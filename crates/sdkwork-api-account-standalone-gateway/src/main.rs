//! Account API server entrypoint.
//!
//! Production bootstrap:
//! - CORS restricted to `SDKWORK_CORS_ALLOWED_ORIGINS` (fail-closed when unset).
//! - Readiness reflects database health via `SELECT 1`.
//! - Graceful shutdown drains in-flight requests on SIGINT / SIGTERM.

use std::sync::Arc;
use std::time::Duration;

use sdkwork_account_service_host::AccountServiceHost;
use sdkwork_api_account_assembly::assemble_api_router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_web_bootstrap::{service_router, ReadinessCheck, ReadinessFuture, ServiceRouterConfig};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let host = match AccountServiceHost::from_env().await {
        Ok(host) => Arc::new(host),
        Err(error) => {
            tracing::error!(target = "account.bootstrap", error = %error, "account service host bootstrap failed");
            return Err(error.into());
        }
    };

    let business = assemble_api_router(host.clone())
        .await
        .router
        .layer(TraceLayer::new_for_http())
        .layer(sdkwork_web_bootstrap::application_cors_layer_from_env(
            &["SDKWORK_ACCOUNT_ENVIRONMENT", "ACCOUNT_ENVIRONMENT"],
            &["SDKWORK_CORS_ALLOWED_ORIGINS"],
        ));

    let readiness = Arc::new(AccountReadiness { host: host.clone() });
    let app = service_router(
        business,
        ServiceRouterConfig::default().with_readiness_check(readiness),
    );

    let addr = std::env::var("ACCOUNT_API_BIND").unwrap_or_else(|_| "0.0.0.0:18095".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(target = "account.bootstrap", %addr, "account api server listening");

    let serve = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    if let Err(error) = serve.await {
        tracing::error!(target = "account.runtime", error = %error, "axum serve failed");
        return Err(error.into());
    }

    tokio::time::timeout(Duration::from_secs(30), host.database_pool().close())
        .await
        .map_err(|_| std::io::Error::other("database pool close timed out after 30s"))?;
    tracing::info!(target = "account.runtime", "account api server stopped");
    Ok(())
}

#[derive(Clone)]
struct AccountReadiness {
    host: Arc<AccountServiceHost>,
}

impl ReadinessCheck for AccountReadiness {
    fn check(&self) -> ReadinessFuture<'_> {
        Box::pin(async move {
            // 服务端权威持久化仅支持 PostgreSQL（DATABASE_SPEC：authoritative-server）
            let DatabasePool::Postgres(pool, _) = self.host.database_pool() else {
                return Err("database is not ready (PostgreSQL pool required)".to_owned());
            };
            let result = sqlx::query_scalar::<_, i64>("SELECT 1")
                .fetch_one(pool)
                .await;
            match result {
                Ok(_) => Ok(()),
                Err(error) => {
                    tracing::error!(target = "account.readiness", error = %error, "database readiness probe failed");
                    Err("database is not ready".to_owned())
                }
            }
        })
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(target = "account.runtime", error = %error, "ctrl_c signal handler failed");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(error) => {
                tracing::warn!(target = "account.runtime", error = %error, "SIGTERM signal handler failed");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!(
        target = "account.runtime",
        "account api server shutdown signal received, draining in-flight requests"
    );
}
