use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_commerce_account_service::{BillingHistoryItem, BillingHistoryListQuery};
use sdkwork_commerce_contract_service::CommerceServiceError;
use sdkwork_commerce_account_repository_sqlx::{
    PostgresCommerceBillingHistoryStore, SqliteCommerceBillingHistoryStore,
};
use sdkwork_iam_context_service::IamAppContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::subject::app_runtime_subject_from_extension;

pub type CommerceBillingHistoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceBillingHistoryStore: Send + Sync {
    fn list_billing_history<'a>(
        &'a self,
        query: BillingHistoryListQuery,
    ) -> CommerceBillingHistoryFuture<'a, Vec<BillingHistoryItem>>;
}

#[derive(Clone)]
struct AppBillingHistoryState {
    store: Arc<dyn CommerceBillingHistoryStore>,
}

#[derive(Debug, Deserialize)]
struct BillingHistoryQueryParams {
    #[serde(rename = "type", alias = "history_type")]
    history_type: Option<String>,
    status: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
    cursor: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppBillingApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BillingHistoryCollectionResponse {
    items: Vec<BillingHistoryItemResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BillingHistoryItemResponse {
    id: String,
    history_no: String,
    #[serde(rename = "type")]
    history_type: String,
    direction: String,
    asset_type: String,
    amount: String,
    currency_code: Option<String>,
    points_delta: i64,
    status: String,
    title: String,
    reference_no: Option<String>,
    source_type: String,
    source_id: String,
    related_order_no: Option<String>,
    payment_method: Option<String>,
    occurred_at: String,
}

impl CommerceBillingHistoryStore for SqliteCommerceBillingHistoryStore {
    fn list_billing_history<'a>(
        &'a self,
        query: BillingHistoryListQuery,
    ) -> CommerceBillingHistoryFuture<'a, Vec<BillingHistoryItem>> {
        Box::pin(async move { self.list_billing_history(query).await })
    }
}

impl CommerceBillingHistoryStore for PostgresCommerceBillingHistoryStore {
    fn list_billing_history<'a>(
        &'a self,
        query: BillingHistoryListQuery,
    ) -> CommerceBillingHistoryFuture<'a, Vec<BillingHistoryItem>> {
        Box::pin(async move { self.list_billing_history(query).await })
    }
}

impl<T: Serialize> AppBillingApiResult<T> {
    fn success(data: T) -> Self {
        Self {
            code: "2000".to_owned(),
            msg: "SUCCESS".to_owned(),
            data: Some(data),
        }
    }
}

impl AppBillingApiResult<()> {
    fn error(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            msg: msg.into(),
            data: None,
        }
    }
}

pub fn app_billing_history_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_billing_history_router(Arc::new(SqliteCommerceBillingHistoryStore::new(pool)))
}

pub fn app_billing_history_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_billing_history_router(Arc::new(PostgresCommerceBillingHistoryStore::new(pool)))
}

pub fn build_app_billing_history_router(
    store: Arc<dyn CommerceBillingHistoryStore>,
) -> Router {
    Router::new()
            .route("/app/v3/api/billing/history", get(fetch_billing_history))
            .with_state(AppBillingHistoryState { store })
}

async fn fetch_billing_history(
    State(state): State<AppBillingHistoryState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(params): Query<BillingHistoryQueryParams>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match BillingHistoryListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        params.history_type.as_deref(),
        params.status.as_deref(),
        params.page,
        params.page_size,
        params.cursor.as_deref(),
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_billing_history(query).await {
        Ok(items) => Json(AppBillingApiResult::success(
            BillingHistoryCollectionResponse {
                items: items.into_iter().map(map_billing_history_item).collect(),
            },
        ))
        .into_response(),
        Err(error) => billing_system_response("billing history read model is unavailable", error),
    }
}

fn map_billing_history_item(value: BillingHistoryItem) -> BillingHistoryItemResponse {
    BillingHistoryItemResponse {
        id: value.id,
        history_no: value.history_no,
        history_type: value.history_type,
        direction: value.direction,
        asset_type: value.asset_type,
        amount: value.amount.as_str().to_owned(),
        currency_code: value.currency_code,
        points_delta: value.points_delta,
        status: value.status,
        title: value.title,
        reference_no: value.reference_no,
        source_type: value.source_type,
        source_id: value.source_id,
        related_order_no: value.related_order_no,
        payment_method: value.payment_method,
        occurred_at: value.occurred_at,
    }
}

fn unauthorized_response(message: String) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(AppBillingApiResult::error("4010", message)),
    )
        .into_response()
}

fn validation_response(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(AppBillingApiResult::error("4001", message)),
    )
        .into_response()
}

fn billing_system_response(context: &str, error: CommerceServiceError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(AppBillingApiResult::error(
            "5000",
            format!("{context}: {}", error.message()),
        )),
    )
        .into_response()
}
