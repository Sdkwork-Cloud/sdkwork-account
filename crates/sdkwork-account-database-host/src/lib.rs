use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_lifecycle::{lifecycle_options_from_env, LifecycleOrchestrator};
use sdkwork_database_spi::{DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool};
use std::path::PathBuf;
use std::sync::Arc;

pub struct AccountDatabaseHost {
    pool: DatabasePool,
    module: Arc<DefaultDatabaseModule>,
}

impl AccountDatabaseHost {
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn module(&self) -> Arc<DefaultDatabaseModule> {
        self.module.clone()
    }
}

pub async fn bootstrap_account_database_from_env() -> Result<AccountDatabaseHost, String> {
    let _ = dotenvy::dotenv();
    let config = DatabaseConfig::from_env("ACCOUNT")
        .map_err(|error| format!("read account database config failed: {error}"))?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create account database pool failed: {error}"))?;
    bootstrap_account_database_host_with_pool(&pool).await
}

/// Bootstrap the account database schema and migrations using an externally
/// provided pool.
///
/// This is used when account is integrated as a federated capability inside a
/// host application (e.g. sdkwork-cloudrouter) that already owns a shared
/// database pool. The function loads the account database module from the
/// account repository's `database/` assets, runs the DDL baseline, and
/// optionally applies migrations — all controlled by the same manifest/env
/// options as the standalone bootstrap (mirrors
/// `bootstrap_membership_database_host_with_pool`).
pub async fn bootstrap_account_database_host_with_pool(
    pool: &DatabasePool,
) -> Result<AccountDatabaseHost, String> {
    if pool.as_postgres().is_none() {
        return Err(
            "account authoritative-server assembly requires a shared PostgreSQL pool".to_owned(),
        );
    }
    let app_root = std::env::var("SDKWORK_ACCOUNT_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let module = Arc::new(
        DefaultDatabaseModule::from_app_root(&app_root)
            .map_err(|error| format!("load account database module failed: {error}"))?,
    );
    let manifest = DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read account database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("ACCOUNT", &manifest);
    let orchestrator =
        LifecycleOrchestrator::new(pool.clone(), module.clone()).with_applied_by("sdkwork-account");
    orchestrator.init().await.map_err(|e| format!("{e}"))?;
    if options.auto_migrate {
        orchestrator.migrate().await.map_err(|e| format!("{e}"))?;
    }
    Ok(AccountDatabaseHost {
        pool: pool.clone(),
        module,
    })
}
