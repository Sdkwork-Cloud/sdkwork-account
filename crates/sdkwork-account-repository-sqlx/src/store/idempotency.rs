use chrono::{DateTime, Utc};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::parse_datetime;

use super::store_error;

/// Default in-flight idempotency lock TTL per `API_SPEC.md` §15.
pub const IDEMPOTENCY_LOCK_TTL_SECS: i64 = 300;

/// Action to take after loading an existing idempotency row with a matching request hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdempotencyRecordAction {
    /// Return the stored outcome without re-executing the mutation.
    Replay,
    /// Re-acquire the lock after a prior `FAILED` or expired `LOCKED` attempt and execute again.
    ReclaimLock,
}

pub fn idempotency_lock_expires_at(now: DateTime<Utc>) -> DateTime<Utc> {
    now + chrono::Duration::seconds(IDEMPOTENCY_LOCK_TTL_SECS)
}

pub fn idempotency_lock_expires_at_rfc3339(now: DateTime<Utc>) -> String {
    idempotency_lock_expires_at(now).to_rfc3339()
}

fn parse_locked_until(value: Option<&str>) -> Option<DateTime<Utc>> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    parse_datetime(raw, None)
}

pub fn is_idempotency_lock_expired(locked_until: Option<&str>, now: DateTime<Utc>) -> bool {
    match parse_locked_until(locked_until) {
        Some(expires_at) => expires_at <= now,
        None => true,
    }
}

pub fn resolve_idempotency_from_row_fields(
    request_hash: &str,
    stored_hash: &str,
    status: &str,
    locked_until: &str,
    now: DateTime<Utc>,
) -> Result<IdempotencyRecordAction, CommerceServiceError> {
    resolve_idempotency_record_action(
        stored_hash,
        status,
        request_hash,
        (!locked_until.trim().is_empty()).then_some(locked_until.trim()),
        now,
    )
}

/// Evaluates an idempotency row against the incoming request hash.
///
/// `LOCKED` rows return [`CommerceServiceError::locked`] while the lock is active.
/// Expired locks are reclaimed per `API_SPEC.md` §15.
pub fn resolve_idempotency_record_action(
    stored_hash: &str,
    status: &str,
    request_hash: &str,
    locked_until: Option<&str>,
    now: DateTime<Utc>,
) -> Result<IdempotencyRecordAction, CommerceServiceError> {
    if stored_hash != request_hash {
        return Err(CommerceServiceError::conflict(
            "idempotency key was used with a different request hash",
        ));
    }

    match status.trim().to_ascii_uppercase().as_str() {
        "COMPLETED" => Ok(IdempotencyRecordAction::Replay),
        "LOCKED" => {
            if is_idempotency_lock_expired(locked_until, now) {
                Ok(IdempotencyRecordAction::ReclaimLock)
            } else {
                Err(CommerceServiceError::locked(
                    "idempotency request in progress; retry with the same idempotency key",
                ))
            }
        }
        "FAILED" => Ok(IdempotencyRecordAction::ReclaimLock),
        other => Err(CommerceServiceError::storage(format!(
            "idempotency record has unsupported status: {other}"
        ))),
    }
}

pub fn map_idempotency_insert_error(context: &str, error: sqlx::Error) -> CommerceServiceError {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.is_unique_violation() {
            return CommerceServiceError::locked(
                "idempotency request in progress; retry with the same idempotency key",
            );
        }
    }
    store_error(context, error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_status_blocks_reexecution() {
        let now = Utc::now();
        let locked_until = idempotency_lock_expires_at_rfc3339(now);
        let error = resolve_idempotency_record_action(
            "hash-a",
            "LOCKED",
            "hash-a",
            Some(&locked_until),
            now,
        )
        .expect_err("locked must error");
        assert_eq!(error.code(), "locked");
    }

    #[test]
    fn expired_locked_status_reclaims() {
        let now = Utc::now();
        let locked_until = (now - chrono::Duration::seconds(1)).to_rfc3339();
        assert_eq!(
            resolve_idempotency_record_action(
                "hash-a",
                "LOCKED",
                "hash-a",
                Some(&locked_until),
                now
            )
            .expect("expired lock reclaims"),
            IdempotencyRecordAction::ReclaimLock
        );
    }

    #[test]
    fn completed_status_replays() {
        let now = Utc::now();
        assert_eq!(
            resolve_idempotency_record_action("hash-a", "COMPLETED", "hash-a", None, now)
                .expect("replay"),
            IdempotencyRecordAction::Replay
        );
    }

    #[test]
    fn failed_status_reclaims() {
        let now = Utc::now();
        assert_eq!(
            resolve_idempotency_record_action("hash-a", "FAILED", "hash-a", None, now)
                .expect("reclaim"),
            IdempotencyRecordAction::ReclaimLock
        );
    }

    #[test]
    fn hash_mismatch_conflicts() {
        let now = Utc::now();
        let error = resolve_idempotency_record_action("hash-a", "COMPLETED", "hash-b", None, now)
            .expect_err("conflict");
        assert_eq!(error.code(), "conflict");
    }
}
