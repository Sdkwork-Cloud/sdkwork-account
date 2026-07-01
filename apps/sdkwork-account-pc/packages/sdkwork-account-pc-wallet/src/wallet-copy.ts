export type SdkworkWalletLocale = "en-US" | "zh-CN";

export type SdkworkWalletMessagesOverrides = DeepPartial<SdkworkWalletMessages>;

export interface SdkworkWalletMessages {
  account: {
    standard: string;
  };
  actions: {
    cancel: string;
    confirmPayment: string;
    confirmRecharge: string;
    confirmWithdraw: string;
    continueToCheckout: string;
    openCenter: string;
    recharge: string;
    rechargeWallet: string;
    withdraw: string;
  };
  balancePanel: {
    accountLevelLabel: string;
    availablePointsLabel: string;
    cashAvailableLabel: string;
    description: string;
    exchangeRateLabel: string;
    eyebrow: string;
    noRechargePackagePublished: string;
    notConfigured: string;
    payProtectionLabel: string;
    primaryAction: string;
    rechargeLaneLabel: string;
    signInToUnlock: string;
    title: string;
    accountStatusLabel: string;
  };
  common: {
    configured: string;
    days: string;
    flexibleDuration: string;
    points: string;
  };
  format: {
    accountLevelSuffixValue: string;
    accountLevelValue: string;
    daysRemainingValue: string;
    pointsForPriceValue: string;
    pointsIncludedValue: string;
    projectedBalanceValue: string;
    remarksValue: string;
    totalPointsValue: string;
  };
  headerEntry: {
    balanceAriaLabel: string;
    pointsSuffix: string;
  };
  page: {
    errorTitle: string;
    loading: string;
  };
  paymentMethod: {
    ALIPAY: string;
    BANKCARD: string;
    WECHAT: string;
  };
  quickPanel: {
    availablePointsLabel: string;
    cashAvailableLabel: string;
    currentAccountLabel: string;
    noRechargePackageDescription: string;
    noRecentActivity: string;
    openCenterAction: string;
    rateLabel: string;
    recentActivityTitle: string;
    rechargeLaneLabel: string;
    signInToUnlock: string;
  };
  rechargeDialog: {
    checkoutFlowDescription: string;
    customAmountLabel: string;
    customAmountPlaceholder: string;
    description: string;
    estimatedPriceLabel: string;
    noPackagesDescription: string;
    noPackagesTitle: string;
    packageGridLabel: string;
    paymentMethodLabel: string;
    rateLabel: string;
    recommendedBadge: string;
    selectedLabel: string;
    selectionEyebrow: string;
    signInRequiredDescription: string;
    signInRequiredTitle: string;
    summaryEyebrow: string;
    title: string;
  };
  status: {
    completed: string;
    failed: string;
    pending: string;
  };
  summaryCards: {
    accountLevelLabel: string;
    cashAvailableLabel: string;
    totalEarnedLabel: string;
    totalSpentLabel: string;
  };
  transactionList: {
    columnAmount: string;
    columnDescription: string;
    columnPoints: string;
    columnStatus: string;
    columnTime: string;
    description: string;
    emptyDescription: string;
    emptyTitle: string;
    fallbackType: string;
    title: string;
  };
  holdList: {
    assetTypeLabel: string;
    description: string;
    emptyDescription: string;
    emptyTitle: string;
    fallbackType: string;
    title: string;
    unknownAsset: string;
  };
  holdStatus: {
    expired: string;
    held: string;
    released: string;
    settled: string;
  };
  withdrawDestination: {
    ALIPAY: {
      description: string;
      label: string;
    };
    WECHAT_PAY: {
      description: string;
      label: string;
    };
    bank_account: {
      description: string;
      label: string;
    };
  };
  withdrawDialog: {
    accountNameLabel: string;
    accountNamePlaceholder: string;
    accountNoLabel: string;
    accountNoPlaceholder: string;
    amountLabel: string;
    amountPlaceholder: string;
    availableCashEyebrow: string;
    bankNameLabel: string;
    bankNamePlaceholder: string;
    description: string;
    insufficientDescription: string;
    insufficientTitle: string;
    invalidRequestNoDescription: string;
    invalidRequestNoTitle: string;
    payoutFlowDescription: string;
    payoutRailLabel: string;
    requestNoLabel: string;
    requestNoPlaceholder: string;
    signInRequiredDescription: string;
    signInRequiredTitle: string;
    title: string;
    withdrawDestinationLabel: string;
  };
}

type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends (...args: never[]) => unknown
    ? T[K]
    : T[K] extends object
      ? DeepPartial<T[K]>
      : T[K];
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function mergeDeep<T>(base: T, overrides?: DeepPartial<T>): T {
  if (!overrides) {
    return base;
  }

  const output: Record<string, unknown> = {
    ...(base as Record<string, unknown>),
  };

  for (const [key, value] of Object.entries(overrides)) {
    if (value === undefined) {
      continue;
    }

    const baseValue = output[key];
    output[key] = isRecord(baseValue) && isRecord(value)
      ? mergeDeep(baseValue, value)
      : value;
  }

  return output as T;
}

const EN_US_MESSAGES: SdkworkWalletMessages = {
  account: {
    standard: "Standard",
  },
  actions: {
    cancel: "Cancel",
    confirmPayment: "Confirm payment",
    confirmRecharge: "Confirm recharge",
    confirmWithdraw: "Confirm withdraw",
    continueToCheckout: "Continue to checkout",
    openCenter: "Open center",
    recharge: "Recharge",
    rechargeWallet: "Recharge wallet",
    withdraw: "Withdraw cash",
  },
  balancePanel: {
    accountLevelLabel: "Account level",
    availablePointsLabel: "Available points",
    cashAvailableLabel: "Cash available",
    description: "View balance, recharge, and transaction history.",
    exchangeRateLabel: "Exchange rate",
    eyebrow: "Commerce",
    noRechargePackagePublished: "No recharge package published",
    notConfigured: "Not configured",
    payProtectionLabel: "Pay protection",
    primaryAction: "Recharge wallet",
    rechargeLaneLabel: "Recharge lane",
    signInToUnlock: "Sign in to unlock wallet billing.",
    title: "Wallet",
    accountStatusLabel: "Account status",
  },
  common: {
    configured: "Configured",
    days: "days",
    flexibleDuration: "Flexible duration",
    points: "points",
  },
  format: {
    accountLevelSuffixValue: "{value} account",
    accountLevelValue: "LV {value}",
    daysRemainingValue: "{value} days remaining",
    pointsForPriceValue: "{points} for {price}",
    pointsIncludedValue: "{points} points included",
    projectedBalanceValue: "Projected balance after payout: {value}",
    remarksValue: "Withdrawal via {value}",
    totalPointsValue: "{value} total points",
  },
  headerEntry: {
    balanceAriaLabel: "Wallet balance",
    pointsSuffix: "pts",
  },
  page: {
    errorTitle: "Wallet center error",
    loading: "Loading wallet center...",
  },
  paymentMethod: {
    ALIPAY: "Alipay",
    BANKCARD: "Bank card",
    WECHAT: "WeChat Pay",
  },
  quickPanel: {
    availablePointsLabel: "Available points",
    cashAvailableLabel: "Cash available",
    currentAccountLabel: "Current account",
    noRechargePackageDescription: "Publish recharge packages in the current commerce service.",
    noRecentActivity: "No recent activity yet.",
    openCenterAction: "Open center",
    rateLabel: "Rate",
    recentActivityTitle: "Recent activity",
    rechargeLaneLabel: "Recharge lane",
    signInToUnlock: "Sign in to unlock wallet billing.",
  },
  rechargeDialog: {
    checkoutFlowDescription: "Checkout creates a unified commerce_order via sdkwork-order and continues in sdkwork-payment.",
    customAmountLabel: "Custom amount",
    customAmountPlaceholder: "Enter points",
    description: "Add points to your balance with a package or custom amount.",
    estimatedPriceLabel: "Estimated price",
    noPackagesDescription: "Recharge packages are published by sdkwork-order. Continue to checkout to create a commerce_order (subject=points_recharge) or enter a custom amount.",
    noPackagesTitle: "No recharge packages available",
    packageGridLabel: "Choose a package",
    paymentMethodLabel: "Payment method",
    rateLabel: "Rate",
    recommendedBadge: "Popular",
    selectedLabel: "Selected",
    selectionEyebrow: "Top up",
    signInRequiredDescription: "Recharge actions require an authenticated wallet session.",
    signInRequiredTitle: "Sign in required",
    summaryEyebrow: "Order summary",
    title: "Recharge balance",
  },
  status: {
    completed: "Completed",
    failed: "Failed",
    pending: "Pending",
  },
  summaryCards: {
    accountLevelLabel: "Account level",
    cashAvailableLabel: "Cash available",
    totalEarnedLabel: "Total earned",
    totalSpentLabel: "Total spent",
  },
  transactionList: {
    columnAmount: "Amount",
    columnDescription: "Description",
    columnPoints: "Points",
    columnStatus: "Status",
    columnTime: "Time",
    description: "Unified recharge, consumption, withdrawal, and balance events.",
    emptyDescription: "Once balance events start flowing, they will appear here.",
    emptyTitle: "No wallet activity yet",
    fallbackType: "Wallet transaction",
    title: "Wallet activity",
  },
  holdList: {
    assetTypeLabel: "Asset",
    description: "Active and recent pre-authorization holds on wallet balances.",
    emptyDescription: "When checkout or order flows create holds, they will appear here.",
    emptyTitle: "No holds yet",
    fallbackType: "Wallet hold",
    title: "Balance holds",
    unknownAsset: "unknown",
  },
  holdStatus: {
    expired: "Expired",
    held: "Held",
    released: "Released",
    settled: "Settled",
  },
  withdrawDestination: {
    ALIPAY: {
      description: "Submit the payout to the linked Alipay settlement rail.",
      label: "Alipay",
    },
    WECHAT_PAY: {
      description: "Submit the payout to the linked WeChat Pay settlement rail.",
      label: "WeChat Pay",
    },
    bank_account: {
      description: "Route the payout through the linked settlement bank account.",
      label: "Bank account",
    },
  },
  withdrawDialog: {
    accountNameLabel: "Account name",
    accountNamePlaceholder: "Enter payee name",
    accountNoLabel: "Account number",
    accountNoPlaceholder: "Enter settlement account",
    amountLabel: "Amount",
    amountPlaceholder: "Enter amount in CNY",
    availableCashEyebrow: "Available cash",
    bankNameLabel: "Bank name",
    bankNamePlaceholder: "Enter bank name",
    description: "Submit a payout request through the shared wallet operations lane and keep desktop commerce apps aligned on one withdraw workflow.",
    insufficientDescription: "Reduce the withdrawal amount so it does not exceed the current withdrawable balance.",
    insufficientTitle: "Insufficient available cash",
    invalidRequestNoDescription: "Request no must use 6-64 letters, numbers, underscores, or hyphens.",
    invalidRequestNoTitle: "Invalid request number",
    payoutFlowDescription: "Payout settlement continues in sdkwork-payment; account debits after payout completes.",
    payoutRailLabel: "Payout rail",
    requestNoLabel: "Request no",
    requestNoPlaceholder: "Optional idempotency key",
    signInRequiredDescription: "Withdrawal actions require an authenticated wallet session.",
    signInRequiredTitle: "Sign in required",
    title: "Withdraw balance",
    withdrawDestinationLabel: "Withdraw destination",
  },
};

const ZH_CN_MESSAGES: SdkworkWalletMessages = {
  account: {
    standard: "\u6807\u51c6\u8d26\u6237",
  },
  actions: {
    cancel: "\u53d6\u6d88",
    confirmPayment: "\u786e\u8ba4\u652f\u4ed8",
    confirmRecharge: "\u786e\u8ba4\u5145\u503c",
    confirmWithdraw: "\u786e\u8ba4\u63d0\u73b0",
    continueToCheckout: "\u524d\u5f80\u7ed3\u7b97",
    openCenter: "\u6253\u5f00\u4e2d\u5fc3",
    recharge: "\u5145\u503c",
    rechargeWallet: "\u5145\u503c\u94b1\u5305",
    withdraw: "\u63d0\u73b0",
  },
  balancePanel: {
    accountLevelLabel: "\u8d26\u6237\u7b49\u7ea7",
    availablePointsLabel: "\u53ef\u7528\u79ef\u5206",
    cashAvailableLabel: "\u53ef\u63d0\u73b0\u4f59\u989d",
    description: "\u67e5\u770b\u4f59\u989d\u3001\u5145\u503c\u4e0e\u4ea4\u6613\u8bb0\u5f55\u3002",
    exchangeRateLabel: "\u5151\u6362\u6bd4\u4f8b",
    eyebrow: "\u5546\u4e1a\u5316",
    noRechargePackagePublished: "\u5f53\u524d\u672a\u53d1\u5e03\u5145\u503c\u5305",
    notConfigured: "\u672a\u914d\u7f6e",
    payProtectionLabel: "\u652f\u4ed8\u4fdd\u62a4",
    primaryAction: "\u5145\u503c\u94b1\u5305",
    rechargeLaneLabel: "\u5145\u503c\u901a\u9053",
    signInToUnlock: "\u767b\u5f55\u540e\u5373\u53ef\u89e3\u9501\u94b1\u5305\u5546\u4e1a\u5316\u80fd\u529b\u3002",
    title: "\u94b1\u5305",
    accountStatusLabel: "\u8d26\u6237\u72b6\u6001",
  },
  common: {
    configured: "\u5df2\u914d\u7f6e",
    days: "\u5929",
    flexibleDuration: "\u65f6\u957f\u7075\u6d3b",
    points: "\u79ef\u5206",
  },
  format: {
    accountLevelSuffixValue: "{value} \u8d26\u6237",
    accountLevelValue: "LV {value}",
    daysRemainingValue: "\u5269\u4f59 {value} \u5929",
    pointsForPriceValue: "{points}\uff0c\u4ef7\u683c {price}",
    pointsIncludedValue: "\u5305\u542b {points} \u79ef\u5206",
    projectedBalanceValue: "\u63d0\u73b0\u540e\u9884\u8ba1\u4f59\u989d\uff1a{value}",
    remarksValue: "\u901a\u8fc7 {value} \u53d1\u8d77\u63d0\u73b0",
    totalPointsValue: "{value} \u603b\u79ef\u5206",
  },
  headerEntry: {
    balanceAriaLabel: "\u94b1\u5305\u4f59\u989d",
    pointsSuffix: "\u79ef\u5206",
  },
  page: {
    errorTitle: "\u94b1\u5305\u4e2d\u5fc3\u5f02\u5e38",
    loading: "\u6b63\u5728\u52a0\u8f7d\u94b1\u5305\u4e2d\u5fc3...",
  },
  paymentMethod: {
    ALIPAY: "\u652f\u4ed8\u5b9d",
    BANKCARD: "\u94f6\u884c\u5361",
    WECHAT: "\u5fae\u4fe1\u652f\u4ed8",
  },
  quickPanel: {
    availablePointsLabel: "\u53ef\u7528\u79ef\u5206",
    cashAvailableLabel: "\u53ef\u63d0\u73b0\u4f59\u989d",
    currentAccountLabel: "\u5f53\u524d\u8d26\u6237",
    noRechargePackageDescription: "\u8bf7\u5148\u5728\u5f53\u524d\u5546\u4e1a\u5316\u670d\u52a1\u4e2d\u53d1\u5e03\u5145\u503c\u5305\u3002",
    noRecentActivity: "\u5f53\u524d\u8fd8\u6ca1\u6709\u6700\u8fd1\u6d3b\u52a8\u3002",
    openCenterAction: "\u6253\u5f00\u4e2d\u5fc3",
    rateLabel: "\u5151\u6362\u6bd4\u4f8b",
    recentActivityTitle: "\u6700\u8fd1\u6d3b\u52a8",
    rechargeLaneLabel: "\u5145\u503c\u901a\u9053",
    signInToUnlock: "\u767b\u5f55\u540e\u5373\u53ef\u89e3\u9501\u94b1\u5305\u5546\u4e1a\u5316\u80fd\u529b\u3002",
  },
  rechargeDialog: {
    checkoutFlowDescription: "\u7ed3\u7b97\u9875\u7531 sdkwork-order \u521b\u5efa\u7edf\u4e00 commerce_order\uff08subject=points_recharge\uff09\uff0c\u7531 sdkwork-payment \u7ee7\u7eed\u6536\u6b3e\u3002",
    customAmountLabel: "\u81ea\u5b9a\u4e49\u6570\u91cf",
    customAmountPlaceholder: "\u8f93\u5165\u79ef\u5206\u6570\u91cf",
    description: "\u9009\u62e9\u5957\u9910\u6216\u81ea\u5b9a\u4e49\u91d1\u989d\u4e3a\u8d26\u6237\u5145\u503c\u3002",
    estimatedPriceLabel: "\u9884\u8ba1\u4ef7\u683c",
    noPackagesDescription: "\u5145\u503c\u5305\u7531 sdkwork-order \u53d1\u5e03\u3002\u524d\u5f80\u7ed3\u7b97\u521b\u5efa\u7edf\u4e00\u8ba2\u5355\uff08subject=points_recharge\uff09\uff0c\u6216\u5728\u4e0b\u65b9\u8f93\u5165\u81ea\u5b9a\u4e49\u6570\u91cf\u3002",
    noPackagesTitle: "\u6682\u65e0\u5145\u503c\u5305",
    packageGridLabel: "\u9009\u62e9\u5145\u503c\u5957\u9910",
    paymentMethodLabel: "\u652f\u4ed8\u65b9\u5f0f",
    rateLabel: "\u5151\u6362\u6bd4\u4f8b",
    recommendedBadge: "\u63a8\u8350",
    selectedLabel: "\u5df2\u9009\u62e9",
    selectionEyebrow: "\u5145\u503c",
    signInRequiredDescription: "\u5145\u503c\u64cd\u4f5c\u9700\u8981\u5df2\u767b\u5f55\u7684\u94b1\u5305\u4f1a\u8bdd\u3002",
    signInRequiredTitle: "\u9700\u8981\u767b\u5f55",
    summaryEyebrow: "\u8ba2\u5355\u6458\u8981",
    title: "\u5145\u503c\u4f59\u989d",
  },
  status: {
    completed: "\u5df2\u5b8c\u6210",
    failed: "\u5931\u8d25",
    pending: "\u5904\u7406\u4e2d",
  },
  summaryCards: {
    accountLevelLabel: "\u8d26\u6237\u7b49\u7ea7",
    cashAvailableLabel: "\u53ef\u63d0\u73b0\u4f59\u989d",
    totalEarnedLabel: "\u7d2f\u8ba1\u83b7\u5f97",
    totalSpentLabel: "\u7d2f\u8ba1\u6d88\u8017",
  },
  transactionList: {
    columnAmount: "\u91d1\u989d",
    columnDescription: "\u8bf4\u660e",
    columnPoints: "\u79ef\u5206",
    columnStatus: "\u72b6\u6001",
    columnTime: "\u65f6\u95f4",
    description: "\u7edf\u4e00\u5c55\u793a\u5145\u503c\u3001\u6d88\u8017\u3001\u63d0\u73b0\u548c\u4f59\u989d\u4e8b\u4ef6\u3002",
    emptyDescription: "\u5f53\u4f59\u989d\u4e8b\u4ef6\u5f00\u59cb\u6d41\u8f6c\u540e\uff0c\u5b83\u4eec\u4f1a\u51fa\u73b0\u5728\u8fd9\u91cc\u3002",
    emptyTitle: "\u6682\u65e0\u94b1\u5305\u6d3b\u52a8",
    fallbackType: "\u94b1\u5305\u4ea4\u6613",
    title: "\u94b1\u5305\u6d3b\u52a8",
  },
  holdList: {
    assetTypeLabel: "\u8d44\u4ea7",
    description: "\u94b1\u5305\u4f59\u989d\u7684\u9884\u6263\u4e0e\u6700\u8fd1\u72b6\u6001\u3002",
    emptyDescription: "\u5f53\u652f\u4ed8\u6216\u8ba2\u5355\u6d41\u7a0b\u521b\u5efa\u9884\u6263\u540e\uff0c\u8bb0\u5f55\u4f1a\u51fa\u73b0\u5728\u8fd9\u91cc\u3002",
    emptyTitle: "\u6682\u65e0\u9884\u6263\u8bb0\u5f55",
    fallbackType: "\u94b1\u5305\u9884\u6263",
    title: "\u4f59\u989d\u9884\u6263",
    unknownAsset: "\u672a\u77e5",
  },
  holdStatus: {
    expired: "\u5df2\u8fc7\u671f",
    held: "\u9884\u6263\u4e2d",
    released: "\u5df2\u91ca\u653e",
    settled: "\u5df2\u7ed3\u7b97",
  },
  withdrawDestination: {
    ALIPAY: {
      description: "\u5c06\u672c\u6b21\u63d0\u73b0\u63d0\u4ea4\u5230\u5df2\u7ed1\u5b9a\u7684\u652f\u4ed8\u5b9d\u7ed3\u7b97\u901a\u9053\u3002",
      label: "\u652f\u4ed8\u5b9d",
    },
    WECHAT_PAY: {
      description: "\u5c06\u672c\u6b21\u63d0\u73b0\u63d0\u4ea4\u5230\u5df2\u7ed1\u5b9a\u7684\u5fae\u4fe1\u652f\u4ed8\u7ed3\u7b97\u901a\u9053\u3002",
      label: "\u5fae\u4fe1\u652f\u4ed8",
    },
    bank_account: {
      description: "\u5c06\u672c\u6b21\u63d0\u73b0\u8def\u7531\u5230\u5df2\u7ed1\u5b9a\u7684\u94f6\u884c\u5361\u7ed3\u7b97\u8d26\u6237\u3002",
      label: "\u94f6\u884c\u5361\u8d26\u6237",
    },
  },
  withdrawDialog: {
    accountNameLabel: "\u6536\u6b3e\u4eba\u59d3\u540d",
    accountNamePlaceholder: "\u8f93\u5165\u6536\u6b3e\u4eba\u59d3\u540d",
    accountNoLabel: "\u6536\u6b3e\u8d26\u53f7",
    accountNoPlaceholder: "\u8f93\u5165\u7ed3\u7b97\u8d26\u53f7",
    amountLabel: "\u63d0\u73b0\u91d1\u989d",
    amountPlaceholder: "\u8f93\u5165\u4eba\u6c11\u5e01\u91d1\u989d",
    availableCashEyebrow: "\u53ef\u63d0\u73b0\u4f59\u989d",
    bankNameLabel: "\u5f00\u6237\u884c\u540d\u79f0",
    bankNamePlaceholder: "\u8f93\u5165\u5f00\u6237\u884c\u540d\u79f0",
    description: "\u901a\u8fc7\u7edf\u4e00\u7684\u94b1\u5305\u8fd0\u8425\u901a\u9053\u63d0\u4ea4\u63d0\u73b0\u7533\u8bf7\uff0c\u8ba9\u684c\u9762\u5546\u4e1a\u5316\u5e94\u7528\u4fdd\u6301\u4e00\u81f4\u7684\u63d0\u73b0\u5de5\u4f5c\u6d41\u3002",
    insufficientDescription: "\u8bf7\u964d\u4f4e\u63d0\u73b0\u91d1\u989d\uff0c\u786e\u4fdd\u5b83\u4e0d\u8d85\u8fc7\u5f53\u524d\u53ef\u63d0\u73b0\u4f59\u989d\u3002",
    insufficientTitle: "\u53ef\u63d0\u73b0\u4f59\u989d\u4e0d\u8db3",
    invalidRequestNoDescription: "\u8bf7\u6c42\u53f7\u5fc5\u987b\u4f7f\u7528 6-64 \u4f4d\u5b57\u6bcd\u3001\u6570\u5b57\u3001\u4e0b\u5212\u7ebf\u6216\u8fde\u5b57\u7b26\u3002",
    invalidRequestNoTitle: "\u8bf7\u6c42\u53f7\u683c\u5f0f\u65e0\u6548",
    payoutFlowDescription: "\u63d0\u73b0\u6253\u6b3e\u5728 sdkwork-payment \u7ee7\u7eed\u5904\u7406\uff1b\u8d26\u6237\u51fa\u8d26\u5728\u6253\u6b3e\u6210\u529f\u540e\u7531 account \u5165\u8d26\u3002",
    payoutRailLabel: "\u63d0\u73b0\u901a\u9053",
    requestNoLabel: "\u8bf7\u6c42\u53f7",
    requestNoPlaceholder: "\u53ef\u9009\u7684\u5e42\u7b49\u8bf7\u6c42\u6807\u8bc6",
    signInRequiredDescription: "\u63d0\u73b0\u64cd\u4f5c\u9700\u8981\u5df2\u767b\u5f55\u7684\u94b1\u5305\u4f1a\u8bdd\u3002",
    signInRequiredTitle: "\u9700\u8981\u767b\u5f55",
    title: "\u63d0\u73b0\u4f59\u989d",
    withdrawDestinationLabel: "\u63d0\u73b0\u65b9\u5f0f",
  },
};

const SDKWORK_WALLET_MESSAGES: Record<SdkworkWalletLocale, SdkworkWalletMessages> = {
  "en-US": EN_US_MESSAGES,
  "zh-CN": ZH_CN_MESSAGES,
};

export function normalizeSdkworkWalletLocale(locale?: string | null): SdkworkWalletLocale {
  const normalized = String(locale || "").trim().toLowerCase();
  if (normalized.startsWith("zh")) {
    return "zh-CN";
  }

  return "en-US";
}

export function createSdkworkWalletMessages(
  locale?: string | null,
  overrides?: SdkworkWalletMessagesOverrides,
): SdkworkWalletMessages {
  return mergeDeep(
    SDKWORK_WALLET_MESSAGES[normalizeSdkworkWalletLocale(locale)],
    overrides,
  );
}
