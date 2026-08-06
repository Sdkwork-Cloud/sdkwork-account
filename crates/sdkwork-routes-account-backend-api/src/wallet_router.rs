use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use sdkwork_account_repository_sqlx::PostgresCommerceAccountStore;
use sdkwork_account_service::{
    AppendLedgerEntryCommand, AppendLedgerEntryOutcome, ExpirePointsLotsCommand,
    ExpirePointsLotsOutcome, PointsLotMismatchItem, PointsReconciliationQuery,
    PointsReconciliationSnapshot, WalletAccountItem, WalletTransactionItem,
};
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney, CommerceRequestHash,
    CommerceServiceError,
};
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_utils_rust::sha256_hash;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::api_response::{
    map_service_error, success_created_item, success_item, unauthorized, validation,
};
use crate::subject::{backend_runtime_subject_from_extension, ensure_backend_owner_user_allowed};

pub type CommerceLedgerWriteFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceAccountLedgerWriteStore: Send + Sync {
    fn append_ledger_entry<'a>(
        &'a self,
        command: AppendLedgerEntryCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceLedgerWriteFuture<'a, AppendLedgerEntryOutcome>;

    fn expire_points_lots<'a>(
        &'a self,
        command: ExpirePointsLotsCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceLedgerWriteFuture<'a, ExpirePointsLotsOutcome>;

    fn reconcile_points_lots<'a>(
        &'a self,
        query: PointsReconciliationQuery,
    ) -> CommerceLedgerWriteFuture<'a, PointsReconciliationSnapshot>;
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
    #[serde(default)]
    direction: String,
    amount: String,
    business_type: String,
    transaction_no: String,
    request_no: String,
    idempotency_key: String,
    #[serde(default)]
    expires_at: Option<String>,
    #[serde(default)]
    reversed_ledger_id: Option<String>,
    /// Owner subject kind (defaults to USER). PARTNER opens settlement accounts.
    #[serde(default)]
    owner_type: Option<String>,
    /// Account purpose (defaults to GENERAL). SETTLEMENT for partner revenue.
    #[serde(default)]
    account_purpose: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpirePointsLotsRequest {
    tenant_id: String,
    #[serde(default)]
    organization_id: Option<String>,
    #[serde(default)]
    owner_user_id: Option<String>,
    #[serde(default)]
    account_id: Option<String>,
    request_no: String,
    idempotency_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpirePointsLotsResponse {
    accepted: bool,
    replayed: bool,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    expired_lot_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    expired_points_total: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PointsReconciliationRequest {
    tenant_id: String,
    #[serde(default)]
    organization_id: Option<String>,
    #[serde(default)]
    owner_user_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PointsLotMismatchResponse {
    account_id: String,
    available_points: String,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    lot_remaining_total: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    delta: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PointsReconciliationResponse {
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    checked_account_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    mismatch_count: i64,
    mismatches: Vec<PointsLotMismatchResponse>,
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
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
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

impl CommerceAccountLedgerWriteStore for PostgresCommerceAccountStore {
    fn append_ledger_entry<'a>(
        &'a self,
        command: AppendLedgerEntryCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceLedgerWriteFuture<'a, AppendLedgerEntryOutcome> {
        Box::pin(async move { self.append_ledger_entry(command, request_hash).await })
    }

    fn expire_points_lots<'a>(
        &'a self,
        command: ExpirePointsLotsCommand,
        request_hash: CommerceRequestHash,
    ) -> CommerceLedgerWriteFuture<'a, ExpirePointsLotsOutcome> {
        Box::pin(async move { self.expire_points_lots(command, request_hash).await })
    }

    fn reconcile_points_lots<'a>(
        &'a self,
        query: PointsReconciliationQuery,
    ) -> CommerceLedgerWriteFuture<'a, PointsReconciliationSnapshot> {
        Box::pin(async move { self.reconcile_points_lots(query).await })
    }
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
            "/backend/v3/api/token_bank/credits",
            post(create_token_bank_credit),
        )
        .route(
            "/backend/v3/api/token_bank/debits",
            post(create_token_bank_debit),
        )
        .route(
            "/backend/v3/api/token_bank/grants",
            post(create_token_bank_grant),
        )
        .route(
            "/backend/v3/api/token_bank/reversals",
            post(create_token_bank_reversal),
        )
        .route(
            "/backend/v3/api/wallet/points/lots/expire",
            post(expire_points_lots),
        )
        .route(
            "/backend/v3/api/wallet/points/reconciliation",
            post(reconcile_points_lots),
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

async fn create_token_bank_credit(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    mut body: axum::Json<CreateWalletAdjustmentRequest>,
) -> Response {
    body.direction = "credit".to_owned();
    create_wallet_adjustment_with_asset(
        state,
        request_context,
        runtime_context,
        body,
        CommerceAccountAssetType::TokenBank,
    )
    .await
}

async fn create_token_bank_debit(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    mut body: axum::Json<CreateWalletAdjustmentRequest>,
) -> Response {
    body.direction = "debit".to_owned();
    create_wallet_adjustment_with_asset(
        state,
        request_context,
        runtime_context,
        body,
        CommerceAccountAssetType::TokenBank,
    )
    .await
}

async fn create_token_bank_grant(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    mut body: axum::Json<CreateWalletAdjustmentRequest>,
) -> Response {
    body.direction = "credit".to_owned();
    create_wallet_adjustment_with_asset(
        state,
        request_context,
        runtime_context,
        body,
        CommerceAccountAssetType::TokenBank,
    )
    .await
}

async fn create_token_bank_reversal(
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
        CommerceAccountAssetType::TokenBank,
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
    let iam_context = runtime_context
        .as_ref()
        .map(|Extension(context)| context.clone());
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

    if let Some(iam) = iam_context.as_ref() {
        if let Err(message) = ensure_backend_owner_user_allowed(iam, body.owner_user_id.trim()) {
            return validation(Some(&ctx), message);
        }
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

    let mut command = match AppendLedgerEntryCommand::with_options(
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
        body.expires_at.as_deref(),
        body.reversed_ledger_id.as_deref(),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(Some(&ctx), error),
    };
    if let (Some(owner_type), Some(account_purpose)) = (body.owner_type.as_deref(), body.account_purpose.as_deref()) {
        command = command.with_account_subject(owner_type, account_purpose);
    }

    let request_hash = match request_hash_from_body(&body) {
        Ok(request_hash) => request_hash,
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    match state.store.append_ledger_entry(command, request_hash).await {
        Ok(outcome) => success_created_item(
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

async fn expire_points_lots(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    body: axum::Json<ExpirePointsLotsRequest>,
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

    let command = match ExpirePointsLotsCommand::new(
        body.tenant_id.trim(),
        body.organization_id.as_deref(),
        body.owner_user_id.as_deref(),
        body.account_id.as_deref(),
        body.request_no.trim(),
        body.idempotency_key.trim(),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    let request_hash = match expire_request_hash_from_body(&body) {
        Ok(request_hash) => request_hash,
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    match state.store.expire_points_lots(command, request_hash).await {
        Ok(outcome) => success_item(
            Some(&ctx),
            ExpirePointsLotsResponse {
                accepted: outcome.accepted,
                replayed: outcome.replayed,
                expired_lot_count: outcome.expired_lot_count,
                expired_points_total: outcome.expired_points_total,
            },
        ),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

async fn reconcile_points_lots(
    State(state): State<BackendWalletState>,
    request_context: Extension<WebRequestContext>,
    runtime_context: Option<Extension<IamAppContext>>,
    body: axum::Json<PointsReconciliationRequest>,
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

    let query = match PointsReconciliationQuery::new(
        body.tenant_id.trim(),
        body.organization_id.as_deref(),
        body.owner_user_id.as_deref(),
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    match state.store.reconcile_points_lots(query).await {
        Ok(snapshot) => success_item(
            Some(&ctx),
            PointsReconciliationResponse {
                checked_account_count: snapshot.checked_account_count,
                mismatch_count: snapshot.mismatch_count,
                mismatches: snapshot
                    .mismatches
                    .into_iter()
                    .map(map_points_mismatch)
                    .collect(),
            },
        ),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

fn map_points_mismatch(value: PointsLotMismatchItem) -> PointsLotMismatchResponse {
    PointsLotMismatchResponse {
        account_id: value.account_id,
        available_points: value.available_points,
        lot_remaining_total: value.lot_remaining_total,
        delta: value.delta,
    }
}

fn request_hash_from_body(
    body: &CreateWalletAdjustmentRequest,
) -> Result<CommerceRequestHash, CommerceServiceError> {
    let canonical = serde_json::to_string(body).map_err(|error| {
        CommerceServiceError::validation(format!("request body is invalid: {error}"))
    })?;
    CommerceRequestHash::new(&sha256_hash(canonical.as_bytes()))
}

fn expire_request_hash_from_body(
    body: &ExpirePointsLotsRequest,
) -> Result<CommerceRequestHash, CommerceServiceError> {
    let canonical = serde_json::to_string(body).map_err(|error| {
        CommerceServiceError::validation(format!("request body is invalid: {error}"))
    })?;
    CommerceRequestHash::new(&sha256_hash(canonical.as_bytes()))
}

fn parse_asset_type(value: &str) -> Result<CommerceAccountAssetType, String> {
    match value.to_ascii_lowercase().as_str() {
        "cash" => Ok(CommerceAccountAssetType::Cash),
        "points" => Ok(CommerceAccountAssetType::Points),
        "token_bank" => Ok(CommerceAccountAssetType::TokenBank),
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn backend_wallet_response_int64_fields_serialize_as_strings() {
        let expire = serde_json::to_value(ExpirePointsLotsResponse {
            accepted: true,
            replayed: false,
            expired_lot_count: 3,
            expired_points_total: 1200,
        })
        .unwrap();

        assert_eq!(expire["expiredLotCount"], json!("3"));
        assert_eq!(expire["expiredPointsTotal"], json!("1200"));

        let reconciliation = serde_json::to_value(PointsReconciliationResponse {
            checked_account_count: 9,
            mismatch_count: 1,
            mismatches: vec![PointsLotMismatchResponse {
                account_id: "account-1".to_owned(),
                available_points: "100".to_owned(),
                lot_remaining_total: 90,
                delta: -10,
            }],
        })
        .unwrap();

        assert_eq!(reconciliation["checkedAccountCount"], json!("9"));
        assert_eq!(reconciliation["mismatchCount"], json!("1"));
        assert_eq!(
            reconciliation["mismatches"][0]["lotRemainingTotal"],
            json!("90")
        );
        assert_eq!(reconciliation["mismatches"][0]["delta"], json!("-10"));
    }
}
