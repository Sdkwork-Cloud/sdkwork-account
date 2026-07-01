mod sqlite_hold;
mod postgres_hold;

pub mod postgres_account;
pub mod postgres_billing;
pub mod sqlite_account;
pub mod sqlite_billing;
pub mod store;

#[cfg(test)]
mod test_sqlite_pool;

pub use postgres_account::PostgresCommerceAccountStore;
pub use postgres_billing::PostgresCommerceBillingHistoryStore;
pub use sqlite_account::SqliteCommerceAccountStore;
pub use sqlite_billing::SqliteCommerceBillingHistoryStore;

use sdkwork_contract_service::{CommerceRequestHash, CommerceServiceError};
use sdkwork_utils_rust::sha256_hash;

pub fn hold_request_hash(body: &str) -> Result<CommerceRequestHash, CommerceServiceError> {
    CommerceRequestHash::new(&sha256_hash(body.as_bytes()))
}
