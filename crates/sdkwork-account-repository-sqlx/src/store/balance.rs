use sdkwork_contract_service::CommerceServiceError;

/// Sums three decimal amount strings (available + frozen + pending).
pub fn sum_amount_strings(available: &str, frozen: &str, pending: &str) -> String {
    let sum = [available, frozen, pending]
        .iter()
        .filter_map(|value| value.trim().parse::<i128>().ok())
        .try_fold(0_i128, |acc, value| acc.checked_add(value))
        .unwrap_or(0);
    sum.to_string()
}

/// Validates that active lot remaining equals account available for points accounts.
pub fn validate_points_lot_balance_invariant(
    available_amount: &str,
    lot_remaining_total: i64,
) -> Result<(), CommerceServiceError> {
    let available = available_amount
        .trim()
        .parse::<i64>()
        .map_err(|_| CommerceServiceError::storage("available_amount is not a valid integer"))?;
    if available != lot_remaining_total {
        return Err(CommerceServiceError::storage(format!(
            "points lot invariant violated: available={available} lot_remaining={lot_remaining_total}"
        )));
    }
    Ok(())
}
