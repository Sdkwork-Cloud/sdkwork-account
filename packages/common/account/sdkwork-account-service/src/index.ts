import {
  APP_ACCOUNT_METHOD_TREE,
  type AccountAppSdkClient,
  type AccountSdkMethod,
  type ClientFromMethodTree,
} from "@sdkwork/account-sdk-ports";
import type { SdkworkAccountMutationStatus } from "@sdkwork/account-contracts";
import { formatCurrency as formatSdkworkCurrency } from "@sdkwork/utils";

type ServiceTemplate = { readonly [key: string]: true | ServiceTemplate };

export type SdkworkAccountWalletService = ClientFromMethodTree<
  (typeof APP_ACCOUNT_METHOD_TREE)["wallet"]
>;
export type SdkworkAccountAccountsService = ClientFromMethodTree<
  (typeof APP_ACCOUNT_METHOD_TREE)["accounts"]
>;
export type SdkworkAccountRechargesService = ClientFromMethodTree<
  (typeof APP_ACCOUNT_METHOD_TREE)["recharges"]
>;

export type SdkworkAccountAppService = {
  wallet: SdkworkAccountWalletService;
  accounts: SdkworkAccountAccountsService;
  recharges: SdkworkAccountRechargesService;
};

export type SdkworkAccountAppServiceProvider = () => SdkworkAccountAppService;

let sdkworkAccountAppServiceProvider: SdkworkAccountAppServiceProvider | null = null;

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
  code?: number | string;
  data?: T;
  message?: string;
  msg?: string;
}

export function configureSdkworkAccountAppServiceProvider(
  provider: SdkworkAccountAppServiceProvider | null,
): void {
  sdkworkAccountAppServiceProvider = provider;
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
    recharges: buildServiceTree<SdkworkAccountRechargesService>(
      APP_ACCOUNT_METHOD_TREE.recharges,
      input.appClient.commerce.recharges,
      ["commerce", "recharges"],
    ),
  };
}

export function unwrapSdkworkAccountResponse<T>(value: unknown, fallbackMessage = "Request failed."): T {
  if (!value || typeof value !== "object") {
    return value as T;
  }
  if (!("data" in value) && !("code" in value)) {
    return value as T;
  }
  const envelope = value as SdkworkAccountResponseEnvelope<T>;
  if (!isSuccessCode(envelope.code)) {
    throw new Error(String(envelope.message || envelope.msg || fallbackMessage).trim());
  }
  return (envelope.data ?? null) as T;
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
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "--";
  }
  return formatSdkworkCurrency(value, "CNY", language) ?? "--";
}

export function formatSdkworkAccountPoints(value: number, language = "en-US"): string {
  return new Intl.NumberFormat(language).format(value);
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

function isSuccessCode(code: number | string | undefined): boolean {
  if (code === undefined || code === null || code === "") {
    return true;
  }
  if (typeof code === "number") {
    return code === 0 || code === 200 || code === 2000;
  }
  const normalized = String(code).trim();
  return normalized === "0" || normalized === "200" || normalized === "2000" || normalized === "SUCCESS";
}
