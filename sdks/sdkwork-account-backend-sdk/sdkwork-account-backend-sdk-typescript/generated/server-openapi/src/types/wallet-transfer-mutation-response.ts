import type { WalletTransferMutationData } from './wallet-transfer-mutation-data';

export interface WalletTransferMutationResponse {
  code: 0;
  data: WalletTransferMutationData;
  traceId: string;
}
