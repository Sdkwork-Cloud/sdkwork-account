use sdkwork_contract_service::{
    assert_status_transition, validate_commerce_context, CapabilityFlag, CommerceAccountAssetType,
    CommerceExchangeStatus, CommerceIdempotencyRecord, CommerceLedgerDirection, CommerceMoney,
    CommercePaymentStatus, CommercePoints, CommerceRechargeStatus, CommerceRequestHash,
    CommerceRuntimeContext, CommerceRuntimeContextInput, CommerceServiceContract,
    CommerceServiceError, CommerceStatusMachine, CommerceSurfaceProfile, DeploymentMode,
    Environment, IdempotencyDecision, IdempotencyRepositoryCommand, OperationExecutionPolicy,
    PromotionCouponStatus, TransactionBoundaryKind,
};

#[test]
fn validates_commerce_runtime_context_for_private_deployments() {
    let context = CommerceRuntimeContext::new(CommerceRuntimeContextInput {
        tenant_id: "100001".to_string(),
        organization_id: Some("300001".to_string()),
        user_id: "30".to_string(),
        session_id: "session-1".to_string(),
        app_id: "sdkwork-router".to_string(),
        deployment_mode: DeploymentMode::Private,
        environment: Environment::Production,
        surface_profile: CommerceSurfaceProfile::Console,
    });

    assert_eq!(context.tenant_id, "100001");
    assert_eq!(context.organization_id.as_deref(), Some("300001"));
    assert_eq!(context.user_id, "30");
    assert_eq!(validate_commerce_context(&context), Ok(()));
}

#[test]
fn rejects_contexts_without_required_tenant_or_user_identity() {
    let context = CommerceRuntimeContext::new(CommerceRuntimeContextInput {
        tenant_id: String::new(),
        organization_id: None,
        user_id: String::new(),
        session_id: "session-1".to_string(),
        app_id: "sdkwork-router".to_string(),
        deployment_mode: DeploymentMode::Local,
        environment: Environment::Test,
        surface_profile: CommerceSurfaceProfile::App,
    });

    assert_eq!(
        validate_commerce_context(&context),
        Err("tenant_id is required")
    );
}

#[test]
fn exposes_standard_asset_and_ledger_direction_names() {
    assert_eq!(CommerceAccountAssetType::Cash.as_str(), "cash");
    assert_eq!(CommerceAccountAssetType::Points.as_str(), "points");
    assert_eq!(CommerceAccountAssetType::Token.as_str(), "token");
    assert_eq!(CommerceLedgerDirection::Credit.as_str(), "credit");
    assert_eq!(CommerceLedgerDirection::Debit.as_str(), "debit");
}

#[test]
fn exposes_standard_promotion_recharge_payment_and_exchange_status_names() {
    assert_eq!(PromotionCouponStatus::Draft.as_str(), "draft");
    assert_eq!(PromotionCouponStatus::Active.as_str(), "active");
    assert_eq!(PromotionCouponStatus::Redeemed.as_str(), "redeemed");
    assert_eq!(PromotionCouponStatus::Expired.as_str(), "expired");
    assert_eq!(PromotionCouponStatus::Disabled.as_str(), "disabled");

    assert_eq!(CommerceRechargeStatus::Pending.as_str(), "pending");
    assert_eq!(CommerceRechargeStatus::Paid.as_str(), "paid");
    assert_eq!(CommerceRechargeStatus::Fulfilled.as_str(), "fulfilled");
    assert_eq!(CommerceRechargeStatus::Closed.as_str(), "closed");

    assert_eq!(CommercePaymentStatus::Pending.as_str(), "pending");
    assert_eq!(CommercePaymentStatus::Succeeded.as_str(), "succeeded");
    assert_eq!(CommercePaymentStatus::Failed.as_str(), "failed");
    assert_eq!(CommercePaymentStatus::Canceled.as_str(), "canceled");

    assert_eq!(CommerceExchangeStatus::Pending.as_str(), "pending");
    assert_eq!(CommerceExchangeStatus::Succeeded.as_str(), "succeeded");
    assert_eq!(CommerceExchangeStatus::Failed.as_str(), "failed");
}

#[test]
fn validates_money_and_points_amount_precision() {
    assert_eq!(CommerceMoney::new("19.90").unwrap().as_str(), "19.90");
    assert!(CommerceMoney::new("19.999").is_err());
    assert!(CommerceMoney::new("-1").is_err());

    assert_eq!(CommercePoints::new("1000").unwrap().as_str(), "1000");
    assert!(CommercePoints::new("1.5").is_err());
    assert!(CommercePoints::new("-1").is_err());
}

#[test]
fn exposes_surface_profiles_for_app_console_and_admin() {
    assert_eq!(CommerceSurfaceProfile::App.as_str(), "app");
    assert_eq!(CommerceSurfaceProfile::Console.as_str(), "console");
    assert_eq!(CommerceSurfaceProfile::Admin.as_str(), "admin");
}

#[test]
fn exposes_standard_service_error_codes() {
    assert_eq!(
        CommerceServiceError::invalid_state("bad transition").code(),
        "invalid-state"
    );
    assert_eq!(
        CommerceServiceError::unsupported_capability("payment").code(),
        "unsupported-capability"
    );
    assert_eq!(
        CommerceServiceError::provider_unavailable("wechat").code(),
        "provider-unavailable"
    );
    assert_eq!(
        CommerceServiceError::validation("amount").message(),
        "amount"
    );
}

#[test]
fn detects_idempotency_replay_and_hash_conflicts() {
    let request_hash = CommerceRequestHash::new("hash-a").unwrap();
    let replay = CommerceIdempotencyRecord::completed(
        "100001",
        "payment.create",
        "idem-1",
        request_hash.clone(),
        "{\"paymentId\":\"pay-1\"}",
    );

    assert_eq!(replay.decide(&request_hash), IdempotencyDecision::Replay);
    assert_eq!(
        replay.decide(&CommerceRequestHash::new("hash-b").unwrap()),
        IdempotencyDecision::Conflict,
    );
}

#[test]
fn exposes_domain_capability_flags() {
    let flag = CapabilityFlag::new("commerce.payment.intent", true).unwrap();

    assert_eq!(flag.name(), "commerce.payment.intent");
    assert!(flag.enabled());
    assert!(CapabilityFlag::new("", true).is_err());
}

#[test]
fn validates_service_contract_metadata_for_reusable_domain_services() {
    let contract = CommerceServiceContract::new(
        "account",
        "commerce.account",
        vec!["ledger.append", "prehold.create"],
        vec!["summary.retrieve", "ledger.list"],
        vec!["account.repository", "idempotency.repository"],
        true,
    );

    assert_eq!(contract.domain, "account");
    assert_eq!(contract.service_name, "commerce.account");
    assert!(contract.requires_idempotency_for_writes);
    assert_eq!(contract.validate(), Ok(()));
    assert!(
        CommerceServiceContract::new("", "commerce.account", vec![], vec![], vec![], true)
            .validate()
            .is_err()
    );
}

#[test]
fn validates_operation_contracts_for_runtime_execution_boundary() {
    let read_contract = sdkwork_contract_service::CommerceOperationContract::new(
        "account.summary.retrieve",
        "commerce.account",
        OperationExecutionPolicy::ReadOnly,
        "commerce.account.summary",
    );
    let write_contract = sdkwork_contract_service::CommerceOperationContract::new(
        "orders.create",
        "commerce.order",
        OperationExecutionPolicy::TransactionalWrite,
        "commerce.order.lifecycle",
    );

    assert_eq!(read_contract.validate(), Ok(()));
    assert_eq!(write_contract.validate(), Ok(()));
    assert!(!read_contract.requires_transaction());
    assert!(!read_contract.requires_idempotency());
    assert!(write_contract.requires_transaction());
    assert!(write_contract.requires_idempotency());
    assert_eq!(write_contract.capability_name, "commerce.order.lifecycle");
}

#[test]
fn rejects_invalid_operation_contracts_before_transport_binding() {
    let invalid = sdkwork_contract_service::CommerceOperationContract::new(
        "",
        "order",
        OperationExecutionPolicy::TransactionalWrite,
        "",
    );

    let error = invalid.validate().unwrap_err();

    assert_eq!(error.code(), "validation");
    assert_eq!(error.message(), "operation_id is required");
}

#[test]
fn validates_operation_request_context_before_service_execution() {
    let contract = sdkwork_contract_service::CommerceOperationContract::new(
        "orders.create",
        "commerce.order",
        OperationExecutionPolicy::TransactionalWrite,
        "commerce.order.lifecycle",
    );
    let context = CommerceRuntimeContext::new(CommerceRuntimeContextInput {
        tenant_id: "100001".to_string(),
        organization_id: Some("300001".to_string()),
        user_id: "30".to_string(),
        session_id: "session-1".to_string(),
        app_id: "sdkwork-router".to_string(),
        deployment_mode: DeploymentMode::Private,
        environment: Environment::Production,
        surface_profile: CommerceSurfaceProfile::App,
    });
    let request = sdkwork_contract_service::CommerceOperationRequest::new(
        context,
        contract.clone(),
        Some("idem-1"),
        Some(CommerceRequestHash::new("hash-1").unwrap()),
    );

    assert_eq!(request.validate_for_execution(), Ok(()));
    assert_eq!(request.idempotency_scope(), "orders.create");

    let missing_idempotency = sdkwork_contract_service::CommerceOperationRequest::new(
        request.context().clone(),
        contract,
        None,
        Some(CommerceRequestHash::new("hash-1").unwrap()),
    );

    assert_eq!(
        missing_idempotency.validate_for_execution(),
        Err(CommerceServiceError::validation(
            "idempotency_key is required for write operation",
        )),
    );
}

#[test]
fn rejects_operation_execution_when_required_capability_is_disabled() {
    let contract = sdkwork_contract_service::CommerceOperationContract::new(
        "payments.intents.create",
        "commerce.payment",
        OperationExecutionPolicy::TransactionalWrite,
        "commerce.payment.intent",
    );
    let disabled = vec![CapabilityFlag::new("commerce.payment.intent", false).unwrap()];
    let enabled = vec![CapabilityFlag::new("commerce.payment.intent", true).unwrap()];

    assert_eq!(
        contract.ensure_capability_enabled(&disabled),
        Err(CommerceServiceError::unsupported_capability(
            "required commerce capability is disabled",
        )),
    );
    assert_eq!(contract.ensure_capability_enabled(&enabled), Ok(()));
}

#[test]
fn exposes_standard_idempotency_repository_commands_and_records() {
    assert_eq!(
        IdempotencyRepositoryCommand::standard_commands(),
        vec![
            IdempotencyRepositoryCommand::Find,
            IdempotencyRepositoryCommand::Lock,
            IdempotencyRepositoryCommand::Complete,
            IdempotencyRepositoryCommand::Fail,
        ],
    );

    let request_hash = CommerceRequestHash::new("hash-lock-1").unwrap();
    let locked = CommerceIdempotencyRecord::locked(
        "100001",
        "orders.create",
        "idem-1",
        request_hash.clone(),
    );
    assert_eq!(
        locked.decide(&request_hash),
        IdempotencyDecision::InProgress
    );
    let failed = locked.clone().mark_failed();
    let completed = locked.mark_completed("{\"orderId\":\"order-1\"}");

    assert_eq!(
        failed.status,
        sdkwork_contract_service::IdempotencyStatus::Failed
    );
    assert_eq!(failed.decide(&request_hash), IdempotencyDecision::Execute);
    assert_eq!(
        completed.status,
        sdkwork_contract_service::IdempotencyStatus::Completed
    );
    assert_eq!(
        completed.response_json.as_deref(),
        Some("{\"orderId\":\"order-1\"}")
    );
    assert_eq!(completed.decide(&request_hash), IdempotencyDecision::Replay);
}

#[test]
fn exposes_standard_transaction_boundary_contracts() {
    let scope = sdkwork_contract_service::CommerceTransactionScope::new(
        "payments.intents.create",
        "commerce.payment",
        TransactionBoundaryKind::Required,
    )
    .unwrap();

    assert_eq!(scope.operation_id(), "payments.intents.create");
    assert_eq!(scope.service_name(), "commerce.payment");
    assert!(scope.requires_transaction());
    assert!(
        sdkwork_contract_service::CommerceTransactionScope::new(
            "",
            "commerce.payment",
            TransactionBoundaryKind::Required,
        )
        .is_err()
    );
}

#[test]
fn validates_status_transitions_without_embedding_domain_logic_in_transports() {
    assert_eq!(
        assert_status_transition(CommerceStatusMachine::Order, "pending_payment", "paid",),
        Ok(()),
    );
    assert_eq!(
        assert_status_transition(CommerceStatusMachine::Payment, "succeeded", "pending",),
        Err(CommerceServiceError::invalid_state(
            "invalid payment status transition"
        )),
    );
}
