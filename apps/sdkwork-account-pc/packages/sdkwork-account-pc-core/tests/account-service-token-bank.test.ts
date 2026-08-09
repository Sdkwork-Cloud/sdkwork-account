import { describe, expect, it, vi } from "vitest";
import {
  createSdkworkAccountAppService,
  createSdkworkAccountBackendService,
} from "@sdkwork/account-service";

describe("sdkwork account service Token Bank facade", () => {
  it("exposes Token Bank app resources without forbidden wallet token aliases", async () => {
    const retrieveTokenBankAccount = vi.fn().mockResolvedValue({
      availableAmount: "4200",
      frozenAmount: "200",
    });
    const appService = createSdkworkAccountAppService({
      appClient: {
        commerce: {
          accounts: {
            current: {
              summary: { retrieve: vi.fn() },
            },
          },
          billing: {
            history: { list: vi.fn() },
          },
          tokenBank: {
            account: { retrieve: retrieveTokenBankAccount },
            holds: { list: vi.fn() },
            ledgerEntries: { list: vi.fn() },
            overview: { retrieve: vi.fn() },
          },
          wallet: {
            accounts: {
              cash: { retrieve: vi.fn() },
              list: vi.fn(),
              points: { retrieve: vi.fn() },
            },
            holds: { list: vi.fn(), retrieve: vi.fn() },
            ledgerEntries: {
              allocations: { list: vi.fn() },
              cash: { list: vi.fn() },
              list: vi.fn(),
              points: { list: vi.fn() },
              retrieve: vi.fn(),
            },
            overview: { retrieve: vi.fn() },
            portfolio: { list: vi.fn() },
            points: {
              lots: { list: vi.fn() },
              summary: { retrieve: vi.fn() },
            },
          },
        },
      },
    });

    await appService.tokenBank.account.retrieve();

    expect(retrieveTokenBankAccount).toHaveBeenCalledTimes(1);
    expect("tokens" in appService.wallet.accounts).toBe(false);
    expect("tokens" in appService.wallet).toBe(false);
  });

  it("exposes Token Bank backend commands without forbidden adjustment token aliases", async () => {
    const creditTokenBank = vi.fn().mockResolvedValue({ ledgerId: "ledger-1" });
    const backendService = createSdkworkAccountBackendService({
      backendClient: {
        commerce: {
          tokenBank: {
            credits: { create: creditTokenBank },
            debits: { create: vi.fn() },
            grants: { create: vi.fn() },
            holds: {
              create: vi.fn(),
              release: vi.fn(),
              settle: vi.fn(),
            },
            reversals: { create: vi.fn() },
            transfers: { create: vi.fn() },
          },
          wallet: {
            adjustments: {
              cash: { create: vi.fn() },
              create: vi.fn(),
              points: { create: vi.fn() },
            },
            health: { retrieve: vi.fn() },
            holds: {
              create: vi.fn(),
              expire: vi.fn(),
              release: vi.fn(),
              settle: vi.fn(),
            },
            outbox: { dispatch: vi.fn() },
            points: {
              lots: { expire: vi.fn() },
              reconciliation: vi.fn(),
            },
            transfers: { create: vi.fn() },
          },
        },
      },
    });

    await backendService.tokenBank.credits.create({ amount: "4200" });

    expect(creditTokenBank).toHaveBeenCalledTimes(1);
    expect("tokens" in backendService.wallet.adjustments).toBe(false);
  });
});
