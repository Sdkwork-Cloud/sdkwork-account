import type { WalletAccountItem } from './wallet-account-item';

export interface WalletAccountItemResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
