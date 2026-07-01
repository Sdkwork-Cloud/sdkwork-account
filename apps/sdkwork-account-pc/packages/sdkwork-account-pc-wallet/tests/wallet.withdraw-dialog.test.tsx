import {
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react/theme";
import {
  SdkworkWalletWithdrawDialog,
  createSdkworkWalletController,
} from "../src";

const overview = {
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
  rechargePackages: [],
  transactions: [],
  holds: [],
};

function createDialogController(withdrawCash = vi.fn()) {
  return createSdkworkWalletController({
    initialState: {
      isBootstrapped: true,
      isLoading: false,
    },
    service: {
      getEmptyOverview: vi.fn().mockReturnValue(overview),
      getOverview: vi.fn().mockResolvedValue(overview),
      rechargePoints: vi.fn(),
      withdrawCash,
    },
  });
}

describe("sdkwork-account-pc-wallet withdraw dialog", () => {
  it("renders available cash, destination choices, and submit guardrails", () => {
    const withdrawCash = vi.fn();
    const controller = createDialogController(withdrawCash);

    render(
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkWalletWithdrawDialog controller={controller} open />
      </SdkworkThemeProvider>,
    );

    expect(screen.getByRole("heading", { name: /withdraw balance/i })).toBeInTheDocument();
    expect(screen.getByText(/available cash/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /bank account/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /alipay/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /wechat pay/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /confirm withdraw/i })).toBeDisabled();

    fireEvent.change(screen.getByLabelText(/amount/i), {
      target: {
        value: "12.50",
      },
    });
    fireEvent.change(screen.getByLabelText(/account name/i), {
      target: {
        value: "SDKWORK Ops",
      },
    });
    fireEvent.change(screen.getByLabelText(/account number/i), {
      target: {
        value: "6222020202020202",
      },
    });

    expect(screen.getByLabelText(/bank name/i)).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText(/request no/i), {
      target: {
        value: "bad request no",
      },
    });
    expect(screen.getByRole("button", { name: /confirm withdraw/i })).toBeDisabled();

    fireEvent.change(screen.getByLabelText(/request no/i), {
      target: {
        value: "REQ_WITHDRAW_300",
      },
    });
    fireEvent.change(screen.getByLabelText(/bank name/i), {
      target: {
        value: "SDKWORK Bank",
      },
    });
    expect(screen.getByRole("button", { name: /confirm withdraw/i })).not.toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: /alipay/i }));
    expect(screen.queryByLabelText(/bank name/i)).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /confirm withdraw/i }));
    expect(withdrawCash).toHaveBeenCalledWith({
      accountName: "SDKWORK Ops",
      accountNo: "6222020202020202",
      amountCny: 12.5,
      destinationCode: "ALIPAY",
      remarks: "Withdrawal via Alipay",
      requestNo: "REQ_WITHDRAW_300",
    });
  });

  it("keeps in-progress settlement fields stable while the dialog stays open", () => {
    const controller = createDialogController();

    render(
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkWalletWithdrawDialog controller={controller} open />
      </SdkworkThemeProvider>,
    );

    const amountInput = screen.getByLabelText(/amount/i);
    const accountNameInput = screen.getByLabelText(/account name/i);
    const accountNumberInput = screen.getByLabelText(/account number/i);
    const requestNoInput = screen.getByLabelText(/request no/i);

    fireEvent.change(amountInput, {
      target: {
        value: "12.50",
      },
    });
    fireEvent.change(accountNameInput, {
      target: {
        value: "SDKWORK Ops",
      },
    });
    fireEvent.change(accountNumberInput, {
      target: {
        value: "6222020202020202",
      },
    });
    fireEvent.change(requestNoInput, {
      target: {
        value: "REQ_WITHDRAW_300",
      },
    });

    expect(amountInput).toHaveValue("12.50");
    expect(accountNameInput).toHaveValue("SDKWORK Ops");
    expect(accountNumberInput).toHaveValue("6222020202020202");
    expect(requestNoInput).toHaveValue("REQ_WITHDRAW_300");

    fireEvent.click(screen.getByRole("button", { name: /alipay/i }));

    expect(screen.getByLabelText(/amount/i)).toHaveValue("12.50");
    expect(screen.getByLabelText(/account name/i)).toHaveValue("SDKWORK Ops");
    expect(screen.getByLabelText(/account number/i)).toHaveValue("6222020202020202");
    expect(screen.getByLabelText(/request no/i)).toHaveValue("REQ_WITHDRAW_300");
    expect(screen.queryByLabelText(/bank name/i)).not.toBeInTheDocument();
  });
});
