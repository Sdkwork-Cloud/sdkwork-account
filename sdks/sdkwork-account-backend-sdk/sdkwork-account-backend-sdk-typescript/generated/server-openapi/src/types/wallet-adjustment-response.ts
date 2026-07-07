import type { WalletAdjustmentData } from './wallet-adjustment-data';

export interface WalletAdjustmentResponse {
  code: 0;
  data: WalletAdjustmentData;
  traceId: string;
}
