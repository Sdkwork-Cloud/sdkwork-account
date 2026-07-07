import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkBackendConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { WalletApi, createWalletApi } from './api/wallet';
import { TokenBankApi, createTokenBankApi } from './api/token-bank';

export class SdkworkAccountBackendClient {
  private httpClient: HttpClient;

  public readonly wallet: WalletApi;
  public readonly tokenBank: TokenBankApi;

  constructor(config: SdkworkBackendConfig) {
    this.httpClient = createHttpClient(config);
    this.wallet = createWalletApi(this.httpClient);

    this.tokenBank = createTokenBankApi(this.httpClient);
  }
  setAuthToken(token: string): this {
    this.httpClient.setAuthToken(token);
    return this;
  }

  setAccessToken(token: string): this {
    this.httpClient.setAccessToken(token);
    return this;
  }

  setTokenManager(manager: AuthTokenManager): this {
    this.httpClient.setTokenManager(manager);
    return this;
  }

  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkBackendConfig): SdkworkAccountBackendClient {
  return new SdkworkAccountBackendClient(config);
}

export default SdkworkAccountBackendClient;
