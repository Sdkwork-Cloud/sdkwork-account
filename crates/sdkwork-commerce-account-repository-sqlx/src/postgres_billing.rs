use sdkwork_commerce_account_service::{BillingHistoryItem, BillingHistoryListQuery};
use sdkwork_commerce_contract_service::CommerceServiceError;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct PostgresCommerceBillingHistoryStore {
    pool: PgPool,
}

impl PostgresCommerceBillingHistoryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_billing_history(
        &self,
        query: BillingHistoryListQuery,
    ) -> Result<Vec<BillingHistoryItem>, CommerceServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, tenant_id, organization_id, owner_user_id, history_no, history_type,
                   direction, asset_type, CAST(amount AS TEXT) AS amount, currency_code,
                   CAST(points_delta AS BIGINT) AS points_delta, status, title, reference_no,
                   source_type, source_id, related_order_id, related_order_no, payment_method,
                   occurred_at
            FROM commerce_billing_history
            WHERE tenant_id = CAST($1 AS TEXT)
              AND ((organization_id = CAST($2 AS TEXT)) OR (organization_id IS NULL AND $2 IS NULL))
              AND owner_user_id = CAST($3 AS TEXT)
              AND ($4 IS NULL OR history_type = $4)
              AND ($5 IS NULL OR status = $5)
            ORDER BY occurred_at DESC, id DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(query.history_type.as_deref())
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
    row: &sqlx::postgres::PgRow,
) -> Result<BillingHistoryItem, CommerceServiceError> {
    BillingHistoryItem::new(
        &string_cell(row, "id"),
        &string_cell(row, "tenant_id"),
        optional_string_cell(row, "organization_id").as_deref(),
        &string_cell(row, "owner_user_id"),
        &string_cell(row, "history_no"),
        &string_cell(row, "history_type"),
        &string_cell(row, "direction"),
        &string_cell(row, "asset_type"),
        &string_cell(row, "amount"),
        optional_string_cell(row, "currency_code").as_deref(),
        integer_cell(row, "points_delta"),
        &string_cell(row, "status"),
        &string_cell(row, "title"),
        optional_string_cell(row, "reference_no").as_deref(),
        &string_cell(row, "source_type"),
        &string_cell(row, "source_id"),
        optional_string_cell(row, "related_order_id").as_deref(),
        optional_string_cell(row, "related_order_no").as_deref(),
        optional_string_cell(row, "payment_method").as_deref(),
        &string_cell(row, "occurred_at"),
    )
}

fn string_cell(row: &sqlx::postgres::PgRow, name: &str) -> String {
    row.try_get::<String, _>(name).unwrap_or_default()
}

fn optional_string_cell(row: &sqlx::postgres::PgRow, name: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(name)
        .ok()
        .flatten()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn integer_cell(row: &sqlx::postgres::PgRow, name: &str) -> i64 {
    row.try_get::<i64, _>(name).unwrap_or_default()
}

fn store_error(context: &str, error: sqlx::Error) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{context}: {error}"))
}
