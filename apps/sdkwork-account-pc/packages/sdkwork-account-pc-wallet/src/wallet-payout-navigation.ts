export type SdkworkWalletPayoutFlow = "checkout" | "direct";

export function resolveWalletPayoutFlow(
  payoutFlow: SdkworkWalletPayoutFlow | undefined,
  onNavigate?: (route: string) => void,
): SdkworkWalletPayoutFlow {
  if (payoutFlow === "direct") {
    return "direct";
  }

  if (payoutFlow === "checkout") {
    return "checkout";
  }

  return onNavigate ? "checkout" : "direct";
}

export interface CreateWalletPayoutRouteIntentOptions {
  basePath?: string;
  focusWindow?: boolean;
}

export interface SdkworkWalletPayoutRouteIntent {
  focusWindow: boolean;
  kind: "wallet-withdraw";
  route: string;
  source: "wallet-workspace";
  type: "wallet-payout-route-intent";
}

function normalizePayoutBasePath(basePath: string | undefined): string {
  const normalized = (basePath ?? "/payments/payout").trim();
  if (!normalized || normalized === "/") {
    return "/payments/payout";
  }

  return normalized.endsWith("/") ? normalized.slice(0, -1) : normalized;
}

export function createWalletPayoutRouteIntent(
  options: CreateWalletPayoutRouteIntentOptions = {},
): SdkworkWalletPayoutRouteIntent {
  const basePath = normalizePayoutBasePath(options.basePath);
  const queryParams = new URLSearchParams({
    kind: "wallet-withdraw",
    source: "wallet-workspace",
  });

  return {
    focusWindow: options.focusWindow !== false,
    kind: "wallet-withdraw",
    route: `${basePath}?${queryParams.toString()}`,
    source: "wallet-workspace",
    type: "wallet-payout-route-intent",
  };
}

export interface NavigateWalletWithdrawPayoutInput {
  onNavigate: (route: string) => void;
  payoutBasePath?: string;
}

export function navigateWalletWithdrawPayout(input: NavigateWalletWithdrawPayoutInput): boolean {
  const intent = createWalletPayoutRouteIntent({
    basePath: input.payoutBasePath,
  });
  input.onNavigate(intent.route);
  return true;
}
