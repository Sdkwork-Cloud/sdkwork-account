use sdkwork_contract_service::CommerceServiceError;

/// Action to take after loading an existing idempotency row with a matching request hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdempotencyRecordAction {
    /// Return the stored outcome without re-executing the mutation.
    Replay,
    /// Re-acquire the lock after a prior `FAILED` attempt and execute again.
    ReclaimLock,
}

/// Evaluates an idempotency row against the incoming request hash.
///
/// `LOCKED` rows return [`CommerceServiceError::locked`] so callers never double-apply
/// in-flight mutations. Hash mismatches remain hard conflicts per `API_SPEC.md` §15.
pub fn resolve_idempotency_record_action(
    stored_hash: &str,
    status: &str,
    request_hash: &str,
) -> Result<IdempotencyRecordAction, CommerceServiceError> {
    if stored_hash != request_hash {
        return Err(CommerceServiceError::conflict(
            "idempotency key was used with a different request hash",
        ));
    }

    match status.trim().to_ascii_uppercase().as_str() {
        "COMPLETED" => Ok(IdempotencyRecordAction::Replay),
        "LOCKED" => Err(CommerceServiceError::locked(
            "idempotency request in progress; retry with the same idempotency key",
        )),
        "FAILED" => Ok(IdempotencyRecordAction::ReclaimLock),
        other => Err(CommerceServiceError::storage(format!(
            "idempotency record has unsupported status: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_status_blocks_reexecution() {
        let error = resolve_idempotency_record_action("hash-a", "LOCKED", "hash-a")
            .expect_err("locked must error");
        assert_eq!(error.code(), "locked");
    }

    #[test]
    fn completed_status_replays() {
        assert_eq!(
            resolve_idempotency_record_action("hash-a", "COMPLETED", "hash-a").expect("replay"),
            IdempotencyRecordAction::Replay
        );
    }

    #[test]
    fn failed_status_reclaims() {
        assert_eq!(
            resolve_idempotency_record_action("hash-a", "FAILED", "hash-a").expect("reclaim"),
            IdempotencyRecordAction::ReclaimLock
        );
    }

    #[test]
    fn hash_mismatch_conflicts() {
        let error = resolve_idempotency_record_action("hash-a", "COMPLETED", "hash-b")
            .expect_err("conflict");
        assert_eq!(error.code(), "conflict");
    }
}
