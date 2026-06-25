use sdkwork_database_sqlx::DatabasePool;
use sdkwork_account_database_host::{bootstrap_account_database_from_env, AccountDatabaseHost};

pub struct AccountServiceHost {
    database: AccountDatabaseHost,
}

impl AccountServiceHost {
    pub async fn new() -> Self {
        Self::from_env().await.expect("account service host bootstrap failed")
    }

    pub async fn from_env() -> Result<Self, String> {
        let database = bootstrap_account_database_from_env().await?;
        Ok(Self { database })
    }

    pub fn database_pool(&self) -> &DatabasePool {
        self.database.pool()
    }

    pub fn database_module(&self) -> std::sync::Arc<sdkwork_database_spi::DefaultDatabaseModule> {
        self.database.module()
    }
}
