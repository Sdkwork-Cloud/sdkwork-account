use sdkwork_contract_service::{
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
pub struct CreateAccountHoldCommand {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub account_id: String,
    pub asset_type: CommerceAccountAssetType,
    pub amount: CommerceMoney,
    pub business_type: String,
    pub business_no: String,
    pub source_type: String,
    pub source_id: String,
    pub request_no: String,
    pub idempotency_key: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettleAccountHoldCommand {
    pub tenant_id: String,
    pub hold_id: String,
    pub business_type: String,
    pub transaction_no: String,
    pub request_no: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseAccountHoldCommand {
    pub tenant_id: String,
    pub hold_id: String,
    pub request_no: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAccountTransferCommand {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub from_account_id: String,
    pub to_account_id: String,
    pub owner_user_id: String,
    pub asset_type: CommerceAccountAssetType,
    pub amount: CommerceMoney,
    pub business_type: String,
    pub business_no: String,
    pub request_no: String,
    pub idempotency_key: String,
}

impl CreateAccountHoldCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        account_id: &str,
        owner_user_id: &str,
        asset_type: CommerceAccountAssetType,
        amount: CommerceMoney,
        business_type: &str,
        business_no: &str,
        source_type: &str,
        source_id: &str,
        request_no: &str,
        idempotency_key: &str,
        expires_at: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            tenant_id: required_text("tenant_id", tenant_id)?,
            organization_id: optional_text(organization_id),
            account_id: optional_account_id(account_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            asset_type,
            amount,
            business_type: required_text("business_type", business_type)?,
            business_no: required_text("business_no", business_no)?,
            source_type: required_text("source_type", source_type)?,
            source_id: required_text("source_id", source_id)?,
            request_no: required_text("request_no", request_no)?,
            idempotency_key: required_text("idempotency_key", idempotency_key)?,
            expires_at: optional_text(expires_at),
        })
    }
}

impl SettleAccountHoldCommand {
    pub fn new(
        tenant_id: &str,
        hold_id: &str,
        business_type: &str,
        transaction_no: &str,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            tenant_id: required_text("tenant_id", tenant_id)?,
            hold_id: required_text("hold_id", hold_id)?,
            business_type: required_text("business_type", business_type)?,
            transaction_no: required_text("transaction_no", transaction_no)?,
            request_no: required_text("request_no", request_no)?,
            idempotency_key: required_text("idempotency_key", idempotency_key)?,
        })
    }
}

impl ReleaseAccountHoldCommand {
    pub fn new(
        tenant_id: &str,
        hold_id: &str,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            tenant_id: required_text("tenant_id", tenant_id)?,
            hold_id: required_text("hold_id", hold_id)?,
            request_no: required_text("request_no", request_no)?,
            idempotency_key: required_text("idempotency_key", idempotency_key)?,
        })
    }
}

impl CreateAccountTransferCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        from_account_id: &str,
        to_account_id: &str,
        owner_user_id: &str,
        asset_type: CommerceAccountAssetType,
        amount: CommerceMoney,
        business_type: &str,
        business_no: &str,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            tenant_id: required_text("tenant_id", tenant_id)?,
            organization_id: optional_text(organization_id),
            from_account_id: required_text("from_account_id", from_account_id)?,
            to_account_id: required_text("to_account_id", to_account_id)?,
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            asset_type,
            amount,
            business_type: required_text("business_type", business_type)?,
            business_no: required_text("business_no", business_no)?,
            request_no: required_text("request_no", request_no)?,
            idempotency_key: required_text("idempotency_key", idempotency_key)?,
        })
    }
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
            account_id: optional_account_id(account_id),
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

fn optional_account_id(value: &str) -> String {
    value.trim().to_string()
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
