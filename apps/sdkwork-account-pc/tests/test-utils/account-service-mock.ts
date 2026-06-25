import {
  configureSdkworkAccountSessionTokenProvider,
  type SdkworkAccountAppService,
  type SdkworkAccountSessionTokens,
} from "@sdkwork/account-service";

type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends (...args: infer TArgs) => infer TReturn
    ? (...args: TArgs) => TReturn
    : DeepPartial<T[K]>;
};

export function createAccountAppServiceMock(
  overrides: DeepPartial<SdkworkAccountAppService> = {},
): SdkworkAccountAppService {
  const base: SdkworkAccountAppService = {
    wallet: createMissingWalletTree(),
    accounts: createMissingAccountsTree(),
    recharges: createMissingRechargesTree(),
  };
  return mergeAccountAppService(base, overrides);
}

export function configureAccountServiceMockSession(
  tokens: SdkworkAccountSessionTokens = { authToken: "account-auth-token" },
): void {
  configureSdkworkAccountSessionTokenProvider(() => tokens);
}

export function resetAccountServiceMockSession(): void {
  configureSdkworkAccountSessionTokenProvider(null);
}

function createMissingWalletTree(): SdkworkAccountAppService["wallet"] {
  const tree: Record<string, unknown> = {};
  for (const method of [
    "ledgerEntries.points.list",
    "accounts.points.retrieve",
    "exchangeRate.retrieve",
    "withdrawalTransfers.create",
  ]) {
    addMissingMethod(tree, method);
  }
  return tree as SdkworkAccountAppService["wallet"];
}

function createMissingAccountsTree(): SdkworkAccountAppService["accounts"] {
  const tree: Record<string, unknown> = {};
  addMissingMethod(tree, "current.summary.retrieve");
  return tree as SdkworkAccountAppService["accounts"];
}

function createMissingRechargesTree(): SdkworkAccountAppService["recharges"] {
  const tree: Record<string, unknown> = {};
  for (const method of ["packages.list", "orders.create"]) {
    addMissingMethod(tree, method);
  }
  return tree as SdkworkAccountAppService["recharges"];
}

function addMissingMethod(root: Record<string, unknown>, method: string): void {
  let node = root;
  const segments = method.split(".");
  for (const segment of segments.slice(0, -1)) {
    if (!node[segment] || typeof node[segment] === "function") {
      node[segment] = {};
    }
    node = node[segment] as Record<string, unknown>;
  }
  node[segments.at(-1)!] = async () => {
    throw new Error(`Missing account service test method: ${method}`);
  };
}

function mergeAccountAppService<T>(base: T, overrides: DeepPartial<T>): T {
  for (const [key, value] of Object.entries(overrides as Record<string, unknown>)) {
    if (
      value &&
      typeof value === "object" &&
      !Array.isArray(value) &&
      typeof (base as Record<string, unknown>)[key] === "object"
    ) {
      mergeAccountAppService((base as Record<string, unknown>)[key], value as DeepPartial<unknown>);
    } else {
      (base as Record<string, unknown>)[key] = value;
    }
  }
  return base;
}
