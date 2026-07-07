//! Account API server entrypoint.
//!
//! Production bootstrap:
//! - CORS restricted to `ACCOUNT_CORS_ALLOW_ORIGINS` (fail-closed when unset).
//! - Readiness reflects database health via `SELECT 1`.
//! - Graceful shutdown drains in-flight requests on SIGINT / SIGTERM.

use std::sync::Arc;
use std::time::Duration;

use sdkwork_account_gateway_assembly::assemble_application_router;
use sdkwork_account_service_host::AccountServiceHost;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_web_bootstrap::{service_router, ReadinessCheck, ReadinessFuture, ServiceRouterConfig};
use tower_http::cors::{AllowOrigin, CorsLayer};
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

    let business = assemble_application_router(host.clone())
        .await
        .router
        .layer(TraceLayer::new_for_http())
        .layer(build_cors_layer());

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
            let result = match self.host.database_pool() {
                DatabasePool::Postgres(pool, _) => {
                    sqlx::query_scalar::<_, i64>("SELECT 1")
                        .fetch_one(pool)
                        .await
                }
                DatabasePool::Sqlite(pool, _) => {
                    sqlx::query_scalar::<_, i64>("SELECT 1")
                        .fetch_one(pool)
                        .await
                }
            };
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

fn build_cors_layer() -> CorsLayer {
    let raw = std::env::var("ACCOUNT_CORS_ALLOW_ORIGINS")
        .unwrap_or_default()
        .trim()
        .to_owned();

    let allow_origin = if raw.is_empty() {
        tracing::warn!(
            target = "account.security",
            "ACCOUNT_CORS_ALLOW_ORIGINS is not set; cross-origin requests are denied"
        );
        AllowOrigin::list([])
    } else if raw == "*" {
        if std::env::var("ACCOUNT_CORS_PERMISSIVE_DEV").as_deref() == Ok("1") {
            tracing::warn!(
                target = "account.security",
                "CORS is permissive (dev mode) — never use in production"
            );
            AllowOrigin::mirror_request()
        } else {
            tracing::error!(
                target = "account.security",
                "ACCOUNT_CORS_ALLOW_ORIGINS='*' ignored without ACCOUNT_CORS_PERMISSIVE_DEV=1; cross-origin requests are denied"
            );
            AllowOrigin::list([])
        }
    } else {
        let origins: Vec<_> = raw
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .filter_map(|value| match value.parse::<axum::http::HeaderValue>() {
                Ok(parsed) => Some(parsed),
                Err(error) => {
                    tracing::warn!(target = "account.security", origin = %value, error = %error, "invalid CORS origin ignored");
                    None
                }
            })
            .collect();
        AllowOrigin::list(origins)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any)
        .allow_credentials(true)
        .max_age(Duration::from_secs(600))
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
