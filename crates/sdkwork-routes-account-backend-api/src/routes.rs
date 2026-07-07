use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_account_repository_sqlx::{PostgresCommerceAccountStore, SqliteCommerceAccountStore};
use sdkwork_account_service_host::AccountServiceHost;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};
use std::sync::Arc;

use crate::web_bootstrap::wrap_router_with_web_framework_from_env;
use crate::{
    backend_hold_router_with_postgres_pool, backend_hold_router_with_sqlite_pool,
    backend_outbox_router_with_postgres_pool, backend_outbox_router_with_sqlite_pool,
    backend_wallet_router_with_postgres_pool, backend_wallet_router_with_sqlite_pool,
};

async fn wallet_health(
    axum::extract::State(host): axum::extract::State<Arc<AccountServiceHost>>,
) -> (
    StatusCode,
    Json<SdkWorkApiResponse<SdkWorkResourceData<serde_json::Value>>>,
) {
    let db_ok = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => sqlx::query("SELECT 1").execute(pool).await.is_ok(),
        DatabasePool::Sqlite(pool, _) => sqlx::query("SELECT 1").execute(pool).await.is_ok(),
    };

    let outbox_pending_lag = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => PostgresCommerceAccountStore::new(pool.clone())
            .pending_outbox_lag()
            .await
            .unwrap_or(-1),
        DatabasePool::Sqlite(pool, _) => SqliteCommerceAccountStore::new(pool.clone())
            .pending_outbox_lag()
            .await
            .unwrap_or(-1),
    };

    let status = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let payload = SdkWorkResourceData {
        item: serde_json::json!({
            "status": if db_ok { "ready" } else { "degraded" },
            "database": if db_ok { "up" } else { "down" },
            "outboxPendingLag": outbox_pending_lag,
        }),
    };
    (
        status,
        Json(SdkWorkApiResponse::success(
            payload,
            sdkwork_utils_rust::uuid(),
        )),
    )
}

pub fn build_account_backend_router(host: Arc<AccountServiceHost>) -> Router {
    let mut router = Router::new().route(
        "/backend/v3/api/wallet/health",
        get(wallet_health).with_state(host.clone()),
    );

    router = router.merge(match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            let pool = pool.clone();
            backend_wallet_router_with_postgres_pool(pool.clone())
                .merge(backend_hold_router_with_postgres_pool(pool.clone()))
                .merge(backend_outbox_router_with_postgres_pool(pool))
        }
        DatabasePool::Sqlite(pool, _) => {
            let pool = pool.clone();
            backend_wallet_router_with_sqlite_pool(pool.clone())
                .merge(backend_hold_router_with_sqlite_pool(pool.clone()))
                .merge(backend_outbox_router_with_sqlite_pool(pool))
        }
    });

    router
}

pub async fn build_account_backend_router_with_framework(host: Arc<AccountServiceHost>) -> Router {
    wrap_router_with_web_framework_from_env(build_account_backend_router(host)).await
}
