import type { WalletAccountItem } from './wallet-account-item';

export interface WalletOverviewResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
