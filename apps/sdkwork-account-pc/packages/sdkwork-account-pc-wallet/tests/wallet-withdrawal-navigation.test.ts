import { describe, expect, it, vi } from "vitest";
import {
  createWalletWithdrawalRouteIntent,
  navigateWalletWithdrawalRequest,
} from "../src";

describe("wallet withdraw order navigation", () => {
  it("defaults cash withdrawal navigation to the order withdrawal request flow", () => {
    const intent = createWalletWithdrawalRouteIntent();

    expect(intent.route).toBe("/withdrawals/requests?kind=wallet-withdraw&source=wallet-workspace");
  });

  it("normalizes blank cash withdrawal base paths to the order withdrawal request flow", () => {
    const intent = createWalletWithdrawalRouteIntent({
      basePath: " ",
    });

    expect(intent.route).toBe("/withdrawals/requests?kind=wallet-withdraw&source=wallet-workspace");
  });

  it("navigates to the configured order-owned withdrawal request route", () => {
    const onNavigate = vi.fn();

    expect(
      navigateWalletWithdrawalRequest({
        onNavigate,
        withdrawalRequestBasePath: "/orders/withdrawals",
      }),
    ).toBe(true);

    expect(onNavigate).toHaveBeenCalledWith(
      "/orders/withdrawals?kind=wallet-withdraw&source=wallet-workspace",
    );
  });
});
