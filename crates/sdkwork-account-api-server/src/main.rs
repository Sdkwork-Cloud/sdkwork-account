use axum::Router;
use sdkwork_router_account_app_api::build_account_app_router_with_framework;
use sdkwork_router_account_backend_api::build_account_backend_router_with_framework;
use sdkwork_account_api_server::account_health_router;
use sdkwork_account_service_host::AccountServiceHost;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let host = Arc::new(AccountServiceHost::new().await);
    let app = Router::new()
        .merge(account_health_router())
        .merge(build_account_app_router_with_framework(host.clone()).await)
        .merge(build_account_backend_router_with_framework(host).await)
        .layer(CorsLayer::permissive());
    let addr = std::env::var("ACCOUNT_API_BIND").unwrap_or_else(|_| "0.0.0.0:18095".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
