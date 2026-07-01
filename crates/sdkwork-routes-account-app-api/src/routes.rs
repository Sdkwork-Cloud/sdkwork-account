use axum::Router;
use sdkwork_account_service_host::AccountServiceHost;
use sdkwork_database_sqlx::DatabasePool;
use std::sync::Arc;

use crate::web_bootstrap::wrap_router_with_web_framework_from_env;
use crate::{
    app_account_wallet_router_with_postgres_pool, app_account_wallet_router_with_sqlite_pool,
    app_billing_history_router_with_postgres_pool, app_billing_history_router_with_sqlite_pool,
};

pub fn build_account_app_router(host: Arc<AccountServiceHost>) -> Router {
    match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            let pool = pool.clone();
            Router::new()
                .merge(app_account_wallet_router_with_postgres_pool(pool.clone()))
                .merge(app_billing_history_router_with_postgres_pool(pool))
        }
        DatabasePool::Sqlite(pool, _) => {
            let pool = pool.clone();
            Router::new()
                .merge(app_account_wallet_router_with_sqlite_pool(pool.clone()))
                .merge(app_billing_history_router_with_sqlite_pool(pool))
        }
    }
}

pub async fn build_account_app_router_with_framework(host: Arc<AccountServiceHost>) -> Router {
    wrap_router_with_web_framework_from_env(build_account_app_router(host)).await
}
