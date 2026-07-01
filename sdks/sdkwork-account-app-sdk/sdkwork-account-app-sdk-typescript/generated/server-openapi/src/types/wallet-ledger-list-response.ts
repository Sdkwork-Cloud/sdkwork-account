import type { PageInfo } from './page-info';
import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletLedgerListResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
