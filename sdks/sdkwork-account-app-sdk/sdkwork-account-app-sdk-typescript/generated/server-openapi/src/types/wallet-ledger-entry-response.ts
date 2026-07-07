import type { WalletLedgerEntryData } from './wallet-ledger-entry-data';

export interface WalletLedgerEntryResponse {
  code: 0;
  data: WalletLedgerEntryData;
  traceId: string;
}
