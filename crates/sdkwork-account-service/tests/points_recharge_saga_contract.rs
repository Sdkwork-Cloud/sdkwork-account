use sdkwork_account_service::AppendLedgerEntryCommand;
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney,
};
use sdkwork_order_service::{
    points_recharge_fulfillment_idempotency_key, points_recharge_fulfillment_transaction_no,
    POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
};

#[test]
fn points_recharge_saga_uses_account_backend_adjustment_contract() {
    let order_id = "order-1001";
    let command = AppendLedgerEntryCommand::new(
        "100001",
        Some("0"),
        "",
        "1",
        CommerceAccountAssetType::Points,
        Some("POINT"),
        CommerceLedgerDirection::Credit,
        CommerceMoney::new("500").expect("money"),
        POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
        &points_recharge_fulfillment_transaction_no(order_id),
        "req-fulfill-1001",
        &points_recharge_fulfillment_idempotency_key(order_id),
    )
    .expect("ledger command");

    assert_eq!(command.business_type, "points_recharge");
    assert_eq!(
        command.idempotency_key,
        "points-recharge:fulfill:order-1001"
    );
    assert_eq!(command.transaction_no, "points-recharge:order-1001");
}
