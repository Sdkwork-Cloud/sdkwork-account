use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use sdkwork_account_repository_sqlx::{PostgresCommerceAccountStore, SqliteCommerceAccountStore};
use sdkwork_account_service::{
    AppendLedgerEntryCommand, AppendLedgerEntryOutcome, WalletAccountItem, WalletTransactionItem,
};
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney, CommerceRequestHash,
    CommerceServiceError,
};
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_utils_rust::sha256_hash;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{map_service_error, success_item, unauthorized, validation};
use crate::subject::backend_runtime_subject_from_extension;

pub type CommerceLedgerWriteFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceAccountLedgerWriteStore: Send + Sync {
    fn append_ledger_entry<'a>(
        &'a self,
        command: AppendLedgerEntryCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceLedgerWriteFuture<'a, AppendLedgerEntryOutcome>;
}

#[derive(Clone)]
struct BackendWalletState {
    store: Arc<dyn CommerceAccountLedgerWriteStore>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateWalletAdjustmentRequest {
    tenant_id: String,
    #[serde(default)]
    organization_id: Option<String>,
    owner_user_id: String,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    asset_type: String,
    #[serde(default)]
    currency_code: Option<String>,
    direction: String,
    amount: String,
    business_type: String,
    transaction_no: String,
    request_no: String,
    idempotency_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletAdjustmentResponse {
    accepted: bool,
    replayed: bool,
    account: WalletAccountItemResponse,
    ledger_entry: WalletTransactionItemResponse,
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

impl CommerceAccountLedgerWriteStore for SqliteCommerceAccountStore {
    fn append_ledger_entry<'a>(
        &'a self,
        command: AppendLedgerEntryCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceLedgerWriteFuture<'a, AppendLedgerEntryOutcome> {
        Box::pin(async move { self.append_ledger_entry(command, request_hash).await })
    }
}

impl CommerceAccountLedgerWriteStore for PostgresCommerceAccountStore {
    fn append_ledger_entry<'a>(
        &'a self,
        command: AppendLedgerEntryCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceLedgerWriteFuture<'a, AppendLedgerEntryOutcome> {
        Box::pin(async move { self.append_ledger_entry(command, request_hash).await })
    }
}

pub fn backend_wallet_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_backend_wallet_router(Arc::new(SqliteCommerceAccountStore::new(pool)))
}

pub fn backend_wallet_router_with_postgres_pool(pool: PgPool) -> Router {
    build_backend_wallet_router(Arc::new(PostgresCommerceAccountStore::new(pool)))
}

pub fn build_backend_wallet_router(store: Arc<dyn CommerceAccountLedgerWriteStore>) -> Router {
    Router::new()
        .route(
            "/backend/v3/api/wallet/adjustments",
            post(create_wallet_adjustment),
        )
        .route(
            "/backend/v3/api/wallet/adjustments/cash",
            post(create_cash_adjustment),
        )
        .route(
            "/backend/v3/api/wallet/adjustments/points",
            post(create_points_adjustment),
        )
        .route(
            "/backend/v3/api/wallet/adjustments/tokens",
            post(create_token_adjustment),
        )
        .with_state(BackendWalletState { store })
}

async fn create_cash_adjustment(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    body: axum::Json<CreateWalletAdjustmentRequest>,
) -> Response {
    create_wallet_adjustment_with_asset(
        state,
        request_context,
        runtime_context,
        body,
        CommerceAccountAssetType::Cash,
    )
    .await
}

async fn create_points_adjustment(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    body: axum::Json<CreateWalletAdjustmentRequest>,
) -> Response {
    create_wallet_adjustment_with_asset(
        state,
        request_context,
        runtime_context,
        body,
        CommerceAccountAssetType::Points,
    )
    .await
}

async fn create_token_adjustment(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    body: axum::Json<CreateWalletAdjustmentRequest>,
) -> Response {
    create_wallet_adjustment_with_asset(
        state,
        request_context,
        runtime_context,
        body,
        CommerceAccountAssetType::Token,
    )
    .await
}

async fn create_wallet_adjustment(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    body: axum::Json<CreateWalletAdjustmentRequest>,
) -> Response {
    let asset_type = match parse_asset_type(body.asset_type.trim()) {
        Ok(asset_type) => asset_type,
        Err(message) => return validation(Some(&request_context.0), message),
    };
    create_wallet_adjustment_with_asset(state, request_context, runtime_context, body, asset_type)
        .await
}

async fn create_wallet_adjustment_with_asset(
    state: BackendWalletState,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    axum::Json(mut body): axum::Json<CreateWalletAdjustmentRequest>,
    asset_type: CommerceAccountAssetType,
) -> Response {
    let ctx = request_context.0;
    let subject = match backend_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };

    if body.tenant_id.trim() != subject.tenant_id {
        return validation(
            Some(&ctx),
            "tenant_id must match authenticated runtime tenant",
        );
    }

    body.asset_type = asset_type.as_str().to_owned();
    let direction = match parse_direction(body.direction.trim()) {
        Ok(direction) => direction,
        Err(message) => return validation(Some(&ctx), message),
    };
    let amount = match parse_amount(&body.amount) {
        Ok(amount) => amount,
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    let command = match AppendLedgerEntryCommand::new(
        body.tenant_id.trim(),
        body.organization_id.as_deref(),
        body.account_id.as_deref().unwrap_or(""),
        body.owner_user_id.trim(),
        asset_type,
        body.currency_code.as_deref(),
        direction,
        amount,
        body.business_type.trim(),
        body.transaction_no.trim(),
        body.request_no.trim(),
        body.idempotency_key.trim(),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    let request_hash = match request_hash_from_body(&body) {
        Ok(request_hash) => request_hash,
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    match state
        .store
        .append_ledger_entry(command, request_hash)
        .await
    {
        Ok(outcome) => success_item(
            Some(&ctx),
            WalletAdjustmentResponse {
                accepted: true,
                replayed: outcome.replayed,
                account: map_wallet_account(outcome.account),
                ledger_entry: map_wallet_transaction(outcome.ledger_entry),
            },
        ),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

fn request_hash_from_body(
    body: &CreateWalletAdjustmentRequest,
) -> Result<CommerceRequestHash, CommerceServiceError> {
    let canonical = serde_json::to_string(body)
        .map_err(|error| CommerceServiceError::validation(format!("request body is invalid: {error}")))?;
    CommerceRequestHash::new(&sha256_hash(canonical.as_bytes()))
}

fn parse_asset_type(value: &str) -> Result<CommerceAccountAssetType, String> {
    match value.to_ascii_lowercase().as_str() {
        "cash" => Ok(CommerceAccountAssetType::Cash),
        "point" | "points" => Ok(CommerceAccountAssetType::Points),
        "token" | "tokens" => Ok(CommerceAccountAssetType::Token),
        _ => Err("asset_type is invalid".to_owned()),
    }
}

fn parse_direction(value: &str) -> Result<CommerceLedgerDirection, String> {
    match value.to_ascii_lowercase().as_str() {
        "credit" => Ok(CommerceLedgerDirection::Credit),
        "debit" => Ok(CommerceLedgerDirection::Debit),
        _ => Err("direction is invalid".to_owned()),
    }
}

fn parse_amount(value: &str) -> Result<CommerceMoney, CommerceServiceError> {
    CommerceMoney::new(value).map_err(CommerceServiceError::validation)
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
