use sdkwork_account_service::{BillingHistoryItem, BillingHistoryListQuery};
use sdkwork_contract_service::CommerceServiceError;
use sqlx::{SqlitePool, Row};

use crate::store::{format_i64, optional_org_string, org_id_from_option, parse_subject_i64, store_error};

#[derive(Debug, Clone)]
pub struct SqliteCommerceBillingHistoryStore {
    pool: SqlitePool,
}

impl SqliteCommerceBillingHistoryStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_billing_history(
        &self,
        query: BillingHistoryListQuery,
    ) -> Result<Vec<BillingHistoryItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;

        let rows = sqlx::query(
            r#"
            SELECT id, tenant_id, organization_id, owner_id, history_no, history_type,
                   direction, asset_code, amount, currency_code, points_delta, status, title,
                   reference_no, source_type, source_id, related_order_id, related_order_no,
                   payment_method, occurred_at
            FROM commerce_billing_history
            WHERE tenant_id = ?
              AND organization_id = ?
              AND owner_id = ?
              AND (? IS NULL OR history_type = ?)
              AND (? IS NULL OR status = ?)
            ORDER BY occurred_at DESC, id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(owner_id)
        .bind(query.history_type.as_deref())
        .bind(query.history_type.as_deref())
        .bind(query.status.as_deref())
        .bind(query.status.as_deref())
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to list billing history", error))?;

        rows.iter().map(map_billing_history_item).collect()
    }
}

fn map_billing_history_item(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<BillingHistoryItem, CommerceServiceError> {
    BillingHistoryItem::new(
        &format_i64(row.try_get::<i64, _>("id").unwrap_or_default()),
        &format_i64(row.try_get::<i64, _>("tenant_id").unwrap_or_default()),
        optional_org_string(row.try_get::<i64, _>("organization_id").unwrap_or_default()).as_deref(),
        &format_i64(row.try_get::<i64, _>("owner_id").unwrap_or_default()),
        &string_cell(row, "history_no"),
        &string_cell(row, "history_type"),
        &string_cell(row, "direction"),
        &string_cell(row, "asset_code"),
        &string_cell(row, "amount"),
        optional_string_cell(row, "currency_code").as_deref(),
        row.try_get::<i64, _>("points_delta").unwrap_or_default(),
        &format_i64(row.try_get::<i64, _>("status").unwrap_or_default()),
        &string_cell(row, "title"),
        optional_string_cell(row, "reference_no").as_deref(),
        &string_cell(row, "source_type"),
        &format_i64(row.try_get::<i64, _>("source_id").unwrap_or_default()),
        row.try_get::<Option<i64>, _>("related_order_id")
            .ok()
            .flatten()
            .map(format_i64)
            .as_deref(),
        optional_string_cell(row, "related_order_no").as_deref(),
        optional_string_cell(row, "payment_method").as_deref(),
        &string_cell(row, "occurred_at"),
    )
}

fn string_cell(row: &sqlx::sqlite::SqliteRow, name: &str) -> String {
    row.try_get::<String, _>(name).unwrap_or_default()
}

fn optional_string_cell(row: &sqlx::sqlite::SqliteRow, name: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(name).ok().flatten()
}
