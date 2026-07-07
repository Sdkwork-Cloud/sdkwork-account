import type { AccountSummaryData } from './account-summary-data';

export interface AccountSummaryResponse {
  code: 0;
  data: AccountSummaryData;
  traceId: string;
}
