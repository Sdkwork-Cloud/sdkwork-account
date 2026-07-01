import type { AccountSummaryItem } from './account-summary-item';

export interface AccountSummaryResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
