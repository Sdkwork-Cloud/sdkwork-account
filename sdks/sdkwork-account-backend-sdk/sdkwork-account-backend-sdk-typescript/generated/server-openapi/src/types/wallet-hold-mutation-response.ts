import type { AccountHoldItem } from './account-hold-item';
import type { WalletAccountItem } from './wallet-account-item';
import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletHoldMutationResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
