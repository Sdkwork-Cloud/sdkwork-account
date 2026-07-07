import type { AccountHoldItem } from './account-hold-item';
import type { WalletAccountItem } from './wallet-account-item';
import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletHoldMutationResult {
  accepted: boolean;
  replayed: boolean;
  hold: AccountHoldItem;
  account: WalletAccountItem;
  ledgerEntry?: WalletLedgerEntryItem;
}
