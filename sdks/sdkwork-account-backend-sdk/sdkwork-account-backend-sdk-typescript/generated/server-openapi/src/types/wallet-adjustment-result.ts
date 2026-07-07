import type { WalletAccountItem } from './wallet-account-item';
import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletAdjustmentResult {
  accepted: boolean;
  replayed: boolean;
  account: WalletAccountItem;
  ledgerEntry: WalletLedgerEntryItem;
}
