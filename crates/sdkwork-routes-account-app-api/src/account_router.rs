use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_account_repository_sqlx::{PostgresCommerceAccountStore, SqliteCommerceAccountStore};
use sdkwork_account_service::{
    AccountConsumptionItem, AccountInvoiceSettings, AccountLoginLog, AccountSecuritySummary,
    AccountHoldDetailQuery, AccountHoldItem, AccountHoldListQuery, AccountSummaryQuery,
    AccountSummarySnapshot, PointsAccountSnapshot, PointsLotItem, PointsLotListQuery, WalletAccountItem, WalletAccountListQuery, WalletOperation,
    WalletOperationQuery, WalletOverview, WalletTransactionDetailQuery, WalletTransactionItem,
    WalletTransactionListQuery,
};
use sdkwork_contract_service::{CommerceAccountAssetType, CommerceServiceError};
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{not_found, success_item, success_items, unauthorized, validation};
use crate::subject::app_runtime_subject_from_extension;

pub type CommerceWalletFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceAccountWalletStore: Send + Sync {
    fn retrieve_account_summary<'a>(
        &'a self,
        query: AccountSummaryQuery,
    ) -> CommerceWalletFuture<'a, AccountSummarySnapshot>;

    fn retrieve_wallet_overview<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, WalletOverview>;

    fn list_wallet_accounts<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, Vec<WalletAccountItem>>;

    fn list_wallet_transactions<'a>(
        &'a self,
        query: WalletTransactionListQuery,
    ) -> CommerceWalletFuture<'a, Vec<WalletTransactionItem>>;

    fn retrieve_wallet_transaction<'a>(
        &'a self,
        query: WalletTransactionDetailQuery,
    ) -> CommerceWalletFuture<'a, Option<WalletTransactionItem>>;

    fn retrieve_wallet_operation<'a>(
        &'a self,
        query: WalletOperationQuery,
    ) -> CommerceWalletFuture<'a, Option<WalletOperation>>;

    fn retrieve_points_account_snapshot<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, PointsAccountSnapshot>;

    fn retrieve_wallet_account_for_asset<'a>(
        &'a self,
        query: WalletAccountListQuery,
        asset_type: CommerceAccountAssetType,
    ) -> CommerceWalletFuture<'a, WalletAccountItem>;

    fn list_points_lots<'a>(
        &'a self,
        query: PointsLotListQuery,
    ) -> CommerceWalletFuture<'a, Vec<PointsLotItem>>;

    fn list_account_holds<'a>(
        &'a self,
        query: AccountHoldListQuery,
    ) -> CommerceWalletFuture<'a, Vec<AccountHoldItem>>;

    fn retrieve_account_hold<'a>(
        &'a self,
        query: AccountHoldDetailQuery,
    ) -> CommerceWalletFuture<'a, Option<AccountHoldItem>>;
}

#[derive(Clone)]
struct AppAccountWalletState {
    store: Arc<dyn CommerceAccountWalletStore>,
}

#[derive(Debug, Deserialize)]
struct WalletAccountQueryParams {
    #[serde(rename = "assetType", alias = "asset_type")]
    asset_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WalletTransactionQueryParams {
    #[serde(rename = "accountId", alias = "account_id")]
    account_id: Option<String>,
    #[serde(rename = "assetType", alias = "asset_type")]
    asset_type: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
    cursor: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletAccountItemResponse {
    id: String,
    uuid: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    asset_type: String,
    currency_code: Option<String>,
    available_amount: String,
    frozen_amount: String,
    pending_amount: String,
    status: String,
    version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletTransactionItemResponse {
    id: String,
    uuid: String,
    account_id: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    asset_type: String,
    direction: String,
    amount: String,
    balance_before: String,
    balance_after: String,
    business_type: String,
    transaction_no: String,
    request_no: String,
    idempotency_key: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletOverviewResponse {
    accounts: Vec<WalletAccountItemResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenBalanceResponse {
    available_tokens: i128,
    frozen_tokens: i128,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CashAccountResponse {
    account_id: String,
    account_uuid: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    currency_code: Option<String>,
    available_amount: String,
    frozen_amount: String,
    pending_amount: String,
    status: String,
    version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PointsAccountResponse {
    account_id: String,
    account_uuid: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    available_points: String,
    frozen_points: String,
    pending_points: String,
    total_points: String,
    active_lot_count: i64,
    expiring_points: String,
    status: String,
    version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PointsLotItemResponse {
    id: String,
    uuid: String,
    account_id: String,
    granted_amount: i64,
    remaining_amount: i64,
    source_type: String,
    source_id: String,
    expires_at: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountSummaryResponse {
    id: String,
    name: String,
    email: String,
    is_verified: bool,
    tier: String,
    organization: String,
    available_credits: f64,
    est_days_remaining: i64,
    monthly_consumption: f64,
    consumption_by_service: Vec<AccountConsumptionItemResponse>,
    invoice_settings: AccountInvoiceSettingsResponse,
    security: AccountSecuritySummaryResponse,
    login_logs: Vec<AccountLoginLogResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountConsumptionItemResponse {
    name: String,
    value: f64,
    color: String,
    percentage: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountInvoiceSettingsResponse {
    org_full: String,
    tax_id: String,
    payment_method: String,
    invoice_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountSecuritySummaryResponse {
    mfa_enabled: bool,
    qps_limit: i64,
    ip_whitelist_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountLoginLogResponse {
    ip: String,
    location: String,
    device: String,
    time: String,
    status: String,
}

impl CommerceAccountWalletStore for SqliteCommerceAccountStore {
    fn retrieve_account_summary<'a>(
        &'a self,
        query: AccountSummaryQuery,
    ) -> CommerceWalletFuture<'a, AccountSummarySnapshot> {
        Box::pin(async move { self.retrieve_account_summary_snapshot(query).await })
    }

    fn retrieve_wallet_overview<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, WalletOverview> {
        Box::pin(async move { self.retrieve_wallet_overview(query).await })
    }

    fn list_wallet_accounts<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, Vec<WalletAccountItem>> {
        Box::pin(async move { self.list_wallet_accounts(query).await })
    }

    fn list_wallet_transactions<'a>(
        &'a self,
        query: WalletTransactionListQuery,
    ) -> CommerceWalletFuture<'a, Vec<WalletTransactionItem>> {
        Box::pin(async move { self.list_wallet_transactions(query).await })
    }

    fn retrieve_wallet_transaction<'a>(
        &'a self,
        query: WalletTransactionDetailQuery,
    ) -> CommerceWalletFuture<'a, Option<WalletTransactionItem>> {
        Box::pin(async move { self.retrieve_wallet_transaction(query).await })
    }

    fn retrieve_wallet_operation<'a>(
        &'a self,
        query: WalletOperationQuery,
    ) -> CommerceWalletFuture<'a, Option<WalletOperation>> {
        Box::pin(async move { self.retrieve_wallet_operation(query).await })
    }

    fn retrieve_points_account_snapshot<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, PointsAccountSnapshot> {
        Box::pin(async move { self.retrieve_points_account_snapshot(query).await })
    }

    fn retrieve_wallet_account_for_asset<'a>(
        &'a self,
        query: WalletAccountListQuery,
        asset_type: CommerceAccountAssetType,
    ) -> CommerceWalletFuture<'a, WalletAccountItem> {
        Box::pin(async move { self.retrieve_wallet_account_for_asset(query, asset_type).await })
    }

    fn list_points_lots<'a>(
        &'a self,
        query: PointsLotListQuery,
    ) -> CommerceWalletFuture<'a, Vec<PointsLotItem>> {
        Box::pin(async move { self.list_points_lots(query).await })
    }

    fn list_account_holds<'a>(
        &'a self,
        query: AccountHoldListQuery,
    ) -> CommerceWalletFuture<'a, Vec<AccountHoldItem>> {
        Box::pin(async move { self.list_account_holds(query).await })
    }

    fn retrieve_account_hold<'a>(
        &'a self,
        query: AccountHoldDetailQuery,
    ) -> CommerceWalletFuture<'a, Option<AccountHoldItem>> {
        Box::pin(async move { self.retrieve_account_hold(query).await })
    }
}

impl CommerceAccountWalletStore for PostgresCommerceAccountStore {
    fn retrieve_account_summary<'a>(
        &'a self,
        query: AccountSummaryQuery,
    ) -> CommerceWalletFuture<'a, AccountSummarySnapshot> {
        Box::pin(async move { self.retrieve_account_summary_snapshot(query).await })
    }

    fn retrieve_wallet_overview<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, WalletOverview> {
        Box::pin(async move { self.retrieve_wallet_overview(query).await })
    }

    fn list_wallet_accounts<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, Vec<WalletAccountItem>> {
        Box::pin(async move { self.list_wallet_accounts(query).await })
    }

    fn list_wallet_transactions<'a>(
        &'a self,
        query: WalletTransactionListQuery,
    ) -> CommerceWalletFuture<'a, Vec<WalletTransactionItem>> {
        Box::pin(async move { self.list_wallet_transactions(query).await })
    }

    fn retrieve_wallet_transaction<'a>(
        &'a self,
        query: WalletTransactionDetailQuery,
    ) -> CommerceWalletFuture<'a, Option<WalletTransactionItem>> {
        Box::pin(async move { self.retrieve_wallet_transaction(query).await })
    }

    fn retrieve_wallet_operation<'a>(
        &'a self,
        query: WalletOperationQuery,
    ) -> CommerceWalletFuture<'a, Option<WalletOperation>> {
        Box::pin(async move { self.retrieve_wallet_operation(query).await })
    }

    fn retrieve_points_account_snapshot<'a>(
        &'a self,
        query: WalletAccountListQuery,
    ) -> CommerceWalletFuture<'a, PointsAccountSnapshot> {
        Box::pin(async move { self.retrieve_points_account_snapshot(query).await })
    }

    fn retrieve_wallet_account_for_asset<'a>(
        &'a self,
        query: WalletAccountListQuery,
        asset_type: CommerceAccountAssetType,
    ) -> CommerceWalletFuture<'a, WalletAccountItem> {
        Box::pin(async move { self.retrieve_wallet_account_for_asset(query, asset_type).await })
    }

    fn list_points_lots<'a>(
        &'a self,
        query: PointsLotListQuery,
    ) -> CommerceWalletFuture<'a, Vec<PointsLotItem>> {
        Box::pin(async move { self.list_points_lots(query).await })
    }

    fn list_account_holds<'a>(
        &'a self,
        query: AccountHoldListQuery,
    ) -> CommerceWalletFuture<'a, Vec<AccountHoldItem>> {
        Box::pin(async move { self.list_account_holds(query).await })
    }

    fn retrieve_account_hold<'a>(
        &'a self,
        query: AccountHoldDetailQuery,
    ) -> CommerceWalletFuture<'a, Option<AccountHoldItem>> {
        Box::pin(async move { self.retrieve_account_hold(query).await })
    }
}

pub fn app_account_wallet_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_account_wallet_router(Arc::new(SqliteCommerceAccountStore::new(pool)))
}

pub fn app_account_wallet_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_account_wallet_router(Arc::new(PostgresCommerceAccountStore::new(pool)))
}

pub fn build_app_account_wallet_router(store: Arc<dyn CommerceAccountWalletStore>) -> Router {
    Router::new()
        .route(
            "/app/v3/api/accounts/current/summary",
            get(fetch_account_summary),
        )
        .route("/app/v3/api/wallet/overview", get(fetch_wallet_overview))
        .route("/app/v3/api/wallet/accounts", get(fetch_wallet_accounts))
        .route("/app/v3/api/wallet/accounts/cash", get(fetch_cash_account))
        .route("/app/v3/api/wallet/accounts/points", get(fetch_points_account))
        .route(
            "/app/v3/api/wallet/accounts/tokens",
            get(fetch_token_account),
        )
        .route(
            "/app/v3/api/wallet/ledger_entries",
            get(fetch_wallet_transactions),
        )
        .route(
            "/app/v3/api/wallet/ledger_entries/cash",
            get(fetch_cash_ledger_entries),
        )
        .route(
            "/app/v3/api/wallet/ledger_entries/points",
            get(fetch_points_ledger_entries),
        )
        .route("/app/v3/api/wallet/points/lots", get(fetch_points_lots))
        .route("/app/v3/api/wallet/holds", get(fetch_account_holds))
        .route(
            "/app/v3/api/wallet/holds/{holdId}",
            get(fetch_account_hold),
        )
        .route(
            "/app/v3/api/wallet/ledger_entries/{ledgerEntryId}",
            get(fetch_wallet_transaction),
        )
        .route("/app/v3/api/wallet/tokens", get(fetch_token_balance))
        .with_state(AppAccountWalletState { store })
}

async fn fetch_account_summary(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let query = match AccountSummaryQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state.store.retrieve_account_summary(query).await {
        Ok(data) => success_item(Some(&ctx), map_account_summary(data)),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_wallet_overview(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(query): Query<WalletAccountQueryParams>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let asset_type = match parse_optional_asset_type(Some(&ctx), query.asset_type.as_deref()) {
        Ok(asset_type) => asset_type,
        Err(response) => return response,
    };
    let query = match WalletAccountListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        asset_type,
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state.store.retrieve_wallet_overview(query).await {
        Ok(data) => success_item(Some(&ctx), map_wallet_overview(data)),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_wallet_accounts(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(query): Query<WalletAccountQueryParams>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let asset_type = match parse_optional_asset_type(Some(&ctx), query.asset_type.as_deref()) {
        Ok(asset_type) => asset_type,
        Err(response) => return response,
    };
    let query = match WalletAccountListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        asset_type,
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state.store.list_wallet_accounts(query).await {
        Ok(data) => {
            let count = data.len() as i64;
            success_items(
                Some(&ctx),
                data.into_iter().map(map_wallet_account).collect(),
                1,
                count,
            )
        }
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_wallet_transactions(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(query): Query<WalletTransactionQueryParams>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let asset_type = match parse_optional_asset_type(Some(&ctx), query.asset_type.as_deref()) {
        Ok(asset_type) => asset_type,
        Err(response) => return response,
    };
    let list_query = match WalletTransactionListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        query.account_id.as_deref(),
        asset_type,
        query.page,
        query.page_size,
        query.cursor.as_deref(),
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };
    let page = list_query.page.unwrap_or(1);
    let page_size = list_query.limit();

    match state.store.list_wallet_transactions(list_query).await {
        Ok(data) => success_items(
            Some(&ctx),
            data.into_iter().map(map_wallet_transaction).collect(),
            page,
            page_size,
        ),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_wallet_transaction(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(transaction_id): Path<String>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let query = match WalletTransactionDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &transaction_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state.store.retrieve_wallet_transaction(query).await {
        Ok(Some(data)) => success_item(Some(&ctx), map_wallet_transaction(data)),
        Ok(None) => not_found(Some(&ctx), "wallet transaction was not found"),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_token_balance(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let query = match wallet_account_list_query(&subject, None) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state
        .store
        .retrieve_wallet_account_for_asset(query, CommerceAccountAssetType::Token)
        .await
    {
        Ok(account) => match map_token_balance(vec![account]) {
            Ok(balance) => success_item(Some(&ctx), balance),
            Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
        },
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_cash_account(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let query = match wallet_account_list_query(&subject, None) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state
        .store
        .retrieve_wallet_account_for_asset(query, CommerceAccountAssetType::Cash)
        .await
    {
        Ok(account) => success_item(Some(&ctx), map_cash_account(account)),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_points_account(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let query = match wallet_account_list_query(&subject, None) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state.store.retrieve_points_account_snapshot(query).await {
        Ok(snapshot) => success_item(Some(&ctx), map_points_account(snapshot)),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_token_account(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let query = match wallet_account_list_query(&subject, None) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state
        .store
        .retrieve_wallet_account_for_asset(query, CommerceAccountAssetType::Token)
        .await
    {
        Ok(account) => success_item(Some(&ctx), map_wallet_account(account)),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_cash_ledger_entries(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(query): Query<WalletTransactionQueryParams>,
) -> Response {
    fetch_asset_ledger_entries(
        state,
        request_context,
        runtime_context,
        query,
        CommerceAccountAssetType::Cash,
    )
    .await
}

async fn fetch_points_ledger_entries(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(query): Query<WalletTransactionQueryParams>,
) -> Response {
    fetch_asset_ledger_entries(
        state,
        request_context,
        runtime_context,
        query,
        CommerceAccountAssetType::Points,
    )
    .await
}

#[derive(Debug, Deserialize)]
struct PointsLotQueryParams {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

async fn fetch_points_lots(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(params): Query<PointsLotQueryParams>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let list_query = match PointsLotListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };
    let page = list_query.page.unwrap_or(1);
    let page_size = list_query.limit();

    match state.store.list_points_lots(list_query).await {
        Ok(items) => success_items(
            Some(&ctx),
            items.into_iter().map(map_points_lot).collect(),
            page,
            page_size,
        ),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

#[derive(Debug, Deserialize)]
struct AccountHoldQueryParams {
    #[serde(rename = "accountId", alias = "account_id")]
    account_id: Option<String>,
    #[serde(rename = "assetType", alias = "asset_type")]
    asset_type: Option<String>,
    status: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountHoldItemResponse {
    id: String,
    uuid: String,
    tenant_id: String,
    organization_id: Option<String>,
    account_id: String,
    owner_user_id: String,
    asset_type: String,
    amount: String,
    settled_amount: String,
    released_amount: String,
    status: String,
    business_type: String,
    business_no: String,
    source_type: String,
    source_id: String,
    request_no: String,
    idempotency_key: String,
    expires_at: Option<String>,
    settled_at: Option<String>,
    released_at: Option<String>,
    version: i64,
    created_at: String,
    updated_at: String,
}

async fn fetch_account_holds(
    State(state): State<AppAccountWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(params): Query<AccountHoldQueryParams>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let asset_type = match params.asset_type.as_deref() {
        Some(value) => match parse_asset_type_filter(value) {
            Ok(asset_type) => Some(asset_type),
            Err(message) => return validation(Some(&ctx), message),
        },
        None => None,
    };
    let list_query = match AccountHoldListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        params.account_id.as_deref(),
        asset_type,
        params.status.as_deref(),
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };
    let page = list_query.page.unwrap_or(1);
    let page_size = list_query.limit();

    match state.store.list_account_holds(list_query).await {
        Ok(items) => success_items(
            Some(&ctx),
            items.into_iter().map(map_account_hold).collect(),
            page,
            page_size,
        ),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

async fn fetch_account_hold(
    State(state): State<AppAccountWalletState>,
    Path(hold_id): Path<String>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let query = match AccountHoldDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        hold_id.trim(),
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };

    match state.store.retrieve_account_hold(query).await {
        Ok(Some(item)) => success_item(Some(&ctx), map_account_hold(item)),
        Ok(None) => crate::api_response::not_found(Some(&ctx), "account hold was not found"),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

fn parse_asset_type_filter(value: &str) -> Result<CommerceAccountAssetType, &'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "cash" => Ok(CommerceAccountAssetType::Cash),
        "point" | "points" => Ok(CommerceAccountAssetType::Points),
        "token" | "tokens" => Ok(CommerceAccountAssetType::Token),
        _ => Err("asset_type is invalid"),
    }
}

async fn fetch_asset_ledger_entries(
    state: AppAccountWalletState,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    query: WalletTransactionQueryParams,
    asset_type: CommerceAccountAssetType,
) -> Response {
    let ctx = request_context.0;
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    let list_query = match WalletTransactionListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        query.account_id.as_deref(),
        Some(asset_type),
        query.page,
        query.page_size,
        query.cursor.as_deref(),
    ) {
        Ok(query) => query,
        Err(error) => return validation(Some(&ctx), error.message()),
    };
    let page = list_query.page.unwrap_or(1);
    let page_size = list_query.limit();

    match state.store.list_wallet_transactions(list_query).await {
        Ok(data) => success_items(
            Some(&ctx),
            data.into_iter().map(map_wallet_transaction).collect(),
            page,
            page_size,
        ),
        Err(error) => crate::api_response::map_service_error(Some(&ctx), error),
    }
}

fn wallet_account_list_query(
    subject: &crate::subject::AppRuntimeSubject,
    asset_type: Option<CommerceAccountAssetType>,
) -> Result<WalletAccountListQuery, CommerceServiceError> {
    WalletAccountListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        asset_type,
    )
}

#[allow(clippy::result_large_err)]
fn parse_optional_asset_type(
    context: Option<&WebRequestContext>,
    value: Option<&str>,
) -> Result<Option<CommerceAccountAssetType>, Response> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => parse_asset_type(context, value).map(Some),
        None => Ok(None),
    }
}

#[allow(clippy::result_large_err)]
fn parse_asset_type(
    context: Option<&WebRequestContext>,
    value: &str,
) -> Result<CommerceAccountAssetType, Response> {
    match value.to_ascii_lowercase().as_str() {
        "cash" => Ok(CommerceAccountAssetType::Cash),
        "point" | "points" => Ok(CommerceAccountAssetType::Points),
        "token" | "tokens" => Ok(CommerceAccountAssetType::Token),
        _ => Err(validation(context, "asset_type is invalid")),
    }
}

fn map_account_summary(value: AccountSummarySnapshot) -> AccountSummaryResponse {
    AccountSummaryResponse {
        id: value.id,
        name: value.name,
        email: value.email,
        is_verified: value.is_verified,
        tier: value.tier,
        organization: value.organization,
        available_credits: value.available_credits,
        est_days_remaining: value.est_days_remaining,
        monthly_consumption: value.monthly_consumption,
        consumption_by_service: value
            .consumption_by_service
            .into_iter()
            .map(map_account_consumption_item)
            .collect(),
        invoice_settings: map_account_invoice_settings(value.invoice_settings),
        security: map_account_security_summary(value.security),
        login_logs: value
            .login_logs
            .into_iter()
            .map(map_account_login_log)
            .collect(),
    }
}

fn map_account_consumption_item(value: AccountConsumptionItem) -> AccountConsumptionItemResponse {
    AccountConsumptionItemResponse {
        name: value.name,
        value: value.value,
        color: value.color,
        percentage: value.percentage,
    }
}

fn map_account_invoice_settings(value: AccountInvoiceSettings) -> AccountInvoiceSettingsResponse {
    AccountInvoiceSettingsResponse {
        org_full: value.org_full,
        tax_id: value.tax_id,
        payment_method: value.payment_method,
        invoice_type: value.invoice_type,
    }
}

fn map_account_security_summary(value: AccountSecuritySummary) -> AccountSecuritySummaryResponse {
    AccountSecuritySummaryResponse {
        mfa_enabled: value.mfa_enabled,
        qps_limit: value.qps_limit,
        ip_whitelist_count: value.ip_whitelist_count,
    }
}

fn map_account_login_log(value: AccountLoginLog) -> AccountLoginLogResponse {
    AccountLoginLogResponse {
        ip: value.ip,
        location: value.location,
        device: value.device,
        time: value.time,
        status: value.status,
    }
}

fn map_wallet_overview(value: WalletOverview) -> WalletOverviewResponse {
    WalletOverviewResponse {
        accounts: value.accounts.into_iter().map(map_wallet_account).collect(),
    }
}

fn map_wallet_account(value: WalletAccountItem) -> WalletAccountItemResponse {
    WalletAccountItemResponse {
        id: value.id,
        uuid: value.uuid,
        tenant_id: value.tenant_id,
        organization_id: value.organization_id,
        owner_user_id: value.owner_user_id,
        asset_type: value.asset_type.as_str().to_owned(),
        currency_code: value.currency_code,
        available_amount: value.available_amount.as_str().to_owned(),
        frozen_amount: value.frozen_amount.as_str().to_owned(),
        pending_amount: value.pending_amount.as_str().to_owned(),
        status: value.status,
        version: value.version,
    }
}

fn map_token_balance(
    accounts: Vec<WalletAccountItem>,
) -> Result<TokenBalanceResponse, CommerceServiceError> {
    let mut available_tokens = 0_i128;
    let mut frozen_tokens = 0_i128;
    for account in accounts {
        available_tokens += parse_token_amount(account.available_amount.as_str())?;
        frozen_tokens += parse_token_amount(account.frozen_amount.as_str())?;
    }
    Ok(TokenBalanceResponse {
        available_tokens,
        frozen_tokens,
    })
}

fn map_wallet_transaction(value: WalletTransactionItem) -> WalletTransactionItemResponse {
    WalletTransactionItemResponse {
        id: value.id,
        uuid: value.uuid,
        account_id: value.account_id,
        tenant_id: value.tenant_id,
        organization_id: value.organization_id,
        owner_user_id: value.owner_user_id,
        asset_type: value.asset_type.as_str().to_owned(),
        direction: value.direction.as_str().to_owned(),
        amount: value.amount.as_str().to_owned(),
        balance_before: value.balance_before.as_str().to_owned(),
        balance_after: value.balance_after.as_str().to_owned(),
        business_type: value.business_type,
        transaction_no: value.transaction_no,
        request_no: value.request_no,
        idempotency_key: value.idempotency_key,
        created_at: value.created_at,
    }
}

fn map_cash_account(value: WalletAccountItem) -> CashAccountResponse {
    CashAccountResponse {
        account_id: value.id,
        account_uuid: value.uuid,
        tenant_id: value.tenant_id,
        organization_id: value.organization_id,
        owner_user_id: value.owner_user_id,
        currency_code: value.currency_code,
        available_amount: value.available_amount.as_str().to_owned(),
        frozen_amount: value.frozen_amount.as_str().to_owned(),
        pending_amount: value.pending_amount.as_str().to_owned(),
        status: value.status,
        version: value.version,
    }
}

fn map_points_account(value: PointsAccountSnapshot) -> PointsAccountResponse {
    let available = value.account.available_amount.as_str();
    let frozen = value.account.frozen_amount.as_str();
    let pending = value.account.pending_amount.as_str();
    PointsAccountResponse {
        account_id: value.account.id,
        account_uuid: value.account.uuid,
        tenant_id: value.account.tenant_id,
        organization_id: value.account.organization_id,
        owner_user_id: value.account.owner_user_id,
        available_points: available.to_owned(),
        frozen_points: frozen.to_owned(),
        pending_points: pending.to_owned(),
        total_points: sum_amount_strings(available, frozen, pending),
        active_lot_count: value.active_lot_count,
        expiring_points: value.expiring_points.to_string(),
        status: value.account.status,
        version: value.account.version,
    }
}

fn map_account_hold(value: AccountHoldItem) -> AccountHoldItemResponse {
    AccountHoldItemResponse {
        id: value.id,
        uuid: value.uuid,
        tenant_id: value.tenant_id,
        organization_id: value.organization_id,
        account_id: value.account_id,
        owner_user_id: value.owner_user_id,
        asset_type: value.asset_type,
        amount: value.amount,
        settled_amount: value.settled_amount,
        released_amount: value.released_amount,
        status: value.status,
        business_type: value.business_type,
        business_no: value.business_no,
        source_type: value.source_type,
        source_id: value.source_id,
        request_no: value.request_no,
        idempotency_key: value.idempotency_key,
        expires_at: value.expires_at,
        settled_at: value.settled_at,
        released_at: value.released_at,
        version: value.version,
        created_at: value.created_at,
        updated_at: value.updated_at,
    }
}

fn map_points_lot(value: PointsLotItem) -> PointsLotItemResponse {
    PointsLotItemResponse {
        id: value.id,
        uuid: value.uuid,
        account_id: value.account_id,
        granted_amount: value.granted_amount,
        remaining_amount: value.remaining_amount,
        source_type: value.source_type,
        source_id: value.source_id,
        expires_at: value.expires_at,
        status: value.status,
        created_at: value.created_at,
        updated_at: value.updated_at,
    }
}

fn sum_amount_strings(left: &str, middle: &str, right: &str) -> String {
    let total = left.parse::<i128>().unwrap_or(0)
        + middle.parse::<i128>().unwrap_or(0)
        + right.parse::<i128>().unwrap_or(0);
    total.to_string()
}

fn parse_token_amount(value: &str) -> Result<i128, CommerceServiceError> {
    let normalized = value.trim();
    if normalized.is_empty() || normalized.starts_with('-') || normalized.starts_with('+') {
        return Err(CommerceServiceError::storage(format!(
            "invalid commerce token amount: {value}"
        )));
    }
    if !normalized
        .chars()
        .all(|character| character.is_ascii_digit())
    {
        return Err(CommerceServiceError::storage(format!(
            "invalid commerce token amount: {value}"
        )));
    }
    normalized.parse::<i128>().map_err(|_| {
        CommerceServiceError::storage(format!("invalid commerce token amount: {value}"))
    })
}
