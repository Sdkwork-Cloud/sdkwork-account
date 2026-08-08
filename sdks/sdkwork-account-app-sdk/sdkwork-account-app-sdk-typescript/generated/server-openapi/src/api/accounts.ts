import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { AccountSummaryItem } from '../types';


export class AccountsCurrentSummaryApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(requestOptions?: ApiRequestOptions): Promise<AccountSummaryItem> {
    return this.client.request<AccountSummaryItem>(appApiPath(`/accounts/current/summary`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export class AccountsCurrentApi {
  private client: HttpClient;
  public readonly summary: AccountsCurrentSummaryApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.summary = new AccountsCurrentSummaryApi(client);
  }

}

export class AccountsApi {
  private client: HttpClient;
  public readonly current: AccountsCurrentApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.current = new AccountsCurrentApi(client);
  }

}

export function createAccountsApi(client: HttpClient): AccountsApi {
  return new AccountsApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}
