import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AccountSummaryItem } from '../types';


export class AccountsCurrentSummaryApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<AccountSummaryItem> {
    return this.client.get<AccountSummaryItem>(appApiPath(`/accounts/current/summary`));
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
