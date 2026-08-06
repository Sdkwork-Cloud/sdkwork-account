use chrono::Utc;
use sdkwork_account_service::{OutboxDispatchItem, OutboxDispatchOutcome};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::MAX_LIST_PAGE_SIZE;
use sqlx::{PgPool, Row};

use super::outbox::{OUTBOX_STATUS_PENDING, OUTBOX_STATUS_PUBLISHED};
use super::{format_i64, store_error, OUTBOX_DISPATCH_BATCH_DEFAULT};

pub fn resolve_outbox_dispatch_batch_size(batch_size: Option<i64>) -> i64 {
    batch_size
        .unwrap_or(OUTBOX_DISPATCH_BATCH_DEFAULT)
        .clamp(1, i64::from(MAX_LIST_PAGE_SIZE))
}

pub async fn count_pending_outbox_postgres(pool: &PgPool) -> Result<i64, CommerceServiceError> {
    let now = Utc::now();
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM acct_outbox_event
        WHERE status = $1
          AND (next_retry_at IS NULL OR next_retry_at <= $2)
        "#,
    )
    .bind(OUTBOX_STATUS_PENDING)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|error| store_error("failed to count pending outbox events", error))
}

pub async fn dispatch_pending_outbox_postgres(
    pool: &PgPool,
    batch_size: Option<i64>,
) -> Result<OutboxDispatchOutcome, CommerceServiceError> {
    let limit = resolve_outbox_dispatch_batch_size(batch_size);
    let now = Utc::now();

    let rows = sqlx::query(
        r#"
        WITH candidates AS (
            SELECT id
            FROM acct_outbox_event
            WHERE status = $1
              AND (next_retry_at IS NULL OR next_retry_at <= $2)
            ORDER BY created_at ASC
            LIMIT $3
            FOR UPDATE SKIP LOCKED
        )
        UPDATE acct_outbox_event AS e
        SET status = $4,
            published_at = $2,
            updated_at = $2
        FROM candidates AS c
        WHERE e.id = c.id
        RETURNING e.id, e.uuid, e.tenant_id, e.aggregate_type, e.aggregate_id,
                  e.event_type, e.event_version, e.event_key, e.payload::text,
                  CAST(e.created_at AS TEXT) AS created_at
        "#,
    )
    .bind(OUTBOX_STATUS_PENDING)
    .bind(now)
    .bind(limit)
    .bind(OUTBOX_STATUS_PUBLISHED)
    .fetch_all(pool)
    .await
    .map_err(|error| store_error("failed to dispatch outbox batch", error))?;

    let items = rows
        .iter()
        .map(map_postgres_outbox_dispatch_row)
        .collect::<Result<Vec<_>, _>>()?;
    let pending_lag = count_pending_outbox_postgres(pool).await?;

    Ok(OutboxDispatchOutcome {
        dispatched_count: items.len() as i64,
        pending_lag,
        items,
    })
}

fn map_postgres_outbox_dispatch_row(
    row: &sqlx::postgres::PgRow,
) -> Result<OutboxDispatchItem, CommerceServiceError> {
    Ok(OutboxDispatchItem {
        id: format_i64(row.try_get::<i64, _>("id").unwrap_or_default()),
        uuid: row.try_get::<String, _>("uuid").unwrap_or_default(),
        tenant_id: format_i64(row.try_get::<i64, _>("tenant_id").unwrap_or_default()),
        aggregate_type: row
            .try_get::<String, _>("aggregate_type")
            .unwrap_or_default(),
        aggregate_id: format_i64(row.try_get::<i64, _>("aggregate_id").unwrap_or_default()),
        event_type: row.try_get::<String, _>("event_type").unwrap_or_default(),
        event_version: row.try_get::<i32, _>("event_version").unwrap_or_default(),
        event_key: row.try_get::<String, _>("event_key").unwrap_or_default(),
        payload: row.try_get::<String, _>("payload").unwrap_or_default(),
        created_at: row.try_get::<String, _>("created_at").unwrap_or_default(),
    })
}
