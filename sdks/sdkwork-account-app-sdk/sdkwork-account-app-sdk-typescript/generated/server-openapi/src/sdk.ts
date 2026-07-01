import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkAppConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { WalletApi, createWalletApi } from './api/wallet';
import { BillingApi, createBillingApi } from './api/billing';
import { AccountsApi, createAccountsApi } from './api/accounts';

export class SdkworkAccountAppClient {
  private httpClient: HttpClient;

  public readonly wallet: WalletApi;
  public readonly billing: BillingApi;
  public readonly accounts: AccountsApi;

  constructor(config: SdkworkAppConfig) {
    this.httpClient = createHttpClient(config);
    this.wallet = createWalletApi(this.httpClient);

    this.billing = createBillingApi(this.httpClient);

    this.accounts = createAccountsApi(this.httpClient);
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

export function createClient(config: SdkworkAppConfig): SdkworkAccountAppClient {
  return new SdkworkAccountAppClient(config);
}

export default SdkworkAccountAppClient;
