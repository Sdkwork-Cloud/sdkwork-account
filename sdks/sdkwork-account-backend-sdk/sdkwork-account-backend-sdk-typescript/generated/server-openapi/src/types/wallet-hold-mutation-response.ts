import type { WalletHoldMutationData } from './wallet-hold-mutation-data';

export interface WalletHoldMutationResponse {
  code: 0;
  data: WalletHoldMutationData;
  traceId: string;
}
