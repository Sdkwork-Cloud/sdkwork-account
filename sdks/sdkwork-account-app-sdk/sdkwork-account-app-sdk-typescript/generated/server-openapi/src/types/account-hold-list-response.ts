import type { AccountHoldListData } from './account-hold-list-data';

export interface AccountHoldListResponse {
  code: 0;
  data: AccountHoldListData;
  traceId: string;
}
