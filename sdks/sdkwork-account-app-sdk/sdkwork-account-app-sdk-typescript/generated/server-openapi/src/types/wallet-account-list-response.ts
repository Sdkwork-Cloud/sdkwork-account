import type { WalletAccountListData } from './wallet-account-list-data';

export interface WalletAccountListResponse {
  code: 0;
  data: WalletAccountListData;
  traceId: string;
}
