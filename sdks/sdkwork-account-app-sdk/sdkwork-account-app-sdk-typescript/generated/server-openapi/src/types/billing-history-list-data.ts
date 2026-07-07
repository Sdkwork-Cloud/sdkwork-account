import type { BillingHistoryItem } from './billing-history-item';
import type { PageInfo } from './page-info';

export interface BillingHistoryListData {
  items: BillingHistoryItem[];
  pageInfo: PageInfo;
}
