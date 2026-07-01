import type { CashAccountItem } from './cash-account-item';

export interface CashAccountResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
