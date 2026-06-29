use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_account_service::{
    AccountBalance, AccountConsumptionItem, AccountInvoiceSettings, AccountLoginLog,
    AccountSecuritySummary, AccountSummary, AccountSummaryQuery, AccountSummarySnapshot,
    AppendLedgerEntryCommand, AppendLedgerEntryOutcome, WalletAccountItem, WalletAccountListQuery,
    WalletOperation, WalletOperationQuery, WalletOverview, WalletTransactionDetailQuery,
    WalletTransactionItem, WalletTransactionListQuery,
};
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney, CommercePoints,
    CommerceRequestHash, CommerceServiceError,
};
use sqlx::{PgPool, Postgres, Row, Transaction};

const LEDGER_APPEND_SCOPE: &str = "ledger.entries.append";
const ACTIVE_STATUS: &str = "active";

const LOAD_ACCOUNT_PROFILE: &str = r#"
SELECT
    CAST(u.id AS TEXT) AS user_id,
    COALESCE(NULLIF(u.display_name, ''), NULLIF(u.username, ''), 'User') AS name,
    COALESCE(u.email, '') AS email,
    COALESCE(o.name, '') AS organization
FROM iam_user u
LEFT JOIN iam_organization o
    ON o.tenant_id = u.tenant_id
   AND o.id = CAST($2 AS TEXT)
WHERE u.tenant_id = CAST($1 AS TEXT)
  AND u.id = CAST($3 AS TEXT)
LIMIT 1
"#;

const LOAD_ACCOUNT_POINTS: &str = r#"
SELECT CAST(COALESCE(SUM(COALESCE(available_amount::numeric, 0)), 0) AS TEXT) AS available_points
FROM commerce_account
WHERE tenant_id = CAST($1 AS TEXT)
  AND (organization_id IS NULL OR organization_id = CAST($2 AS TEXT))
  AND owner_user_id = CAST($3 AS TEXT)
  AND asset_type = $4
  AND status = 'active'
"#;

const LOAD_MONTHLY_CONSUMPTION: &str = r#"
SELECT CAST(COALESCE(SUM(COALESCE(customer_charge_amount, cost_amount, 0)), 0) AS TEXT) AS monthly_consumption
FROM ai_usage_fact
WHERE status = 1
  AND tenant_id = CAST($1 AS TEXT)
  AND organization_id = CAST($2 AS TEXT)
  AND user_id = CAST($3 AS TEXT)
  AND occurred_at >= date_trunc('month', now())
"#;

const LOAD_CONSUMPTION_BY_SERVICE: &str = r#"
SELECT
    modality,
    CAST(COALESCE(SUM(COALESCE(customer_charge_amount, cost_amount, 0)), 0) AS TEXT) AS value
FROM ai_usage_fact
WHERE status = 1
  AND tenant_id = CAST($1 AS TEXT)
  AND organization_id = CAST($2 AS TEXT)
  AND user_id = CAST($3 AS TEXT)
  AND occurred_at >= date_trunc('month', now())
GROUP BY modality
ORDER BY modality ASC
"#;

const LOAD_INVOICE_SETTINGS: &str = r#"
SELECT
    COALESCE(NULLIF(name, ''), '') AS org_full,
    COALESCE(NULLIF(tax_no, ''), '') AS tax_id,
    '' AS payment_method,
    COALESCE(NULLIF(title_type, ''), '') AS invoice_type
FROM commerce_invoice_title
WHERE tenant_id = CAST($1 AS TEXT)
  AND owner_user_id = CAST($2 AS TEXT)
ORDER BY updated_at DESC NULLS LAST, id DESC
LIMIT 1
"#;

const LOAD_SECURITY: &str = r#"
SELECT
    mfa_enabled,
    COALESCE(trusted_device_count, 0) AS trusted_device_count
FROM iam_user_security_setting
WHERE tenant_id = CAST($1 AS TEXT)
  AND organization_id = CAST($2 AS TEXT)
  AND user_id = CAST($3 AS TEXT)
  AND deleted_at IS NULL
ORDER BY updated_at DESC NULLS LAST, id DESC
LIMIT 1
"#;

const LOAD_LOGIN_LOGS: &str = r#"
SELECT
    COALESCE(NULLIF(client_ip_masked, ''), '-') AS ip,
    COALESCE(NULLIF(client_ip_region, ''), '-') AS location,
    COALESCE(NULLIF(device_label, ''), '-') AS device,
    CAST(COALESCE(occurred_at, created_at) AS TEXT) AS time,
    login_result,
    risk_level
FROM iam_user_login_event
WHERE tenant_id = CAST($1 AS TEXT)
  AND organization_id = CAST($2 AS TEXT)
  AND user_id = CAST($3 AS TEXT)
ORDER BY COALESCE(occurred_at, created_at) DESC NULLS LAST, id DESC
LIMIT 5
"#;

#[derive(Debug, Clone)]
pub struct PostgresCommerceAccountStore {
    pool: PgPool,
}

#[derive(Debug, Clone)]
struct StoredAccount {
    id: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    asset_type: CommerceAccountAssetType,
    currency_code: Option<String>,
    available_amount: String,
    frozen_amount: String,
    status: String,
    version: i64,
}

#[derive(Debug, Clone)]
struct AccountProfile {
    id: String,
    name: String,
    email: String,
    organization: String,
    available_points: f64,
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
                    cash_available += parse_money_minor(account.available_amount.as_str())?;
                    cash_frozen += parse_money_minor(account.frozen_amount.as_str())?;
                }
                CommerceAccountAssetType::Points => {
                    points_available += parse_points_amount(account.available_amount.as_str())?;
                    points_frozen += parse_points_amount(account.frozen_amount.as_str())?;
                }
                CommerceAccountAssetType::Token => {
                    token_available += parse_points_amount(account.available_amount.as_str())?;
                    token_frozen += parse_points_amount(account.frozen_amount.as_str())?;
                }
            }
        }

        Ok(AccountSummary {
            cash: AccountBalance::new(
                CommerceMoney::new(&format_money_minor(cash_available))
                    .map_err(CommerceServiceError::storage)?,
                CommerceMoney::new(&format_money_minor(cash_frozen))
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
        let profile = self.load_profile(&query).await?;
        let monthly_consumption = self.load_monthly_consumption(&query).await?;
        let consumption_by_service = self.load_consumption_by_service(&query).await?;
        let invoice_settings = self.load_invoice_settings(&query).await?;
        let security = self.load_security(&query).await?;
        let login_logs = self.load_login_logs(&query).await?;
        let is_verified =
            !invoice_settings.org_full.is_empty() || !invoice_settings.tax_id.is_empty();

        Ok(AccountSummarySnapshot {
            id: profile.id,
            name: profile.name,
            email: profile.email,
            is_verified,
            tier: "Standard".to_owned(),
            organization: profile.organization,
            available_credits: profile.available_points,
            est_days_remaining: estimate_days_remaining(
                profile.available_points,
                monthly_consumption,
            ),
            monthly_consumption,
            consumption_by_service,
            invoice_settings,
            security,
            login_logs,
        })
    }

    pub async fn list_wallet_accounts(
        &self,
        query: WalletAccountListQuery,
    ) -> Result<Vec<WalletAccountItem>, CommerceServiceError> {
        let asset_type = query
            .asset_type
            .as_ref()
            .map(CommerceAccountAssetType::as_str);
        let rows = sqlx::query(
            r#"
            SELECT id, tenant_id, organization_id, owner_user_id, asset_type, currency_code,
                   CAST(available_amount AS TEXT) AS available_amount,
                   CAST(frozen_amount AS TEXT) AS frozen_amount,
                   status,
                   CAST(version AS BIGINT) AS version
            FROM commerce_account
            WHERE tenant_id = CAST($1 AS TEXT)
              AND ((organization_id = CAST($2 AS TEXT)) OR (organization_id IS NULL AND $2 IS NULL))
              AND owner_user_id = CAST($3 AS TEXT)
              AND ($4 IS NULL OR asset_type = $4)
              AND status = 'active'
            ORDER BY asset_type ASC, currency_code ASC NULLS LAST, id ASC
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(asset_type)
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
        let asset_type = query
            .asset_type
            .as_ref()
            .map(CommerceAccountAssetType::as_str);
        let rows = sqlx::query(
            r#"
            SELECT id, account_id, tenant_id, organization_id, owner_user_id, asset_type,
                   direction,
                   CAST(amount AS TEXT) AS amount,
                   CAST(balance_after AS TEXT) AS balance_after,
                   business_type, transaction_no, request_no, idempotency_key,
                   CAST(created_at AS TEXT) AS created_at
            FROM commerce_account_ledger_entry
            WHERE tenant_id = CAST($1 AS TEXT)
              AND ((organization_id = CAST($2 AS TEXT)) OR (organization_id IS NULL AND $2 IS NULL))
              AND owner_user_id = CAST($3 AS TEXT)
              AND ($4 IS NULL OR account_id = $4)
              AND ($5 IS NULL OR asset_type = $5)
              AND ($6 IS NULL OR created_at < $6)
            ORDER BY created_at DESC NULLS LAST, id DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(query.account_id.as_deref())
        .bind(asset_type)
        .bind(query.cursor.as_deref())
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to list wallet transactions", error))?;

        rows.iter().map(map_wallet_transaction).collect()
    }

    pub async fn retrieve_wallet_transaction(
        &self,
        query: WalletTransactionDetailQuery,
    ) -> Result<Option<WalletTransactionItem>, CommerceServiceError> {
        let row = sqlx::query(
            r#"
            SELECT id, account_id, tenant_id, organization_id, owner_user_id, asset_type,
                   direction,
                   CAST(amount AS TEXT) AS amount,
                   CAST(balance_after AS TEXT) AS balance_after,
                   business_type, transaction_no, request_no, idempotency_key,
                   CAST(created_at AS TEXT) AS created_at
            FROM commerce_account_ledger_entry
            WHERE tenant_id = CAST($1 AS TEXT)
              AND ((organization_id = CAST($2 AS TEXT)) OR (organization_id IS NULL AND $2 IS NULL))
              AND owner_user_id = CAST($3 AS TEXT)
              AND id = CAST($4 AS TEXT)
            LIMIT 1
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(&query.transaction_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("failed to retrieve wallet transaction", error))?;

        row.as_ref().map(map_wallet_transaction).transpose()
    }

    pub async fn retrieve_wallet_operation(
        &self,
        query: WalletOperationQuery,
    ) -> Result<Option<WalletOperation>, CommerceServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, account_id, tenant_id, organization_id, owner_user_id, asset_type,
                   direction,
                   CAST(amount AS TEXT) AS amount,
                   CAST(balance_after AS TEXT) AS balance_after,
                   business_type, transaction_no, request_no, idempotency_key,
                   CAST(created_at AS TEXT) AS created_at
            FROM commerce_account_ledger_entry
            WHERE tenant_id = CAST($1 AS TEXT)
              AND ((organization_id = CAST($2 AS TEXT)) OR (organization_id IS NULL AND $2 IS NULL))
              AND owner_user_id = CAST($3 AS TEXT)
              AND request_no = CAST($4 AS TEXT)
            ORDER BY created_at DESC NULLS LAST, id DESC
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
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

    async fn load_profile(
        &self,
        query: &AccountSummaryQuery,
    ) -> Result<AccountProfile, CommerceServiceError> {
        let row = sqlx::query(LOAD_ACCOUNT_PROFILE)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .bind(&query.owner_user_id)
            .fetch_optional(&self.pool)
            .await
            .or_else(optional_postgres_row_when_read_model_is_missing)?;

        let mut profile = row
            .as_ref()
            .map(|row| AccountProfile {
                id: string_cell(row, "user_id"),
                name: string_cell(row, "name"),
                email: string_cell(row, "email"),
                organization: string_cell(row, "organization"),
                available_points: 0.0,
            })
            .unwrap_or_else(|| default_account_profile(query));
        profile.available_points = self.load_account_points(query).await?;
        Ok(profile)
    }

    async fn load_account_points(
        &self,
        query: &AccountSummaryQuery,
    ) -> Result<f64, CommerceServiceError> {
        let Some(row) = sqlx::query(LOAD_ACCOUNT_POINTS)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .bind(&query.owner_user_id)
            .bind(CommerceAccountAssetType::Points.as_str())
            .fetch_one(&self.pool)
            .await
            .map(Some)
            .or_else(optional_postgres_row_when_read_model_is_missing)?
        else {
            return Ok(0.0);
        };
        Ok(decimal_cell(&row, "available_points"))
    }

    async fn load_monthly_consumption(
        &self,
        query: &AccountSummaryQuery,
    ) -> Result<f64, CommerceServiceError> {
        let Some(row) = sqlx::query(LOAD_MONTHLY_CONSUMPTION)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .bind(&query.owner_user_id)
            .fetch_one(&self.pool)
            .await
            .map(Some)
            .or_else(optional_postgres_row_when_read_model_is_missing)?
        else {
            return Ok(0.0);
        };
        Ok(decimal_cell(&row, "monthly_consumption"))
    }

    async fn load_consumption_by_service(
        &self,
        query: &AccountSummaryQuery,
    ) -> Result<Vec<AccountConsumptionItem>, CommerceServiceError> {
        let rows = sqlx::query(LOAD_CONSUMPTION_BY_SERVICE)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .bind(&query.owner_user_id)
            .fetch_all(&self.pool)
            .await
            .or_else(empty_postgres_rows_when_read_model_is_missing)?;
        let mut items = rows
            .iter()
            .map(|row| {
                let modality = optional_integer_cell(row, "modality");
                AccountConsumptionItem {
                    name: modality_label(modality).to_owned(),
                    value: decimal_cell(row, "value"),
                    color: modality_color(modality).to_owned(),
                    percentage: 0.0,
                }
            })
            .collect::<Vec<_>>();
        apply_percentages(&mut items);
        Ok(items)
    }

    async fn load_invoice_settings(
        &self,
        query: &AccountSummaryQuery,
    ) -> Result<AccountInvoiceSettings, CommerceServiceError> {
        let row = sqlx::query(LOAD_INVOICE_SETTINGS)
            .bind(&query.tenant_id)
            .bind(&query.owner_user_id)
            .fetch_optional(&self.pool)
            .await
            .or_else(optional_postgres_row_when_read_model_is_missing)?;

        Ok(row
            .as_ref()
            .map(|row| AccountInvoiceSettings {
                org_full: string_cell(row, "org_full"),
                tax_id: string_cell(row, "tax_id"),
                payment_method: string_cell(row, "payment_method"),
                invoice_type: invoice_type_label(&string_cell(row, "invoice_type")).to_owned(),
            })
            .unwrap_or_default())
    }

    async fn load_security(
        &self,
        query: &AccountSummaryQuery,
    ) -> Result<AccountSecuritySummary, CommerceServiceError> {
        let row = sqlx::query(LOAD_SECURITY)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .bind(&query.owner_user_id)
            .fetch_optional(&self.pool)
            .await
            .or_else(optional_postgres_row_when_read_model_is_missing)?;

        Ok(row
            .as_ref()
            .map(|row| AccountSecuritySummary {
                mfa_enabled: bool_cell(row, "mfa_enabled"),
                qps_limit: 0,
                ip_whitelist_count: integer_cell(row, "trusted_device_count"),
            })
            .unwrap_or_default())
    }

    async fn load_login_logs(
        &self,
        query: &AccountSummaryQuery,
    ) -> Result<Vec<AccountLoginLog>, CommerceServiceError> {
        let rows = sqlx::query(LOAD_LOGIN_LOGS)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .bind(&query.owner_user_id)
            .fetch_all(&self.pool)
            .await
            .or_else(empty_postgres_rows_when_read_model_is_missing)?;

        Ok(rows
            .iter()
            .map(|row| AccountLoginLog {
                ip: string_cell(row, "ip"),
                location: string_cell(row, "location"),
                device: string_cell(row, "device"),
                time: string_cell(row, "time"),
                status: login_status(
                    optional_integer_cell(row, "login_result"),
                    optional_integer_cell(row, "risk_level"),
                )
                .to_owned(),
            })
            .collect())
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
        let now = current_timestamp_string();

        if let Some(row) = load_idempotency_row(&mut tx, &command).await? {
            let stored_hash = string_cell(&row, "request_hash");
            if stored_hash != request_hash.as_str() {
                return Err(CommerceServiceError::conflict(
                    "idempotency key was used with a different request hash",
                ));
            }

            if string_cell(&row, "status") == "completed" {
                let outcome = load_replayed_outcome(&mut tx, &command).await?;
                tx.commit()
                    .await
                    .map_err(|error| store_error("failed to commit ledger replay", error))?;
                return Ok(outcome);
            }

            sqlx::query(
                r#"
                UPDATE commerce_idempotency_key
                SET status = 'locked',
                    locked_until = $1,
                    expires_at = $2,
                    updated_at = $3
                WHERE tenant_id = $4 AND scope = $5 AND idempotency_key = $6
                "#,
            )
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .bind(&command.tenant_id)
            .bind(LEDGER_APPEND_SCOPE)
            .bind(&command.idempotency_key)
            .execute(&mut *tx)
            .await
            .map_err(|error| store_error("failed to refresh idempotency lock", error))?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO commerce_idempotency_key
                    (id, tenant_id, organization_id, scope, idempotency_key, request_hash,
                     response_json, status, locked_until, expires_at, created_at, updated_at)
                VALUES
                    ($1, $2, $3, $4, $5, $6, NULL, 'locked', $7, $8, $9, $10)
                "#,
            )
            .bind(idempotency_id(&command))
            .bind(&command.tenant_id)
            .bind(command.organization_id.as_deref())
            .bind(LEDGER_APPEND_SCOPE)
            .bind(&command.idempotency_key)
            .bind(request_hash.as_str())
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|error| store_error("failed to insert idempotency lock", error))?;
        }

        let mut account = load_or_create_account_for_append(&mut tx, &command, &now).await?;
        let current_balance =
            parse_stored_ledger_amount(&command.asset_type, &account.available_amount)?;
        let amount = parse_command_ledger_amount(&command.asset_type, command.amount.as_str())?;
        let next_balance = match command.direction {
            CommerceLedgerDirection::Credit => checked_ledger_add(current_balance, amount)?,
            CommerceLedgerDirection::Debit => {
                if current_balance < amount {
                    return Err(CommerceServiceError::invalid_state(
                        "insufficient account balance",
                    ));
                }
                current_balance.checked_sub(amount).ok_or_else(|| {
                    CommerceServiceError::storage("commerce account balance subtraction overflow")
                })?
            }
        };
        let next_balance_text = format_ledger_amount(&command.asset_type, next_balance);
        let next_version = checked_account_version_increment(account.version)?;

        let account_update = sqlx::query(
            r#"
            UPDATE commerce_account
            SET available_amount = $1,
                frozen_amount = $2,
                version = $3,
                status = 'active',
                updated_at = $4
            WHERE id = $5
              AND version = $6
            "#,
        )
        .bind(&next_balance_text)
        .bind(&account.frozen_amount)
        .bind(next_version)
        .bind(&now)
        .bind(&account.id)
        .bind(account.version)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to update commerce account balance", error))?;
        if account_update.rows_affected() != 1 {
            return Err(CommerceServiceError::conflict(
                "commerce account balance update was not applied atomically",
            ));
        }

        account.available_amount = next_balance_text.clone();
        account.status = ACTIVE_STATUS.to_string();
        account.version = next_version;

        let ledger_id = ledger_entry_id(&command);
        sqlx::query(
            r#"
            INSERT INTO commerce_account_ledger_entry
                (id, tenant_id, organization_id, account_id, owner_user_id, asset_type, direction,
                 amount, balance_after, business_type, transaction_no, request_no, idempotency_key,
                 source_type, source_id, remark, created_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NULL, NULL, NULL, $14)
            "#,
        )
        .bind(&ledger_id)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(&account.id)
        .bind(&command.owner_user_id)
        .bind(command.asset_type.as_str())
        .bind(command.direction.as_str())
        .bind(command.amount.as_str())
        .bind(&next_balance_text)
        .bind(&command.business_type)
        .bind(&command.transaction_no)
        .bind(&command.request_no)
        .bind(&command.idempotency_key)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to append commerce account ledger entry", error))?;

        let account_item = account.to_wallet_item()?;
        let ledger_entry = WalletTransactionItem::new(
            &ledger_id,
            &account.id,
            &command.tenant_id,
            command.organization_id.as_deref(),
            &command.owner_user_id,
            command.asset_type.clone(),
            command.direction.clone(),
            command.amount.as_str(),
            &next_balance_text,
            &command.business_type,
            &command.transaction_no,
            &command.request_no,
            &command.idempotency_key,
            &now,
        )?;
        let response_json = serde_json::json!({
            "accountId": account_item.id,
            "ledgerEntryId": ledger_entry.id,
            "requestNo": ledger_entry.request_no,
            "transactionNo": ledger_entry.transaction_no,
        })
        .to_string();

        sqlx::query(
            r#"
            UPDATE commerce_idempotency_key
            SET response_json = $1,
                status = 'completed',
                locked_until = NULL,
                updated_at = $2
            WHERE tenant_id = $3 AND scope = $4 AND idempotency_key = $5
            "#,
        )
        .bind(response_json)
        .bind(&now)
        .bind(&command.tenant_id)
        .bind(LEDGER_APPEND_SCOPE)
        .bind(&command.idempotency_key)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to complete idempotency record", error))?;

        tx.commit()
            .await
            .map_err(|error| store_error("failed to commit ledger transaction", error))?;

        Ok(AppendLedgerEntryOutcome::executed(
            account_item,
            ledger_entry,
        ))
    }
}

impl StoredAccount {
    fn to_wallet_item(&self) -> Result<WalletAccountItem, CommerceServiceError> {
        WalletAccountItem::new(
            &self.id,
            &self.tenant_id,
            self.organization_id.as_deref(),
            &self.owner_user_id,
            self.asset_type.clone(),
            self.currency_code.as_deref(),
            &self.available_amount,
            &self.frozen_amount,
            &self.status,
            self.version,
        )
    }
}

async fn load_idempotency_row(
    tx: &mut Transaction<'_, Postgres>,
    command: &AppendLedgerEntryCommand,
) -> Result<Option<sqlx::postgres::PgRow>, CommerceServiceError> {
    sqlx::query(
        r#"
        SELECT request_hash, status
        FROM commerce_idempotency_key
        WHERE tenant_id = $1 AND scope = $2 AND idempotency_key = $3
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(&command.tenant_id)
    .bind(LEDGER_APPEND_SCOPE)
    .bind(&command.idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load idempotency record", error))
}

async fn load_or_create_account_for_append(
    tx: &mut Transaction<'_, Postgres>,
    command: &AppendLedgerEntryCommand,
    now: &str,
) -> Result<StoredAccount, CommerceServiceError> {
    if let Some(account) = load_account_by_id(tx, command).await? {
        return Ok(account);
    }

    if let Some(account) = load_account_by_owner_asset(tx, command).await? {
        return Ok(account);
    }

    if matches!(command.direction, CommerceLedgerDirection::Debit) {
        return Err(CommerceServiceError::invalid_state(
            "insufficient account balance",
        ));
    }

    sqlx::query(
        r#"
        INSERT INTO commerce_account
            (id, tenant_id, organization_id, owner_user_id, asset_type, currency_code,
             available_amount, frozen_amount, version, status, created_at, updated_at)
        VALUES
            ($1, $2, $3, $4, $5, $6, '0', '0', 0, 'active', $7, $8)
        "#,
    )
    .bind(&command.account_id)
    .bind(&command.tenant_id)
    .bind(command.organization_id.as_deref())
    .bind(&command.owner_user_id)
    .bind(command.asset_type.as_str())
    .bind(command.currency_code.as_deref())
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to create commerce account", error))?;

    load_account_by_id(tx, command).await?.ok_or_else(|| {
        CommerceServiceError::storage("created commerce account could not be loaded")
    })
}

async fn load_account_by_id(
    tx: &mut Transaction<'_, Postgres>,
    command: &AppendLedgerEntryCommand,
) -> Result<Option<StoredAccount>, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, organization_id, owner_user_id, asset_type, currency_code,
               CAST(available_amount AS TEXT) AS available_amount,
               CAST(frozen_amount AS TEXT) AS frozen_amount,
               status,
               CAST(version AS BIGINT) AS version
        FROM commerce_account
        WHERE id = $1
          AND tenant_id = $2
          AND ((organization_id = CAST($3 AS TEXT)) OR (organization_id IS NULL AND $3 IS NULL))
          AND owner_user_id = $4
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(&command.account_id)
    .bind(&command.tenant_id)
    .bind(command.organization_id.as_deref())
    .bind(&command.owner_user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load commerce account by id", error))?;

    row.as_ref().map(map_stored_account).transpose()
}

async fn load_account_by_owner_asset(
    tx: &mut Transaction<'_, Postgres>,
    command: &AppendLedgerEntryCommand,
) -> Result<Option<StoredAccount>, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, organization_id, owner_user_id, asset_type, currency_code,
               CAST(available_amount AS TEXT) AS available_amount,
               CAST(frozen_amount AS TEXT) AS frozen_amount,
               status,
               CAST(version AS BIGINT) AS version
        FROM commerce_account
        WHERE tenant_id = $1
          AND ((organization_id = CAST($2 AS TEXT)) OR (organization_id IS NULL AND $2 IS NULL))
          AND owner_user_id = $3
          AND asset_type = $4
          AND ((currency_code = CAST($5 AS TEXT)) OR (currency_code IS NULL AND $5 IS NULL))
        ORDER BY updated_at DESC NULLS LAST, id DESC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(&command.tenant_id)
    .bind(command.organization_id.as_deref())
    .bind(&command.owner_user_id)
    .bind(command.asset_type.as_str())
    .bind(command.currency_code.as_deref())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load commerce account by owner asset", error))?;

    row.as_ref().map(map_stored_account).transpose()
}

async fn load_replayed_outcome(
    tx: &mut Transaction<'_, Postgres>,
    command: &AppendLedgerEntryCommand,
) -> Result<AppendLedgerEntryOutcome, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, account_id, tenant_id, organization_id, owner_user_id, asset_type,
               direction,
               CAST(amount AS TEXT) AS amount,
               CAST(balance_after AS TEXT) AS balance_after,
               business_type, transaction_no, request_no, idempotency_key,
               CAST(created_at AS TEXT) AS created_at
        FROM commerce_account_ledger_entry
        WHERE tenant_id = $1
          AND owner_user_id = $2
          AND idempotency_key = $3
        ORDER BY created_at DESC NULLS LAST, id DESC
        LIMIT 1
        "#,
    )
    .bind(&command.tenant_id)
    .bind(&command.owner_user_id)
    .bind(&command.idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load replayed ledger entry", error))?
    .ok_or_else(|| CommerceServiceError::invalid_state("idempotency record has no ledger entry"))?;

    let ledger_entry = map_wallet_transaction(&row)?;
    let account = load_account_item_for_replay(tx, &ledger_entry.account_id).await?;
    Ok(AppendLedgerEntryOutcome::replayed(account, ledger_entry))
}

async fn load_account_item_for_replay(
    tx: &mut Transaction<'_, Postgres>,
    account_id: &str,
) -> Result<WalletAccountItem, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, organization_id, owner_user_id, asset_type, currency_code,
               CAST(available_amount AS TEXT) AS available_amount,
               CAST(frozen_amount AS TEXT) AS frozen_amount,
               status,
               CAST(version AS BIGINT) AS version
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

fn map_stored_account(row: &sqlx::postgres::PgRow) -> Result<StoredAccount, CommerceServiceError> {
    Ok(StoredAccount {
        id: string_cell(row, "id"),
        tenant_id: string_cell(row, "tenant_id"),
        organization_id: optional_string_cell(row, "organization_id"),
        owner_user_id: string_cell(row, "owner_user_id"),
        asset_type: parse_asset_type(&string_cell(row, "asset_type"))?,
        currency_code: optional_string_cell(row, "currency_code"),
        available_amount: string_cell(row, "available_amount"),
        frozen_amount: string_cell(row, "frozen_amount"),
        status: string_cell(row, "status"),
        version: integer_cell(row, "version"),
    })
}

fn map_wallet_account(
    row: &sqlx::postgres::PgRow,
) -> Result<WalletAccountItem, CommerceServiceError> {
    map_stored_account(row)?.to_wallet_item()
}

fn map_wallet_transaction(
    row: &sqlx::postgres::PgRow,
) -> Result<WalletTransactionItem, CommerceServiceError> {
    WalletTransactionItem::new(
        &string_cell(row, "id"),
        &string_cell(row, "account_id"),
        &string_cell(row, "tenant_id"),
        optional_string_cell(row, "organization_id").as_deref(),
        &string_cell(row, "owner_user_id"),
        parse_asset_type(&string_cell(row, "asset_type"))?,
        parse_ledger_direction(&string_cell(row, "direction"))?,
        &string_cell(row, "amount"),
        &string_cell(row, "balance_after"),
        &string_cell(row, "business_type"),
        &string_cell(row, "transaction_no"),
        &string_cell(row, "request_no"),
        &string_cell(row, "idempotency_key"),
        &string_cell(row, "created_at"),
    )
}

fn parse_asset_type(value: &str) -> Result<CommerceAccountAssetType, CommerceServiceError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "cash" => Ok(CommerceAccountAssetType::Cash),
        "point" | "points" => Ok(CommerceAccountAssetType::Points),
        "token" | "tokens" => Ok(CommerceAccountAssetType::Token),
        _ => Err(CommerceServiceError::storage(format!(
            "unknown commerce account asset type: {value}"
        ))),
    }
}

fn parse_ledger_direction(value: &str) -> Result<CommerceLedgerDirection, CommerceServiceError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "credit" => Ok(CommerceLedgerDirection::Credit),
        "debit" => Ok(CommerceLedgerDirection::Debit),
        _ => Err(CommerceServiceError::storage(format!(
            "unknown commerce ledger direction: {value}"
        ))),
    }
}

fn parse_money_minor(value: &str) -> Result<i128, CommerceServiceError> {
    let normalized = value.trim();
    if normalized.is_empty() || normalized.starts_with('-') || normalized.starts_with('+') {
        return Err(CommerceServiceError::storage(format!(
            "invalid commerce money amount: {value}"
        )));
    }

    let mut parts = normalized.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if parts.next().is_some()
        || integer.is_empty()
        || !integer.chars().all(|character| character.is_ascii_digit())
    {
        return Err(CommerceServiceError::storage(format!(
            "invalid commerce money amount: {value}"
        )));
    }

    let integer_value = integer.parse::<i128>().map_err(|_| {
        CommerceServiceError::storage(format!("invalid commerce money amount: {value}"))
    })?;
    let integer_minor = integer_value.checked_mul(100).ok_or_else(|| {
        CommerceServiceError::storage(format!("commerce money amount is too large: {value}"))
    })?;
    let fraction_minor = match fraction {
        Some(fraction) => {
            if fraction.is_empty()
                || fraction.len() > 2
                || !fraction.chars().all(|character| character.is_ascii_digit())
            {
                return Err(CommerceServiceError::storage(format!(
                    "invalid commerce money amount: {value}"
                )));
            }
            let padded = if fraction.len() == 1 {
                format!("{fraction}0")
            } else {
                fraction.to_string()
            };
            padded.parse::<i128>().map_err(|_| {
                CommerceServiceError::storage(format!("invalid commerce money amount: {value}"))
            })?
        }
        None => 0,
    };

    integer_minor.checked_add(fraction_minor).ok_or_else(|| {
        CommerceServiceError::storage(format!("commerce money amount is too large: {value}"))
    })
}

fn parse_points_amount(value: &str) -> Result<i128, CommerceServiceError> {
    let normalized = value.trim();
    if normalized.is_empty() || normalized.starts_with('-') || normalized.starts_with('+') {
        return Err(CommerceServiceError::storage(format!(
            "invalid commerce points amount: {value}"
        )));
    }
    if !normalized
        .chars()
        .all(|character| character.is_ascii_digit())
    {
        return Err(CommerceServiceError::storage(format!(
            "invalid commerce points amount: {value}"
        )));
    }
    normalized.parse::<i128>().map_err(|_| {
        CommerceServiceError::storage(format!("invalid commerce points amount: {value}"))
    })
}

fn parse_stored_ledger_amount(
    asset_type: &CommerceAccountAssetType,
    value: &str,
) -> Result<i128, CommerceServiceError> {
    match asset_type {
        CommerceAccountAssetType::Cash => parse_money_minor(value),
        CommerceAccountAssetType::Points | CommerceAccountAssetType::Token => {
            parse_points_amount(value)
        }
    }
}

fn parse_command_ledger_amount(
    asset_type: &CommerceAccountAssetType,
    value: &str,
) -> Result<i128, CommerceServiceError> {
    match asset_type {
        CommerceAccountAssetType::Cash => parse_money_minor(value),
        CommerceAccountAssetType::Points | CommerceAccountAssetType::Token => {
            parse_points_amount(value).map_err(|_| {
                CommerceServiceError::validation(
                    "points and token ledger amounts must be non-negative integers",
                )
            })
        }
    }
}

fn format_ledger_amount(asset_type: &CommerceAccountAssetType, value: i128) -> String {
    match asset_type {
        CommerceAccountAssetType::Cash => format_money_minor(value),
        CommerceAccountAssetType::Points | CommerceAccountAssetType::Token => value.to_string(),
    }
}

fn checked_ledger_add(left: i128, right: i128) -> Result<i128, CommerceServiceError> {
    left.checked_add(right)
        .ok_or_else(|| CommerceServiceError::storage("commerce account balance addition overflow"))
}

fn checked_account_version_increment(version: i64) -> Result<i64, CommerceServiceError> {
    version
        .checked_add(1)
        .ok_or_else(|| CommerceServiceError::storage("commerce account version overflow"))
}

fn format_money_minor(value: i128) -> String {
    let integer = value / 100;
    let fraction = value % 100;
    if fraction == 0 {
        integer.to_string()
    } else if fraction % 10 == 0 {
        format!("{integer}.{}", fraction / 10)
    } else {
        format!("{integer}.{fraction:02}")
    }
}

fn idempotency_id(command: &AppendLedgerEntryCommand) -> String {
    stable_storage_id(&[
        "idem",
        &command.tenant_id,
        LEDGER_APPEND_SCOPE,
        &command.idempotency_key,
    ])
}

fn ledger_entry_id(command: &AppendLedgerEntryCommand) -> String {
    stable_storage_id(&["ledger", &command.tenant_id, &command.transaction_no])
}

fn stable_storage_id(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|part| {
            part.chars()
                .map(|character| {
                    if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                        character
                    } else {
                        '-'
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn default_account_profile(query: &AccountSummaryQuery) -> AccountProfile {
    AccountProfile {
        id: query.owner_user_id.clone(),
        name: String::new(),
        email: String::new(),
        organization: String::new(),
        available_points: 0.0,
    }
}

fn estimate_days_remaining(available_credits: f64, monthly_consumption: f64) -> i64 {
    if available_credits <= 0.0 || monthly_consumption <= 0.0 {
        return 0;
    }
    let daily_average = monthly_consumption / 30.0;
    (available_credits / daily_average).floor().max(0.0) as i64
}

fn apply_percentages(items: &mut [AccountConsumptionItem]) {
    let total: f64 = items.iter().map(|item| item.value).sum();
    if total <= 0.0 {
        return;
    }
    for item in items {
        item.percentage = ((item.value / total) * 100.0).clamp(0.0, 100.0);
    }
}

fn modality_label(value: Option<i64>) -> &'static str {
    match value {
        Some(1) => "Text",
        Some(2) => "Image",
        Some(3) => "Video",
        Some(4) => "Audio",
        Some(5) => "Music",
        None => "Unknown",
        Some(_) => "Unknown",
    }
}

fn modality_color(value: Option<i64>) -> &'static str {
    match value {
        Some(1) => "bg-emerald-500",
        Some(2) => "bg-blue-500",
        Some(3) => "bg-violet-500",
        Some(4) => "bg-amber-500",
        Some(5) => "bg-pink-500",
        None => "bg-slate-500",
        Some(_) => "bg-slate-500",
    }
}

fn invoice_type_label(value: &str) -> &'static str {
    match value.to_ascii_uppercase().as_str() {
        "COMPANY" | "PERSONAL" | "PERSON" => "NORMAL",
        "SPECIAL" | "2" => "SPECIAL",
        "ELECTRONIC" | "3" => "ELECTRONIC",
        "PAPER" | "4" => "PAPER",
        "NORMAL" | "1" => "NORMAL",
        _ => "",
    }
}

fn login_status(login_result: Option<i64>, risk_level: Option<i64>) -> &'static str {
    match (login_result, risk_level) {
        (Some(1), Some(0..=2)) => "success",
        _ => "warning",
    }
}

fn optional_string_cell(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn string_cell(row: &sqlx::postgres::PgRow, column: &str) -> String {
    optional_string_cell(row, column).unwrap_or_default()
}

fn integer_cell(row: &sqlx::postgres::PgRow, column: &str) -> i64 {
    optional_integer_cell(row, column).unwrap_or(0)
}

fn optional_integer_cell(row: &sqlx::postgres::PgRow, column: &str) -> Option<i64> {
    row.try_get::<Option<i64>, _>(column)
        .ok()
        .flatten()
        .or_else(|| row.try_get::<i64, _>(column).ok())
        .or_else(|| row.try_get::<i32, _>(column).ok().map(i64::from))
        .or_else(|| parse_integer_text(&string_cell(row, column)))
}

fn parse_integer_text(value: &str) -> Option<i64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let digits = value.strip_prefix('-').unwrap_or(value);
    if digits.is_empty() || !digits.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    value.parse::<i64>().ok()
}

fn decimal_cell(row: &sqlx::postgres::PgRow, column: &str) -> f64 {
    row.try_get::<Option<f64>, _>(column)
        .ok()
        .flatten()
        .or_else(|| row.try_get::<f64, _>(column).ok())
        .or_else(|| string_cell(row, column).parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn bool_cell(row: &sqlx::postgres::PgRow, column: &str) -> bool {
    row.try_get::<Option<bool>, _>(column)
        .ok()
        .flatten()
        .or_else(|| row.try_get::<bool, _>(column).ok())
        .or_else(|| optional_integer_cell(row, column).map(|value| value != 0))
        .unwrap_or(false)
}

fn optional_postgres_row_when_read_model_is_missing(
    error: sqlx::Error,
) -> Result<Option<sqlx::postgres::PgRow>, CommerceServiceError> {
    if is_missing_postgres_read_model(&error) {
        Ok(None)
    } else {
        Err(store_error("failed to read account summary", error))
    }
}

fn empty_postgres_rows_when_read_model_is_missing(
    error: sqlx::Error,
) -> Result<Vec<sqlx::postgres::PgRow>, CommerceServiceError> {
    if is_missing_postgres_read_model(&error) {
        Ok(Vec::new())
    } else {
        Err(store_error("failed to read account summary", error))
    }
}

fn is_missing_postgres_read_model(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database_error) => {
            let code = database_error
                .code()
                .map(|code| code.to_string())
                .unwrap_or_default();
            let message = database_error.message().to_ascii_lowercase();
            code == "42P01"
                || code == "42703"
                || message.contains("does not exist")
                || message.contains("undefined table")
                || message.contains("undefined column")
        }
        _ => false,
    }
}

fn store_error(context: &str, error: sqlx::Error) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{context}: {error}"))
}

fn current_timestamp_string() -> String {
    format_unix_timestamp(current_unix_timestamp())
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn format_unix_timestamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use sdkwork_account_service::{
        AccountSummaryQuery, AppendLedgerEntryCommand, WalletAccountListQuery,
        WalletOperationQuery, WalletTransactionDetailQuery, WalletTransactionListQuery,
    };
    use sdkwork_contract_service::CommerceRequestHash;

    use super::PostgresCommerceAccountStore;

    #[test]
    fn postgres_store_api_is_publicly_constructible() {
        let _: fn(sqlx::PgPool) -> PostgresCommerceAccountStore = PostgresCommerceAccountStore::new;
        let _ = PostgresCommerceAccountStore::retrieve_summary;
        let _ = PostgresCommerceAccountStore::list_wallet_accounts;
        let _ = PostgresCommerceAccountStore::retrieve_wallet_overview;
        let _ = PostgresCommerceAccountStore::list_wallet_transactions;
        let _ = PostgresCommerceAccountStore::retrieve_wallet_transaction;
        let _ = PostgresCommerceAccountStore::retrieve_wallet_operation;
        let _ = PostgresCommerceAccountStore::append_ledger_entry;

        let _ = std::mem::size_of::<AccountSummaryQuery>();
        let _ = std::mem::size_of::<WalletAccountListQuery>();
        let _ = std::mem::size_of::<WalletTransactionListQuery>();
        let _ = std::mem::size_of::<WalletTransactionDetailQuery>();
        let _ = std::mem::size_of::<WalletOperationQuery>();
        let _ = std::mem::size_of::<AppendLedgerEntryCommand>();
        let _ = std::mem::size_of::<CommerceRequestHash>();
    }

    #[test]
    fn postgres_account_balance_update_is_version_guarded() {
        let source = include_str!("postgres_account.rs");
        let update_section = source
            .split("UPDATE commerce_account")
            .nth(1)
            .expect("commerce account update section");

        assert!(update_section.contains("AND version = $6"));
        assert!(source.contains("account_update.rows_affected() != 1"));
    }

    #[test]
    fn postgres_account_integer_cells_never_parse_through_f64() {
        let source = include_str!("postgres_account.rs");
        let forbidden = ["parse", "::<", "f64"].join("");
        let integer_section = source
            .split("fn optional_integer_cell")
            .nth(1)
            .expect("integer helper section")
            .split("fn decimal_cell")
            .next()
            .expect("integer helper boundary");

        assert!(!integer_section.contains(&forbidden));
    }
}
