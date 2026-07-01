use sdkwork_account_service::AppendLedgerEntryCommand;
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::sha256_hash;
use serde::Serialize;

pub const OUTBOX_STATUS_PENDING: &str = "PENDING";
pub const OUTBOX_AGGREGATE_TYPE_ACCOUNT: &str = "commerce_account";
pub const OUTBOX_EVENT_TYPE_LEDGER_APPENDED: &str = "account.ledger_appended";
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
    let payload_json = serde_json::to_string(&payload).map_err(|error| {
        CommerceServiceError::storage(format!("failed to serialize outbox payload: {error}"))
    })?;
    let payload_hash = sha256_hash(payload_json.as_bytes());
    Ok((event_key, payload_json, payload_hash))
}
