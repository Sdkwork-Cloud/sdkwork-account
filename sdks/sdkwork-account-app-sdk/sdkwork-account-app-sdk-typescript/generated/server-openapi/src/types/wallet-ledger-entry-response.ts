import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletLedgerEntryResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
