use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::parse_datetime;

/// Ensures the loaded account belongs to the command owner and organization scope.
pub fn validate_account_owner_scope(
    account_organization_id: i64,
    account_owner_id: i64,
    organization_id: i64,
    owner_id: i64,
) -> Result<(), CommerceServiceError> {
    if account_organization_id != organization_id {
        return Err(CommerceServiceError::validation(
            "account organization_id does not match command scope",
        ));
    }
    if account_owner_id != owner_id {
        return Err(CommerceServiceError::validation(
            "account owner_id does not match command owner_user_id",
        ));
    }
    Ok(())
}

/// Rejects zero or negative ledger/hold/transfer amounts.
pub fn require_positive_amount(amount: i128, field_name: &str) -> Result<(), CommerceServiceError> {
    if amount <= 0 {
        return Err(CommerceServiceError::validation(format!(
            "{field_name} must be greater than zero"
        )));
    }
    Ok(())
}

/// Rejects hold settle/release when the hold has passed its expiry timestamp.
pub fn ensure_hold_not_expired(
    expires_at: Option<&str>,
    now_rfc3339: &str,
) -> Result<(), CommerceServiceError> {
    let Some(expires_at) = expires_at.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    let Some(expiry) = parse_datetime(expires_at, None) else {
        return Err(CommerceServiceError::validation(
            "hold expires_at must be a valid RFC3339 timestamp",
        ));
    };
    let Some(now) = parse_datetime(now_rfc3339, None) else {
        return Err(CommerceServiceError::storage(
            "hold expiry check could not parse current timestamp",
        ));
    };
    if expiry <= now {
        return Err(CommerceServiceError::invalid_state(
            "hold has expired and cannot be settled or released",
        ));
    }
    Ok(())
}

/// Ensures transfer operates on same organization and the command owner controls the source account.
/// Cross-user transfers are allowed when `to_account` belongs to a different owner in the same org.
pub fn validate_transfer_account_pair(
    from_organization_id: i64,
    from_owner_id: i64,
    to_organization_id: i64,
    command_owner_id: i64,
) -> Result<(), CommerceServiceError> {
    if from_organization_id != to_organization_id {
        return Err(CommerceServiceError::validation(
            "transfer accounts must share the same organization_id",
        ));
    }
    if from_owner_id != command_owner_id {
        return Err(CommerceServiceError::validation(
            "from_account owner_id does not match command owner_user_id",
        ));
    }
    Ok(())
}

/// After FIFO lot consumption, remaining debit amount must be zero.
pub fn ensure_points_lot_debit_complete(remaining: i64) -> Result<(), CommerceServiceError> {
    if remaining > 0 {
        return Err(CommerceServiceError::invalid_state(
            "insufficient points lots to satisfy debit amount",
        ));
    }
    Ok(())
}
