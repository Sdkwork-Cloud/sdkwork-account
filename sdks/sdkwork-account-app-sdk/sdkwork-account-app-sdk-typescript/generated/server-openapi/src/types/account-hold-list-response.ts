import type { AccountHoldItem } from './account-hold-item';
import type { PageInfo } from './page-info';

export interface AccountHoldListResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
