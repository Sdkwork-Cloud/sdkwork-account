export type SdkworkWalletWithdrawalFlow = "checkout" | "direct";

export function resolveWalletWithdrawalFlow(
  withdrawalFlow: SdkworkWalletWithdrawalFlow | undefined,
  onNavigate?: (route: string) => void,
): SdkworkWalletWithdrawalFlow {
  if (withdrawalFlow === "direct") {
    return "direct";
  }

  if (withdrawalFlow === "checkout") {
    return "checkout";
  }

  return onNavigate ? "checkout" : "direct";
}

export interface CreateWalletWithdrawalRouteIntentOptions {
  basePath?: string;
  focusWindow?: boolean;
}

export interface SdkworkWalletWithdrawalRouteIntent {
  focusWindow: boolean;
  kind: "wallet-withdraw";
  route: string;
  source: "wallet-workspace";
  type: "wallet-withdrawal-route-intent";
}

const DEFAULT_WITHDRAWAL_REQUEST_BASE_PATH = "/withdrawals/requests";

function normalizeWithdrawalRequestBasePath(basePath: string | undefined): string {
  const normalized = (basePath ?? DEFAULT_WITHDRAWAL_REQUEST_BASE_PATH).trim();
  if (!normalized || normalized === "/") {
    return DEFAULT_WITHDRAWAL_REQUEST_BASE_PATH;
  }

  return normalized.endsWith("/") ? normalized.slice(0, -1) : normalized;
}

export function createWalletWithdrawalRouteIntent(
  options: CreateWalletWithdrawalRouteIntentOptions = {},
): SdkworkWalletWithdrawalRouteIntent {
  const basePath = normalizeWithdrawalRequestBasePath(options.basePath);
  const queryParams = new URLSearchParams({
    kind: "wallet-withdraw",
    source: "wallet-workspace",
  });

  return {
    focusWindow: options.focusWindow !== false,
    kind: "wallet-withdraw",
    route: `${basePath}?${queryParams.toString()}`,
    source: "wallet-workspace",
    type: "wallet-withdrawal-route-intent",
  };
}

export interface NavigateWalletWithdrawalRequestInput {
  onNavigate: (route: string) => void;
  withdrawalRequestBasePath?: string;
}

export function navigateWalletWithdrawalRequest(input: NavigateWalletWithdrawalRequestInput): boolean {
  const intent = createWalletWithdrawalRouteIntent({
    basePath: input.withdrawalRequestBasePath,
  });
  input.onNavigate(intent.route);
  return true;
}
