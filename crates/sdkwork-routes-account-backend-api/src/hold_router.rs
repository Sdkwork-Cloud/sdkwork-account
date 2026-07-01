use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use sdkwork_account_repository_sqlx::{hold_request_hash, PostgresCommerceAccountStore, SqliteCommerceAccountStore};
use sdkwork_account_service::{
    AccountHoldItem, CreateAccountHoldCommand, CreateAccountTransferCommand, HoldMutationOutcome,
    ReleaseAccountHoldCommand, SettleAccountHoldCommand, TransferMutationOutcome, WalletAccountItem,
    WalletTransactionItem,
};
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceMoney, CommerceRequestHash, CommerceServiceError,
};
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{map_service_error, success_item, unauthorized, validation};
use crate::subject::backend_runtime_subject_from_extension;

pub type CommerceHoldWriteFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceAccountHoldWriteStore: Send + Sync {
    fn create_account_hold<'a>(
        &'a self,
        command: CreateAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome>;

    fn settle_account_hold<'a>(
        &'a self,
        command: SettleAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome>;

    fn release_account_hold<'a>(
        &'a self,
        command: ReleaseAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome>;

    fn create_account_transfer<'a>(
        &'a self,
        command: CreateAccountTransferCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, TransferMutationOutcome>;
}

#[derive(Clone)]
pub struct BackendHoldState {
    store: Arc<dyn CommerceAccountHoldWriteStore>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateAccountHoldRequest {
    tenant_id: String,
    #[serde(default)]
    organization_id: Option<String>,
    owner_user_id: String,
    #[serde(default)]
    account_id: Option<String>,
    asset_type: String,
    amount: String,
    business_type: String,
    business_no: String,
    source_type: String,
    source_id: String,
    request_no: String,
    idempotency_key: String,
    #[serde(default)]
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettleAccountHoldRequest {
    tenant_id: String,
    business_type: String,
    transaction_no: String,
    request_no: String,
    idempotency_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseAccountHoldRequest {
    tenant_id: String,
    request_no: String,
    idempotency_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateAccountTransferRequest {
    tenant_id: String,
    #[serde(default)]
    organization_id: Option<String>,
    from_account_id: String,
    to_account_id: String,
    owner_user_id: String,
    asset_type: String,
    amount: String,
    business_type: String,
    business_no: String,
    request_no: String,
    idempotency_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HoldMutationResponse {
    accepted: bool,
    replayed: bool,
    hold: AccountHoldItemResponse,
    account: WalletAccountItemResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    ledger_entry: Option<WalletTransactionItemResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransferMutationResponse {
    accepted: bool,
    replayed: bool,
    transfer: AccountTransferItemResponse,
    from_account: WalletAccountItemResponse,
    to_account: WalletAccountItemResponse,
    debit_entry: WalletTransactionItemResponse,
    credit_entry: WalletTransactionItemResponse,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountTransferItemResponse {
    id: String,
    uuid: String,
    tenant_id: String,
    organization_id: Option<String>,
    from_account_id: String,
    to_account_id: String,
    owner_user_id: String,
    asset_type: String,
    amount: String,
    status: String,
    business_type: String,
    business_no: String,
    request_no: String,
    idempotency_key: String,
    journal_id: String,
    trace_id: String,
    created_at: String,
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

impl CommerceAccountHoldWriteStore for SqliteCommerceAccountStore {
    fn create_account_hold<'a>(
        &'a self,
        command: CreateAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome> {
        Box::pin(async move { self.create_account_hold(command, request_hash).await })
    }

    fn settle_account_hold<'a>(
        &'a self,
        command: SettleAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome> {
        Box::pin(async move { self.settle_account_hold(command, request_hash).await })
    }

    fn release_account_hold<'a>(
        &'a self,
        command: ReleaseAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome> {
        Box::pin(async move { self.release_account_hold(command, request_hash).await })
    }

    fn create_account_transfer<'a>(
        &'a self,
        command: CreateAccountTransferCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, TransferMutationOutcome> {
        Box::pin(async move { self.create_account_transfer(command, request_hash).await })
    }
}

impl CommerceAccountHoldWriteStore for PostgresCommerceAccountStore {
    fn create_account_hold<'a>(
        &'a self,
        command: CreateAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome> {
        Box::pin(async move { self.create_account_hold(command, request_hash).await })
    }

    fn settle_account_hold<'a>(
        &'a self,
        command: SettleAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome> {
        Box::pin(async move { self.settle_account_hold(command, request_hash).await })
    }

    fn release_account_hold<'a>(
        &'a self,
        command: ReleaseAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, HoldMutationOutcome> {
        Box::pin(async move { self.release_account_hold(command, request_hash).await })
    }

    fn create_account_transfer<'a>(
        &'a self,
        command: CreateAccountTransferCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceHoldWriteFuture<'a, TransferMutationOutcome> {
        Box::pin(async move { self.create_account_transfer(command, request_hash).await })
    }
}

pub fn backend_hold_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_backend_hold_router(Arc::new(SqliteCommerceAccountStore::new(pool)))
}

pub fn backend_hold_router_with_postgres_pool(pool: PgPool) -> Router {
    build_backend_hold_router(Arc::new(PostgresCommerceAccountStore::new(pool)))
}

pub fn build_backend_hold_router(store: Arc<dyn CommerceAccountHoldWriteStore>) -> Router {
    Router::new()
        .route("/backend/v3/api/wallet/holds", post(create_account_hold))
        .route(
            "/backend/v3/api/wallet/holds/{holdId}/settle",
            post(settle_account_hold),
        )
        .route(
            "/backend/v3/api/wallet/holds/{holdId}/release",
            post(release_account_hold),
        )
        .route("/backend/v3/api/wallet/transfers", post(create_account_transfer))
        .with_state(BackendHoldState { store })
}

async fn create_account_hold(
    State(state): State<BackendHoldState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    axum::Json(body): axum::Json<CreateAccountHoldRequest>,
) -> Response {
    let ctx = request_context.0;
    let subject = match backend_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    if body.tenant_id.trim() != subject.tenant_id {
        return validation(Some(&ctx), "tenant_id must match authenticated runtime tenant");
    }
    let asset_type = match parse_asset_type(body.asset_type.trim()) {
        Ok(asset_type) => asset_type,
        Err(message) => return validation(Some(&ctx), message),
    };
    let amount = match CommerceMoney::new(body.amount.trim()).map_err(CommerceServiceError::validation) {
        Ok(amount) => amount,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    let command = match CreateAccountHoldCommand::new(
        body.tenant_id.trim(),
        body.organization_id.as_deref(),
        body.account_id.as_deref().unwrap_or(""),
        body.owner_user_id.trim(),
        asset_type,
        amount,
        body.business_type.trim(),
        body.business_no.trim(),
        body.source_type.trim(),
        body.source_id.trim(),
        body.request_no.trim(),
        body.idempotency_key.trim(),
        body.expires_at.as_deref(),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    let request_hash = match hold_request_hash(&serde_json::to_string(&body).unwrap_or_default()) {
        Ok(hash) => hash,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    match state.store.create_account_hold(command, request_hash).await {
        Ok(outcome) => success_item(Some(&ctx), map_hold_outcome(outcome)),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

async fn settle_account_hold(
    State(state): State<BackendHoldState>,
    Path(hold_id): Path<String>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    axum::Json(body): axum::Json<SettleAccountHoldRequest>,
) -> Response {
    let ctx = request_context.0;
    let subject = match backend_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    if body.tenant_id.trim() != subject.tenant_id {
        return validation(Some(&ctx), "tenant_id must match authenticated runtime tenant");
    }
    let command = match SettleAccountHoldCommand::new(
        body.tenant_id.trim(),
        hold_id.trim(),
        body.business_type.trim(),
        body.transaction_no.trim(),
        body.request_no.trim(),
        body.idempotency_key.trim(),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    let request_hash = match hold_request_hash(&serde_json::to_string(&body).unwrap_or_default()) {
        Ok(hash) => hash,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    match state.store.settle_account_hold(command, request_hash).await {
        Ok(outcome) => success_item(Some(&ctx), map_hold_outcome(outcome)),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

async fn release_account_hold(
    State(state): State<BackendHoldState>,
    Path(hold_id): Path<String>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    axum::Json(body): axum::Json<ReleaseAccountHoldRequest>,
) -> Response {
    let ctx = request_context.0;
    let subject = match backend_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    if body.tenant_id.trim() != subject.tenant_id {
        return validation(Some(&ctx), "tenant_id must match authenticated runtime tenant");
    };
    let command = match ReleaseAccountHoldCommand::new(
        body.tenant_id.trim(),
        hold_id.trim(),
        body.request_no.trim(),
        body.idempotency_key.trim(),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    let request_hash = match hold_request_hash(&serde_json::to_string(&body).unwrap_or_default()) {
        Ok(hash) => hash,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    match state.store.release_account_hold(command, request_hash).await {
        Ok(outcome) => success_item(Some(&ctx), map_hold_outcome(outcome)),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

async fn create_account_transfer(
    State(state): State<BackendHoldState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    axum::Json(body): axum::Json<CreateAccountTransferRequest>,
) -> Response {
    let ctx = request_context.0;
    let subject = match backend_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(Some(&ctx), message),
    };
    if body.tenant_id.trim() != subject.tenant_id {
        return validation(Some(&ctx), "tenant_id must match authenticated runtime tenant");
    }
    let asset_type = match parse_asset_type(body.asset_type.trim()) {
        Ok(asset_type) => asset_type,
        Err(message) => return validation(Some(&ctx), message),
    };
    let amount = match CommerceMoney::new(body.amount.trim()).map_err(CommerceServiceError::validation) {
        Ok(amount) => amount,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    let command = match CreateAccountTransferCommand::new(
        body.tenant_id.trim(),
        body.organization_id.as_deref(),
        body.from_account_id.trim(),
        body.to_account_id.trim(),
        body.owner_user_id.trim(),
        asset_type,
        amount,
        body.business_type.trim(),
        body.business_no.trim(),
        body.request_no.trim(),
        body.idempotency_key.trim(),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    let request_hash = match hold_request_hash(&serde_json::to_string(&body).unwrap_or_default()) {
        Ok(hash) => hash,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    match state.store.create_account_transfer(command, request_hash).await {
        Ok(outcome) => success_item(Some(&ctx), map_transfer_outcome(outcome)),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

fn parse_asset_type(value: &str) -> Result<CommerceAccountAssetType, &'static str> {
    match value.to_ascii_lowercase().as_str() {
        "cash" => Ok(CommerceAccountAssetType::Cash),
        "point" | "points" => Ok(CommerceAccountAssetType::Points),
        "token" | "tokens" => Ok(CommerceAccountAssetType::Token),
        _ => Err("asset_type is invalid"),
    }
}

fn map_hold_outcome(outcome: HoldMutationOutcome) -> HoldMutationResponse {
    HoldMutationResponse {
        accepted: true,
        replayed: outcome.replayed,
        hold: map_hold_item(outcome.hold),
        account: map_wallet_account(outcome.account),
        ledger_entry: outcome.ledger_entry.map(map_wallet_transaction),
    }
}

fn map_transfer_outcome(outcome: TransferMutationOutcome) -> TransferMutationResponse {
    TransferMutationResponse {
        accepted: true,
        replayed: outcome.replayed,
        transfer: map_transfer_item(outcome.transfer),
        from_account: map_wallet_account(outcome.from_account),
        to_account: map_wallet_account(outcome.to_account),
        debit_entry: map_wallet_transaction(outcome.debit_entry),
        credit_entry: map_wallet_transaction(outcome.credit_entry),
    }
}

fn map_hold_item(value: AccountHoldItem) -> AccountHoldItemResponse {
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

fn map_transfer_item(value: sdkwork_account_service::AccountTransferItem) -> AccountTransferItemResponse {
    AccountTransferItemResponse {
        id: value.id,
        uuid: value.uuid,
        tenant_id: value.tenant_id,
        organization_id: value.organization_id,
        from_account_id: value.from_account_id,
        to_account_id: value.to_account_id,
        owner_user_id: value.owner_user_id,
        asset_type: value.asset_type,
        amount: value.amount,
        status: value.status,
        business_type: value.business_type,
        business_no: value.business_no,
        request_no: value.request_no,
        idempotency_key: value.idempotency_key,
        journal_id: value.journal_id,
        trace_id: value.trace_id,
        created_at: value.created_at,
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
