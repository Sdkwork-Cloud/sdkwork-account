use axum::routing::get;
use axum::Router;
use sdkwork_account_service_host::AccountServiceHost;
use sdkwork_database_sqlx::DatabasePool;
use std::sync::Arc;

use crate::web_bootstrap::wrap_router_with_web_framework_from_env;
use crate::{
    backend_hold_router_with_postgres_pool, backend_hold_router_with_sqlite_pool,
    backend_wallet_router_with_postgres_pool, backend_wallet_router_with_sqlite_pool,
};

pub fn build_account_backend_router(host: Arc<AccountServiceHost>) -> Router {
    let mut router = Router::new().route("/backend/v3/api/wallet/health", get(|| async { "ok" }));

    router = router.merge(match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            let pool = pool.clone();
            backend_wallet_router_with_postgres_pool(pool.clone())
                .merge(backend_hold_router_with_postgres_pool(pool))
        }
        DatabasePool::Sqlite(pool, _) => {
            let pool = pool.clone();
            backend_wallet_router_with_sqlite_pool(pool.clone())
                .merge(backend_hold_router_with_sqlite_pool(pool))
        }
    });

    router
}

pub async fn build_account_backend_router_with_framework(host: Arc<AccountServiceHost>) -> Router {
    wrap_router_with_web_framework_from_env(build_account_backend_router(host)).await
}
