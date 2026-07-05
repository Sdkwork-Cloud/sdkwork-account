use sdkwork_contract_service::CommerceServiceError;

pub fn require_non_empty(field: &str, value: &str) -> Result<(), CommerceServiceError> {
    if value.trim().is_empty() {
        return Err(CommerceServiceError::validation(format!(
            "{field} is required"
        )));
    }

    Ok(())
}

pub fn validate_ledger_business_type(value: &str) -> Result<(), CommerceServiceError> {
    sdkwork_contract_service::CommerceLedgerBusinessType::validate(value)
}
