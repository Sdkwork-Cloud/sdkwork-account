use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use sdkwork_account_repository_sqlx::{
    PostgresCommerceAccountStore, SqliteCommerceAccountStore,
};
use sdkwork_account_service::OutboxDispatchOutcome;
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{map_service_error, success_item, unauthorized};
use crate::subject::backend_runtime_subject_from_extension;

pub type CommerceOutboxRelayFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceOutboxRelayStore: Send + Sync {
    fn dispatch_outbox_batch<'a>(
        &'a self,
        batch_size: Option<i64>,
    ) -> CommerceOutboxRelayFuture<'a, OutboxDispatchOutcome>;
}

#[derive(Clone)]
struct BackendOutboxRelayState {
    store: Arc<dyn CommerceOutboxRelayStore>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DispatchOutboxRequest {
    #[serde(default)]
    batch_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutboxDispatchItemResponse {
    id: String,
    uuid: String,
    tenant_id: String,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    event_version: i32,
    event_key: String,
    payload: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutboxDispatchResponse {
    accepted: bool,
    dispatched_count: i64,
    pending_lag: i64,
    items: Vec<OutboxDispatchItemResponse>,
}

impl CommerceOutboxRelayStore for SqliteCommerceAccountStore {
    fn dispatch_outbox_batch<'a>(
        &'a self,
        batch_size: Option<i64>,
    ) -> CommerceOutboxRelayFuture<'a, OutboxDispatchOutcome> {
        Box::pin(async move { self.dispatch_outbox_batch(batch_size).await })
    }
}

impl CommerceOutboxRelayStore for PostgresCommerceAccountStore {
    fn dispatch_outbox_batch<'a>(
        &'a self,
        batch_size: Option<i64>,
    ) -> CommerceOutboxRelayFuture<'a, OutboxDispatchOutcome> {
        Box::pin(async move { self.dispatch_outbox_batch(batch_size).await })
    }
}

pub fn backend_outbox_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_backend_outbox_router(Arc::new(SqliteCommerceAccountStore::new(pool)))
}

pub fn backend_outbox_router_with_postgres_pool(pool: PgPool) -> Router {
    build_backend_outbox_router(Arc::new(PostgresCommerceAccountStore::new(pool)))
}

pub fn build_backend_outbox_router(store: Arc<dyn CommerceOutboxRelayStore>) -> Router {
    Router::new()
        .route("/backend/v3/api/wallet/outbox/dispatch", post(dispatch_outbox_batch))
        .with_state(BackendOutboxRelayState { store })
}

async fn dispatch_outbox_batch(
    State(state): State<BackendOutboxRelayState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    axum::Json(body): axum::Json<DispatchOutboxRequest>,
) -> Response {
    let ctx = request_context.0;
    if backend_runtime_subject_from_extension(runtime_context).is_err() {
        return unauthorized(Some(&ctx), "authenticated runtime context is required");
    }

    match state.store.dispatch_outbox_batch(body.batch_size).await {
        Ok(outcome) => success_item(Some(&ctx), map_outbox_dispatch_response(outcome)),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

fn map_outbox_dispatch_response(outcome: OutboxDispatchOutcome) -> OutboxDispatchResponse {
    OutboxDispatchResponse {
        accepted: true,
        dispatched_count: outcome.dispatched_count,
        pending_lag: outcome.pending_lag,
        items: outcome
            .items
            .into_iter()
            .map(|item| OutboxDispatchItemResponse {
                id: item.id,
                uuid: item.uuid,
                tenant_id: item.tenant_id,
                aggregate_type: item.aggregate_type,
                aggregate_id: item.aggregate_id,
                event_type: item.event_type,
                event_version: item.event_version,
                event_key: item.event_key,
                payload: item.payload,
                created_at: item.created_at,
            })
            .collect(),
    }
}
