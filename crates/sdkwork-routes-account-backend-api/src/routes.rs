use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use sdkwork_account_service_host::AccountServiceHost;

pub fn build_account_backend_router(_host: Arc<AccountServiceHost>) -> Router {
    Router::new().route(
        "/backend/v3/api/wallet/health",
        get(|| async { "ok" }),
    )
}

pub async fn build_account_backend_router_with_framework(host: Arc<AccountServiceHost>) -> Router {
    build_account_backend_router(host)
}
