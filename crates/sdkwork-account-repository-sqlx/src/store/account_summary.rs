use sdkwork_account_service::{AccountConsumptionItem, AccountSummarySnapshot};
use sdkwork_contract_service::CommerceServiceError;
use sqlx::Row;

use crate::store::{optional_org_string, store_error};

pub struct WalletSummaryStats {
    pub monthly_points_consumed: i64,
    pub consumption_by_service: Vec<AccountConsumptionItem>,
    pub est_days_remaining: i64,
}

pub fn build_account_summary_snapshot(
    owner_user_id: &str,
    organization_id: i64,
    available_points: i128,
    stats: WalletSummaryStats,
) -> AccountSummarySnapshot {
    AccountSummarySnapshot {
        id: owner_user_id.to_owned(),
        name: String::new(),
        email: String::new(),
        is_verified: false,
        tier: String::new(),
        organization: optional_org_string(organization_id).unwrap_or_default(),
        available_points: available_points.to_string(),
        est_days_remaining: stats.est_days_remaining,
        monthly_points_consumed: stats.monthly_points_consumed.to_string(),
        consumption_by_service: stats.consumption_by_service,
    }
}

pub fn consumption_items_from_rows(
    rows: &[(String, i64)],
    monthly_total: i64,
) -> Vec<AccountConsumptionItem> {
    rows.iter()
        .map(|(name, value)| {
            let percentage = if monthly_total > 0 {
                (*value as f64 / monthly_total as f64) * 100.0
            } else {
                0.0
            };
            AccountConsumptionItem {
                name: name.clone(),
                points_consumed: value.to_string(),
                color: String::new(),
                percentage,
            }
        })
        .collect()
}

pub fn estimate_days_remaining(available_points: i128, monthly_points_consumed: i64) -> i64 {
    if monthly_points_consumed <= 0 || available_points <= 0 {
        return 0;
    }
    let daily = monthly_points_consumed as f64 / 30.0;
    if daily <= 0.0 {
        return 0;
    }
    (available_points as f64 / daily).floor() as i64
}

pub async fn load_wallet_summary_stats_postgres(
    pool: &sqlx::PgPool,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
) -> Result<WalletSummaryStats, CommerceServiceError> {
    let monthly_total: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(ABS(points_delta)), 0)
        FROM acct_billing_history
        WHERE tenant_id = $1
          AND organization_id = $2
          AND owner_id = $3
          AND direction = 'debit'
          AND occurred_at >= date_trunc('month', NOW())
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(owner_id)
    .fetch_one(pool)
    .await
    .map_err(|error| store_error("failed to load monthly consumption total", error))?;

    let breakdown_rows = sqlx::query(
        r#"
        SELECT history_type, COALESCE(SUM(ABS(points_delta)), 0) AS total
        FROM acct_billing_history
        WHERE tenant_id = $1
          AND organization_id = $2
          AND owner_id = $3
          AND direction = 'debit'
          AND occurred_at >= date_trunc('month', NOW())
        GROUP BY history_type
        ORDER BY total DESC
        LIMIT 20
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(owner_id)
    .fetch_all(pool)
    .await
    .map_err(|error| store_error("failed to load monthly consumption breakdown", error))?;

    let breakdown: Vec<(String, i64)> = breakdown_rows
        .iter()
        .map(|row| {
            (
                row.try_get::<String, _>("history_type")
                    .unwrap_or_else(|_| "unknown".to_owned()),
                row.try_get::<i64, _>("total").unwrap_or(0),
            )
        })
        .collect();

    Ok(WalletSummaryStats {
        monthly_points_consumed: monthly_total,
        consumption_by_service: consumption_items_from_rows(&breakdown, monthly_total),
        est_days_remaining: 0,
    })
}

pub async fn sum_spendable_points_lots_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: i64,
    account_id: i64,
    now: &str,
) -> Result<i64, CommerceServiceError> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(remaining_amount), 0)
        FROM acct_points_lot
        WHERE tenant_id = $1
          AND account_id = $2
          AND status = 1
          AND remaining_amount > 0
          AND (expires_at IS NULL OR expires_at > $3::timestamptz)
        "#,
    )
    .bind(tenant_id)
    .bind(account_id)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| store_error("failed to sum spendable points lots", error))
}
