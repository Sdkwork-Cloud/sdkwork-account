use crate::{
    AccountLedgerQuery, AccountSummary, AccountSummaryQuery, AppendLedgerEntryCommand,
    AppendLedgerEntryOutcome, BillingHistoryItem, BillingHistoryListQuery, LedgerEntryDraft,
    WalletAccountItem, WalletAccountListQuery, WalletOperation, WalletOperationQuery,
    WalletOverview, WalletTransactionDetailQuery, WalletTransactionItem,
    WalletTransactionListQuery,
};
use sdkwork_contract_service::CommerceRequestHash;
use sdkwork_contract_service::CommerceServiceError;

pub trait AccountRepositoryPort {
    fn retrieve_summary(
        &self,
        query: &AccountSummaryQuery,
    ) -> Result<AccountSummary, CommerceServiceError>;

    fn append_ledger_entry(&self, draft: &LedgerEntryDraft) -> Result<(), CommerceServiceError>;
}

pub trait AccountWalletReadPort {
    fn retrieve_summary(
        &self,
        query: &AccountLedgerQuery,
    ) -> Result<AccountSummary, CommerceServiceError>;

    fn list_wallet_accounts(
        &self,
        query: &WalletAccountListQuery,
    ) -> Result<Vec<WalletAccountItem>, CommerceServiceError>;

    fn retrieve_wallet_overview(
        &self,
        query: &WalletAccountListQuery,
    ) -> Result<WalletOverview, CommerceServiceError>;

    fn list_wallet_transactions(
        &self,
        query: &WalletTransactionListQuery,
    ) -> Result<Vec<WalletTransactionItem>, CommerceServiceError>;

    fn retrieve_wallet_transaction(
        &self,
        query: &WalletTransactionDetailQuery,
    ) -> Result<Option<WalletTransactionItem>, CommerceServiceError>;

    fn retrieve_wallet_operation(
        &self,
        query: &WalletOperationQuery,
    ) -> Result<Option<WalletOperation>, CommerceServiceError>;
}

pub trait AccountLedgerWritePort {
    fn append_ledger_entry(
        &self,
        command: &AppendLedgerEntryCommand,
        request_hash: &CommerceRequestHash,
    ) -> Result<AppendLedgerEntryOutcome, CommerceServiceError>;
}

pub trait BillingHistoryReadPort {
    fn list_billing_history(
        &self,
        query: &BillingHistoryListQuery,
    ) -> Result<Vec<BillingHistoryItem>, CommerceServiceError>;
}

pub const ACCOUNT_REPOSITORY_PORT: &str = "account.repository";
pub const ACCOUNT_WALLET_READ_PORT: &str = "account.wallet.read";
pub const ACCOUNT_LEDGER_WRITE_PORT: &str = "account.ledger.write";
pub const BILLING_HISTORY_READ_PORT: &str = "billing.history.read";
pub const IDEMPOTENCY_REPOSITORY_PORT: &str = "idempotency.repository";
