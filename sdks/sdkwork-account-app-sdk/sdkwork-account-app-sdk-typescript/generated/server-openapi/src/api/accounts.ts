import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { AccountSummaryItem } from '../types';


export class AccountsCurrentSummaryApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(requestOptions?: ApiRequestOptions): Promise<AccountSummaryItem> {
    return this.client.request<AccountSummaryItem>(appApiPath(`/accounts/current/summary`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export class AccountsCurrentApi {
  public readonly summary: AccountsCurrentSummaryApi;

  constructor(client: HttpClient) {
    this.summary = new AccountsCurrentSummaryApi(client);
  }

}

export class AccountsApi {
  public readonly current: AccountsCurrentApi;

  constructor(client: HttpClient) {
    this.current = new AccountsCurrentApi(client);
  }

}

export function createAccountsApi(client: HttpClient): AccountsApi {
  return new AccountsApi(client);
}
