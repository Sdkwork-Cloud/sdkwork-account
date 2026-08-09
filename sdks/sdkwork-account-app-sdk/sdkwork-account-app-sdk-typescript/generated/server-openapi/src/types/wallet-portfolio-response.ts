import type { WalletPortfolioData } from './wallet-portfolio-data';

export interface WalletPortfolioResponse {
  code: 0;
  data: WalletPortfolioData;
  traceId: string;
}
