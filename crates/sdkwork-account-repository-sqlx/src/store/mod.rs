pub mod outbox;

use chrono::{DateTime, Utc};
use sdkwork_account_service::AppendLedgerEntryCommand;
use sdkwork_contract_service::{CommerceAccountAssetType, CommerceServiceError};
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_utils_rust::{parse_datetime, uuid as new_uuid};
use std::sync::Mutex;

pub const LEDGER_APPEND_SCOPE: &str = "wallet.adjustments.create";
pub const HOLD_CREATE_SCOPE: &str = "wallet.holds.create";
pub const HOLD_SETTLE_SCOPE: &str = "wallet.holds.settle";
pub const HOLD_RELEASE_SCOPE: &str = "wallet.holds.release";
pub const TRANSFER_CREATE_SCOPE: &str = "wallet.transfers.create";
pub const OWNER_TYPE_USER: &str = "USER";
pub const ACCOUNT_STATUS_ACTIVE: i32 = 1;
pub const ACCOUNT_PURPOSE_GENERAL: &str = "GENERAL";
pub const HOLD_STATUS_HELD: i32 = 1;
pub const HOLD_STATUS_SETTLED: i32 = 2;
pub const HOLD_STATUS_RELEASED: i32 = 3;
pub const HOLD_STATUS_EXPIRED: i32 = 4;
pub const TRANSFER_STATUS_COMPLETED: i32 = 2;

pub fn parse_subject_i64(field_name: &str, value: &str) -> Result<i64, CommerceServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CommerceServiceError::validation(format!(
            "{field_name} is required"
        )));
    }
    trimmed.parse::<i64>().map_err(|_| {
        CommerceServiceError::validation(format!("{field_name} must be a valid int64"))
    })
}

pub fn org_id_from_option(value: Option<&str>) -> Result<i64, CommerceServiceError> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => parse_subject_i64("organization_id", value),
        None => Ok(0),
    }
}

pub fn asset_code_from_type(asset_type: &CommerceAccountAssetType) -> &'static str {
    asset_type.as_str()
}

pub fn asset_type_from_code(value: &str) -> Result<CommerceAccountAssetType, CommerceServiceError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "cash" => Ok(CommerceAccountAssetType::Cash),
        "point" | "points" => Ok(CommerceAccountAssetType::Points),
        "token" | "tokens" => Ok(CommerceAccountAssetType::Token),
        _ => Err(CommerceServiceError::validation("asset_code is invalid")),
    }
}

pub fn points_lot_status_label(status: i32) -> &'static str {
    match status {
        1 => "active",
        2 => "depleted",
        3 => "expired",
        _ => "unknown",
    }
}

pub fn default_currency_code(asset_type: &CommerceAccountAssetType) -> &'static str {
    match asset_type {
        CommerceAccountAssetType::Cash => "",
        CommerceAccountAssetType::Points => "POINT",
        CommerceAccountAssetType::Token => "TOKEN",
    }
}

pub fn currency_code_for_command(command: &AppendLedgerEntryCommand) -> String {
    command
        .currency_code
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(default_currency_code(&command.asset_type))
        .to_string()
}

pub fn hold_status_label(status: i32) -> &'static str {
    match status {
        HOLD_STATUS_HELD => "held",
        HOLD_STATUS_SETTLED => "settled",
        HOLD_STATUS_RELEASED => "released",
        HOLD_STATUS_EXPIRED => "expired",
        _ => "unknown",
    }
}

pub fn account_status_label(status: i32) -> &'static str {
    match status {
        1 => "active",
        2 => "frozen",
        3 => "closed",
        _ => "unknown",
    }
}

pub fn store_error(context: &str, error: impl std::fmt::Display) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{context}: {error}"))
}

pub fn parse_wallet_transaction_cursor(
    cursor: Option<&str>,
) -> Result<Option<DateTime<Utc>>, CommerceServiceError> {
    let Some(raw) = cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if let Some(parsed) = parse_datetime(raw, None) {
        return Ok(Some(parsed));
    }
    if let Ok(millis) = raw.parse::<i64>() {
        return DateTime::from_timestamp_millis(millis)
            .ok_or_else(|| {
                CommerceServiceError::validation("cursor must be an RFC3339 timestamp or unix millis")
            })
            .map(Some);
    }
    Err(CommerceServiceError::validation(
        "cursor must be an RFC3339 timestamp or unix millis",
    ))
}

pub struct AccountIdGenerator {
    snowflake: SnowflakeIdGenerator,
}

impl AccountIdGenerator {
    pub fn new() -> Result<Self, CommerceServiceError> {
        SnowflakeIdGenerator::new(0)
            .map(|snowflake| Self { snowflake })
            .map_err(|error| CommerceServiceError::storage(error.to_string()))
    }

    pub fn next_id(&self) -> Result<i64, CommerceServiceError> {
        self.snowflake
            .generate()
            .map_err(|error| CommerceServiceError::storage(error.to_string()))
    }

    pub fn next_uuid(&self) -> String {
        new_uuid()
    }
}

impl Default for AccountIdGenerator {
    fn default() -> Self {
        Self::new().expect("account id generator must initialize")
    }
}

thread_local! {
    static ID_GENERATOR: Mutex<AccountIdGenerator> =
        Mutex::new(AccountIdGenerator::new().expect("account id generator must initialize"));
}

pub fn next_entity_id() -> Result<i64, CommerceServiceError> {
    ID_GENERATOR.with(|generator| generator.lock().expect("id generator lock").next_id())
}

pub fn next_entity_uuid() -> String {
    ID_GENERATOR.with(|generator| generator.lock().expect("id generator lock").next_uuid())
}

pub fn format_i64(value: i64) -> String {
    value.to_string()
}

pub fn optional_org_string(value: i64) -> Option<String> {
    if value == 0 {
        None
    } else {
        Some(value.to_string())
    }
}
