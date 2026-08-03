import { backendApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { CreateTokenBankHoldRequest, CreateTokenBankLedgerMutationRequest, CreateTokenBankReversalRequest, CreateTokenBankTransferRequest, ReleaseAccountHoldRequest, SettleAccountHoldRequest, WalletAdjustmentResult, WalletHoldMutationResult, WalletTransferMutationResult } from '../types';


export class TokenBankTransfersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankTransferRequest, requestOptions?: ApiRequestOptions): Promise<WalletTransferMutationResult> {
    return this.client.request<WalletTransferMutationResult>(backendApiPath(`/token_bank/transfers`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class TokenBankHoldsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankHoldRequest, requestOptions?: ApiRequestOptions): Promise<WalletHoldMutationResult> {
    return this.client.request<WalletHoldMutationResult>(backendApiPath(`/token_bank/holds`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async settle(holdId: string, body: SettleAccountHoldRequest, requestOptions?: ApiRequestOptions): Promise<WalletHoldMutationResult> {
    return this.client.request<WalletHoldMutationResult>(backendApiPath(`/token_bank/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}/settle`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }

async release(holdId: string, body: ReleaseAccountHoldRequest, requestOptions?: ApiRequestOptions): Promise<WalletHoldMutationResult> {
    return this.client.request<WalletHoldMutationResult>(backendApiPath(`/token_bank/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}/release`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class TokenBankReversalsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankReversalRequest, requestOptions?: ApiRequestOptions): Promise<WalletAdjustmentResult> {
    return this.client.request<WalletAdjustmentResult>(backendApiPath(`/token_bank/reversals`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class TokenBankGrantsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankLedgerMutationRequest, requestOptions?: ApiRequestOptions): Promise<WalletAdjustmentResult> {
    return this.client.request<WalletAdjustmentResult>(backendApiPath(`/token_bank/grants`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class TokenBankDebitsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankLedgerMutationRequest, requestOptions?: ApiRequestOptions): Promise<WalletAdjustmentResult> {
    return this.client.request<WalletAdjustmentResult>(backendApiPath(`/token_bank/debits`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class TokenBankCreditsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankLedgerMutationRequest, requestOptions?: ApiRequestOptions): Promise<WalletAdjustmentResult> {
    return this.client.request<WalletAdjustmentResult>(backendApiPath(`/token_bank/credits`), { signal: requestOptions?.signal, timeout: requestOptions?.timeout, method: 'POST' as any, body, contentType: 'application/json', sdkworkUnwrapKind: 'item' });
  }
}

export class TokenBankApi {
  private client: HttpClient;
  public readonly credits: TokenBankCreditsApi;
  public readonly debits: TokenBankDebitsApi;
  public readonly grants: TokenBankGrantsApi;
  public readonly reversals: TokenBankReversalsApi;
  public readonly holds: TokenBankHoldsApi;
  public readonly transfers: TokenBankTransfersApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.credits = new TokenBankCreditsApi(client);
    this.debits = new TokenBankDebitsApi(client);
    this.grants = new TokenBankGrantsApi(client);
    this.reversals = new TokenBankReversalsApi(client);
    this.holds = new TokenBankHoldsApi(client);
    this.transfers = new TokenBankTransfersApi(client);
  }

}

export function createTokenBankApi(client: HttpClient): TokenBankApi {
  return new TokenBankApi(client);
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
