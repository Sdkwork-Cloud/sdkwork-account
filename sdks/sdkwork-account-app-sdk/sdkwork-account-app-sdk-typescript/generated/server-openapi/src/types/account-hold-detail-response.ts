import type { AccountHoldDetailData } from './account-hold-detail-data';

export interface AccountHoldDetailResponse {
  code: 0;
  data: AccountHoldDetailData;
  traceId: string;
}
