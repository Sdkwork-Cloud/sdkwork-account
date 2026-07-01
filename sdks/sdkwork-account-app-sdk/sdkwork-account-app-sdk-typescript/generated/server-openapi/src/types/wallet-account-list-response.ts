import type { PageInfo } from './page-info';
import type { WalletAccountItem } from './wallet-account-item';

export interface WalletAccountListResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
