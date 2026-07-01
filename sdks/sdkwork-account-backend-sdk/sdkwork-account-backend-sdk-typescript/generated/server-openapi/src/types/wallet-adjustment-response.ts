import type { WalletAccountItem } from './wallet-account-item';
import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletAdjustmentResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
