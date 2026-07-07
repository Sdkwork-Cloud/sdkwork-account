import type { PageInfo } from './page-info';
import type { WalletAccountItem } from './wallet-account-item';

export interface WalletAccountListData {
  items: WalletAccountItem[];
  pageInfo: PageInfo;
}
