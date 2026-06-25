use sdkwork_commerce_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney, CommerceServiceError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppendLedgerEntryCommand {
    pub account_id: String,
    pub amount: CommerceMoney,
    pub asset_type: CommerceAccountAssetType,
    pub business_type: String,
    pub currency_code: Option<String>,
    pub direction: CommerceLedgerDirection,
    pub idempotency_key: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub request_no: String,
    pub tenant_id: String,
    pub transaction_no: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatePreholdCommand {
    pub account_id: String,
    pub amount: CommerceMoney,
    pub idempotency_key: String,
    pub owner_user_id: String,
    pub request_no: String,
    pub tenant_id: String,
}

impl AppendLedgerEntryCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        account_id: &str,
        owner_user_id: &str,
        asset_type: CommerceAccountAssetType,
        currency_code: Option<&str>,
        direction: CommerceLedgerDirection,
        amount: CommerceMoney,
        business_type: &str,
        transaction_no: &str,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            account_id: required_text("account_id", account_id)?,
            amount,
            asset_type,
            business_type: required_text("business_type", business_type)?,
            currency_code: optional_text(currency_code),
            direction,
            idempotency_key: required_text("idempotency_key", idempotency_key)?,
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            request_no: required_text("request_no", request_no)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
            transaction_no: required_text("transaction_no", transaction_no)?,
        })
    }
}

fn required_text(field_name: &str, value: &str) -> Result<String, CommerceServiceError> {
    crate::validation::require_non_empty(field_name, value)?;
    Ok(value.trim().to_string())
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
