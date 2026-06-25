import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react/theme";
import {
  SdkworkWalletQuickPanel,
} from "../src";

describe("sdkwork-account-pc-wallet quick panel", () => {
  it("renders the reusable wallet quick panel with commerce shortcuts and recent activity", () => {
    const QuickPanel = SdkworkWalletQuickPanel;

    expect(QuickPanel).toBeTypeOf("function");

    const { container } = render(
      <SdkworkThemeProvider defaultTheme="light">
        <QuickPanel
          onOpenPage={() => undefined}
          onRecharge={() => undefined}
          onWithdraw={() => undefined}
          overview={{
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
          }}
        />
      </SdkworkThemeProvider>,
    );

    expect(screen.getByText(/available points/i)).toBeInTheDocument();
    expect(screen.getByText(/cash available/i)).toBeInTheDocument();
    expect(screen.getByText(/current account/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /recharge/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /withdraw/i })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /manage membership/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /open center/i })).toBeInTheDocument();
    expect(screen.getAllByText("Points usage").length).toBeGreaterThan(0);
    expect(container.innerHTML).not.toContain("border-white/10");
    expect(container.innerHTML).not.toContain("bg-white/8");
    expect(container.innerHTML).not.toContain("text-white/70");
    expect(container.innerHTML).not.toContain("text-white/60");
    expect(container.innerHTML).not.toContain("text-white/55");
  });
});
