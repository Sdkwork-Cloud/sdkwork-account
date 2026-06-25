use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_commerce_account_service::{
    AccountConsumptionItem, AccountInvoiceSettings, AccountLoginLog, AccountSecuritySummary,
    AccountSummaryQuery, AccountSummarySnapshot, WalletAccountItem, WalletAccountListQuery,
    WalletOperation, WalletOperationQuery, WalletOverview, WalletTransactionDetailQuery,
    WalletTransactionItem, WalletTransactionListQuery,
};
use sdkwork_commerce_contract_service::{CommerceAccountAssetType, CommerceServiceError};
use sdkwork_commerce_account_repository_sqlx::{
    PostgresCommerceAccountStore, SqliteCommerceAccountStore,
};
use sdkwork_iam_context_service::IamAppContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

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
struct AppWalletApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletAccountItemResponse {
    id: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    asset_type: String,
    currency_code: Option<String>,
    available_amount: String,
    frozen_amount: String,
    status: String,
    version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletTransactionItemResponse {
    id: String,
    account_id: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    asset_type: String,
    direction: String,
    amount: String,
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
}

impl<T: Serialize> AppWalletApiResult<T> {
    fn success(data: T) -> Self {
        Self {
            code: "2000".to_owned(),
            msg: "SUCCESS".to_owned(),
            data: Some(data),
        }
    }
}

impl AppWalletApiResult<()> {
    fn error(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            msg: msg.into(),
            data: None,
        }
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
            .route(
                "/app/v3/api/wallet/ledger_entries",
                get(fetch_wallet_transactions),
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
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match AccountSummaryQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_account_summary(query).await {
        Ok(data) => Json(AppWalletApiResult::success(map_account_summary(data))).into_response(),
        Err(error) => wallet_system_response("account summary read model is unavailable", error),
    }
}

async fn fetch_wallet_overview(
    State(state): State<AppAccountWalletState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(query): Query<WalletAccountQueryParams>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let asset_type = match parse_optional_asset_type(query.asset_type.as_deref()) {
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
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_wallet_overview(query).await {
        Ok(data) => Json(AppWalletApiResult::success(map_wallet_overview(data))).into_response(),
        Err(error) => wallet_system_response("wallet overview read model is unavailable", error),
    }
}

async fn fetch_wallet_accounts(
    State(state): State<AppAccountWalletState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(query): Query<WalletAccountQueryParams>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let asset_type = match parse_optional_asset_type(query.asset_type.as_deref()) {
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
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_wallet_accounts(query).await {
        Ok(data) => Json(AppWalletApiResult::success(
            data.into_iter().map(map_wallet_account).collect::<Vec<_>>(),
        ))
        .into_response(),
        Err(error) => wallet_system_response("wallet accounts read model is unavailable", error),
    }
}

async fn fetch_wallet_transactions(
    State(state): State<AppAccountWalletState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(query): Query<WalletTransactionQueryParams>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let asset_type = match parse_optional_asset_type(query.asset_type.as_deref()) {
        Ok(asset_type) => asset_type,
        Err(response) => return response,
    };
    let query = match WalletTransactionListQuery::new(
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
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_wallet_transactions(query).await {
        Ok(data) => Json(AppWalletApiResult::success(
            data.into_iter()
                .map(map_wallet_transaction)
                .collect::<Vec<_>>(),
        ))
        .into_response(),
        Err(error) => {
            wallet_system_response("wallet transactions read model is unavailable", error)
        }
    }
}

async fn fetch_wallet_transaction(
    State(state): State<AppAccountWalletState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(transaction_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match WalletTransactionDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &transaction_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_wallet_transaction(query).await {
        Ok(Some(data)) => {
            Json(AppWalletApiResult::success(map_wallet_transaction(data))).into_response()
        }
        Ok(None) => not_found_response("wallet transaction was not found"),
        Err(error) => wallet_system_response("wallet transaction read model is unavailable", error),
    }
}

async fn fetch_token_balance(
    State(state): State<AppAccountWalletState>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match WalletAccountListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        Some(CommerceAccountAssetType::Token),
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_wallet_accounts(query).await {
        Ok(accounts) => match map_token_balance(accounts) {
            Ok(balance) => Json(AppWalletApiResult::success(balance)).into_response(),
            Err(error) => wallet_system_response("token balance read model is unavailable", error),
        },
        Err(error) => wallet_system_response("token balance read model is unavailable", error),
    }
}

#[allow(clippy::result_large_err)]
fn parse_optional_asset_type(
    value: Option<&str>,
) -> Result<Option<CommerceAccountAssetType>, Response> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => parse_asset_type(value).map(Some),
        None => Ok(None),
    }
}

#[allow(clippy::result_large_err)]
fn parse_asset_type(value: &str) -> Result<CommerceAccountAssetType, Response> {
    match value.to_ascii_lowercase().as_str() {
        "cash" => Ok(CommerceAccountAssetType::Cash),
        "point" | "points" => Ok(CommerceAccountAssetType::Points),
        "token" | "tokens" => Ok(CommerceAccountAssetType::Token),
        _ => Err(validation_response("asset_type is invalid")),
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
        tenant_id: value.tenant_id,
        organization_id: value.organization_id,
        owner_user_id: value.owner_user_id,
        asset_type: value.asset_type.as_str().to_owned(),
        currency_code: value.currency_code,
        available_amount: value.available_amount.as_str().to_owned(),
        frozen_amount: value.frozen_amount.as_str().to_owned(),
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
        account_id: value.account_id,
        tenant_id: value.tenant_id,
        organization_id: value.organization_id,
        owner_user_id: value.owner_user_id,
        asset_type: value.asset_type.as_str().to_owned(),
        direction: value.direction.as_str().to_owned(),
        amount: value.amount.as_str().to_owned(),
        balance_after: value.balance_after.as_str().to_owned(),
        business_type: value.business_type,
        transaction_no: value.transaction_no,
        request_no: value.request_no,
        idempotency_key: value.idempotency_key,
        created_at: value.created_at,
    }
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

fn unauthorized_response(message: String) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(AppWalletApiResult::error("4010", message)),
    )
        .into_response()
}

fn validation_response(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(AppWalletApiResult::error("4001", message)),
    )
        .into_response()
}

fn not_found_response(message: impl Into<String>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(AppWalletApiResult::error("4040", message)),
    )
        .into_response()
}

fn wallet_system_response(context: &str, error: CommerceServiceError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(AppWalletApiResult::error(
            "5000",
            format!("{context}: {}", error.message()),
        )),
    )
        .into_response()
}
