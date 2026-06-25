import {
  createContext,
  useContext,
  useMemo,
  type PropsWithChildren,
} from "react";
import {
  formatSdkworkAccountCurrencyCny,
  formatSdkworkAccountPoints,
  formatSdkworkAccountPointsRate,
} from "@sdkwork/account-service";
import type {
  SdkworkWalletAccount,
  SdkworkWalletRechargePackage,
  SdkworkWalletTransaction,
} from "./wallet-service";
import {
  formatSdkworkWalletDelta,
  type SdkworkWalletWithdrawDestinationCode,
} from "./wallet";
import {
  createSdkworkWalletMessages,
  normalizeSdkworkWalletLocale,
  type SdkworkWalletMessages,
  type SdkworkWalletMessagesOverrides,
} from "./wallet-copy";

export interface SdkworkWalletIntlValue {
  copy: SdkworkWalletMessages;
  formatAccountLevelLabel: (account: Pick<SdkworkWalletAccount, "level" | "levelName">) => string;
  formatAccountLevelSummary: (account: Pick<SdkworkWalletAccount, "level" | "levelName">) => string;
  formatAccountState: (
    account: Pick<SdkworkWalletAccount, "status" | "statusName" | "totalPoints">,
    isAuthenticated: boolean,
  ) => string;
  formatCurrencyCny: (value: number | null | undefined) => string;
  formatDurationDays: (value: number | null | undefined) => string;
  formatPaymentMethod: (value: string | null | undefined) => string;
  formatPayProtection: (enabled: boolean) => string;
  formatPoints: (value: number) => string;
  formatPointsIncluded: (value: number | null | undefined) => string;
  formatPointsRate: (value: number | null | undefined) => string;
  formatProjectedBalance: (value: number | null | undefined) => string;
  formatRechargePackageSummary: (
    rechargePackage: Pick<SdkworkWalletRechargePackage, "points" | "priceCny">,
  ) => string;
  formatTransactionStatus: (status: string | undefined) => string;
  formatTransactionTimestamp: (value: string) => string;
  formatWalletDelta: (value: number) => string;
  formatWithdrawDestinationDescription: (code: SdkworkWalletWithdrawDestinationCode | string | undefined) => string;
  formatWithdrawDestinationLabel: (code: SdkworkWalletWithdrawDestinationCode | string | undefined) => string;
  formatWithdrawRemarks: (label: string) => string;
  locale: string;
}

export interface SdkworkWalletIntlProviderProps extends PropsWithChildren {
  locale?: string | null;
  messages?: SdkworkWalletMessagesOverrides;
}

function interpolateSdkworkWalletTemplate(
  template: string,
  values: Record<string, string>,
): string {
  return Object.entries(values).reduce(
    (output, [key, value]) => output.replaceAll(`{${key}}`, value),
    template,
  );
}

function normalizePaymentMethodKey(value: string | null | undefined): keyof SdkworkWalletMessages["paymentMethod"] {
  const normalized = String(value || "").trim().toUpperCase();

  if (normalized === "ALIPAY") {
    return "ALIPAY";
  }

  if (normalized === "BANKCARD" || normalized === "BANK_CARD") {
    return "BANKCARD";
  }

  return "WECHAT";
}

function normalizeWithdrawDestinationKey(
  value: string | null | undefined,
): keyof SdkworkWalletMessages["withdrawDestination"] {
  const normalized = String(value || "").trim();

  if (normalized === "bank_account") {
    return "bank_account";
  }

  if (normalized === "ALIPAY") {
    return "ALIPAY";
  }

  return "WECHAT_PAY";
}

function createSdkworkWalletIntlValue(
  locale?: string | null,
  overrides?: SdkworkWalletMessagesOverrides,
): SdkworkWalletIntlValue {
  const resolvedLocale = normalizeSdkworkWalletLocale(locale);
  const copy = createSdkworkWalletMessages(resolvedLocale, overrides);

  function formatCurrency(value: number | null | undefined): string {
    return formatSdkworkAccountCurrencyCny(value, resolvedLocale);
  }

  function formatPointsValue(value: number): string {
    return formatSdkworkAccountPoints(value, resolvedLocale);
  }

  function resolveAccountLevelLabel(
    account: Pick<SdkworkWalletAccount, "level" | "levelName">,
  ): string {
    const levelName = account.levelName?.trim();
    if (levelName) {
      return levelName;
    }

    if (account.level !== null && account.level !== undefined) {
      return interpolateSdkworkWalletTemplate(copy.format.accountLevelValue, {
        value: String(account.level),
      });
    }

    return copy.account.standard;
  }

  return {
    copy,
    formatAccountLevelLabel(account) {
      return resolveAccountLevelLabel(account);
    },
    formatAccountLevelSummary(account) {
      return interpolateSdkworkWalletTemplate(copy.format.accountLevelSuffixValue, {
        value: resolveAccountLevelLabel(account),
      });
    },
    formatAccountState(account, isAuthenticated) {
      if (!isAuthenticated) {
        return copy.balancePanel.signInToUnlock;
      }

      const status = account.statusName?.trim() || account.status?.trim();
      if (status) {
        return status;
      }

      return interpolateSdkworkWalletTemplate(copy.format.totalPointsValue, {
        value: formatPointsValue(account.totalPoints),
      });
    },
    formatCurrencyCny: formatCurrency,
    formatDurationDays(value) {
      if (value === null || value === undefined) {
        return copy.common.flexibleDuration;
      }

      return `${value} ${copy.common.days}`;
    },
    formatPaymentMethod(value) {
      return copy.paymentMethod[normalizePaymentMethodKey(value)];
    },
    formatPayProtection(enabled) {
      return enabled ? copy.common.configured : copy.balancePanel.notConfigured;
    },
    formatPoints: formatPointsValue,
    formatPointsIncluded(value) {
      return interpolateSdkworkWalletTemplate(copy.format.pointsIncludedValue, {
        points: formatPointsValue(value ?? 0),
      });
    },
    formatPointsRate(value) {
      if (!value || value <= 0) {
        return formatSdkworkAccountPointsRate(value ?? 0, resolvedLocale);
      }

      if (resolvedLocale === "zh-CN") {
        return `${formatPointsValue(value)} ${copy.headerEntry.pointsSuffix} / 1 元`;
      }

      return `${formatPointsValue(value)} ${copy.headerEntry.pointsSuffix} / CNY 1`;
    },
    formatProjectedBalance(value) {
      return interpolateSdkworkWalletTemplate(copy.format.projectedBalanceValue, {
        value: formatCurrency(value),
      });
    },
    formatRechargePackageSummary(rechargePackage) {
      return interpolateSdkworkWalletTemplate(copy.format.pointsForPriceValue, {
        points: formatPointsValue(rechargePackage.points),
        price: formatCurrency(rechargePackage.priceCny),
      });
    },
    formatTransactionStatus(status) {
      const normalized = String(status || "").trim().toUpperCase();

      if (normalized === "SUCCESS" || normalized === "COMPLETED") {
        return copy.status.completed;
      }

      if (normalized === "FAILED") {
        return copy.status.failed;
      }

      return copy.status.pending;
    },
    formatTransactionTimestamp(value) {
      const timestamp = new Date(value);

      if (Number.isNaN(timestamp.getTime())) {
        return value;
      }

      return new Intl.DateTimeFormat(resolvedLocale, {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(timestamp);
    },
    formatWalletDelta(value) {
      return formatSdkworkWalletDelta(value, resolvedLocale);
    },
    formatWithdrawDestinationDescription(code) {
      return copy.withdrawDestination[normalizeWithdrawDestinationKey(code)].description;
    },
    formatWithdrawDestinationLabel(code) {
      return copy.withdrawDestination[normalizeWithdrawDestinationKey(code)].label;
    },
    formatWithdrawRemarks(label) {
      return interpolateSdkworkWalletTemplate(copy.format.remarksValue, {
        value: label,
      });
    },
    locale: resolvedLocale,
  };
}

const DEFAULT_SDKWORK_WALLET_INTL = createSdkworkWalletIntlValue();

const SdkworkWalletIntlContext = createContext<SdkworkWalletIntlValue>(
  DEFAULT_SDKWORK_WALLET_INTL,
);

export function SdkworkWalletIntlProvider({
  children,
  locale,
  messages,
}: SdkworkWalletIntlProviderProps) {
  const value = useMemo(
    () => createSdkworkWalletIntlValue(locale, messages),
    [locale, messages],
  );

  return (
    <SdkworkWalletIntlContext.Provider value={value}>
      {children}
    </SdkworkWalletIntlContext.Provider>
  );
}

export function useSdkworkWalletIntl(): SdkworkWalletIntlValue {
  return useContext(SdkworkWalletIntlContext);
}
