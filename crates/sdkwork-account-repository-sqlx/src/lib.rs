mod sqlite_hold;
mod postgres_points_ops;
mod sqlite_points_ops;
mod postgres_hold;

pub mod postgres_account;
pub mod postgres_billing;
pub mod sqlite_account;
pub mod sqlite_billing;
pub mod store;

#[cfg(any(test, feature = "test-support"))]
mod test_sqlite_pool;

#[cfg(any(test, feature = "test-support"))]
pub use test_sqlite_pool::account_migrated_sqlite_memory_pool;

pub use postgres_account::PostgresCommerceAccountStore;
pub use postgres_billing::PostgresCommerceBillingHistoryStore;
pub use sqlite_account::SqliteCommerceAccountStore;
pub use sqlite_billing::SqliteCommerceBillingHistoryStore;

use sdkwork_contract_service::{CommerceRequestHash, CommerceServiceError};
use sdkwork_utils_rust::sha256_hash;

pub fn hold_request_hash(body: &str) -> Result<CommerceRequestHash, CommerceServiceError> {
    CommerceRequestHash::new(&sha256_hash(body.as_bytes()))
}
