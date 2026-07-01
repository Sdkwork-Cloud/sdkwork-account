import type { AccountTransferItem } from './account-transfer-item';
import type { WalletAccountItem } from './wallet-account-item';
import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletTransferMutationResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
