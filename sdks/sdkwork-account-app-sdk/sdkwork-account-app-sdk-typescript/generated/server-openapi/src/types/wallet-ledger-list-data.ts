import type { PageInfo } from './page-info';
import type { WalletLedgerEntryItem } from './wallet-ledger-entry-item';

export interface WalletLedgerListData {
  items: WalletLedgerEntryItem[];
  pageInfo: PageInfo;
}
