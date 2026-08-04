use sdkwork_contract_service::CommerceServiceError;
use sdkwork_database_id::{
    NodeAllocatorConfig, NodeLease, SnowflakeIdGenerator, SnowflakeNodeAllocator,
};
use sdkwork_database_sqlx::DatabasePool;

use crate::store::{init_account_id_generator, AccountIdGenerator};

const ACCOUNT_SERVICE_NAME: &str = "account-service";

/// Allocates a snowflake generator from the database node registry when possible,
/// then installs it for repository writes. Keeps the returned [`NodeLease`] alive
/// for the process lifetime.
pub async fn bootstrap_and_install_account_id_generator(
    pool: &DatabasePool,
) -> Result<Option<NodeLease>, CommerceServiceError> {
    let config = NodeAllocatorConfig::from_service_name(ACCOUNT_SERVICE_NAME);
    match SnowflakeNodeAllocator::allocate_process_generator(pool, &config).await {
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
            if account_is_production_like() {
                return Err(CommerceServiceError::provider_unavailable(format!(
                    "account Snowflake database node allocation failed in production-like environment: {error}"
                )));
            }
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

fn account_is_production_like() -> bool {
    let lifecycle = ["SDKWORK_ACCOUNT_ENVIRONMENT", "SDKWORK_CLOUDROUTER_ENVIRONMENT"]
        .into_iter()
        .find_map(|key| {
            std::env::var(key)
                .ok()
                .map(|value| value.trim().to_ascii_lowercase())
        });
    let deployment_is_explicit = [
        "SDKWORK_ACCOUNT_DEPLOYMENT_PROFILE",
        "SDKWORK_CLOUDROUTER_DEPLOYMENT_PROFILE",
        "SDKWORK_ACCOUNT_RUNTIME_TARGET",
        "SDKWORK_CLOUDROUTER_RUNTIME_TARGET",
    ]
    .into_iter()
    .any(|key| std::env::var(key).is_ok());
    let explicit_override = std::env::var("SDKWORK_ACCOUNT_ALLOW_UNSAFE_ID_FALLBACK")
        .ok()
        .is_some_and(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    account_fallback_is_forbidden(
        lifecycle.as_deref(),
        deployment_is_explicit,
        explicit_override,
        cfg!(debug_assertions),
    )
}

fn account_fallback_is_forbidden(
    lifecycle: Option<&str>,
    deployment_is_explicit: bool,
    explicit_override: bool,
    debug_build: bool,
) -> bool {
    if let Some(value) = lifecycle {
        return !matches!(value, "development" | "dev" | "test");
    }
    if deployment_is_explicit {
        return true;
    }
    !explicit_override && !debug_build
}

impl AccountIdGenerator {
    pub fn from_snowflake(snowflake: SnowflakeIdGenerator) -> Self {
        Self { snowflake }
    }
}

#[cfg(test)]
mod tests {
    use super::account_fallback_is_forbidden;

    #[test]
    fn fallback_policy_cannot_override_production_or_explicit_deployment() {
        assert!(account_fallback_is_forbidden(
            Some("production"),
            false,
            true,
            true
        ));
        assert!(account_fallback_is_forbidden(None, true, true, true));
        assert!(account_fallback_is_forbidden(
            Some("unknown"),
            false,
            false,
            true
        ));
        assert!(!account_fallback_is_forbidden(
            Some("development"),
            true,
            false,
            false
        ));
        assert!(account_fallback_is_forbidden(None, false, false, false));
    }
}
