import {
  createWalletCheckoutRouteIntent,
  resolveWalletRechargePayableAmountCny,
} from "./wallet";
import type { SdkworkWalletRechargePackage } from "./wallet-service";

export type SdkworkWalletRechargeFlow = "checkout" | "direct";

export function resolveWalletRechargeFlow(
  rechargeFlow: SdkworkWalletRechargeFlow | undefined,
  onNavigate?: (route: string) => void,
): SdkworkWalletRechargeFlow {
  if (rechargeFlow === "direct") {
    return "direct";
  }

  if (rechargeFlow === "checkout") {
    return "checkout";
  }

  return onNavigate ? "checkout" : "direct";
}

export interface NavigateWalletRechargeCheckoutInput {
  checkoutBasePath?: string;
  onNavigate: (route: string) => void;
  package?: Pick<SdkworkWalletRechargePackage, "id" | "points" | "priceCny">;
  points?: number;
  pointsToCashRate?: number | null;
  priceCny?: number;
}

export function navigateWalletRechargeCheckout(
  input: NavigateWalletRechargeCheckoutInput,
): boolean {
  const rechargePackage = input.package;
  const points = rechargePackage?.points ?? input.points;
  if (!points || points <= 0) {
    return false;
  }

  const priceCny = rechargePackage?.priceCny
    ?? input.priceCny
    ?? resolveWalletRechargePayableAmountCny(points, input.pointsToCashRate ?? null);

  if (!priceCny || priceCny <= 0) {
    return false;
  }

  input.onNavigate(
    createWalletCheckoutRouteIntent({
      basePath: input.checkoutBasePath,
      ...(rechargePackage ? { package: rechargePackage } : { points, priceCny }),
    }).route,
  );

  return true;
}
