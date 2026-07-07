import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CreateTokenBankHoldRequest, CreateTokenBankLedgerMutationRequest, CreateTokenBankReversalRequest, CreateTokenBankTransferRequest, ReleaseAccountHoldRequest, SettleAccountHoldRequest, WalletAdjustmentResult, WalletHoldMutationResult, WalletTransferMutationResult } from '../types';


export class TokenBankTransfersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankTransferRequest): Promise<WalletTransferMutationResult> {
    return this.client.post<WalletTransferMutationResult>(backendApiPath(`/token_bank/transfers`), body, undefined, undefined, 'application/json');
  }
}

export class TokenBankHoldsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankHoldRequest): Promise<WalletHoldMutationResult> {
    return this.client.post<WalletHoldMutationResult>(backendApiPath(`/token_bank/holds`), body, undefined, undefined, 'application/json');
  }

async settle(holdId: string, body: SettleAccountHoldRequest): Promise<WalletHoldMutationResult> {
    return this.client.post<WalletHoldMutationResult>(backendApiPath(`/token_bank/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}/settle`), body, undefined, undefined, 'application/json');
  }

async release(holdId: string, body: ReleaseAccountHoldRequest): Promise<WalletHoldMutationResult> {
    return this.client.post<WalletHoldMutationResult>(backendApiPath(`/token_bank/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}/release`), body, undefined, undefined, 'application/json');
  }
}

export class TokenBankReversalsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankReversalRequest): Promise<WalletAdjustmentResult> {
    return this.client.post<WalletAdjustmentResult>(backendApiPath(`/token_bank/reversals`), body, undefined, undefined, 'application/json');
  }
}

export class TokenBankGrantsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankLedgerMutationRequest): Promise<WalletAdjustmentResult> {
    return this.client.post<WalletAdjustmentResult>(backendApiPath(`/token_bank/grants`), body, undefined, undefined, 'application/json');
  }
}

export class TokenBankDebitsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankLedgerMutationRequest): Promise<WalletAdjustmentResult> {
    return this.client.post<WalletAdjustmentResult>(backendApiPath(`/token_bank/debits`), body, undefined, undefined, 'application/json');
  }
}

export class TokenBankCreditsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: CreateTokenBankLedgerMutationRequest): Promise<WalletAdjustmentResult> {
    return this.client.post<WalletAdjustmentResult>(backendApiPath(`/token_bank/credits`), body, undefined, undefined, 'application/json');
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
