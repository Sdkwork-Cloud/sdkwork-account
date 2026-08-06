pub mod postgres_account;
pub mod postgres_billing;
mod postgres_hold;
mod postgres_points_ops;
pub mod store;

pub use postgres_account::PostgresCommerceAccountStore;
pub use postgres_billing::PostgresCommerceBillingHistoryStore;

use sdkwork_contract_service::{CommerceRequestHash, CommerceServiceError};
use sdkwork_utils_rust::sha256_hash;

pub fn hold_request_hash(body: &str) -> Result<CommerceRequestHash, CommerceServiceError> {
    CommerceRequestHash::new(&sha256_hash(body.as_bytes()))
}
