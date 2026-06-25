use sdkwork_commerce_contract_service::CommerceServiceContract;

pub fn account_service_contract() -> CommerceServiceContract {
    CommerceServiceContract::new(
        "account",
        "commerce.account",
        vec!["wallet.adjustments.create"],
        vec![
            "accounts.current.summary.retrieve",
            "wallet.overview.retrieve",
            "wallet.accounts.list",
            "wallet.ledgerEntries.list",
            "wallet.ledgerEntries.retrieve",
            "wallet.tokens.retrieve",
        ],
        vec![
            crate::ports::ACCOUNT_REPOSITORY_PORT,
            crate::ports::ACCOUNT_WALLET_READ_PORT,
            crate::ports::ACCOUNT_LEDGER_WRITE_PORT,
            crate::ports::IDEMPOTENCY_REPOSITORY_PORT,
        ],
        true,
    )
}
