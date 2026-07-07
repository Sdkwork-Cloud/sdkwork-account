import type { TokenBankBalanceData } from './token-bank-balance-data';

export interface TokenBankBalanceResponse {
  code: 0;
  data: TokenBankBalanceData;
  traceId: string;
}
