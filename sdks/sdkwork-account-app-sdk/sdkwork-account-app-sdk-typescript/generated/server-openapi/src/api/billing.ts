import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { BillingHistoryListData } from '../types';


export class BillingHistoryApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(requestOptions?: ApiRequestOptions): Promise<BillingHistoryListData> {
    return this.client.request<BillingHistoryListData>(appApiPath(`/billing/history`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'page' });
  }
}

export class BillingApi {
  public readonly history: BillingHistoryApi;

  constructor(client: HttpClient) {
    this.history = new BillingHistoryApi(client);
  }

}

export function createBillingApi(client: HttpClient): BillingApi {
  return new BillingApi(client);
}
