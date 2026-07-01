pub mod account_router;
pub mod api_response;
pub mod billing_router;
pub mod routes;
pub mod subject;
pub mod web_bootstrap;

pub use account_router::{
    app_account_wallet_router_with_postgres_pool, app_account_wallet_router_with_sqlite_pool,
    build_app_account_wallet_router, CommerceAccountWalletStore, CommerceWalletFuture,
};
pub use billing_router::{
    app_billing_history_router_with_postgres_pool, app_billing_history_router_with_sqlite_pool,
    build_app_billing_history_router, CommerceBillingHistoryFuture, CommerceBillingHistoryStore,
};
pub use routes::build_account_app_router_with_framework;
pub use web_bootstrap::wrap_router_with_web_framework_from_env;

use axum::Router;
use sdkwork_account_service_host::AccountServiceHost;
use std::sync::Arc;

pub async fn gateway_mount(host: Arc<AccountServiceHost>) -> Router {
    build_account_app_router_with_framework(host).await
}
