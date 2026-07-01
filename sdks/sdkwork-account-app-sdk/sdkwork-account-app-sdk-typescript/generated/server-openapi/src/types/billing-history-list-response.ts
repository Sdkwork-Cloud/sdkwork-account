import type { BillingHistoryItem } from './billing-history-item';
import type { PageInfo } from './page-info';

export interface BillingHistoryListResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
