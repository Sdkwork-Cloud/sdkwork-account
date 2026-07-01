import {
  createSdkworkAppCapabilityManifest,
  type CreateSdkworkAppCapabilityManifestOptions,
  type SdkworkAppCapabilityManifest,
} from "@sdkwork/appbase-pc-react";
import {
  formatSdkworkAccountPointsDelta,
} from "@sdkwork/account-service";
import type { SdkworkWalletAccount } from "./wallet-service";

export interface SdkworkWalletWorkspaceManifest extends SdkworkAppCapabilityManifest {
  capability: "wallet";
  routePath: string;
}

export interface CreateWalletWorkspaceManifestOptions
  extends Partial<
    Pick<CreateSdkworkAppCapabilityManifestOptions, "description" | "host" | "id" | "packageNames" | "theme" | "title">
  > {
  routePath?: string;
}

export interface SdkworkWalletRouteIntent {
  focusWindow: boolean;
  route: string;
  sectionId?: string;
  source: "wallet-workspace";
  type: "wallet-route-intent";
}

export type SdkworkWalletWithdrawDestinationCode = "bank_account" | "ALIPAY" | "WECHAT_PAY";

export interface SdkworkWalletWithdrawDestination {
  code: SdkworkWalletWithdrawDestinationCode;
  description: string;
  id: string;
  label: string;
}

export interface CreateWalletRouteIntentOptions {
  basePath?: string;
  focusWindow?: boolean;
  sectionId?: string;
}

function normalizeBasePath(basePath: string | undefined): string {
  const normalized = (basePath ?? "/wallet").trim();
  if (!normalized || normalized === "/") {
    return "/wallet";
  }

  return normalized.endsWith("/") ? normalized.slice(0, -1) : normalized;
}

export function formatSdkworkWalletDelta(
  value: number,
  language = "en-US",
): string {
  return formatSdkworkAccountPointsDelta(value, language);
}

export function getSdkworkWalletAccountLevelLabel(
  account: Pick<SdkworkWalletAccount, "level" | "levelName">,
): string {
  const levelName = account.levelName?.trim();
  if (levelName) {
    return levelName;
  }

  if (account.level !== null) {
    return `LV ${account.level}`;
  }

  return "Standard";
}

export function createWalletWorkspaceManifest({
  description = "Wallet workspace for balances, holds, ledger history, and commerce checkout navigation.",
  host,
  id = "sdkwork-wallet",
  packageNames = ["@sdkwork/account-pc-wallet"],
  routePath = "/wallet",
  theme,
  title = "Wallet",
}: CreateWalletWorkspaceManifestOptions = {}): SdkworkWalletWorkspaceManifest {
  return {
    ...createSdkworkAppCapabilityManifest({
      description,
      host,
      id,
      packageNames,
      theme,
      title,
    }),
    capability: "wallet",
    routePath: normalizeBasePath(routePath),
  };
}

export interface CreateWalletCheckoutRouteIntentOptions {
  basePath?: string;
  focusWindow?: boolean;
  package?: Pick<SdkworkWalletRechargePackageInput, "id" | "points" | "priceCny">;
  points?: number;
  priceCny?: number;
}

export interface SdkworkWalletRechargePackageInput {
  id: number;
  points: number;
  priceCny: number;
}

export interface SdkworkWalletCheckoutRouteIntent {
  focusWindow: boolean;
  kind: "wallet-recharge";
  route: string;
  source: "wallet-workspace";
  sourceId: string;
  type: "wallet-checkout-route-intent";
}

function normalizeCheckoutBasePath(basePath: string | undefined): string {
  const normalized = (basePath ?? "/checkout").trim();
  if (!normalized || normalized === "/") {
    return "/checkout";
  }

  return normalized.endsWith("/") ? normalized.slice(0, -1) : normalized;
}

export function createWalletCheckoutSourceId(
  input: { packageId: number } | { points: number },
): string {
  if ("packageId" in input) {
    return `wallet-recharge-package-${input.packageId}`;
  }

  return `wallet-recharge-points-${input.points}`;
}

export function createWalletCheckoutRouteIntent(
  options: CreateWalletCheckoutRouteIntentOptions,
): SdkworkWalletCheckoutRouteIntent {
  const basePath = normalizeCheckoutBasePath(options.basePath);
  const rechargePackage = options.package;
  const points = rechargePackage?.points ?? options.points;
  const priceCny = rechargePackage?.priceCny ?? options.priceCny;

  if (!points || points <= 0) {
    throw new Error("Wallet checkout requires a positive points amount.");
  }

  if (!priceCny || priceCny <= 0) {
    throw new Error("Wallet checkout requires a positive payable amount.");
  }

  const sourceId = rechargePackage
    ? createWalletCheckoutSourceId({ packageId: rechargePackage.id })
    : createWalletCheckoutSourceId({ points });

  const queryParams = new URLSearchParams({
    kind: "wallet-recharge",
    points: String(points),
    priceCny: String(priceCny),
    sourceId,
  });

  if (rechargePackage) {
    queryParams.set("packageId", String(rechargePackage.id));
  }

  return {
    focusWindow: options.focusWindow !== false,
    kind: "wallet-recharge",
    route: `${basePath}?${queryParams.toString()}`,
    source: "wallet-workspace",
    sourceId,
    type: "wallet-checkout-route-intent",
  };
}

export function resolveWalletRechargePayableAmountCny(
  points: number,
  pointsToCashRate: number | null,
): number | null {
  if (!Number.isFinite(points) || points <= 0 || !pointsToCashRate || pointsToCashRate <= 0) {
    return null;
  }

  return Number((points / pointsToCashRate).toFixed(2));
}

export function createWalletRouteIntent(
  options: CreateWalletRouteIntentOptions = {},
): SdkworkWalletRouteIntent {
  const basePath = normalizeBasePath(options.basePath);
  const queryParams = new URLSearchParams();

  if (options.sectionId) {
    queryParams.set("section", options.sectionId);
  }

  const querySuffix = queryParams.toString() ? `?${queryParams.toString()}` : "";

  return {
    focusWindow: options.focusWindow !== false,
    route: `${basePath}${querySuffix}`,
    ...(options.sectionId ? { sectionId: options.sectionId } : {}),
    source: "wallet-workspace",
    type: "wallet-route-intent",
  };
}

export function createDefaultSdkworkWalletWithdrawDestinations(): SdkworkWalletWithdrawDestination[] {
  return [
    {
      code: "bank_account",
      description: "Route the payout through the linked settlement bank account.",
      id: "withdraw-bank-account",
      label: "Bank account",
    },
    {
      code: "ALIPAY",
      description: "Submit the payout to the linked Alipay settlement rail.",
      id: "withdraw-alipay",
      label: "Alipay",
    },
    {
      code: "WECHAT_PAY",
      description: "Submit the payout to the linked WeChat Pay settlement rail.",
      id: "withdraw-wechat-pay",
      label: "WeChat Pay",
    },
  ];
}

export const walletPackageMeta = {
  architecture: "pc-react",
  domain: "commerce",
  package: "@sdkwork/account-pc-wallet",
  status: "ready",
} as const;

export type WalletPackageMeta = typeof walletPackageMeta;
