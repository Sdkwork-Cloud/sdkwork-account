import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AccountHoldItem, AccountTransferItem, CreateAccountHoldRequest, CreateAccountTransferRequest, CreateWalletAdjustmentRequest, ReleaseAccountHoldRequest, SettleAccountHoldRequest, WalletAccountItem, WalletLedgerEntryItem } from '../types';


export class WalletTransfersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateAccountTransferRequest): Promise<Record<string, unknown>> {
    return this.client.post<Record<string, unknown>>(backendApiPath(`/wallet/transfers`), body, undefined, undefined, 'application/json');
  }
}

export class WalletHoldsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateAccountHoldRequest): Promise<Record<string, unknown>> {
    return this.client.post<Record<string, unknown>>(backendApiPath(`/wallet/holds`), body, undefined, undefined, 'application/json');
  }

async settle(holdId: string, body: SettleAccountHoldRequest): Promise<Record<string, unknown>> {
    return this.client.post<Record<string, unknown>>(backendApiPath(`/wallet/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}/settle`), body, undefined, undefined, 'application/json');
  }

async release(holdId: string, body: ReleaseAccountHoldRequest): Promise<Record<string, unknown>> {
    return this.client.post<Record<string, unknown>>(backendApiPath(`/wallet/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}/release`), body, undefined, undefined, 'application/json');
  }
}

export class WalletAdjustmentsTokensApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateWalletAdjustmentRequest): Promise<Record<string, unknown>> {
    return this.client.post<Record<string, unknown>>(backendApiPath(`/wallet/adjustments/tokens`), body, undefined, undefined, 'application/json');
  }
}

export class WalletAdjustmentsPointsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateWalletAdjustmentRequest): Promise<Record<string, unknown>> {
    return this.client.post<Record<string, unknown>>(backendApiPath(`/wallet/adjustments/points`), body, undefined, undefined, 'application/json');
  }
}

export class WalletAdjustmentsCashApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateWalletAdjustmentRequest): Promise<Record<string, unknown>> {
    return this.client.post<Record<string, unknown>>(backendApiPath(`/wallet/adjustments/cash`), body, undefined, undefined, 'application/json');
  }
}

export class WalletAdjustmentsApi {
  private client: HttpClient;
  public readonly cash: WalletAdjustmentsCashApi;
  public readonly points: WalletAdjustmentsPointsApi;
  public readonly tokens: WalletAdjustmentsTokensApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.cash = new WalletAdjustmentsCashApi(client);
    this.points = new WalletAdjustmentsPointsApi(client);
    this.tokens = new WalletAdjustmentsTokensApi(client);
  }


async create(body: CreateWalletAdjustmentRequest): Promise<Record<string, unknown>> {
    return this.client.post<Record<string, unknown>>(backendApiPath(`/wallet/adjustments`), body, undefined, undefined, 'application/json');
  }
}

export class WalletHealthApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(backendApiPath(`/wallet/health`));
  }
}

export class WalletApi {
  private client: HttpClient;
  public readonly health: WalletHealthApi;
  public readonly adjustments: WalletAdjustmentsApi;
  public readonly holds: WalletHoldsApi;
  public readonly transfers: WalletTransfersApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.health = new WalletHealthApi(client);
    this.adjustments = new WalletAdjustmentsApi(client);
    this.holds = new WalletHoldsApi(client);
    this.transfers = new WalletTransfersApi(client);
  }

}

export function createWalletApi(client: HttpClient): WalletApi {
  return new WalletApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
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
