use chrono::Utc;
use sdkwork_account_service::{
    AccountBalance, AccountSummary, AccountSummaryQuery, AccountSummarySnapshot,
    AppendLedgerEntryCommand, AppendLedgerEntryOutcome, OutboxDispatchOutcome,
    PointsAccountSnapshot, PointsLotItem, PointsLotListQuery, StoreListPage, WalletAccountItem,
    WalletAccountListQuery, WalletOperation, WalletOperationQuery, WalletOverview,
    WalletTransactionDetailQuery, WalletTransactionItem, WalletTransactionListQuery,
};
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney, CommercePoints,
    CommerceRequestHash, CommerceServiceError,
};
use sdkwork_utils_rust::LIST_TOTAL_SQL_COLUMN;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::store::{
    account_guard::{ensure_points_lot_debit_complete, require_positive_amount},
    account_status_label, account_summary, asset_code_from_type, asset_type_from_code, balance,
    billing_projection, currency_code_for_command, default_currency_code, finalize_list_page,
    format_i64, idempotency_lock_expires_at_rfc3339, map_idempotency_insert_error, next_entity_id,
    next_entity_uuid, optional_org_string, org_id_from_option, parse_subject_i64,
    points_lot_status_label, resolve_idempotency_from_row_fields, resolve_list_sql_paging,
    store_error, IdempotencyRecordAction, ACCOUNT_PURPOSE_GENERAL, ACCOUNT_STATUS_ACTIVE,
    LEDGER_APPEND_SCOPE, OWNER_TYPE_USER, POINTS_LOT_DEBIT_BATCH_SIZE, POINTS_LOT_STATUS_DEPLETED,
};

#[derive(Debug, Clone)]
pub struct SqliteCommerceAccountStore {
    pub(crate) pool: SqlitePool,
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

impl SqliteCommerceAccountStore {
    pub fn new(pool: SqlitePool) -> Self {
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
        let mut token_bank_available = 0_i128;
        let mut token_bank_frozen = 0_i128;

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
                CommerceAccountAssetType::TokenBank => {
                    token_bank_available += parse_amount_minor(account.available_amount.as_str())?;
                    token_bank_frozen += parse_amount_minor(account.frozen_amount.as_str())?;
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
            token_bank: AccountBalance::new(
                CommerceMoney::new(&format_amount_minor(token_bank_available))
                    .map_err(CommerceServiceError::storage)?,
                CommerceMoney::new(&format_amount_minor(token_bank_frozen))
                    .map_err(CommerceServiceError::storage)?,
            )?,
        })
    }

    pub async fn retrieve_account_summary_snapshot(
        &self,
        query: AccountSummaryQuery,
    ) -> Result<AccountSummarySnapshot, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let summary = self.retrieve_summary(query.clone()).await?;
        let available_points = summary
            .points
            .available
            .as_str()
            .parse::<i128>()
            .map_err(|_| {
                CommerceServiceError::storage("invalid points amount in account summary")
            })?;
        let mut stats = account_summary::load_wallet_summary_stats_sqlite(
            &self.pool,
            tenant_id,
            organization_id,
            owner_id,
        )
        .await?;
        stats.est_days_remaining = account_summary::estimate_days_remaining(
            available_points,
            stats.monthly_points_consumed,
        );
        Ok(account_summary::build_account_summary_snapshot(
            &query.owner_user_id,
            organization_id,
            available_points,
            stats,
        ))
    }

    pub async fn list_wallet_accounts(
        &self,
        query: WalletAccountListQuery,
    ) -> Result<Vec<WalletAccountItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let asset_code = query.asset_type.as_ref().map(asset_code_from_type);

        let rows = sqlx::query(
            r#"
            SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
                   available_amount, frozen_amount, pending_amount, status, version
            FROM acct_account
            WHERE tenant_id = ?
              AND organization_id = ?
              AND owner_type = ?
              AND owner_id = ?
              AND (? IS NULL OR asset_code = ?)
              AND status = ?
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
    ) -> Result<StoreListPage<WalletTransactionItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let account_id = match query.account_id.as_deref() {
            Some(value) => Some(parse_subject_i64("account_id", value)?),
            None => None,
        };
        let asset_code = query.asset_type.as_ref().map(asset_code_from_type);
        let paging = resolve_list_sql_paging(query.page, query.page_size, query.cursor.as_deref())?;
        let fetch_limit = paging.fetch_limit;
        let sql_offset = paging.sql_offset;
        let keyset_created_at = paging.keyset_before;

        let rows = if let Some(cursor) = keyset_created_at {
            sqlx::query(sqlx::AssertSqlSafe(format!(
                r#"
                SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
                       direction, amount, balance_before, balance_after, business_type, business_no,
                       request_no, idempotency_key, created_at,
                       COUNT(*) OVER() AS {LIST_TOTAL_SQL_COLUMN}
                FROM acct_ledger_entry
                WHERE tenant_id = ?
                  AND organization_id = ?
                  AND owner_id = ?
                  AND (? IS NULL OR account_id = ?)
                  AND (? IS NULL OR asset_code = ?)
                  AND created_at < ?
                ORDER BY created_at DESC, id DESC
                LIMIT ?
                "#
            )))
            .bind(tenant_id)
            .bind(organization_id)
            .bind(owner_id)
            .bind(account_id)
            .bind(account_id)
            .bind(asset_code)
            .bind(asset_code)
            .bind(cursor)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(sqlx::AssertSqlSafe(format!(
                r#"
                SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
                       direction, amount, balance_before, balance_after, business_type, business_no,
                       request_no, idempotency_key, created_at,
                       COUNT(*) OVER() AS {LIST_TOTAL_SQL_COLUMN}
                FROM acct_ledger_entry
                WHERE tenant_id = ?
                  AND organization_id = ?
                  AND owner_id = ?
                  AND (? IS NULL OR account_id = ?)
                  AND (? IS NULL OR asset_code = ?)
                ORDER BY created_at DESC, id DESC
                LIMIT ? OFFSET ?
                "#
            )))
            .bind(tenant_id)
            .bind(organization_id)
            .bind(owner_id)
            .bind(account_id)
            .bind(account_id)
            .bind(asset_code)
            .bind(asset_code)
            .bind(fetch_limit)
            .bind(sql_offset)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|error| store_error("failed to list wallet transactions", error))?;

        let total_items = rows
            .first()
            .map(|row| integer_cell(row, LIST_TOTAL_SQL_COLUMN))
            .unwrap_or(0);
        let items: Result<Vec<_>, _> = rows.iter().map(map_wallet_transaction).collect();
        Ok(finalize_list_page(
            items?,
            paging.params.page_size,
            total_items,
        ))
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
            FROM acct_ledger_entry
            WHERE tenant_id = ?
              AND organization_id = ?
              AND owner_id = ?
              AND (uuid = ? OR CAST(id AS TEXT) = ?)
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
            FROM acct_ledger_entry
            WHERE tenant_id = ?
              AND organization_id = ?
              AND owner_id = ?
              AND request_no = ?
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
                         AND expires_at <= datetime('now', '+30 days')
                    THEN remaining_amount
                    ELSE 0
                END), 0) AS expiring_points
            FROM acct_points_lot
            WHERE tenant_id = ?
              AND account_id = ?
              AND status = ?
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
    ) -> Result<StoreListPage<PointsLotItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let paging = resolve_list_sql_paging(query.page, query.page_size, None)?;

        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            r#"
            SELECT lot.id, lot.uuid, lot.account_id, lot.granted_amount, lot.remaining_amount,
                   lot.source_type, lot.source_id, lot.expires_at, lot.status,
                   lot.created_at, lot.updated_at,
                   COUNT(*) OVER() AS {LIST_TOTAL_SQL_COLUMN}
            FROM acct_points_lot lot
            INNER JOIN acct_account account
                ON account.id = lot.account_id
               AND account.tenant_id = lot.tenant_id
            WHERE lot.tenant_id = ?
              AND account.organization_id = ?
              AND account.owner_type = ?
              AND account.owner_id = ?
              AND account.asset_code = ?
            ORDER BY
                CASE WHEN lot.expires_at IS NULL THEN 1 ELSE 0 END,
                lot.expires_at ASC,
                lot.created_at ASC
            LIMIT ? OFFSET ?
            "#
        )))
        .bind(tenant_id)
        .bind(organization_id)
        .bind(OWNER_TYPE_USER)
        .bind(owner_id)
        .bind(asset_code_from_type(&CommerceAccountAssetType::Points))
        .bind(paging.fetch_limit)
        .bind(paging.sql_offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to list points lots", error))?;

        let total_items = rows
            .first()
            .map(|row| integer_cell(row, LIST_TOTAL_SQL_COLUMN))
            .unwrap_or(0);
        let items: Result<Vec<_>, _> = rows.iter().map(map_points_lot).collect();
        Ok(finalize_list_page(
            items?,
            paging.params.page_size,
            total_items,
        ))
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
        let now_dt = Utc::now();
        let now = now_dt.to_rfc3339();
        let lock_expires = idempotency_lock_expires_at_rfc3339(now_dt);
        let tenant_id = parse_subject_i64("tenant_id", &command.tenant_id)?;
        let organization_id = org_id_from_option(command.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &command.owner_user_id)?;

        if let Some(row) =
            load_idempotency_row(&mut tx, tenant_id, &command.idempotency_key).await?
        {
            match resolve_idempotency_from_row_fields(
                request_hash.as_str(),
                &string_cell(&row, "request_hash"),
                &string_cell(&row, "status"),
                &string_cell(&row, "locked_until"),
                now_dt,
            )? {
                IdempotencyRecordAction::Replay => {
                    let outcome =
                        load_replayed_outcome(&mut tx, tenant_id, owner_id, &command).await?;
                    tx.commit()
                        .await
                        .map_err(|error| store_error("failed to commit ledger replay", error))?;
                    return Ok(outcome);
                }
                IdempotencyRecordAction::ReclaimLock => {
                    crate::sqlite_hold::reclaim_idempotency_scoped_public(
                        &mut tx,
                        tenant_id,
                        LEDGER_APPEND_SCOPE,
                        &command.idempotency_key,
                        request_hash.as_str(),
                        &now,
                    )
                    .await?;
                }
            }
        } else {
            insert_idempotency_lock(
                &mut tx,
                tenant_id,
                &command,
                request_hash.as_str(),
                &now,
                &lock_expires,
            )
            .await?;
        }

        let mut account = load_or_create_account_for_append(
            &mut tx,
            &command,
            tenant_id,
            organization_id,
            owner_id,
            &now,
        )
        .await?;
        let current_balance = parse_amount_minor(&account.available_amount)?;
        let amount = parse_amount_minor(command.amount.as_str())?;
        require_positive_amount(amount, "amount")?;
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
            UPDATE acct_account
            SET available_amount = ?, version = ?, updated_at = ?
            WHERE id = ? AND version = ?
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
            INSERT INTO acct_journal
                (id, uuid, tenant_id, business_type, business_no, request_no, idempotency_key,
                 status, trace_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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

        let reversed_ledger_id = command
            .reversed_ledger_id
            .as_deref()
            .map(|value| parse_subject_i64("reversed_ledger_id", value))
            .transpose()?;

        sqlx::query(
            r#"
            INSERT INTO acct_ledger_entry
                (id, uuid, tenant_id, organization_id, account_id, journal_id, owner_type, owner_id,
                 asset_code, currency_code, ledger_type, entry_type, direction, amount,
                 balance_before, balance_after, business_type, business_no, request_no,
                 idempotency_key, reversed_ledger_id, trace_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'AVAILABLE', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(reversed_ledger_id)
        .bind(&trace_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to append ledger entry", error))?;

        sqlx::query(
            r#"
            INSERT INTO acct_journal_line
                (id, journal_id, account_id, direction, amount, ledger_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
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
                PointsLotMovementInput {
                    account_id: account.id,
                    amount: lot_amount,
                    direction: command.direction.clone(),
                    expires_at: command.expires_at.as_deref(),
                    ledger_id,
                    now: &now,
                    source_type: &command.business_type,
                    tenant_id,
                },
            )
            .await?;
        }

        billing_projection::insert_billing_history_for_ledger_append(
            &mut *tx,
            tenant_id,
            organization_id,
            owner_id,
            ledger_id,
            &command,
            &now,
        )
        .await?;

        insert_ledger_appended_outbox(
            &mut tx,
            LedgerAppendedOutboxInput {
                account_id: account.id,
                account_uuid: &account.uuid,
                command: &command,
                journal_uuid: &journal_uuid,
                ledger_uuid: &ledger_uuid,
                now: &now,
                tenant_id,
            },
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

        Ok(AppendLedgerEntryOutcome::executed(
            account_item,
            ledger_entry,
        ))
    }

    pub async fn dispatch_outbox_batch(
        &self,
        batch_size: Option<i64>,
    ) -> Result<OutboxDispatchOutcome, CommerceServiceError> {
        crate::store::outbox_relay::dispatch_pending_outbox_sqlite(&self.pool, batch_size).await
    }

    pub async fn pending_outbox_lag(&self) -> Result<i64, CommerceServiceError> {
        crate::store::outbox_relay::count_pending_outbox_sqlite(&self.pool).await
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

struct LedgerAppendedOutboxInput<'a> {
    account_id: i64,
    account_uuid: &'a str,
    command: &'a AppendLedgerEntryCommand,
    journal_uuid: &'a str,
    ledger_uuid: &'a str,
    now: &'a str,
    tenant_id: i64,
}

async fn insert_ledger_appended_outbox(
    tx: &mut Transaction<'_, Sqlite>,
    input: LedgerAppendedOutboxInput<'_>,
) -> Result<(), CommerceServiceError> {
    let (event_key, payload, payload_hash) = crate::store::outbox::build_ledger_appended_outbox(
        input.journal_uuid,
        input.ledger_uuid,
        input.account_uuid,
        input.command,
    )?;
    crate::store::outbox::insert_outbox_event_sqlite(
        &mut **tx,
        crate::store::outbox::OutboxEventInsert {
            aggregate_id: input.account_id,
            event_key: &event_key,
            event_type: crate::store::outbox::OUTBOX_EVENT_TYPE_LEDGER_APPENDED,
            now: input.now,
            payload: &payload,
            payload_hash: &payload_hash,
            tenant_id: input.tenant_id,
        },
    )
    .await
}

async fn load_idempotency_row(
    tx: &mut Transaction<'_, Sqlite>,
    tenant_id: i64,
    idempotency_key: &str,
) -> Result<Option<sqlx::sqlite::SqliteRow>, CommerceServiceError> {
    sqlx::query(
        r#"
        SELECT request_hash, status
        FROM acct_idempotency_record
        WHERE tenant_id = ? AND scope = ? AND idempotency_key = ?
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
    tx: &mut Transaction<'_, Sqlite>,
    tenant_id: i64,
    command: &AppendLedgerEntryCommand,
    request_hash: &str,
    now: &str,
    lock_expires: &str,
) -> Result<(), CommerceServiceError> {
    sqlx::query(
        r#"
        INSERT INTO acct_idempotency_record
            (id, uuid, tenant_id, scope, idempotency_key, request_hash, target_type, status,
             locked_until, expire_at, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, 'ledger', 'LOCKED', ?, ?, ?, ?)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(next_entity_uuid())
    .bind(tenant_id)
    .bind(LEDGER_APPEND_SCOPE)
    .bind(&command.idempotency_key)
    .bind(request_hash)
    .bind(lock_expires)
    .bind(lock_expires)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| map_idempotency_insert_error("failed to insert idempotency lock", error))?;
    Ok(())
}

async fn complete_idempotency(
    tx: &mut Transaction<'_, Sqlite>,
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
        UPDATE acct_idempotency_record
        SET status = 'COMPLETED',
            target_id = ?,
            response_snapshot = ?,
            locked_until = NULL,
            updated_at = ?
        WHERE tenant_id = ? AND scope = ? AND idempotency_key = ?
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
    tx: &mut Transaction<'_, Sqlite>,
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

    if let Some(account) =
        load_account_by_owner_asset(tx, command, tenant_id, organization_id, owner_id).await?
    {
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
        INSERT INTO acct_account
            (id, uuid, tenant_id, organization_id, owner_type, owner_id, asset_code, currency_code,
             account_purpose, available_amount, frozen_amount, pending_amount, status, version,
             created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, '0', '0', '0', ?, 0, ?, ?)
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
        .ok_or_else(|| {
            CommerceServiceError::storage("created commerce account could not be loaded")
        })
}

fn parse_optional_account_id(value: &str) -> Result<Option<i64>, CommerceServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    parse_subject_i64("account_id", trimmed).map(Some)
}

async fn load_account_by_id(
    tx: &mut Transaction<'_, Sqlite>,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
    account_id: i64,
) -> Result<Option<StoredAccount>, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
               available_amount, frozen_amount, pending_amount, status, version
        FROM acct_account
        WHERE id = ? AND tenant_id = ? AND organization_id = ? AND owner_id = ?
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
    tx: &mut Transaction<'_, Sqlite>,
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
        FROM acct_account
        WHERE tenant_id = ?
          AND organization_id = ?
          AND owner_type = ?
          AND owner_id = ?
          AND asset_code = ?
          AND currency_code = ?
          AND account_purpose = ?
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
    tx: &mut Transaction<'_, Sqlite>,
    tenant_id: i64,
    owner_id: i64,
    command: &AppendLedgerEntryCommand,
) -> Result<AppendLedgerEntryOutcome, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
               direction, amount, balance_before, balance_after, business_type, business_no,
               request_no, idempotency_key, created_at
        FROM acct_ledger_entry
        WHERE tenant_id = ? AND owner_id = ? AND idempotency_key = ?
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
    tx: &mut Transaction<'_, Sqlite>,
    account_id: i64,
) -> Result<WalletAccountItem, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
               available_amount, frozen_amount, pending_amount, status, version
        FROM acct_account
        WHERE id = ?
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

pub(crate) fn map_stored_account(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<StoredAccount, CommerceServiceError> {
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

fn map_wallet_account(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<WalletAccountItem, CommerceServiceError> {
    map_stored_account(row)?.to_wallet_item()
}

fn map_wallet_transaction(
    row: &sqlx::sqlite::SqliteRow,
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

fn map_points_lot(row: &sqlx::sqlite::SqliteRow) -> Result<PointsLotItem, CommerceServiceError> {
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
        _ => Err(CommerceServiceError::validation(
            "ledger direction is invalid",
        )),
    }
}

pub(crate) fn string_cell(row: &sqlx::sqlite::SqliteRow, name: &str) -> String {
    row.try_get::<String, _>(name).unwrap_or_default()
}

pub(crate) fn integer_cell(row: &sqlx::sqlite::SqliteRow, name: &str) -> i64 {
    row.try_get::<i64, _>(name).unwrap_or_default()
}

pub(crate) fn parse_amount_minor(value: &str) -> Result<i128, CommerceServiceError> {
    let amount = CommerceMoney::new(value).map_err(CommerceServiceError::validation)?;
    amount
        .as_str()
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

pub(crate) struct PointsLotMovementInput<'a> {
    pub account_id: i64,
    pub amount: i64,
    pub direction: CommerceLedgerDirection,
    pub expires_at: Option<&'a str>,
    pub ledger_id: i64,
    pub now: &'a str,
    pub source_type: &'a str,
    pub tenant_id: i64,
}

pub(crate) async fn apply_points_lot_movement(
    tx: &mut Transaction<'_, Sqlite>,
    input: PointsLotMovementInput<'_>,
) -> Result<(), CommerceServiceError> {
    let PointsLotMovementInput {
        account_id,
        amount,
        direction,
        expires_at,
        ledger_id,
        now,
        source_type,
        tenant_id,
    } = input;
    match direction {
        CommerceLedgerDirection::Credit => {
            let lot_id = next_entity_id()?;
            let lot_uuid = next_entity_uuid();
            sqlx::query(
                r#"
                INSERT INTO acct_points_lot
                    (id, uuid, tenant_id, account_id, granted_amount, remaining_amount,
                     source_type, source_id, expires_at, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(lot_id)
            .bind(&lot_uuid)
            .bind(tenant_id)
            .bind(account_id)
            .bind(amount)
            .bind(amount)
            .bind(source_type)
            .bind(ledger_id)
            .bind(expires_at)
            .bind(ACCOUNT_STATUS_ACTIVE)
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await
            .map_err(|error| store_error("failed to insert points lot", error))?;
        }
        CommerceLedgerDirection::Debit => {
            let mut remaining = amount;
            let mut last_lot_id: Option<i64> = None;

            loop {
                if remaining <= 0 {
                    break;
                }

                let rows = if let Some(last_id) = last_lot_id {
                    sqlx::query(
                        r#"
                        SELECT id, remaining_amount
                        FROM acct_points_lot
                        WHERE tenant_id = ?
                          AND account_id = ?
                          AND status = ?
                          AND remaining_amount > 0
                          AND (expires_at IS NULL OR expires_at > ?)
                          AND id > ?
                        ORDER BY
                            CASE WHEN expires_at IS NULL THEN 1 ELSE 0 END,
                            expires_at ASC,
                            created_at ASC,
                            id ASC
                        LIMIT ?
                        "#,
                    )
                    .bind(tenant_id)
                    .bind(account_id)
                    .bind(ACCOUNT_STATUS_ACTIVE)
                    .bind(now)
                    .bind(last_id)
                    .bind(POINTS_LOT_DEBIT_BATCH_SIZE)
                    .fetch_all(&mut **tx)
                    .await
                    .map_err(|error| store_error("failed to load points lots for debit", error))?
                } else {
                    sqlx::query(
                        r#"
                        SELECT id, remaining_amount
                        FROM acct_points_lot
                        WHERE tenant_id = ?
                          AND account_id = ?
                          AND status = ?
                          AND remaining_amount > 0
                          AND (expires_at IS NULL OR expires_at > ?)
                        ORDER BY
                            CASE WHEN expires_at IS NULL THEN 1 ELSE 0 END,
                            expires_at ASC,
                            created_at ASC,
                            id ASC
                        LIMIT ?
                        "#,
                    )
                    .bind(tenant_id)
                    .bind(account_id)
                    .bind(ACCOUNT_STATUS_ACTIVE)
                    .bind(now)
                    .bind(POINTS_LOT_DEBIT_BATCH_SIZE)
                    .fetch_all(&mut **tx)
                    .await
                    .map_err(|error| store_error("failed to load points lots for debit", error))?
                };

                if rows.is_empty() {
                    break;
                }

                for row in rows {
                    if remaining <= 0 {
                        break;
                    }
                    let lot_id = integer_cell(&row, "id");
                    let lot_remaining = integer_cell(&row, "remaining_amount");
                    let consume = remaining.min(lot_remaining);
                    let next_remaining = lot_remaining - consume;
                    let next_status = if next_remaining == 0 {
                        POINTS_LOT_STATUS_DEPLETED
                    } else {
                        ACCOUNT_STATUS_ACTIVE
                    };
                    sqlx::query(
                        r#"
                        UPDATE acct_points_lot
                        SET remaining_amount = ?, status = ?, updated_at = ?
                        WHERE id = ? AND tenant_id = ?
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

                    sqlx::query(
                        r#"
                        INSERT INTO acct_points_lot_allocation
                            (id, uuid, tenant_id, account_id, ledger_id, lot_id, amount, created_at)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                        "#,
                    )
                    .bind(next_entity_id()?)
                    .bind(next_entity_uuid())
                    .bind(tenant_id)
                    .bind(account_id)
                    .bind(ledger_id)
                    .bind(lot_id)
                    .bind(consume)
                    .bind(now)
                    .execute(&mut **tx)
                    .await
                    .map_err(|error| {
                        store_error("failed to insert points lot allocation", error)
                    })?;

                    remaining -= consume;
                    last_lot_id = Some(lot_id);
                }
            }
            ensure_points_lot_debit_complete(remaining)?;
            let lot_sum: i64 = sqlx::query_scalar(
                r#"
                SELECT COALESCE(SUM(remaining_amount), 0)
                FROM acct_points_lot
                WHERE tenant_id = ? AND account_id = ?
                "#,
            )
            .bind(tenant_id)
            .bind(account_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|error| store_error("failed to sum points lot remaining", error))?;
            let available: String = sqlx::query_scalar(
                "SELECT available_amount FROM acct_account WHERE tenant_id = ? AND id = ?",
            )
            .bind(tenant_id)
            .bind(account_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|error| {
                store_error("failed to load account available for lot invariant", error)
            })?;
            balance::validate_points_lot_balance_invariant(&available, lot_sum)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_account_service::{PointsLotListQuery, WalletAccountListQuery};
    use sdkwork_contract_service::{
        CommerceAccountAssetType, CommerceLedgerBusinessType, CommerceLedgerDirection,
        CommerceMoney, CommerceRequestHash,
    };
    use sqlx::SqlitePool;

    async fn migrated_pool() -> SqlitePool {
        crate::test_sqlite_pool::account_migrated_sqlite_memory_pool().await
    }

    fn credit_command(idempotency_key: &str, transaction_no: &str) -> AppendLedgerEntryCommand {
        AppendLedgerEntryCommand::new(
            "100001",
            Some("0"),
            "",
            "1",
            CommerceAccountAssetType::Points,
            Some("POINT"),
            CommerceLedgerDirection::Credit,
            CommerceMoney::new("100").expect("money"),
            CommerceLedgerBusinessType::POINTS_RECHARGE,
            transaction_no,
            "request-1",
            idempotency_key,
        )
        .expect("command")
    }

    #[test]
    fn sqlite_account_balance_update_is_version_guarded() {
        let source = include_str!("sqlite_account.rs");
        assert!(source.contains("AND version = ?"));
        assert!(source.contains("rows_affected() != 1"));
    }

    #[test]
    fn sqlite_amount_parser_requires_integer_smallest_units() {
        assert_eq!(parse_amount_minor("1990").expect("integer amount"), 1990);
        assert!(parse_amount_minor("19.90").is_err());
        assert!(parse_amount_minor("-1").is_err());
    }

    #[tokio::test]
    async fn sqlite_wallet_lists_accounts_after_ledger_credit() {
        let pool = migrated_pool().await;
        let store = SqliteCommerceAccountStore::new(pool);

        let outcome = store
            .append_ledger_entry(
                credit_command("idem-1", "txn-1"),
                CommerceRequestHash::new("hash-1").expect("request hash"),
            )
            .await
            .expect("append ledger");

        assert!(!outcome.replayed);

        let accounts = store
            .list_wallet_accounts(
                WalletAccountListQuery::new(
                    "100001",
                    Some("0"),
                    "1",
                    Some(CommerceAccountAssetType::Points),
                )
                .expect("query"),
            )
            .await
            .expect("accounts");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].available_amount.as_str(), "100");
        assert!(!accounts[0].uuid.is_empty());
    }

    #[tokio::test]
    async fn sqlite_ledger_append_replays_same_idempotency_key() {
        let pool = migrated_pool().await;
        let store = SqliteCommerceAccountStore::new(pool);

        let first = store
            .append_ledger_entry(
                credit_command("idem-1", "txn-1"),
                CommerceRequestHash::new("hash-1").expect("request hash"),
            )
            .await
            .expect("first append");
        let second = store
            .append_ledger_entry(
                credit_command("idem-1", "txn-1"),
                CommerceRequestHash::new("hash-1").expect("request hash"),
            )
            .await
            .expect("second append");

        assert!(!first.replayed);
        assert!(second.replayed);
        assert_eq!(first.ledger_entry.uuid, second.ledger_entry.uuid);
    }

    fn debit_command(idempotency_key: &str, amount: &str) -> AppendLedgerEntryCommand {
        AppendLedgerEntryCommand::new(
            "100001",
            Some("0"),
            "",
            "1",
            CommerceAccountAssetType::Points,
            Some("POINT"),
            CommerceLedgerDirection::Debit,
            CommerceMoney::new(amount).expect("money"),
            CommerceLedgerBusinessType::POINTS_BURN,
            "txn-debit",
            "request-debit",
            idempotency_key,
        )
        .expect("command")
    }

    #[tokio::test]
    async fn sqlite_points_debit_consumes_lots_fifo() {
        let pool = migrated_pool().await;
        let store = SqliteCommerceAccountStore::new(pool);

        store
            .append_ledger_entry(
                credit_command("fifo-credit-1", "txn-fifo-1"),
                CommerceRequestHash::new("hash-c1").expect("request hash"),
            )
            .await
            .expect("first credit");
        store
            .append_ledger_entry(
                credit_command("fifo-credit-2", "txn-fifo-2"),
                CommerceRequestHash::new("hash-c2").expect("request hash"),
            )
            .await
            .expect("second credit");
        store
            .append_ledger_entry(
                debit_command("fifo-debit-1", "150"),
                CommerceRequestHash::new("hash-d1").expect("request hash"),
            )
            .await
            .expect("debit");

        let lots = store
            .list_points_lots(
                PointsLotListQuery::new("100001", Some("0"), "1", None, None).expect("query"),
            )
            .await
            .expect("lots");

        assert_eq!(lots.items.len(), 2);
        assert_eq!(lots.items[0].remaining_amount, 0);
        assert_eq!(lots.items[1].remaining_amount, 50);
        assert_eq!(lots.items[0].granted_amount, 100);
        assert_eq!(lots.items[1].granted_amount, 100);

        let accounts = store
            .list_wallet_accounts(
                WalletAccountListQuery::new(
                    "100001",
                    Some("0"),
                    "1",
                    Some(CommerceAccountAssetType::Points),
                )
                .expect("query"),
            )
            .await
            .expect("accounts");
        assert_eq!(accounts[0].available_amount.as_str(), "50");
    }

    #[tokio::test]
    async fn sqlite_points_debit_rejects_insufficient_lots() {
        let pool = migrated_pool().await;
        let store = SqliteCommerceAccountStore::new(pool);

        store
            .append_ledger_entry(
                credit_command("lot-credit", "txn-lot-credit"),
                CommerceRequestHash::new("hash-lot-credit").expect("request hash"),
            )
            .await
            .expect("credit");

        let error = store
            .append_ledger_entry(
                debit_command("lot-debit-over", "200"),
                CommerceRequestHash::new("hash-lot-debit").expect("request hash"),
            )
            .await
            .expect_err("debit must fail when lots are insufficient");

        assert_eq!(error.code(), "invalid-state");
    }

    #[tokio::test]
    async fn sqlite_points_expire_sweep_debits_expired_lots() {
        use sdkwork_account_service::{
            ExpirePointsLotsCommand, PointsLotAllocationListQuery, WalletTransactionListQuery,
        };

        let pool = migrated_pool().await;
        let store = SqliteCommerceAccountStore::new(pool);

        let credit = AppendLedgerEntryCommand::with_options(
            "100001",
            Some("0"),
            "",
            "1",
            CommerceAccountAssetType::Points,
            Some("POINT"),
            CommerceLedgerDirection::Credit,
            CommerceMoney::new("80").expect("money"),
            CommerceLedgerBusinessType::POINTS_RECHARGE,
            "txn-expire-credit",
            "request-expire",
            "expire-credit",
            Some("2020-01-01T00:00:00Z"),
            None,
        )
        .expect("command");

        store
            .append_ledger_entry(
                credit,
                CommerceRequestHash::new("hash-expire-credit").expect("request hash"),
            )
            .await
            .expect("credit with past expiry");

        let command = ExpirePointsLotsCommand::new(
            "100001",
            Some("0"),
            Some("1"),
            None,
            "request-expire-sweep",
            "expire-sweep-1",
        )
        .expect("expire command");

        let outcome = store
            .expire_points_lots(
                command,
                CommerceRequestHash::new("hash-expire-sweep").expect("request hash"),
            )
            .await
            .expect("expire sweep");

        assert!(!outcome.replayed);
        assert_eq!(outcome.expired_lot_count, 1);
        assert_eq!(outcome.expired_points_total, 80);

        let accounts = store
            .list_wallet_accounts(
                WalletAccountListQuery::new(
                    "100001",
                    Some("0"),
                    "1",
                    Some(CommerceAccountAssetType::Points),
                )
                .expect("query"),
            )
            .await
            .expect("accounts");
        assert_eq!(accounts[0].available_amount.as_str(), "0");

        let summary = store
            .retrieve_points_summary(
                WalletAccountListQuery::new(
                    "100001",
                    Some("0"),
                    "1",
                    Some(CommerceAccountAssetType::Points),
                )
                .expect("query"),
            )
            .await
            .expect("summary");
        assert_eq!(summary.unswept_expired_points, 0);
        assert_eq!(summary.month_debit_points, 80);

        let ledger_entries = store
            .list_wallet_transactions(
                WalletTransactionListQuery::new(
                    "100001",
                    Some("0"),
                    "1",
                    None,
                    Some(CommerceAccountAssetType::Points),
                    None,
                    None,
                    None,
                )
                .expect("query"),
            )
            .await
            .expect("ledger entries");
        let expire_entry = ledger_entries
            .items
            .iter()
            .find(|entry| entry.business_type == CommerceLedgerBusinessType::POINTS_EXPIRE)
            .expect("expire ledger entry");

        let allocations = store
            .list_points_lot_allocations(
                PointsLotAllocationListQuery::new("100001", Some("0"), "1", &expire_entry.id)
                    .expect("query"),
            )
            .await
            .expect("allocations");
        assert_eq!(allocations.len(), 1);
        assert_eq!(allocations[0].amount, 80);
    }

    #[tokio::test]
    async fn sqlite_reconcile_points_lots_reports_no_mismatch_after_credit() {
        use sdkwork_account_service::PointsReconciliationQuery;

        let pool = migrated_pool().await;
        let store = SqliteCommerceAccountStore::new(pool);

        store
            .append_ledger_entry(
                credit_command("reconcile-credit", "txn-reconcile"),
                CommerceRequestHash::new("hash-reconcile").expect("hash"),
            )
            .await
            .expect("credit");

        let snapshot = store
            .reconcile_points_lots(
                PointsReconciliationQuery::new("100001", Some("0"), Some("1")).expect("query"),
            )
            .await
            .expect("reconcile");

        assert_eq!(snapshot.checked_account_count, 1);
        assert_eq!(snapshot.mismatch_count, 0);
        assert!(snapshot.mismatches.is_empty());
    }
}
