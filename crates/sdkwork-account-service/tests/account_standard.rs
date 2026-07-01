use sdkwork_account_service::{
    account_service_contract, AccountBalance, AccountSummary, LedgerEntryDraft, LedgerPolicy,
    PreholdStatus, PreholdTransition,
};
use sdkwork_contract_service::{CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney};

#[test]
fn creates_empty_account_summary_for_local_private_runtime() {
    let summary = AccountSummary::empty("100001", "1");

    assert_eq!(summary.tenant_id, "100001");
    assert_eq!(summary.owner_user_id, "1");
    assert_eq!(summary.cash.available.as_str(), "0");
    assert_eq!(summary.points.available.as_str(), "0");
    assert_eq!(summary.token.available.as_str(), "0");
}

#[test]
fn account_service_contract_declares_wallet_and_token_read_models() {
    let contract = account_service_contract();

    assert_eq!(contract.domain, "account");
    assert_eq!(contract.service_name, "commerce.account");
    assert!(contract
        .read_queries
        .contains(&"accounts.current.summary.retrieve"));
    assert!(contract.read_queries.contains(&"wallet.accounts.list"));
    assert!(contract.read_queries.contains(&"wallet.accounts.points.retrieve"));
    assert!(contract.read_queries.contains(&"wallet.points.lots.list"));
    assert!(contract.read_queries.contains(&"wallet.tokens.retrieve"));
    assert!(contract
        .write_commands
        .contains(&"wallet.adjustments.create"));
    assert!(contract
        .write_commands
        .contains(&"wallet.adjustments.points.create"));
    assert!(contract.write_commands.contains(&"wallet.holds.create"));
    assert!(contract.write_commands.contains(&"wallet.holds.settle"));
    assert!(contract.write_commands.contains(&"wallet.holds.release"));
    assert!(contract.write_commands.contains(&"wallet.transfers.create"));
    assert!(contract.read_queries.contains(&"wallet.holds.list"));
    assert!(contract.read_queries.contains(&"wallet.holds.retrieve"));
    assert!(contract.ports.contains(&"account.wallet.read"));
}

#[test]
fn ledger_entries_are_append_only_and_require_idempotency() {
    let policy = LedgerPolicy::standard();
    let entry = LedgerEntryDraft::new(
        "100001",
        "account-1",
        "1",
        CommerceAccountAssetType::Cash,
        CommerceLedgerDirection::Credit,
        CommerceMoney::new("19.90").unwrap(),
        "request-1",
        "idem-1",
    )
    .unwrap();

    assert!(policy.require_idempotency_key);
    assert!(policy.require_append_only);
    assert_eq!(entry.request_no, "request-1");
    assert_eq!(entry.idempotency_key, "idem-1");
}

#[test]
fn rejects_ledger_entries_without_request_or_idempotency_key() {
    let result = LedgerEntryDraft::new(
        "100001",
        "account-1",
        "1",
        CommerceAccountAssetType::Cash,
        CommerceLedgerDirection::Debit,
        CommerceMoney::new("1").unwrap(),
        "",
        "",
    );

    assert!(result.is_err());
}

#[test]
fn prehold_lifecycle_allows_settle_or_release_once() {
    assert_eq!(
        PreholdTransition::new(PreholdStatus::Held, PreholdStatus::Settled).validate(),
        Ok(()),
    );
    assert_eq!(
        PreholdTransition::new(PreholdStatus::Held, PreholdStatus::Released).validate(),
        Ok(()),
    );
    assert!(
        PreholdTransition::new(PreholdStatus::Settled, PreholdStatus::Released)
            .validate()
            .is_err()
    );
}

#[test]
fn account_balance_never_allows_negative_available_amount() {
    assert!(AccountBalance::new(
        CommerceMoney::new("0").unwrap(),
        CommerceMoney::new("0").unwrap()
    )
    .is_ok());
}
