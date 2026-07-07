use sdkwork_contract_service::CommerceServiceError;
use sdkwork_database_id::{NodeAllocatorConfig, NodeLease, SnowflakeIdGenerator, SnowflakeNodeAllocator};
use sdkwork_database_sqlx::DatabasePool;

use crate::store::{AccountIdGenerator, init_account_id_generator};

const ACCOUNT_SERVICE_NAME: &str = "account-service";

/// Allocates a snowflake generator from the database node registry when possible,
/// then installs it for repository writes. Keeps the returned [`NodeLease`] alive
/// for the process lifetime.
pub async fn bootstrap_and_install_account_id_generator(
    pool: &DatabasePool,
) -> Result<Option<NodeLease>, CommerceServiceError> {
    let config = NodeAllocatorConfig::from_service_name(ACCOUNT_SERVICE_NAME);
    match SnowflakeNodeAllocator::allocate_generator(pool, &config).await {
        Ok((snowflake, lease)) => {
            tracing::info!(
                target = "account.id",
                node_id = lease.node_id(),
                "allocated snowflake node id from database registry"
            );
            init_account_id_generator(AccountIdGenerator::from_snowflake(snowflake));
            Ok(Some(lease))
        }
        Err(error) => {
            tracing::warn!(
                target = "account.id",
                error = %error,
                "database snowflake allocation failed; falling back to ACCOUNT_SNOWFLAKE_WORKER_ID"
            );
            init_account_id_generator(AccountIdGenerator::new()?);
            Ok(None)
        }
    }
}

impl AccountIdGenerator {
    pub fn from_snowflake(snowflake: SnowflakeIdGenerator) -> Self {
        Self { snowflake }
    }
}
