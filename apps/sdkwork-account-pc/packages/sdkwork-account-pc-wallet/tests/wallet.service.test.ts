import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  configureAccountServiceMockSession,
  createAccountAppServiceMock,
  resetAccountServiceMockSession,
} from "../../../tests/test-utils/account-service-mock";
import { createSdkworkWalletService } from "../src";

describe("sdkwork-account-pc-wallet service", () => {
  beforeEach(() => {
    configureAccountServiceMockSession({ authToken: "wallet-auth-token" });
  });

  afterEach(() => {
    resetAccountServiceMockSession();
  });

  it("maps dedicated cash, points, Token Bank, and unified ledger endpoints into a wallet overview", async () => {
    const accountAppService = createAccountAppServiceMock({
      wallet: {
        accounts: {
          cash: {
            retrieve: vi.fn().mockResolvedValue({
              code: 0,
              data: {
                item: {
                  availableAmount: "88.50",
                  frozenAmount: "10.00",
                  pendingAmount: "0",
                },
              },
            }),
          },
          points: {
            retrieve: vi.fn().mockResolvedValue({
              code: 0,
              data: {
                item: {
                  availablePoints: "1200",
                  frozenPoints: "30",
                  pendingPoints: "0",
                  totalPoints: "1230",
                  status: "active",
                },
              },
            }),
          },
        },
        ledgerEntries: {
          list: vi.fn().mockResolvedValue({
            code: 0,
            data: {
              items: [
                {
                  amount: "240",
                  assetType: "points",
                  balanceAfter: "1200",
                  balanceBefore: "1440",
                  businessType: "POINTS_USAGE",
                  createdAt: "2026-04-01T12:00:00.000Z",
                  direction: "debit",
                  uuid: "history-2",
                },
                {
                  amount: "6.00",
                  assetType: "cash",
                  businessType: "CASH_WITHDRAW",
                  createdAt: "2026-04-01T11:00:00.000Z",
                  direction: "debit",
                  uuid: "cash-history-1",
                },
              ],
              pageInfo: {
                mode: "cursor",
                pageSize: 20,
                hasMore: false,
              },
            },
          }),
        },
        points: {
          summary: {
            retrieve: vi.fn().mockResolvedValue({
              code: 0,
              data: {
                item: {
                  monthCreditPoints: "9600",
                  monthDebitPoints: "7200",
                  totalPoints: "1230",
                },
              },
            }),
          },
        },
        holds: { list: vi.fn() },
      },
      tokenBank: {
        account: {
          retrieve: vi.fn().mockResolvedValue({
            code: 0,
            data: {
              item: {
                availableAmount: "4200",
                frozenAmount: "200",
              },
            },
          }),
        },
        holds: {
          list: vi.fn().mockResolvedValue({
            code: 0,
            data: {
              items: [
                {
                  amount: "1500",
                  assetType: "token_bank",
                  businessNo: "AI-JOB-1001",
                  businessType: "token_bank.hold",
                  createdAt: "2026-04-01T10:00:00.000Z",
                  uuid: "token-bank-hold-1",
                  status: "held",
                },
              ],
              pageInfo: {
                mode: "offset",
                hasMore: false,
              },
            },
          }),
        },
        ledgerEntries: { list: vi.fn() },
        overview: { retrieve: vi.fn() },
      },
    } as any);

    const service = createSdkworkWalletService({
      accountAppService,
    });

    const overview = await service.getOverview({
      pageSize: 20,
    });

    expect(overview.isAuthenticated).toBe(true);
    expect(overview.account.availablePoints).toBe(1200);
    expect(overview.account.cashAvailable).toBe(88.5);
    expect(overview.account.tokenBankAvailable).toBe(4200);
    expect(overview.account.tokenBankFrozen).toBe(200);
    expect(overview.account.totalEarned).toBe(9600);
    expect(overview.account.totalSpent).toBe(7200);
    expect(overview.transactions).toHaveLength(2);
    expect(overview.transactions[0]).toMatchObject({
      id: "history-2",
      pointsDelta: -240,
      tokenBankDelta: 0,
      title: "POINTS_USAGE",
    });
    expect(overview.transactions[1]).toMatchObject({
      id: "cash-history-1",
      cashAmountCny: 6,
      title: "CASH_WITHDRAW",
    });
    expect(overview.holds).toHaveLength(1);
    expect(overview.holds[0]).toMatchObject({
      amount: 1500,
      assetType: "token_bank",
      businessNo: "AI-JOB-1001",
      holdId: "token-bank-hold-1",
      status: "held",
    });
    expect(accountAppService.tokenBank.account.retrieve).toHaveBeenCalledTimes(1);
    expect(accountAppService.tokenBank.holds.list).toHaveBeenCalledWith({
      page: 1,
      pageSize: 20,
    });
  });

  it("returns a guest-safe empty overview when runtime auth is missing", async () => {
    resetAccountServiceMockSession();
    const service = createSdkworkWalletService();

    const overview = await service.getOverview();

    expect(overview.isAuthenticated).toBe(false);
    expect(overview.account.availablePoints).toBe(0);
    expect(overview.transactions).toEqual([]);
    expect(overview.holds).toEqual([]);
  });

  it("rejects recharge without order SDK and withdraw until payout flows are implemented", async () => {
    const service = createSdkworkWalletService({
      accountAppService: createAccountAppServiceMock(),
    });

    await expect(
      service.rechargePoints({
        paymentMethod: "WECHAT",
        points: 1200,
      }),
    ).rejects.toThrow(/sdkwork-order/i);

    await expect(
      service.withdrawCash({
        accountName: "SDKWORK Ops",
        accountNo: "6222020202020202",
        amountCny: 12.5,
        destinationCode: "bank_account",
      }),
    ).rejects.toThrow(/sdkwork-payment/i);
  });

  it("maps recharge packages from order SDK into wallet overview", async () => {
    const accountAppService = createAccountAppServiceMock({
      wallet: {
        accounts: {
          cash: { retrieve: vi.fn().mockResolvedValue({ code: 0, data: { item: {} } }) },
          points: { retrieve: vi.fn().mockResolvedValue({ code: 0, data: { item: {} } }) },
        },
        ledgerEntries: {
          list: vi.fn().mockResolvedValue({ code: 0, data: { items: [], pageInfo: { mode: "offset", hasMore: false } } }),
        },
        points: {
          summary: {
            retrieve: vi.fn().mockResolvedValue({ code: 0, data: { item: {} } }),
          },
        },
        holds: { list: vi.fn() },
      },
      tokenBank: {
        account: { retrieve: vi.fn().mockResolvedValue({ code: 0, data: { item: {} } }) },
        holds: {
          list: vi.fn().mockResolvedValue({
            code: 0,
            data: {
              items: [],
              pageInfo: {
                mode: "offset",
                hasMore: false,
              },
            },
          }),
        },
        ledgerEntries: { list: vi.fn() },
        overview: { retrieve: vi.fn() },
      },
    } as any);
    const orderAppService = {
      recharges: {
        packages: {
          list: vi.fn().mockResolvedValue({
            code: 0,
            data: {
              items: [
                {
                  id: "pkg-1",
                  points: "1000",
                  priceAmount: "10.00",
                  currencyCode: "CNY",
                  title: "Starter pack",
                },
              ],
              pageInfo: {
                mode: "offset",
                hasMore: false,
              },
            },
          }),
        },
        settings: {
          retrieve: vi.fn().mockResolvedValue({
            code: 0,
            data: {
              item: {
                basePointsPerCny: "100",
              },
            },
          }),
        },
        orders: {
          create: vi.fn(),
          retrieve: vi.fn(),
          list: vi.fn(),
          cancel: vi.fn(),
        },
      },
      orders: {} as never,
    };

    const service = createSdkworkWalletService({
      accountAppService,
      orderAppService,
    });

    const overview = await service.getOverview();
    expect(overview.rechargePackages).toHaveLength(1);
    expect(overview.rechargePackages[0]).toMatchObject({
      id: 1,
      points: 1000,
      priceCny: 10,
      title: "Starter pack",
    });
    expect(overview.pointsToCashRate).toBe(100);
  });
});
