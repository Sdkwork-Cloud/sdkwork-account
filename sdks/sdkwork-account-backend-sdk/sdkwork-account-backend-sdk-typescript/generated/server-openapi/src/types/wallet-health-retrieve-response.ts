import type { WalletHealthData } from './wallet-health-data';

export interface WalletHealthRetrieveResponse {
  code: 0;
  data: WalletHealthData;
  traceId: string;
}
