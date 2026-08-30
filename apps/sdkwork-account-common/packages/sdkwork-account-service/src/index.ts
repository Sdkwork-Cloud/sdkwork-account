import {
  APP_ACCOUNT_METHOD_TREE,
  BACKEND_ACCOUNT_METHOD_TREE,
  type AccountAppSdkClient,
  type AccountBackendSdkClient,
  type AccountSdkMethod,
  type ClientFromMethodTree,
} from "@sdkwork/account-sdk-ports";
import type { SdkworkAccountMutationStatus } from "@sdkwork/account-contracts";
import { formatMoney } from "@sdkwork/utils/money";
import { TOKEN_POINTS_SCALE, microToDecimalString } from "@sdkwork/utils";
import {
  createAccountBackendSdkClientFromTransport,
  createAccountBackendTransportClient,
  type BootstrapSdkworkAccountBackendServiceInput,
} from "./backend-transport.ts";
import {
  createAccountAppSdkClientFromTransport,
  createAccountAppTransportClient,
  type BootstrapSdkworkAccountAppServiceInput,
} from "./transport.ts";

type ServiceTemplate = { readonly [key: string]: true | ServiceTemplate };

export type SdkworkAccountBillingService = ClientFromMethodTree<
  (typeof APP_ACCOUNT_METHOD_TREE)["billing"]
>;
export type SdkworkAccountWalletService = ClientFromMethodTree<
  (typeof APP_ACCOUNT_METHOD_TREE)["wallet"]
>;
export type SdkworkAccountAccountsService = ClientFromMethodTree<
  (typeof APP_ACCOUNT_METHOD_TREE)["accounts"]
>;
export type SdkworkAccountTokenBankService = ClientFromMethodTree<
  (typeof APP_ACCOUNT_METHOD_TREE)["tokenBank"]
>;

export type SdkworkAccountAppService = {
  billing: SdkworkAccountBillingService;
  wallet: SdkworkAccountWalletService;
  accounts: SdkworkAccountAccountsService;
  tokenBank: SdkworkAccountTokenBankService;
};

export type SdkworkAccountBackendWalletService = ClientFromMethodTree<
  (typeof BACKEND_ACCOUNT_METHOD_TREE)["wallet"]
>;
export type SdkworkAccountBackendTokenBankService = ClientFromMethodTree<
  (typeof BACKEND_ACCOUNT_METHOD_TREE)["tokenBank"]
>;

export type SdkworkAccountBackendService = {
  wallet: SdkworkAccountBackendWalletService;
  tokenBank: SdkworkAccountBackendTokenBankService;
};

export type SdkworkAccountAppServiceProvider = () => SdkworkAccountAppService;
export type SdkworkAccountBackendServiceProvider = () => SdkworkAccountBackendService;

let sdkworkAccountAppServiceProvider: SdkworkAccountAppServiceProvider | null = null;
let sdkworkAccountBackendServiceProvider: SdkworkAccountBackendServiceProvider | null = null;

export interface SdkworkAccountSessionTokens {
  accessToken?: string;
  authToken?: string;
  refreshToken?: string;
}

export type SdkworkAccountSessionTokenProvider = () => SdkworkAccountSessionTokens;

let sdkworkAccountSessionTokenProvider: SdkworkAccountSessionTokenProvider = () => ({});

export interface CreateSdkworkAccountAppServiceInput {
  appClient: AccountAppSdkClient;
}

export interface SdkworkAccountResponseEnvelope<T> {
  code: number;
  data: T;
  traceId?: string;
}

export function configureSdkworkAccountAppServiceProvider(
  provider: SdkworkAccountAppServiceProvider | null,
): void {
  sdkworkAccountAppServiceProvider = provider;
}

export function configureSdkworkAccountBackendServiceProvider(
  provider: SdkworkAccountBackendServiceProvider | null,
): void {
  sdkworkAccountBackendServiceProvider = provider;
}

export function configureSdkworkAccountSessionTokenProvider(
  provider: SdkworkAccountSessionTokenProvider | null,
): void {
  sdkworkAccountSessionTokenProvider = provider ?? (() => ({}));
}

export function getSdkworkAccountService(): SdkworkAccountAppService {
  if (!sdkworkAccountAppServiceProvider) {
    throw new Error(
      "SDKWork account service provider is not configured. Call configureSdkworkAccountAppServiceProvider() from account PC bootstrap.",
    );
  }
  return sdkworkAccountAppServiceProvider();
}

export function getSdkworkAccountBackendService(): SdkworkAccountBackendService {
  if (!sdkworkAccountBackendServiceProvider) {
    throw new Error(
      "SDKWork account backend service provider is not configured. Call configureSdkworkAccountBackendServiceProvider() from backend bootstrap.",
    );
  }
  return sdkworkAccountBackendServiceProvider();
}

export function getSdkworkAccountSessionTokens(): SdkworkAccountSessionTokens {
  const tokens = sdkworkAccountSessionTokenProvider();
  return {
    accessToken: normalizeSessionToken(tokens.accessToken),
    authToken: normalizeSessionToken(tokens.authToken),
    refreshToken: normalizeSessionToken(tokens.refreshToken),
  };
}

export function hasSdkworkAccountSession(): boolean {
  const tokens = getSdkworkAccountSessionTokens();
  return Boolean(normalizeSessionToken(tokens.authToken) || normalizeSessionToken(tokens.accessToken));
}

export function requireSdkworkAccountSession(message = "Authentication required"): void {
  if (!hasSdkworkAccountSession()) {
    throw new Error(message);
  }
}

export function createSdkworkAccountAppService(
  input: CreateSdkworkAccountAppServiceInput,
): SdkworkAccountAppService {
  return {
    billing: buildServiceTree<SdkworkAccountBillingService>(
      APP_ACCOUNT_METHOD_TREE.billing,
      input.appClient.commerce.billing,
      ["commerce", "billing"],
    ),
    wallet: buildServiceTree<SdkworkAccountWalletService>(
      APP_ACCOUNT_METHOD_TREE.wallet,
      input.appClient.commerce.wallet,
      ["commerce", "wallet"],
    ),
    accounts: buildServiceTree<SdkworkAccountAccountsService>(
      APP_ACCOUNT_METHOD_TREE.accounts,
      input.appClient.commerce.accounts,
      ["commerce", "accounts"],
    ),
    tokenBank: buildServiceTree<SdkworkAccountTokenBankService>(
      APP_ACCOUNT_METHOD_TREE.tokenBank,
      input.appClient.commerce.tokenBank,
      ["commerce", "tokenBank"],
    ),
  };
}

export interface CreateSdkworkAccountBackendServiceInput {
  backendClient: AccountBackendSdkClient;
}

export function createSdkworkAccountBackendService(
  input: CreateSdkworkAccountBackendServiceInput,
): SdkworkAccountBackendService {
  return {
    wallet: buildServiceTree<SdkworkAccountBackendWalletService>(
      BACKEND_ACCOUNT_METHOD_TREE.wallet,
      input.backendClient.commerce.wallet,
      ["commerce", "wallet"],
    ),
    tokenBank: buildServiceTree<SdkworkAccountBackendTokenBankService>(
      BACKEND_ACCOUNT_METHOD_TREE.tokenBank,
      input.backendClient.commerce.tokenBank,
      ["commerce", "tokenBank"],
    ),
  };
}

export function unwrapSdkworkAccountResponse<T>(value: unknown, fallbackMessage = "Request failed."): T {
  if (!value || typeof value !== "object") {
    return value as T;
  }
  const envelopeCandidate = value as Record<string, unknown>;
  const hasCode = Object.prototype.hasOwnProperty.call(envelopeCandidate, "code");
  const hasData = Object.prototype.hasOwnProperty.call(envelopeCandidate, "data");
  if (!hasCode && !hasData) {
    return value as T;
  }
  if (!hasCode || !hasData) {
    throw new Error("SDKWork account response envelope must include numeric code 0 and data.");
  }
  if (typeof envelopeCandidate.code !== "number") {
    throw new Error("SDKWork account response envelope success code must be numeric code 0.");
  }
  if (envelopeCandidate.code !== 0) {
    throw new Error(fallbackMessage.trim() || "Request failed.");
  }
  const envelope = value as SdkworkAccountResponseEnvelope<T>;
  return envelope.data;
}

export function unwrapSdkworkAccountResource<T>(
  value: unknown,
  fallbackMessage = "Request failed.",
): T {
  const data = unwrapSdkworkAccountResponse<{ item?: T } | T>(value, fallbackMessage);
  if (data && typeof data === "object" && "item" in data && (data as { item?: T }).item !== undefined) {
    return (data as { item: T }).item;
  }
  return data as T;
}

export interface SdkworkAccountPageInfo {
  hasMore?: boolean;
  mode?: "cursor" | "offset";
  nextCursor?: string | null;
  page?: number;
  pageSize?: number;
  totalItems?: number;
}

export interface SdkworkAccountListPage<T> {
  items: T[];
  pageInfo: SdkworkAccountPageInfo | null;
}

export function unwrapSdkworkAccountListPage<T>(
  value: unknown,
  fallbackMessage = "Request failed.",
): SdkworkAccountListPage<T> {
  const data = unwrapSdkworkAccountResponse<{
    items?: T[];
    pageInfo?: SdkworkAccountPageInfo | null;
  } | T[]>(value, fallbackMessage);
  if (Array.isArray(data)) {
    throw new Error("SDKWork account list payload must include items and pageInfo.");
  }
  if (!data || typeof data !== "object" || !Array.isArray(data.items) || !data.pageInfo) {
    throw new Error("SDKWork account list payload must include items and pageInfo.");
  }
  return {
    items: data.items,
    pageInfo: data.pageInfo,
  };
}

export function unwrapSdkworkAccountPage<T>(
  value: unknown,
  fallbackMessage = "Request failed.",
): T[] {
  return unwrapSdkworkAccountListPage<T>(value, fallbackMessage).items;
}

export function toSdkworkAccountOptionalString(value: unknown): string | undefined {
  const normalized = typeof value === "string" ? value.trim() : String(value ?? "").trim();
  return normalized || undefined;
}

export function toNullableSdkworkAccountNumber(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string" && value.trim()) {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

export function toSdkworkAccountNumber(value: unknown, fallback = 0): number {
  return toNullableSdkworkAccountNumber(value) ?? fallback;
}

export function toSdkworkAccountMutationStatus(status: unknown): SdkworkAccountMutationStatus {
  const normalized = String(status ?? "").trim().toUpperCase();
  if (normalized === "SUCCESS" || normalized === "COMPLETED" || normalized === "PAID") {
    return "completed";
  }
  if (normalized === "FAILED" || normalized === "REJECTED") {
    return "failed";
  }
  return "pending";
}

export function formatSdkworkAccountCurrencyCny(value: number | null | undefined, language = "en-US"): string {
  return formatMoney(value, { currency: "CNY", locale: language, mode: "symbol" }) ?? "--";
}

export function formatSdkworkAccountPoints(value: number, language = "en-US"): string {
  if (!Number.isFinite(value)) {
    return "0";
  }
  return new Intl.NumberFormat(language, {
    maximumFractionDigits: TOKEN_POINTS_SCALE,
  }).format(value);
}

export function toSdkworkAccountPointsFromMicro(value: unknown): number {
  const raw = toSdkworkAccountNumber(value);
  if (!Number.isFinite(raw) || raw === 0) {
    return 0;
  }
  const sign = raw < 0 ? -1 : 1;
  const micro = BigInt(Math.round(Math.abs(raw)));
  return Number(microToDecimalString(micro)) * sign;
}

export function formatSdkworkAccountPointsRate(points: number, language = "en-US"): string {
  return language === "zh-CN"
    ? `${formatSdkworkAccountPoints(points, language)} \u79ef\u5206 / 1 \u5143`
    : `${formatSdkworkAccountPoints(points, language)} pts / CNY 1`;
}

export function formatSdkworkAccountPointsDelta(value: number, language = "en-US"): string {
  const formatted = formatSdkworkAccountPoints(Math.abs(value), language);
  if (value > 0) {
    return `+${formatted}`;
  }
  if (value < 0) {
    return `-${formatted}`;
  }
  return "0";
}

function buildServiceTree<TService>(
  template: ServiceTemplate,
  client: unknown,
  missingPathPrefix: readonly string[],
  servicePath: readonly string[] = [],
): TService {
  const service: Record<string, unknown> = {};
  for (const [key, marker] of Object.entries(template)) {
    const nextServicePath = [...servicePath, key];
    if (marker === true) {
      const missingPath = [...missingPathPrefix, ...nextServicePath].join(".");
      service[key] = (...args: Parameters<AccountSdkMethod>) =>
        callAccount(readMethod(client, nextServicePath), missingPath, ...args);
    } else {
      service[key] = buildServiceTree<Record<string, unknown>>(
        marker,
        client,
        missingPathPrefix,
        nextServicePath,
      );
    }
  }
  return service as TService;
}

function readMethod(root: unknown, path: readonly string[]): AccountSdkMethod | undefined {
  let node: unknown = root;
  for (const segment of path) {
    if (!node || typeof node !== "object") {
      return undefined;
    }
    const parent = node;
    node = (parent as Record<string, unknown>)[segment];
    if (typeof node === "function") {
      return node.bind(parent) as AccountSdkMethod;
    }
  }
  return typeof node === "function" ? (node as AccountSdkMethod) : undefined;
}

async function callAccount(
  method: AccountSdkMethod | undefined,
  name: string,
  ...args: Parameters<AccountSdkMethod>
): Promise<unknown> {
  if (!method) {
    throw new Error(`Missing SDKWork account SDK resource: ${name}`);
  }
  return method(...args);
}

function normalizeSessionToken(value: unknown): string | undefined {
  const normalized = typeof value === "string" ? value.trim() : "";
  return normalized || undefined;
}

export {
  createAccountAppSdkClientFromTransport,
  createAccountAppTransportClient,
  resolveAccountAppApiOrigin,
  type BootstrapSdkworkAccountAppServiceInput,
} from "./transport.ts";

export function bootstrapSdkworkAccountAppService(
  input: BootstrapSdkworkAccountAppServiceInput,
): SdkworkAccountAppService {
  const transport = createAccountAppTransportClient(input);
  const service = createSdkworkAccountAppService({
    appClient: createAccountAppSdkClientFromTransport(transport),
  });
  configureSdkworkAccountAppServiceProvider(() => service);
  return service;
}

export {
  createAccountBackendSdkClientFromTransport,
  createAccountBackendTransportClient,
  resolveAccountBackendApiOrigin,
  type BootstrapSdkworkAccountBackendServiceInput,
} from "./backend-transport.ts";

export function bootstrapSdkworkAccountBackendService(
  input: BootstrapSdkworkAccountBackendServiceInput,
): SdkworkAccountBackendService {
  const transport = createAccountBackendTransportClient(input);
  const service = createSdkworkAccountBackendService({
    backendClient: createAccountBackendSdkClientFromTransport(transport),
  });
  configureSdkworkAccountBackendServiceProvider(() => service);
  return service;
}
