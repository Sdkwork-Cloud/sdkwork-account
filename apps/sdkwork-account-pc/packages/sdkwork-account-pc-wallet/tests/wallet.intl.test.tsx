import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { formatSdkworkAccountPointsRate } from "@sdkwork/account-service";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react/theme";
import {
  SdkworkWalletIntlProvider,
  SdkworkWalletPage,
  SdkworkWalletQuickPanel,
  createSdkworkWalletController,
} from "../src";

function createOverview() {
  return {
    account: {
      availablePoints: 2400,
      cashAvailable: 88.5,
      cashFrozen: 0,
      experience: 18,
      frozenPoints: 30,
      hasPayPassword: true,
      level: 2,
      levelName: "Silver",
      totalEarned: 9600,
      tokenBalance: 42,
      totalPoints: 2430,
      totalSpent: 7200,
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
        pointsAfter: 2400,
        pointsBefore: 2640,
        pointsDelta: -240,
        status: "SUCCESS",
        title: "Points usage",
        transactionTypeName: "Points usage",
      },
    ],
  };
}

function createEmptyOverview() {
  return {
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
  };
}

describe("sdkwork-account-pc-wallet intl", () => {
  it("renders Chinese copy across the wallet page when a Chinese locale is provided", async () => {
    const WalletPage = SdkworkWalletPage;
    const controller = createSdkworkWalletController({
      service: {
        getEmptyOverview: vi.fn().mockReturnValue(createEmptyOverview()),
        getOverview: vi.fn().mockResolvedValue(createOverview()),
        rechargePoints: vi.fn(),
        withdrawCash: vi.fn(),
      },
    });

    render(
      <SdkworkThemeProvider defaultTheme="light">
        <WalletPage controller={controller} locale="zh-CN" />
      </SdkworkThemeProvider>,
    );

    expect(
      await screen.findByRole("heading", {
        name: "\u94b1\u5305\u4e2d\u5fc3",
      }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "\u5145\u503c\u94b1\u5305" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "\u63d0\u73b0" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "\u94b1\u5305\u6d3b\u52a8" })).toBeInTheDocument();
  });

  it("applies host message overrides on top of the localized wallet surface", async () => {
    const WalletPage = SdkworkWalletPage;
    const controller = createSdkworkWalletController({
      service: {
        getEmptyOverview: vi.fn().mockReturnValue(createEmptyOverview()),
        getOverview: vi.fn().mockResolvedValue(createOverview()),
        rechargePoints: vi.fn(),
        withdrawCash: vi.fn(),
      },
    });

    render(
      <SdkworkThemeProvider defaultTheme="light">
        <WalletPage
          controller={controller}
          locale="zh-CN"
          messages={{
            balancePanel: {
              primaryAction: "Launch top-up",
              title: "Host wallet cockpit",
            },
          }}
        />
      </SdkworkThemeProvider>,
    );

    expect(
      await screen.findByRole("heading", {
        name: "Host wallet cockpit",
      }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Launch top-up" })).toBeInTheDocument();
  });

  it("falls back to built-in English copy for standalone wallet components without a host intl provider", () => {
    const WalletQuickPanel = SdkworkWalletQuickPanel;

    render(
      <SdkworkThemeProvider defaultTheme="light">
        <WalletQuickPanel
          onOpenPage={vi.fn()}
          onRecharge={vi.fn()}
          onWithdraw={vi.fn()}
          overview={createOverview()}
        />
      </SdkworkThemeProvider>,
    );

    expect(screen.getAllByText("Available points").length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: /open center/i })).toBeInTheDocument();
    expect(formatSdkworkAccountPointsRate(200, "en-US")).toBe("200 pts / CNY 1");
  });

  it("lets standalone wallet components consume Chinese copy through the intl provider", () => {
    const WalletIntlProvider = SdkworkWalletIntlProvider;
    const WalletQuickPanel = SdkworkWalletQuickPanel;

    expect(WalletIntlProvider).toBeTypeOf("function");

    if (typeof WalletIntlProvider !== "function") {
      return;
    }

    render(
      <SdkworkThemeProvider defaultTheme="light">
        <WalletIntlProvider locale="zh-CN">
          <WalletQuickPanel
            onOpenPage={vi.fn()}
            onRecharge={vi.fn()}
            onWithdraw={vi.fn()}
            overview={createOverview()}
          />
        </WalletIntlProvider>
      </SdkworkThemeProvider>,
    );

    expect(screen.getAllByText("\u53ef\u7528\u79ef\u5206").length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "\u6253\u5f00\u4e2d\u5fc3" })).toBeInTheDocument();
    expect(formatSdkworkAccountPointsRate(200, "zh-CN")).toBe("200 \u79ef\u5206 / 1 \u5143");
  });
});
