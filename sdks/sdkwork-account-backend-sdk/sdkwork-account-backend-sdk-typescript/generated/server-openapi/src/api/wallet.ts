import { backendApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { CreateAccountHoldRequest, CreateAccountTransferRequest, CreateWalletAdjustmentRequest, DispatchOutboxRequest, DispatchOutboxResult, ExpireExpiredHoldsRequest, ExpireExpiredHoldsResult, ExpirePointsLotsRequest, ExpirePointsLotsResult, PointsReconciliationRequest, PointsReconciliationResult, ReleaseAccountHoldRequest, SettleAccountHoldRequest, WalletAdjustmentResult, WalletHealthItem, WalletHoldMutationResult, WalletTransferMutationResult } from '../types';


export class WalletTransfersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateAccountTransferRequest, requestOptions?: ApiRequestOptions): Promise<WalletTransferMutationResult> {
    return this.client.request<WalletTransferMutationResult>(backendApiPath(`/wallet/transfers`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class WalletHoldsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateAccountHoldRequest, requestOptions?: ApiRequestOptions): Promise<WalletHoldMutationResult> {
    return this.client.request<WalletHoldMutationResult>(backendApiPath(`/wallet/holds`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async settle(holdId: string, body: SettleAccountHoldRequest, requestOptions?: ApiRequestOptions): Promise<WalletHoldMutationResult> {
    return this.client.request<WalletHoldMutationResult>(backendApiPath(`/wallet/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}/settle`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async release(holdId: string, body: ReleaseAccountHoldRequest, requestOptions?: ApiRequestOptions): Promise<WalletHoldMutationResult> {
    return this.client.request<WalletHoldMutationResult>(backendApiPath(`/wallet/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}/release`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async expire(body: ExpireExpiredHoldsRequest, requestOptions?: ApiRequestOptions): Promise<ExpireExpiredHoldsResult> {
    return this.client.request<ExpireExpiredHoldsResult>(backendApiPath(`/wallet/holds/expire`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class WalletPointsLotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async expire(body: ExpirePointsLotsRequest, requestOptions?: ApiRequestOptions): Promise<ExpirePointsLotsResult> {
    return this.client.request<ExpirePointsLotsResult>(backendApiPath(`/wallet/points/lots/expire`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class WalletPointsApi {
  private client: HttpClient;
  public readonly lots: WalletPointsLotsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.lots = new WalletPointsLotsApi(client);
  }


async reconciliation(body: PointsReconciliationRequest, requestOptions?: ApiRequestOptions): Promise<PointsReconciliationResult> {
    return this.client.request<PointsReconciliationResult>(backendApiPath(`/wallet/points/reconciliation`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class WalletAdjustmentsPointsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateWalletAdjustmentRequest, requestOptions?: ApiRequestOptions): Promise<WalletAdjustmentResult> {
    return this.client.request<WalletAdjustmentResult>(backendApiPath(`/wallet/adjustments/points`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class WalletAdjustmentsCashApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateWalletAdjustmentRequest, requestOptions?: ApiRequestOptions): Promise<WalletAdjustmentResult> {
    return this.client.request<WalletAdjustmentResult>(backendApiPath(`/wallet/adjustments/cash`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class WalletAdjustmentsApi {
  private client: HttpClient;
  public readonly cash: WalletAdjustmentsCashApi;
  public readonly points: WalletAdjustmentsPointsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.cash = new WalletAdjustmentsCashApi(client);
    this.points = new WalletAdjustmentsPointsApi(client);
  }


async create(body: CreateWalletAdjustmentRequest, requestOptions?: ApiRequestOptions): Promise<WalletAdjustmentResult> {
    return this.client.request<WalletAdjustmentResult>(backendApiPath(`/wallet/adjustments`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class WalletOutboxApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async dispatch(body?: DispatchOutboxRequest, requestOptions?: ApiRequestOptions): Promise<DispatchOutboxResult> {
    return this.client.request<DispatchOutboxResult>(backendApiPath(`/wallet/outbox/dispatch`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'POST' as any, ...(body !== undefined ? { body, contentType: 'application/json' } : {}), sdkworkUnwrapKind: 'item' });
  }
}

export class WalletHealthApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(requestOptions?: ApiRequestOptions): Promise<WalletHealthItem> {
    return this.client.request<WalletHealthItem>(backendApiPath(`/wallet/health`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export class WalletApi {
  public readonly health: WalletHealthApi;
  public readonly outbox: WalletOutboxApi;
  public readonly adjustments: WalletAdjustmentsApi;
  public readonly points: WalletPointsApi;
  public readonly holds: WalletHoldsApi;
  public readonly transfers: WalletTransfersApi;

  constructor(client: HttpClient) {
    this.health = new WalletHealthApi(client);
    this.outbox = new WalletOutboxApi(client);
    this.adjustments = new WalletAdjustmentsApi(client);
    this.points = new WalletPointsApi(client);
    this.holds = new WalletHoldsApi(client);
    this.transfers = new WalletTransfersApi(client);
  }

}

export function createWalletApi(client: HttpClient): WalletApi {
  return new WalletApi(client);
}



interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
