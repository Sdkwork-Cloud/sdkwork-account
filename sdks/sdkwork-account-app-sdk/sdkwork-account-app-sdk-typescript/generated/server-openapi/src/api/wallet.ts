import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AccountHoldItem, CashAccountItem, PageInfo, PointsAccountItem, PointsLotItem, WalletAccountItem, WalletLedgerEntryItem } from '../types';


export interface WalletHoldsListParams {
  accountId?: string;
  assetType?: 'cash' | 'points' | 'token';
  status?: string;
  page?: string;
  pageSize?: string;
}

export class WalletHoldsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: WalletHoldsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'accountId', value: params?.accountId, style: 'form', explode: true, allowReserved: false },
      { name: 'assetType', value: params?.assetType, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'pageSize', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/wallet/holds`), query));
  }

async retrieve(holdId: string): Promise<AccountHoldItem> {
    return this.client.get<AccountHoldItem>(appApiPath(`/wallet/holds/${serializePathParameter(holdId, { name: 'holdId', style: 'simple', explode: false })}`));
  }
}

export class WalletTokensApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(appApiPath(`/wallet/tokens`));
  }
}

export interface WalletPointsLotsListParams {
  accountId?: string;
  status?: string;
  page?: string;
  pageSize?: string;
  cursor?: string;
}

export class WalletPointsLotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: WalletPointsLotsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'accountId', value: params?.accountId, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'pageSize', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/wallet/points/lots`), query));
  }
}

export class WalletPointsApi {
  private client: HttpClient;
  public readonly lots: WalletPointsLotsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.lots = new WalletPointsLotsApi(client);
  }

}

export interface WalletLedgerEntriesPointsListParams {
  accountId?: string;
  page?: string;
  pageSize?: string;
  cursor?: string;
}

export class WalletLedgerEntriesPointsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: WalletLedgerEntriesPointsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'accountId', value: params?.accountId, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'pageSize', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/wallet/ledger_entries/points`), query));
  }
}

export interface WalletLedgerEntriesCashListParams {
  accountId?: string;
  page?: string;
  pageSize?: string;
  cursor?: string;
}

export class WalletLedgerEntriesCashApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: WalletLedgerEntriesCashListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'accountId', value: params?.accountId, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'pageSize', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/wallet/ledger_entries/cash`), query));
  }
}

export interface WalletLedgerEntriesListParams {
  accountId?: string;
  assetType?: 'cash' | 'points' | 'token';
  page?: string;
  pageSize?: string;
  cursor?: string;
}

export class WalletLedgerEntriesApi {
  private client: HttpClient;
  public readonly cash: WalletLedgerEntriesCashApi;
  public readonly points: WalletLedgerEntriesPointsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.cash = new WalletLedgerEntriesCashApi(client);
    this.points = new WalletLedgerEntriesPointsApi(client);
  }


async list(params?: WalletLedgerEntriesListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'accountId', value: params?.accountId, style: 'form', explode: true, allowReserved: false },
      { name: 'assetType', value: params?.assetType, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'pageSize', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/wallet/ledger_entries`), query));
  }

async retrieve(ledgerEntryId: string): Promise<WalletLedgerEntryItem> {
    return this.client.get<WalletLedgerEntryItem>(appApiPath(`/wallet/ledger_entries/${serializePathParameter(ledgerEntryId, { name: 'ledgerEntryId', style: 'simple', explode: false })}`));
  }
}

export class WalletAccountsTokensApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<WalletAccountItem> {
    return this.client.get<WalletAccountItem>(appApiPath(`/wallet/accounts/tokens`));
  }
}

export class WalletAccountsPointsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<PointsAccountItem> {
    return this.client.get<PointsAccountItem>(appApiPath(`/wallet/accounts/points`));
  }
}

export class WalletAccountsCashApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<CashAccountItem> {
    return this.client.get<CashAccountItem>(appApiPath(`/wallet/accounts/cash`));
  }
}

export interface WalletAccountsListParams {
  assetType?: 'cash' | 'points' | 'token';
}

export class WalletAccountsApi {
  private client: HttpClient;
  public readonly cash: WalletAccountsCashApi;
  public readonly points: WalletAccountsPointsApi;
  public readonly tokens: WalletAccountsTokensApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.cash = new WalletAccountsCashApi(client);
    this.points = new WalletAccountsPointsApi(client);
    this.tokens = new WalletAccountsTokensApi(client);
  }


async list(params?: WalletAccountsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'assetType', value: params?.assetType, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/wallet/accounts`), query));
  }
}

export class WalletOverviewApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(appApiPath(`/wallet/overview`));
  }
}

export class WalletApi {
  private client: HttpClient;
  public readonly overview: WalletOverviewApi;
  public readonly accounts: WalletAccountsApi;
  public readonly ledgerEntries: WalletLedgerEntriesApi;
  public readonly points: WalletPointsApi;
  public readonly tokens: WalletTokensApi;
  public readonly holds: WalletHoldsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.overview = new WalletOverviewApi(client);
    this.accounts = new WalletAccountsApi(client);
    this.ledgerEntries = new WalletLedgerEntriesApi(client);
    this.points = new WalletPointsApi(client);
    this.tokens = new WalletTokensApi(client);
    this.holds = new WalletHoldsApi(client);
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
interface QueryParameterSpec {
  name: string;
  value: unknown;
  style: string;
  explode: boolean;
  allowReserved: boolean;
  contentType?: string;
}

function buildQueryString(parameters: QueryParameterSpec[]): string {
  const pairs: string[] = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

function appendSerializedParameter(pairs: string[], parameter: QueryParameterSpec): void {
  if (parameter.value === undefined || parameter.value === null) {
    return;
  }

  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }

  const style = parameter.style || 'form';
  if (style === 'deepObject') {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }

  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }

  if (typeof parameter.value === 'object') {
    appendObjectParameter(pairs, parameter.name, parameter.value as Record<string, unknown>, style, parameter.explode, parameter.allowReserved);
    return;
  }

  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}

function appendArrayParameter(
  pairs: string[],
  name: string,
  value: unknown[],
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const values = value
    .filter((item) => item !== undefined && item !== null)
    .map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }

  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(','), allowReserved)}`);
}

function appendObjectParameter(
  pairs: string[],
  name: string,
  value: Record<string, unknown>,
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }

  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(',');
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}

function appendDeepObjectParameter(
  pairs: string[],
  name: string,
  value: unknown,
  allowReserved: boolean,
): void {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }

  for (const [key, entryValue] of Object.entries(value as Record<string, unknown>)) {
    if (entryValue === undefined || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}

function serializePrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function encodeQueryComponent(value: string): string {
  return encodeURIComponent(value);
}

function encodeQueryValue(value: string, allowReserved: boolean): string {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ':')
    .replace(/%2F/gi, '/')
    .replace(/%3F/gi, '?')
    .replace(/%23/gi, '#')
    .replace(/%5B/gi, '[')
    .replace(/%5D/gi, ']')
    .replace(/%40/gi, '@')
    .replace(/%21/gi, '!')
    .replace(/%24/gi, '$')
    .replace(/%26/gi, '&')
    .replace(/%27/gi, "'")
    .replace(/%28/gi, '(')
    .replace(/%29/gi, ')')
    .replace(/%2A/gi, '*')
    .replace(/%2B/gi, '+')
    .replace(/%2C/gi, ',')
    .replace(/%3B/gi, ';')
    .replace(/%3D/gi, '=');
}
