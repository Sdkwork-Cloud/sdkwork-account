import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { BillingHistoryListData } from '../types';


export class BillingHistoryApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(requestOptions?: ApiRequestOptions): Promise<BillingHistoryListData> {
    return this.client.request<BillingHistoryListData>(appApiPath(`/billing/history`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export class BillingApi {
  private client: HttpClient;
  public readonly history: BillingHistoryApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.history = new BillingHistoryApi(client);
  }

}

export function createBillingApi(client: HttpClient): BillingApi {
  return new BillingApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}
