import {
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react/theme";
import {
  SdkworkWalletHeaderEntry,
  createSdkworkWalletController,
} from "../src";

describe("sdkwork-account-pc-wallet header entry", () => {
  it("opens the wallet quick panel and launches recharge and withdraw flows from header context", async () => {
    const onOpenPage = vi.fn();
    const controller = createSdkworkWalletController({
      service: {
        getEmptyOverview: vi.fn().mockReturnValue({
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
        }),
        getOverview: vi.fn().mockResolvedValue({
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
        }),
        rechargePoints: vi.fn().mockResolvedValue({
          cashAmountCny: 24,
          paymentMethod: "WECHAT",
          points: 5000,
          status: "completed",
          transactionId: "TXN-200",
        }),
        withdrawCash: vi.fn().mockResolvedValue({
          amountCny: 12.5,
          destinationCode: "bank_account",
          remainingCashAvailable: 76,
          status: "pending",
          transactionId: "TXN-300",
        }),
      },
    });

    render(
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkWalletHeaderEntry controller={controller} onOpenPage={onOpenPage} />
      </SdkworkThemeProvider>,
    );

    fireEvent.click(
      await screen.findByRole("button", {
        name: /wallet balance/i,
      }),
    );

    expect(await screen.findByText(/cash available/i)).toBeInTheDocument();
    expect(screen.getAllByText("Points usage").length).toBeGreaterThan(0);

    fireEvent.click(
      screen.getByRole("button", {
        name: /open center/i,
      }),
    );
    expect(onOpenPage).toHaveBeenCalledTimes(1);

    fireEvent.click(
      screen.getByRole("button", {
        name: /wallet balance/i,
      }),
    );
    fireEvent.click(
      screen.getByRole("button", {
        name: /recharge/i,
      }),
    );
    expect(
      await screen.findByRole("heading", {
        name: /recharge balance/i,
      }),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", {
        name: /cancel/i,
      }),
    );
    fireEvent.click(
      screen.getByRole("button", {
        name: /wallet balance/i,
      }),
    );
    fireEvent.click(
      screen.getByRole("button", {
        name: /withdraw/i,
      }),
    );
    expect(
      await screen.findByRole("heading", {
        name: /withdraw balance/i,
      }),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getAllByRole("button", {
        name: /cancel/i,
      })[0],
    );
    const accountButton = screen.getByRole("button", {
      name: /account/i,
    });
    expect(accountButton).toBeInTheDocument();
    fireEvent.click(accountButton);
    expect(onOpenPage).toHaveBeenCalledTimes(2);
  });
});
