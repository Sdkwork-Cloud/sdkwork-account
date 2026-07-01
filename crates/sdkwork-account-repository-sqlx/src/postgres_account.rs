use chrono::Utc;
use sdkwork_account_service::{
    AccountBalance, AccountSummary, AccountSummaryQuery, AccountSummarySnapshot,
    AppendLedgerEntryCommand, AppendLedgerEntryOutcome, PointsAccountSnapshot, PointsLotItem,
    PointsLotListQuery, WalletAccountItem, WalletAccountListQuery, WalletOperation,
    WalletOperationQuery, WalletOverview, WalletTransactionDetailQuery, WalletTransactionItem,
    WalletTransactionListQuery,
};
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney, CommercePoints,
    CommerceRequestHash, CommerceServiceError,
};
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::store::{
    account_status_label, asset_code_from_type, asset_type_from_code, currency_code_for_command,
    default_currency_code, format_i64, next_entity_id, next_entity_uuid, optional_org_string,
    outbox::{
        build_ledger_appended_outbox, OUTBOX_AGGREGATE_TYPE_ACCOUNT, OUTBOX_EVENT_TYPE_LEDGER_APPENDED,
        OUTBOX_EVENT_VERSION, OUTBOX_STATUS_PENDING,
    },
    org_id_from_option, parse_subject_i64, parse_wallet_transaction_cursor, points_lot_status_label,
    store_error,
    ACCOUNT_PURPOSE_GENERAL, ACCOUNT_STATUS_ACTIVE, LEDGER_APPEND_SCOPE, OWNER_TYPE_USER,
};

#[derive(Debug, Clone)]
pub struct PostgresCommerceAccountStore {
    pub(crate) pool: PgPool,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredAccount {
    pub(crate) id: i64,
    pub(crate) uuid: String,
    pub(crate) tenant_id: i64,
    pub(crate) organization_id: i64,
    pub(crate) owner_id: i64,
    pub(crate) asset_type: CommerceAccountAssetType,
    pub(crate) currency_code: String,
    pub(crate) available_amount: String,
    pub(crate) frozen_amount: String,
    pub(crate) pending_amount: String,
    pub(crate) status: i32,
    pub(crate) version: i64,
}

impl PostgresCommerceAccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn retrieve_summary(
        &self,
        query: AccountSummaryQuery,
    ) -> Result<AccountSummary, CommerceServiceError> {
        let accounts = self
            .list_wallet_accounts(WalletAccountListQuery::new(
                &query.tenant_id,
                query.organization_id.as_deref(),
                &query.owner_user_id,
                None,
            )?)
            .await?;

        let mut cash_available = 0_i128;
        let mut cash_frozen = 0_i128;
        let mut points_available = 0_i128;
        let mut points_frozen = 0_i128;
        let mut token_available = 0_i128;
        let mut token_frozen = 0_i128;

        for account in accounts {
            match account.asset_type {
                CommerceAccountAssetType::Cash => {
                    cash_available += parse_amount_minor(account.available_amount.as_str())?;
                    cash_frozen += parse_amount_minor(account.frozen_amount.as_str())?;
                }
                CommerceAccountAssetType::Points => {
                    points_available += parse_amount_minor(account.available_amount.as_str())?;
                    points_frozen += parse_amount_minor(account.frozen_amount.as_str())?;
                }
                CommerceAccountAssetType::Token => {
                    token_available += parse_amount_minor(account.available_amount.as_str())?;
                    token_frozen += parse_amount_minor(account.frozen_amount.as_str())?;
                }
            }
        }

        Ok(AccountSummary {
            cash: AccountBalance::new(
                CommerceMoney::new(&format_amount_minor(cash_available))
                    .map_err(CommerceServiceError::storage)?,
                CommerceMoney::new(&format_amount_minor(cash_frozen))
                    .map_err(CommerceServiceError::storage)?,
            )?,
            owner_user_id: query.owner_user_id,
            points: AccountBalance::new(
                CommercePoints::new(&points_available.to_string())
                    .map_err(CommerceServiceError::storage)?,
                CommercePoints::new(&points_frozen.to_string())
                    .map_err(CommerceServiceError::storage)?,
            )?,
            tenant_id: query.tenant_id,
            token: AccountBalance::new(
                CommercePoints::new(&token_available.to_string())
                    .map_err(CommerceServiceError::storage)?,
                CommercePoints::new(&token_frozen.to_string())
                    .map_err(CommerceServiceError::storage)?,
            )?,
        })
    }

    pub async fn retrieve_account_summary_snapshot(
        &self,
        query: AccountSummaryQuery,
    ) -> Result<AccountSummarySnapshot, CommerceServiceError> {
        let summary = self.retrieve_summary(query.clone()).await?;
        let available_points = summary
            .points
            .available
            .as_str()
            .parse::<f64>()
            .unwrap_or(0.0);

        Ok(AccountSummarySnapshot {
            id: query.owner_user_id.clone(),
            name: "User".to_owned(),
            email: String::new(),
            is_verified: false,
            tier: "Standard".to_owned(),
            organization: String::new(),
            available_credits: available_points,
            est_days_remaining: 0,
            monthly_consumption: 0.0,
            consumption_by_service: Vec::new(),
            invoice_settings: Default::default(),
            security: Default::default(),
            login_logs: Vec::new(),
        })
    }

    pub async fn list_wallet_accounts(
        &self,
        query: WalletAccountListQuery,
    ) -> Result<Vec<WalletAccountItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let asset_code = query
            .asset_type
            .as_ref()
            .map(asset_code_from_type);

        let rows = sqlx::query(
            r#"
            SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
                   available_amount, frozen_amount, pending_amount, status, version
            FROM commerce_account
            WHERE tenant_id = $1
              AND organization_id = $2
              AND owner_type = $3
              AND owner_id = $4
              AND ($5 IS NULL OR asset_code = $6)
              AND status = $7
            ORDER BY asset_code ASC, currency_code ASC, id ASC
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(OWNER_TYPE_USER)
        .bind(owner_id)
        .bind(asset_code)
        .bind(asset_code)
        .bind(ACCOUNT_STATUS_ACTIVE)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to list wallet accounts", error))?;

        rows.iter().map(map_wallet_account).collect()
    }

    pub async fn retrieve_wallet_overview(
        &self,
        query: WalletAccountListQuery,
    ) -> Result<WalletOverview, CommerceServiceError> {
        Ok(WalletOverview::new(self.list_wallet_accounts(query).await?))
    }

    pub async fn list_wallet_transactions(
        &self,
        query: WalletTransactionListQuery,
    ) -> Result<Vec<WalletTransactionItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let account_id = match query.account_id.as_deref() {
            Some(value) => Some(parse_subject_i64("account_id", value)?),
            None => None,
        };
        let asset_code = query
            .asset_type
            .as_ref()
            .map(asset_code_from_type);
        let cursor_created_at =
            parse_wallet_transaction_cursor(query.cursor.as_deref())?;

        let rows = if let Some(cursor) = cursor_created_at {
            sqlx::query(
                r#"
                SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
                       direction, amount, balance_before, balance_after, business_type, business_no,
                       request_no, idempotency_key, created_at
                FROM commerce_account_ledger
                WHERE tenant_id = $1
                  AND organization_id = $2
                  AND owner_id = $3
                  AND ($4 IS NULL OR account_id = $5)
                  AND ($6 IS NULL OR asset_code = $7)
                  AND created_at < $8
                ORDER BY created_at DESC, id DESC
                LIMIT $9 OFFSET $10
                "#,
            )
            .bind(tenant_id)
            .bind(organization_id)
            .bind(owner_id)
            .bind(account_id)
            .bind(account_id)
            .bind(asset_code)
            .bind(asset_code)
            .bind(cursor)
            .bind(query.limit())
            .bind(query.offset())
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
                       direction, amount, balance_before, balance_after, business_type, business_no,
                       request_no, idempotency_key, created_at
                FROM commerce_account_ledger
                WHERE tenant_id = $1
                  AND organization_id = $2
                  AND owner_id = $3
                  AND ($4 IS NULL OR account_id = $5)
                  AND ($6 IS NULL OR asset_code = $7)
                ORDER BY created_at DESC, id DESC
                LIMIT $8 OFFSET $9
                "#,
            )
            .bind(tenant_id)
            .bind(organization_id)
            .bind(owner_id)
            .bind(account_id)
            .bind(account_id)
            .bind(asset_code)
            .bind(asset_code)
            .bind(query.limit())
            .bind(query.offset())
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|error| store_error("failed to list wallet transactions", error))?;

        rows.iter().map(map_wallet_transaction).collect()
    }

    pub async fn retrieve_wallet_transaction(
        &self,
        query: WalletTransactionDetailQuery,
    ) -> Result<Option<WalletTransactionItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let ledger_key = query.transaction_id.trim();

        let row = sqlx::query(
            r#"
            SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
                   direction, amount, balance_before, balance_after, business_type, business_no,
                   request_no, idempotency_key, created_at
            FROM commerce_account_ledger
            WHERE tenant_id = $1
              AND organization_id = $2
              AND owner_id = $3
              AND (uuid = $4 OR CAST(id AS TEXT) = $5)
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(owner_id)
        .bind(ledger_key)
        .bind(ledger_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("failed to retrieve wallet transaction", error))?;

        row.as_ref().map(map_wallet_transaction).transpose()
    }

    pub async fn retrieve_wallet_operation(
        &self,
        query: WalletOperationQuery,
    ) -> Result<Option<WalletOperation>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;

        let rows = sqlx::query(
            r#"
            SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
                   direction, amount, balance_before, balance_after, business_type, business_no,
                   request_no, idempotency_key, created_at
            FROM commerce_account_ledger
            WHERE tenant_id = $1
              AND organization_id = $2
              AND owner_id = $3
              AND request_no = $4
            ORDER BY created_at DESC, id DESC
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(owner_id)
        .bind(&query.request_no)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to retrieve wallet operation", error))?;

        if rows.is_empty() {
            return Ok(None);
        }

        let transactions = rows
            .iter()
            .map(map_wallet_transaction)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(WalletOperation::new(&query.request_no, transactions)?))
    }

    pub async fn retrieve_wallet_account_for_asset(
        &self,
        query: WalletAccountListQuery,
        asset_type: CommerceAccountAssetType,
    ) -> Result<WalletAccountItem, CommerceServiceError> {
        let scoped = WalletAccountListQuery::new(
            &query.tenant_id,
            query.organization_id.as_deref(),
            &query.owner_user_id,
            Some(asset_type.clone()),
        )?;
        let accounts = self.list_wallet_accounts(scoped).await?;
        let currency_code = default_currency_code(&asset_type);
        Ok(accounts.into_iter().next().unwrap_or_else(|| {
            WalletAccountItem::zero_for_owner(
                &query.tenant_id,
                query.organization_id.as_deref(),
                &query.owner_user_id,
                asset_type,
                Some(currency_code),
            )
            .expect("zero wallet account")
        }))
    }

    pub async fn retrieve_points_account_snapshot(
        &self,
        query: WalletAccountListQuery,
    ) -> Result<PointsAccountSnapshot, CommerceServiceError> {
        let account = self
            .retrieve_wallet_account_for_asset(query.clone(), CommerceAccountAssetType::Points)
            .await?;
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let account_id = account.id.parse::<i64>().unwrap_or(0);
        if account_id <= 0 {
            return Ok(PointsAccountSnapshot {
                account,
                active_lot_count: 0,
                expiring_points: 0,
            });
        }

        let stats = sqlx::query(
            r#"
            SELECT
                COUNT(*) AS active_lot_count,
                COALESCE(SUM(CASE
                    WHEN expires_at IS NOT NULL
                         AND expires_at <= NOW() + INTERVAL '30 days'
                    THEN remaining_amount
                    ELSE 0
                END), 0) AS expiring_points
            FROM commerce_points_lot
            WHERE tenant_id = $1
              AND account_id = $2
              AND status = $3
            "#,
        )
        .bind(tenant_id)
        .bind(account_id)
        .bind(ACCOUNT_STATUS_ACTIVE)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("failed to load points lot stats", error))?;

        Ok(PointsAccountSnapshot {
            account,
            active_lot_count: integer_cell(&stats, "active_lot_count"),
            expiring_points: integer_cell(&stats, "expiring_points"),
        })
    }

    pub async fn list_points_lots(
        &self,
        query: PointsLotListQuery,
    ) -> Result<Vec<PointsLotItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let limit = query.limit();
        let offset = query.offset();

        let rows = sqlx::query(
            r#"
            SELECT lot.id, lot.uuid, lot.account_id, lot.granted_amount, lot.remaining_amount,
                   lot.source_type, lot.source_id, lot.expires_at, lot.status,
                   lot.created_at, lot.updated_at
            FROM commerce_points_lot lot
            INNER JOIN commerce_account account
                ON account.id = lot.account_id
               AND account.tenant_id = lot.tenant_id
            WHERE lot.tenant_id = $1
              AND account.organization_id = $2
              AND account.owner_type = $3
              AND account.owner_id = $4
              AND account.asset_code = $5
            ORDER BY lot.expires_at NULLS LAST, lot.created_at ASC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(OWNER_TYPE_USER)
        .bind(owner_id)
        .bind(asset_code_from_type(&CommerceAccountAssetType::Points))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to list points lots", error))?;

        rows.iter().map(map_points_lot).collect()
    }

    pub async fn append_ledger_entry(
        &self,
        command: AppendLedgerEntryCommand,
        request_hash: CommerceRequestHash,
    ) -> Result<AppendLedgerEntryOutcome, CommerceServiceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("failed to begin ledger transaction", error))?;
        let now = Utc::now().to_rfc3339();
        let tenant_id = parse_subject_i64("tenant_id", &command.tenant_id)?;
        let organization_id = org_id_from_option(command.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &command.owner_user_id)?;

        if let Some(row) = load_idempotency_row(&mut tx, tenant_id, &command.idempotency_key).await?
        {
            let stored_hash = string_cell(&row, "request_hash");
            if stored_hash != request_hash.as_str() {
                return Err(CommerceServiceError::conflict(
                    "idempotency key was used with a different request hash",
                ));
            }
            if string_cell(&row, "status") == "COMPLETED" {
                let outcome = load_replayed_outcome(&mut tx, tenant_id, owner_id, &command).await?;
                tx.commit()
                    .await
                    .map_err(|error| store_error("failed to commit ledger replay", error))?;
                return Ok(outcome);
            }
        } else {
            insert_idempotency_lock(
                &mut tx,
                tenant_id,
                &command,
                request_hash.as_str(),
                &now,
            )
            .await?;
        }

        let mut account = load_or_create_account_for_append(&mut tx, &command, tenant_id, organization_id, owner_id, &now).await?;
        let current_balance = parse_amount_minor(&account.available_amount)?;
        let amount = parse_amount_minor(command.amount.as_str())?;
        let next_balance = match command.direction {
            CommerceLedgerDirection::Credit => checked_add(current_balance, amount)?,
            CommerceLedgerDirection::Debit => {
                if current_balance < amount {
                    return Err(CommerceServiceError::invalid_state(
                        "insufficient account balance",
                    ));
                }
                current_balance
                    .checked_sub(amount)
                    .ok_or_else(|| CommerceServiceError::storage("balance subtraction overflow"))?
            }
        };
        let balance_before = format_amount_minor(current_balance);
        let balance_after = format_amount_minor(next_balance);
        let next_version = account.version.checked_add(1).ok_or_else(|| {
            CommerceServiceError::storage("commerce account version increment overflow")
        })?;

        let update = sqlx::query(
            r#"
            UPDATE commerce_account
            SET available_amount = $1, version = $2, updated_at = $3
            WHERE id = $4 AND version = $5
            "#,
        )
        .bind(&balance_after)
        .bind(next_version)
        .bind(&now)
        .bind(account.id)
        .bind(account.version)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to update commerce account balance", error))?;
        if update.rows_affected() != 1 {
            return Err(CommerceServiceError::conflict(
                "commerce account balance update was not applied atomically",
            ));
        }

        account.available_amount = balance_after.clone();
        account.version = next_version;

        let journal_id = next_entity_id()?;
        let journal_uuid = next_entity_uuid();
        let ledger_id = next_entity_id()?;
        let ledger_uuid = next_entity_uuid();
        let trace_id = next_entity_uuid();

        sqlx::query(
            r#"
            INSERT INTO commerce_account_journal
                (id, uuid, tenant_id, business_type, business_no, request_no, idempotency_key,
                 status, trace_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(journal_id)
        .bind(&journal_uuid)
        .bind(tenant_id)
        .bind(&command.business_type)
        .bind(&command.transaction_no)
        .bind(&command.request_no)
        .bind(&command.idempotency_key)
        .bind(ACCOUNT_STATUS_ACTIVE)
        .bind(&trace_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert journal", error))?;

        let entry_type = match command.direction {
            CommerceLedgerDirection::Credit => "CREDIT",
            CommerceLedgerDirection::Debit => "DEBIT",
        };

        sqlx::query(
            r#"
            INSERT INTO commerce_account_ledger
                (id, uuid, tenant_id, organization_id, account_id, journal_id, owner_type, owner_id,
                 asset_code, currency_code, ledger_type, entry_type, direction, amount,
                 balance_before, balance_after, business_type, business_no, request_no,
                 idempotency_key, trace_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'AVAILABLE', $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            "#,
        )
        .bind(ledger_id)
        .bind(&ledger_uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(account.id)
        .bind(journal_id)
        .bind(OWNER_TYPE_USER)
        .bind(owner_id)
        .bind(asset_code_from_type(&command.asset_type))
        .bind(currency_code_for_command(&command))
        .bind(entry_type)
        .bind(command.direction.as_str())
        .bind(command.amount.as_str())
        .bind(&balance_before)
        .bind(&balance_after)
        .bind(&command.business_type)
        .bind(&command.transaction_no)
        .bind(&command.request_no)
        .bind(&command.idempotency_key)
        .bind(&trace_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to append ledger entry", error))?;

        sqlx::query(
            r#"
            INSERT INTO commerce_account_journal_line
                (id, journal_id, account_id, direction, amount, ledger_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(next_entity_id()?)
        .bind(journal_id)
        .bind(account.id)
        .bind(command.direction.as_str())
        .bind(command.amount.as_str())
        .bind(ledger_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert journal line", error))?;

        if command.asset_type == CommerceAccountAssetType::Points {
            let lot_amount = i64::try_from(amount).map_err(|_| {
                CommerceServiceError::validation("points amount exceeds supported lot range")
            })?;
            apply_points_lot_movement(
                &mut tx,
                tenant_id,
                account.id,
                &command.direction,
                lot_amount,
                &command.business_type,
                ledger_id,
                &now,
            )
            .await?;
        }

        insert_ledger_appended_outbox(
            &mut tx,
            tenant_id,
            account.id,
            &account.uuid,
            &journal_uuid,
            &ledger_uuid,
            &command,
            &now,
        )
        .await?;

        let account_item = account.to_wallet_item()?;
        let ledger_entry = WalletTransactionItem::new(
            &format_i64(ledger_id),
            &ledger_uuid,
            &format_i64(account.id),
            &command.tenant_id,
            optional_org_string(organization_id).as_deref(),
            &command.owner_user_id,
            command.asset_type.clone(),
            command.direction.clone(),
            command.amount.as_str(),
            &balance_before,
            &balance_after,
            &command.business_type,
            &command.transaction_no,
            &command.request_no,
            &command.idempotency_key,
            &now,
        )?;

        complete_idempotency(
            &mut tx,
            tenant_id,
            &command.idempotency_key,
            ledger_id,
            &ledger_entry,
            &now,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|error| store_error("failed to commit ledger transaction", error))?;

        Ok(AppendLedgerEntryOutcome::executed(account_item, ledger_entry))
    }
}

impl StoredAccount {
    pub(crate) fn to_wallet_item(&self) -> Result<WalletAccountItem, CommerceServiceError> {
        WalletAccountItem::new(
            &format_i64(self.id),
            &self.uuid,
            &format_i64(self.tenant_id),
            optional_org_string(self.organization_id).as_deref(),
            &format_i64(self.owner_id),
            self.asset_type.clone(),
            Some(self.currency_code.as_str()).filter(|value| !value.is_empty()),
            &self.available_amount,
            &self.frozen_amount,
            &self.pending_amount,
            account_status_label(self.status),
            self.version,
        )
    }
}

async fn insert_ledger_appended_outbox(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    account_id: i64,
    account_uuid: &str,
    journal_uuid: &str,
    ledger_uuid: &str,
    command: &AppendLedgerEntryCommand,
    now: &str,
) -> Result<(), CommerceServiceError> {
    let (event_key, payload, payload_hash) =
        build_ledger_appended_outbox(journal_uuid, ledger_uuid, account_uuid, command)?;
    sqlx::query(
        r#"
        INSERT INTO commerce_outbox_event
            (id, uuid, tenant_id, aggregate_type, aggregate_id, event_type, event_version,
             event_key, payload, payload_hash, status, retry_count, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 0, $12, $13)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(next_entity_uuid())
    .bind(tenant_id)
    .bind(OUTBOX_AGGREGATE_TYPE_ACCOUNT)
    .bind(account_id)
    .bind(OUTBOX_EVENT_TYPE_LEDGER_APPENDED)
    .bind(OUTBOX_EVENT_VERSION)
    .bind(event_key)
    .bind(payload)
    .bind(payload_hash)
    .bind(OUTBOX_STATUS_PENDING)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert ledger outbox event", error))?;
    Ok(())
}

async fn load_idempotency_row(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    idempotency_key: &str,
) -> Result<Option<sqlx::postgres::PgRow>, CommerceServiceError> {
    sqlx::query(
        r#"
        SELECT request_hash, status
        FROM commerce_idempotency_record
        WHERE tenant_id = $1 AND scope = $2 AND idempotency_key = $3
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(LEDGER_APPEND_SCOPE)
    .bind(idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load idempotency record", error))
}

async fn insert_idempotency_lock(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    command: &AppendLedgerEntryCommand,
    request_hash: &str,
    now: &str,
) -> Result<(), CommerceServiceError> {
    sqlx::query(
        r#"
        INSERT INTO commerce_idempotency_record
            (id, uuid, tenant_id, scope, idempotency_key, request_hash, target_type, status,
             locked_until, expire_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'ledger', 'LOCKED', $7, $8, $9, $10)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(next_entity_uuid())
    .bind(tenant_id)
    .bind(LEDGER_APPEND_SCOPE)
    .bind(&command.idempotency_key)
    .bind(request_hash)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert idempotency lock", error))?;
    Ok(())
}

async fn complete_idempotency(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    idempotency_key: &str,
    ledger_id: i64,
    ledger_entry: &WalletTransactionItem,
    now: &str,
) -> Result<(), CommerceServiceError> {
    let response_json = serde_json::json!({
        "accountUuid": ledger_entry.account_id,
        "ledgerEntryUuid": ledger_entry.uuid,
        "requestNo": ledger_entry.request_no,
        "businessNo": ledger_entry.transaction_no,
    })
    .to_string();

    sqlx::query(
        r#"
        UPDATE commerce_idempotency_record
        SET status = 'COMPLETED',
            target_id = $1,
            response_snapshot = $2,
            locked_until = NULL,
            updated_at = $3
        WHERE tenant_id = $4 AND scope = $5 AND idempotency_key = $6
        "#,
    )
    .bind(ledger_id)
    .bind(response_json)
    .bind(now)
    .bind(tenant_id)
    .bind(LEDGER_APPEND_SCOPE)
    .bind(idempotency_key)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to complete idempotency record", error))?;
    Ok(())
}

async fn load_or_create_account_for_append(
    tx: &mut Transaction<'_, Postgres>,
    command: &AppendLedgerEntryCommand,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
    now: &str,
) -> Result<StoredAccount, CommerceServiceError> {
    if let Some(account_id) = parse_optional_account_id(&command.account_id)? {
        if let Some(account) =
            load_account_by_id(tx, tenant_id, organization_id, owner_id, account_id).await?
        {
            return Ok(account);
        }
    }

    if let Some(account) = load_account_by_owner_asset(tx, command, tenant_id, organization_id, owner_id).await? {
        return Ok(account);
    }

    if matches!(command.direction, CommerceLedgerDirection::Debit) {
        return Err(CommerceServiceError::invalid_state(
            "insufficient account balance",
        ));
    }

    let account_id = next_entity_id()?;
    let account_uuid = next_entity_uuid();
    let currency_code = currency_code_for_command(command);

    sqlx::query(
        r#"
        INSERT INTO commerce_account
            (id, uuid, tenant_id, organization_id, owner_type, owner_id, asset_code, currency_code,
             account_purpose, available_amount, frozen_amount, pending_amount, status, version,
             created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '0', '0', '0', $10, 0, $11, $12)
        "#,
    )
    .bind(account_id)
    .bind(&account_uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(OWNER_TYPE_USER)
    .bind(owner_id)
    .bind(asset_code_from_type(&command.asset_type))
    .bind(&currency_code)
    .bind(ACCOUNT_PURPOSE_GENERAL)
    .bind(ACCOUNT_STATUS_ACTIVE)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to create commerce account", error))?;

    load_account_by_id(tx, tenant_id, organization_id, owner_id, account_id)
        .await?
        .ok_or_else(|| CommerceServiceError::storage("created commerce account could not be loaded"))
}

fn parse_optional_account_id(value: &str) -> Result<Option<i64>, CommerceServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    parse_subject_i64("account_id", trimmed).map(Some)
}

async fn load_account_by_id(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
    account_id: i64,
) -> Result<Option<StoredAccount>, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
               available_amount, frozen_amount, pending_amount, status, version
        FROM commerce_account
        WHERE id = $1 AND tenant_id = $2 AND organization_id = $3 AND owner_id = $4
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(owner_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load commerce account by id", error))?;

    row.as_ref().map(map_stored_account).transpose()
}

async fn load_account_by_owner_asset(
    tx: &mut Transaction<'_, Postgres>,
    command: &AppendLedgerEntryCommand,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
) -> Result<Option<StoredAccount>, CommerceServiceError> {
    let currency_code = currency_code_for_command(command);
    let row = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
               available_amount, frozen_amount, pending_amount, status, version
        FROM commerce_account
        WHERE tenant_id = $1
          AND organization_id = $2
          AND owner_type = $3
          AND owner_id = $4
          AND asset_code = $5
          AND currency_code = $6
          AND account_purpose = $7
        ORDER BY updated_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(OWNER_TYPE_USER)
    .bind(owner_id)
    .bind(asset_code_from_type(&command.asset_type))
    .bind(currency_code)
    .bind(ACCOUNT_PURPOSE_GENERAL)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load commerce account by owner asset", error))?;

    row.as_ref().map(map_stored_account).transpose()
}

async fn load_replayed_outcome(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    owner_id: i64,
    command: &AppendLedgerEntryCommand,
) -> Result<AppendLedgerEntryOutcome, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
               direction, amount, balance_before, balance_after, business_type, business_no,
               request_no, idempotency_key, created_at
        FROM commerce_account_ledger
        WHERE tenant_id = $1 AND owner_id = $2 AND idempotency_key = $3
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(owner_id)
    .bind(&command.idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load replayed ledger entry", error))?
    .ok_or_else(|| CommerceServiceError::invalid_state("idempotency record has no ledger entry"))?;

    let ledger_entry = map_wallet_transaction(&row)?;
    let account_id = parse_subject_i64("account_id", &ledger_entry.account_id)?;
    let account = load_account_item_for_replay(tx, account_id).await?;
    Ok(AppendLedgerEntryOutcome::replayed(account, ledger_entry))
}

async fn load_account_item_for_replay(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
) -> Result<WalletAccountItem, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
               available_amount, frozen_amount, pending_amount, status, version
        FROM commerce_account
        WHERE id = $1
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load replayed account", error))?
    .ok_or_else(|| CommerceServiceError::invalid_state("ledger account is missing"))?;

    map_wallet_account(&row)
}

pub(crate) fn map_stored_account(row: &sqlx::postgres::PgRow) -> Result<StoredAccount, CommerceServiceError> {
    Ok(StoredAccount {
        id: integer_cell(row, "id"),
        uuid: string_cell(row, "uuid"),
        tenant_id: integer_cell(row, "tenant_id"),
        organization_id: integer_cell(row, "organization_id"),
        owner_id: integer_cell(row, "owner_id"),
        asset_type: asset_type_from_code(&string_cell(row, "asset_code"))?,
        currency_code: string_cell(row, "currency_code"),
        available_amount: string_cell(row, "available_amount"),
        frozen_amount: string_cell(row, "frozen_amount"),
        pending_amount: string_cell(row, "pending_amount"),
        status: integer_cell(row, "status") as i32,
        version: integer_cell(row, "version"),
    })
}

fn map_wallet_account(row: &sqlx::postgres::PgRow) -> Result<WalletAccountItem, CommerceServiceError> {
    map_stored_account(row)?.to_wallet_item()
}

fn map_wallet_transaction(
    row: &sqlx::postgres::PgRow,
) -> Result<WalletTransactionItem, CommerceServiceError> {
    WalletTransactionItem::new(
        &format_i64(integer_cell(row, "id")),
        &string_cell(row, "uuid"),
        &format_i64(integer_cell(row, "account_id")),
        &format_i64(integer_cell(row, "tenant_id")),
        optional_org_string(integer_cell(row, "organization_id")).as_deref(),
        &format_i64(integer_cell(row, "owner_id")),
        asset_type_from_code(&string_cell(row, "asset_code"))?,
        parse_direction(&string_cell(row, "direction"))?,
        &string_cell(row, "amount"),
        &string_cell(row, "balance_before"),
        &string_cell(row, "balance_after"),
        &string_cell(row, "business_type"),
        &string_cell(row, "business_no"),
        &string_cell(row, "request_no"),
        &string_cell(row, "idempotency_key"),
        &string_cell(row, "created_at"),
    )
}

fn map_points_lot(row: &sqlx::postgres::PgRow) -> Result<PointsLotItem, CommerceServiceError> {
    let expires_at = row
        .try_get::<Option<String>, _>("expires_at")
        .unwrap_or(None)
        .filter(|value| !value.trim().is_empty());
    Ok(PointsLotItem {
        id: format_i64(integer_cell(row, "id")),
        uuid: string_cell(row, "uuid"),
        account_id: format_i64(integer_cell(row, "account_id")),
        granted_amount: integer_cell(row, "granted_amount"),
        remaining_amount: integer_cell(row, "remaining_amount"),
        source_type: string_cell(row, "source_type"),
        source_id: format_i64(integer_cell(row, "source_id")),
        expires_at,
        status: points_lot_status_label(integer_cell(row, "status") as i32).to_string(),
        created_at: string_cell(row, "created_at"),
        updated_at: string_cell(row, "updated_at"),
    })
}

fn parse_direction(value: &str) -> Result<CommerceLedgerDirection, CommerceServiceError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "credit" => Ok(CommerceLedgerDirection::Credit),
        "debit" => Ok(CommerceLedgerDirection::Debit),
        _ => Err(CommerceServiceError::validation("ledger direction is invalid")),
    }
}

fn string_cell(row: &sqlx::postgres::PgRow, name: &str) -> String {
    row.try_get::<String, _>(name).unwrap_or_default()
}

fn integer_cell(row: &sqlx::postgres::PgRow, name: &str) -> i64 {
    row.try_get::<i64, _>(name).unwrap_or_default()
}

pub(crate) fn parse_amount_minor(value: &str) -> Result<i128, CommerceServiceError> {
    if value.contains('.') {
        let parts: Vec<_> = value.split('.').collect();
        if parts.len() != 2 {
            return Err(CommerceServiceError::validation("amount is invalid"));
        }
        let whole = parts[0].parse::<i128>().map_err(|_| {
            CommerceServiceError::validation("amount whole part is invalid")
        })?;
        let fraction = parts[1].parse::<i128>().map_err(|_| {
            CommerceServiceError::validation("amount fraction is invalid")
        })?;
        let scale = 10_i128.pow(parts[1].len() as u32);
        return Ok(whole * scale + fraction);
    }
    value
        .parse::<i128>()
        .map_err(|_| CommerceServiceError::validation("amount is invalid"))
}

pub(crate) fn format_amount_minor(value: i128) -> String {
    if value == 0 {
        return "0".to_string();
    }
    value.to_string()
}

fn checked_add(left: i128, right: i128) -> Result<i128, CommerceServiceError> {
    left.checked_add(right)
        .ok_or_else(|| CommerceServiceError::storage("amount addition overflow"))
}

pub(crate) async fn apply_points_lot_movement_for_hold_postgres(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    account_id: i64,
    direction: &CommerceLedgerDirection,
    amount: i64,
    source_type: &str,
    source_ledger_id: i64,
    now: &str,
) -> Result<(), CommerceServiceError> {
    apply_points_lot_movement(
        tx,
        tenant_id,
        account_id,
        direction,
        amount,
        source_type,
        source_ledger_id,
        now,
    )
    .await
}

async fn apply_points_lot_movement(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    account_id: i64,
    direction: &CommerceLedgerDirection,
    amount: i64,
    source_type: &str,
    source_ledger_id: i64,
    now: &str,
) -> Result<(), CommerceServiceError> {
    match direction {
        CommerceLedgerDirection::Credit => {
            let lot_id = next_entity_id()?;
            let lot_uuid = next_entity_uuid();
            sqlx::query(
                r#"
                INSERT INTO commerce_points_lot
                    (id, uuid, tenant_id, account_id, granted_amount, remaining_amount,
                     source_type, source_id, expires_at, status, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, $9, $10, $11)
                "#,
            )
            .bind(lot_id)
            .bind(&lot_uuid)
            .bind(tenant_id)
            .bind(account_id)
            .bind(amount)
            .bind(amount)
            .bind(source_type)
            .bind(source_ledger_id)
            .bind(ACCOUNT_STATUS_ACTIVE)
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await
            .map_err(|error| store_error("failed to insert points lot", error))?;
        }
        CommerceLedgerDirection::Debit => {
            let rows = sqlx::query(
                r#"
                SELECT id, remaining_amount
                FROM commerce_points_lot
                WHERE tenant_id = $1
                  AND account_id = $2
                  AND status = $3
                  AND remaining_amount > 0
                ORDER BY expires_at NULLS LAST, created_at ASC
                "#,
            )
            .bind(tenant_id)
            .bind(account_id)
            .bind(ACCOUNT_STATUS_ACTIVE)
            .fetch_all(&mut **tx)
            .await
            .map_err(|error| store_error("failed to load points lots for debit", error))?;

            let mut remaining = amount;
            for row in rows {
                if remaining <= 0 {
                    break;
                }
                let lot_id = integer_cell(&row, "id");
                let lot_remaining = integer_cell(&row, "remaining_amount");
                let consume = remaining.min(lot_remaining);
                let next_remaining = lot_remaining - consume;
                let next_status = if next_remaining == 0 { 2 } else { ACCOUNT_STATUS_ACTIVE };
                sqlx::query(
                    r#"
                    UPDATE commerce_points_lot
                    SET remaining_amount = $1, status = $2, updated_at = $3
                    WHERE id = $4 AND tenant_id = $5
                    "#,
                )
                .bind(next_remaining)
                .bind(next_status)
                .bind(now)
                .bind(lot_id)
                .bind(tenant_id)
                .execute(&mut **tx)
                .await
                .map_err(|error| store_error("failed to consume points lot", error))?;
                remaining -= consume;
            }
        }
    }
    Ok(())
}
