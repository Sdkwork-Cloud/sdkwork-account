import type { AccountTransferItem } from './account-transfer-item';
import type { WalletAccountItem } from './wallet-account-item';
import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletTransferMutationResult {
  accepted: boolean;
  replayed: boolean;
  transfer: AccountTransferItem;
  fromAccount: WalletAccountItem;
  toAccount: WalletAccountItem;
  debitEntry: WalletLedgerEntryItem;
  creditEntry: WalletLedgerEntryItem;
}
