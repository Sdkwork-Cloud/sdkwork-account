use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_account_repository_sqlx::{
    PostgresCommerceBillingHistoryStore, SqliteCommerceBillingHistoryStore,
};
use sdkwork_account_service::{BillingHistoryItem, BillingHistoryListQuery, StoreListPage};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_utils_rust::OffsetListPageParams;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{success_cursor_list_page, success_offset_list_page, unauthorized, validation};
use crate::subject::app_runtime_subject_from_extension;

pub type CommerceBillingHistoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceBillingHistoryStore: Send + Sync {
    fn list_billing_history<'a>(
        &'a self,
        query: BillingHistoryListQuery,
    ) -> CommerceBillingHistoryFuture<'a, StoreListPage<BillingHistoryItem>>;
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
    ) -> CommerceBillingHistoryFuture<'a, StoreListPage<BillingHistoryItem>> {
        Box::pin(async move { self.list_billing_history(query).await })
    }
}

impl CommerceBillingHistoryStore for PostgresCommerceBillingHistoryStore {
    fn list_billing_history<'a>(
        &'a self,
        query: BillingHistoryListQuery,
    ) -> CommerceBillingHistoryFuture<'a, StoreListPage<BillingHistoryItem>> {
        Box::pin(async move { self.list_billing_history(query).await })
    }
}

pub fn app_billing_history_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_billing_history_router(Arc::new(SqliteCommerceBillingHistoryStore::new(pool)))
}

pub fn app_billing_history_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_billing_history_router(Arc::new(PostgresCommerceBillingHistoryStore::new(pool)))
}

pub fn build_app_billing_history_router(store: Arc<dyn CommerceBillingHistoryStore>) -> Router {
    Router::new()
        .route("/app/v3/api/billing/history", get(fetch_billing_history))
        .with_state(AppBillingHistoryState { store })
}

async fn fetch_billing_history(
    State(state): State<AppBillingHistoryState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(params): Query<BillingHistoryQueryParams>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let list_query = match BillingHistoryListQuery::new(
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
        Err(error) => return validation(Some(&ctx), error.message()),
    };
    let list_query_for_response = list_query.clone();
    let paging = OffsetListPageParams::parse(list_query.page, list_query.page_size);
    let numeric_cursor = list_query
        .cursor
        .as_deref()
        .and_then(|raw| raw.trim().parse::<i64>().ok());

    match state.store.list_billing_history(list_query).await {
        Ok(page) => billing_history_list_response(
            &ctx,
            &list_query_for_response,
            page,
            paging,
            numeric_cursor,
        ),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

fn billing_history_list_response(
    ctx: &WebRequestContext,
    list_query: &BillingHistoryListQuery,
    page: StoreListPage<BillingHistoryItem>,
    paging: OffsetListPageParams,
    numeric_cursor: Option<i64>,
) -> Response {
    let mapped: Vec<_> = page
        .items
        .into_iter()
        .map(map_billing_history_item)
        .collect();

    if numeric_cursor.is_some() {
        let offset = numeric_cursor.unwrap_or(paging.offset);
        let next_cursor = page
            .has_more
            .then(|| (offset + mapped.len() as i64).to_string());
        return success_cursor_list_page(
            Some(ctx),
            mapped,
            paging.page_size,
            next_cursor,
            page.has_more,
        );
    }

    if list_query
        .cursor
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        let next_cursor = page.has_more.then(|| {
            mapped
                .last()
                .map(|item| item.occurred_at.clone())
                .unwrap_or_default()
        });
        return success_cursor_list_page(
            Some(ctx),
            mapped,
            paging.page_size,
            next_cursor,
            page.has_more,
        );
    }

    success_offset_list_page(
        Some(ctx),
        StoreListPage {
            items: mapped,
            total_items: page.total_items,
            has_more: page.has_more,
        },
        paging,
    )
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
