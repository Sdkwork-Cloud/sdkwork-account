import { describe, expect, it, vi } from "vitest";
import {
  createSdkworkWalletController,
  type SdkworkWalletOverview,
} from "../src";

function createOverview(
  overrides: Partial<SdkworkWalletOverview> = {},
): SdkworkWalletOverview {
  return {
    account: {
      availablePoints: 1200,
      cashAvailable: 88.5,
      cashFrozen: 0,
      experience: 18,
      frozenPoints: 30,
      hasPayPassword: true,
      level: 2,
      levelName: "Silver",
      totalEarned: 9600,
      tokenBalance: 42,
      totalPoints: 1230,
      totalSpent: 8370,
    },
    isAuthenticated: true,
    pointsToCashRate: 200,
    rechargePackages: [
      {
        id: 202,
        points: 5000,
        priceCny: 24,
        recommended: true,
        sortWeight: 20,
        title: "Growth 5K",
      },
    ],
    transactions: [
      {
        cashAmountCny: 6,
        createdAt: "2026-04-02T12:00:00.000Z",
        id: "wallet-transaction-1",
        pointsAfter: 1200,
        pointsBefore: 1440,
        pointsDelta: -240,
        status: "SUCCESS",
        title: "Points usage",
        transactionTypeName: "Points usage",
      },
    ],
    holds: [],
    ...overrides,
  };
}

describe("sdkwork-account-pc-wallet controller", () => {
  it("bootstraps overview state and refreshes after recharge actions", async () => {
    const initialOverview = createOverview();
    const reloadedOverview = createOverview({
      account: {
        ...createOverview().account,
        availablePoints: 6200,
      },
    });
    const getOverview = vi.fn()
      .mockResolvedValueOnce(initialOverview)
      .mockResolvedValueOnce(reloadedOverview)
      .mockResolvedValueOnce(reloadedOverview);
    const controller = createSdkworkWalletController({
      service: {
        getEmptyOverview: vi.fn().mockReturnValue(
          createOverview({
            account: {
              availablePoints: 0,
              cashAvailable: 0,
              cashFrozen: 0,
              experience: null,
              frozenPoints: 0,
              hasPayPassword: false,
              level: null,
              totalEarned: 0,
              tokenBalance: 0,
              totalPoints: 0,
              totalSpent: 0,
            },
            isAuthenticated: false,
            pointsToCashRate: null,
            rechargePackages: [],
            transactions: [],
            holds: [],
          }),
        ),
        getOverview,
        rechargePoints: vi.fn().mockResolvedValue({
          cashAmountCny: 24,
          paymentMethod: "WECHAT",
          points: 5000,
          status: "completed",
          transactionId: "TXN-200",
        }),
      },
    });

    await controller.bootstrap();

    expect(controller.getState().isBootstrapped).toBe(true);
    expect(controller.getState().overview.account.availablePoints).toBe(1200);

    controller.openRecharge();
    await controller.rechargePoints({
      paymentMethod: "WECHAT",
      points: 5000,
    });

    expect(controller.getState().isRechargeOpen).toBe(false);
    expect(controller.getState().overview.account.availablePoints).toBe(6200);
    expect(getOverview).toHaveBeenCalledTimes(2);
  });

  it("opens the withdraw dialog and refreshes overview after a withdraw mutation", async () => {
    const initialOverview = createOverview();
    const refreshedOverview = createOverview({
      account: {
        ...createOverview().account,
        cashAvailable: 76,
      },
    });
    const getOverview = vi.fn()
      .mockResolvedValueOnce(initialOverview)
      .mockResolvedValueOnce(refreshedOverview);
    const controller = createSdkworkWalletController({
      service: {
        getEmptyOverview: vi.fn().mockReturnValue(
          createOverview({
            account: {
              availablePoints: 0,
              cashAvailable: 0,
              cashFrozen: 0,
              experience: null,
              frozenPoints: 0,
              hasPayPassword: false,
              level: null,
              totalEarned: 0,
              tokenBalance: 0,
              totalPoints: 0,
              totalSpent: 0,
            },
            isAuthenticated: false,
            pointsToCashRate: null,
            rechargePackages: [],
            transactions: [],
            holds: [],
          }),
        ),
        getOverview,
        rechargePoints: vi.fn(),
        withdrawCash: vi.fn().mockResolvedValue({
          amountCny: 12.5,
          destinationCode: "bank_account",
          remainingCashAvailable: 76,
          status: "pending",
          transactionId: "TXN-300",
        }),
      },
    });

    await controller.bootstrap();

    controller.openWithdraw();
    expect(controller.getState().isWithdrawOpen).toBe(true);

    await controller.withdrawCash({
      accountName: "SDKWORK Ops",
      accountNo: "6222020202020202",
      amountCny: 12.5,
      bankName: "SDKWORK Bank",
      destinationCode: "bank_account",
    });

    expect(controller.getState().isWithdrawOpen).toBe(false);
    expect(controller.getState().overview.account.cashAvailable).toBe(76);
    expect(getOverview).toHaveBeenCalledTimes(2);

    controller.openWithdraw();
    controller.closeWithdraw();
    expect(controller.getState().isWithdrawOpen).toBe(false);
  });
});
