//! API assembly for sdkwork-account.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.

mod bootstrap;
mod generated;

pub use bootstrap::{assemble_api_router, ApiAssembly};

pub async fn assemble_api_router_from_env() -> Result<ApiAssembly, String> {
    let host = sdkwork_account_service_host::AccountServiceHost::from_env().await?;
    Ok(assemble_api_router(std::sync::Arc::new(host)).await)
}

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
