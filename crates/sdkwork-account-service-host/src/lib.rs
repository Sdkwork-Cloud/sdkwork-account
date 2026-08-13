use sdkwork_account_database_host::{
    bootstrap_account_database_from_env, bootstrap_account_database_host_with_pool,
    AccountDatabaseHost,
};
use sdkwork_database_sqlx::DatabasePool;

pub struct AccountServiceHost {
    database: AccountDatabaseHost,
}

impl AccountServiceHost {
    pub async fn new() -> Self {
        Self::from_env()
            .await
            .expect("account service host bootstrap failed")
    }

    pub async fn from_env() -> Result<Self, String> {
        let database = bootstrap_account_database_from_env().await?;
        Ok(Self { database })
    }

    /// Build the account service host on a shared pool owned by the consuming
    /// host (same-origin dependency composition). Mirrors the membership
    /// `MembershipServiceHost::from_pool` pattern; the consuming host already
    /// owns the database lifecycle for this pool.
    pub async fn from_pool(pool: &DatabasePool) -> Result<Self, String> {
        let database = bootstrap_account_database_host_with_pool(pool).await?;
        Ok(Self { database })
    }

    pub fn database_pool(&self) -> &DatabasePool {
        self.database.pool()
    }

    pub fn database_module(&self) -> std::sync::Arc<sdkwork_database_spi::DefaultDatabaseModule> {
        self.database.module()
    }

    pub async fn close_database_pool(&self) {
        self.database.pool().close().await;
    }
}
