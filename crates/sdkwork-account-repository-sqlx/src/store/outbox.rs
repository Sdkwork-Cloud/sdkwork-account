use sdkwork_account_service::AppendLedgerEntryCommand;
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::sha256_hash;
use serde::Serialize;
use sqlx::{Executor, Sqlite};

use super::{next_entity_id, next_entity_uuid, store_error};

pub const OUTBOX_STATUS_PENDING: &str = "PENDING";
pub const OUTBOX_STATUS_PUBLISHED: &str = "PUBLISHED";
pub const OUTBOX_AGGREGATE_TYPE_ACCOUNT: &str = "account";
pub const OUTBOX_EVENT_TYPE_LEDGER_APPENDED: &str = "account.ledger_appended";
pub const OUTBOX_EVENT_TYPE_HOLD_CREATED: &str = "account.hold_created";
pub const OUTBOX_EVENT_TYPE_HOLD_SETTLED: &str = "account.hold_settled";
pub const OUTBOX_EVENT_TYPE_HOLD_RELEASED: &str = "account.hold_released";
pub const OUTBOX_EVENT_TYPE_HOLD_EXPIRED: &str = "account.hold_expired";
pub const OUTBOX_EVENT_TYPE_TRANSFER_COMPLETED: &str = "account.transfer_completed";
pub const OUTBOX_EVENT_TYPE_POINTS_LOTS_EXPIRED: &str = "account.points_lots_expired";
pub const OUTBOX_EVENT_VERSION: i32 = 1;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LedgerAppendedOutboxPayload {
    journal_uuid: String,
    ledger_entry_uuid: String,
    account_uuid: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    asset_type: String,
    direction: String,
    amount: String,
    business_type: String,
    transaction_no: String,
    request_no: String,
    idempotency_key: String,
}

pub fn build_ledger_appended_outbox(
    journal_uuid: &str,
    ledger_entry_uuid: &str,
    account_uuid: &str,
    command: &AppendLedgerEntryCommand,
) -> Result<(String, String, String), CommerceServiceError> {
    let event_key = format!(
        "{}:{}:{}",
        command.tenant_id.trim(),
        command.idempotency_key.trim(),
        OUTBOX_EVENT_TYPE_LEDGER_APPENDED
    );
    let payload = LedgerAppendedOutboxPayload {
        journal_uuid: journal_uuid.to_owned(),
        ledger_entry_uuid: ledger_entry_uuid.to_owned(),
        account_uuid: account_uuid.to_owned(),
        tenant_id: command.tenant_id.trim().to_owned(),
        organization_id: command.organization_id.clone(),
        owner_user_id: command.owner_user_id.trim().to_owned(),
        asset_type: command.asset_type.as_str().to_owned(),
        direction: command.direction.as_str().to_owned(),
        amount: command.amount.as_str().to_owned(),
        business_type: command.business_type.clone(),
        transaction_no: command.transaction_no.clone(),
        request_no: command.request_no.clone(),
        idempotency_key: command.idempotency_key.clone(),
    };
    serialize_outbox_payload(&event_key, &payload)
}

pub fn build_domain_outbox(
    tenant_id: i64,
    event_type: &str,
    idempotency_key: &str,
    payload: &impl Serialize,
) -> Result<(String, String, String), CommerceServiceError> {
    let event_key = format!("{tenant_id}:{idempotency_key}:{event_type}");
    serialize_outbox_payload(&event_key, payload)
}

fn serialize_outbox_payload(
    event_key: &str,
    payload: &impl Serialize,
) -> Result<(String, String, String), CommerceServiceError> {
    let payload_json = serde_json::to_string(payload).map_err(|error| {
        CommerceServiceError::storage(format!("failed to serialize outbox payload: {error}"))
    })?;
    let payload_hash = sha256_hash(payload_json.as_bytes());
    Ok((event_key.to_owned(), payload_json, payload_hash))
}

pub struct OutboxEventInsert<'a> {
    pub aggregate_id: i64,
    pub event_key: &'a str,
    pub event_type: &'a str,
    pub now: &'a str,
    pub payload: &'a str,
    pub payload_hash: &'a str,
    pub tenant_id: i64,
}

pub async fn insert_outbox_event_sqlite<'e, E>(
    executor: E,
    input: OutboxEventInsert<'_>,
) -> Result<(), CommerceServiceError>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query(
        r#"
        INSERT INTO acct_outbox_event
            (id, uuid, tenant_id, aggregate_type, aggregate_id, event_type, event_version,
             event_key, payload, payload_hash, status, retry_count, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(next_entity_uuid())
    .bind(input.tenant_id)
    .bind(OUTBOX_AGGREGATE_TYPE_ACCOUNT)
    .bind(input.aggregate_id)
    .bind(input.event_type)
    .bind(OUTBOX_EVENT_VERSION)
    .bind(input.event_key)
    .bind(input.payload)
    .bind(input.payload_hash)
    .bind(OUTBOX_STATUS_PENDING)
    .bind(input.now)
    .bind(input.now)
    .execute(executor)
    .await
    .map_err(|error| store_error("failed to insert outbox event", error))?;
    Ok(())
}

pub async fn emit_domain_outbox_sqlite(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    tenant_id: i64,
    aggregate_id: i64,
    event_type: &str,
    idempotency_key: &str,
    payload: &impl Serialize,
    now: &str,
) -> Result<(), CommerceServiceError> {
    let (event_key, payload_json, payload_hash) =
        build_domain_outbox(tenant_id, event_type, idempotency_key, payload)?;
    insert_outbox_event_sqlite(
        &mut **tx,
        OutboxEventInsert {
            aggregate_id,
            event_key: &event_key,
            event_type,
            now,
            payload: &payload_json,
            payload_hash: &payload_hash,
            tenant_id,
        },
    )
    .await
}

pub async fn emit_domain_outbox_postgres(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: i64,
    aggregate_id: i64,
    event_type: &str,
    idempotency_key: &str,
    payload: &impl Serialize,
    now: &str,
) -> Result<(), CommerceServiceError> {
    let (event_key, payload_json, payload_hash) =
        build_domain_outbox(tenant_id, event_type, idempotency_key, payload)?;
    insert_outbox_event_postgres(
        &mut **tx,
        OutboxEventInsert {
            aggregate_id,
            event_key: &event_key,
            event_type,
            now,
            payload: &payload_json,
            payload_hash: &payload_hash,
            tenant_id,
        },
    )
    .await
}

pub async fn insert_outbox_event_postgres<'e, E>(
    executor: E,
    input: OutboxEventInsert<'_>,
) -> Result<(), CommerceServiceError>
where
    E: Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
        INSERT INTO acct_outbox_event
            (id, uuid, tenant_id, aggregate_type, aggregate_id, event_type, event_version,
             event_key, payload, payload_hash, status, retry_count, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 0, $12, $13)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(next_entity_uuid())
    .bind(input.tenant_id)
    .bind(OUTBOX_AGGREGATE_TYPE_ACCOUNT)
    .bind(input.aggregate_id)
    .bind(input.event_type)
    .bind(OUTBOX_EVENT_VERSION)
    .bind(input.event_key)
    .bind(input.payload)
    .bind(input.payload_hash)
    .bind(OUTBOX_STATUS_PENDING)
    .bind(input.now)
    .bind(input.now)
    .execute(executor)
    .await
    .map_err(|error| store_error("failed to insert outbox event", error))?;
    Ok(())
}
