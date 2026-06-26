use axum::Router;
use sdkwork_account_service_host::AccountServiceHost;
use std::sync::Arc;

pub mod routes;
pub mod web_bootstrap;

pub use routes::build_account_backend_router_with_framework;

pub async fn gateway_mount(host: Arc<AccountServiceHost>) -> Router {
    build_account_backend_router_with_framework(host).await
}
