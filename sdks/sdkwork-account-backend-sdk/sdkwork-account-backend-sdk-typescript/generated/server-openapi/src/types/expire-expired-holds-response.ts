import type { ExpireExpiredHoldsData } from './expire-expired-holds-data';

export interface ExpireExpiredHoldsResponse {
  code: 0;
  data: ExpireExpiredHoldsData;
  traceId: string;
}
