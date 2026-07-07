import {
  toSdkworkAccountNumber,
  toSdkworkAccountOptionalString,
  unwrapSdkworkAccountListPage,
  unwrapSdkworkAccountResource,
} from "@sdkwork/account-service";
import type {
  SdkworkWalletRechargeInput,
  SdkworkWalletRechargePackage,
  SdkworkWalletRechargeResult,
} from "./wallet-service.ts";

const WALLET_PAYMENT_METHOD_ALIASES: Record<string, string> = {
  ALIPAY: "alipay",
  BANKCARD: "balance",
  WECHAT: "wechat_pay",
  WECHAT_PAY: "wechat_pay",
};

function normalizeWalletPaymentMethod(value: string | undefined): string {
  const trimmed = value?.trim();
  if (!trimmed) {
    return "wechat_pay";
  }
  const upper = trimmed.toUpperCase();
  return WALLET_PAYMENT_METHOD_ALIASES[upper] ?? trimmed.toLowerCase();
}

export interface RemoteRechargePackage {
  id?: string | number;
  points?: number | string;
  grantAmount?: number | string;
  grant_amount?: number | string;
  bonusPoints?: number | string;
  bonus_points?: number | string;
  priceAmount?: string | number;
  price_amount?: string | number;
  currencyCode?: string;
  currency_code?: string;
  title?: string;
  description?: string;
  recommended?: boolean;
  sortWeight?: number | string | null;
  sort_weight?: number | string | null;
}

export interface RemoteRechargeSettings {
  baseCurrencyCode?: string;
  base_currency_code?: string;
  basePointsPerCny?: string | number;
  base_points_per_cny?: string | number;
}

export interface RemoteRechargeOrderOutcome {
  orderNo?: string;
  order_no?: string;
  outTradeNo?: string;
  out_trade_no?: string;
  amount?: string | number;
  currencyCode?: string;
  currency_code?: string;
  points?: number | string;
  status?: string;
  paymentMethod?: string;
  payment_method?: string;
  cashierUrl?: string;
  cashier_url?: string;
  nextAction?: string;
  next_action?: string;
}

export interface CreateSdkworkWalletRechargeOrderRequest {
  amount: number;
  clientRequestNo?: string;
  currencyCode: string;
  paymentMethod: string;
  source: string;
}

export interface SdkworkWalletRechargeOrderService {
  recharges: {
    orders: {
      create(input: CreateSdkworkWalletRechargeOrderRequest): Promise<unknown>;
    };
    packages: {
      list(): Promise<unknown>;
    };
    settings: {
      retrieve(): Promise<unknown>;
    };
  };
}

export interface CreateSdkworkWalletRechargeServiceOptions {
  orderAppService: SdkworkWalletRechargeOrderService;
}

export interface SdkworkWalletRechargeService {
  listPackages(): Promise<SdkworkWalletRechargePackage[]>;
  retrievePointsToCashRate(): Promise<number | null>;
  createRechargeOrder(input: SdkworkWalletRechargeInput): Promise<SdkworkWalletRechargeResult>;
}

function readRemotePriceCny(item: RemoteRechargePackage): number {
  const raw = item.priceAmount ?? item.price_amount ?? "0";
  const amount = toSdkworkAccountNumber(raw);
  const currency = (item.currencyCode ?? item.currency_code ?? "CNY").toUpperCase();
  return currency === "CNY" ? amount : amount;
}

function mapRechargePackage(item: RemoteRechargePackage, index: number): SdkworkWalletRechargePackage {
  const idValue = item.id ?? index + 1;
  const numericId = typeof idValue === "number" ? idValue : Number.parseInt(String(idValue), 10);
  const points = toSdkworkAccountNumber(item.points ?? item.grantAmount ?? item.grant_amount);
  const bonus = toSdkworkAccountNumber(item.bonusPoints ?? item.bonus_points);
  const title =
    toSdkworkAccountOptionalString(item.title)
    || (bonus > 0 ? `${points} + ${bonus} bonus points` : `${points} points`);

  return {
    description: toSdkworkAccountOptionalString(item.description),
    id: Number.isFinite(numericId) ? numericId : index + 1,
    points: points + bonus,
    priceCny: readRemotePriceCny(item),
    recommended: Boolean(item.recommended),
    sortWeight: item.sortWeight === null || item.sort_weight === null
      ? null
      : toSdkworkAccountNumber(item.sortWeight ?? item.sort_weight, index),
    title,
  };
}

function mapRechargeOutcome(outcome: RemoteRechargeOrderOutcome, input: SdkworkWalletRechargeInput): SdkworkWalletRechargeResult {
  const status = toSdkworkAccountOptionalString(outcome.status)?.toLowerCase() ?? "pending";
  const normalizedStatus =
    status === "paid" || status === "completed" || status === "success"
      ? "completed"
      : status === "failed" || status === "cancelled"
        ? "failed"
        : "pending";

  return {
    cashAmountCny: toSdkworkAccountNumber(outcome.amount),
    paymentMethod: toSdkworkAccountOptionalString(outcome.paymentMethod ?? outcome.payment_method) ?? input.paymentMethod,
    points: toSdkworkAccountNumber(outcome.points, input.points),
    processedAt: undefined,
    remainingPoints: null,
    requestNo:
      toSdkworkAccountOptionalString(outcome.orderNo ?? outcome.order_no)
      ?? toSdkworkAccountOptionalString(outcome.outTradeNo ?? outcome.out_trade_no)
      ?? input.requestNo,
    status: normalizedStatus,
    transactionId: toSdkworkAccountOptionalString(outcome.outTradeNo ?? outcome.out_trade_no),
  };
}

export function createSdkworkWalletRechargeService(
  options: CreateSdkworkWalletRechargeServiceOptions,
): SdkworkWalletRechargeService {
  const getOrderAppService = () => options.orderAppService;

  return {
    async listPackages() {
      const payload = await getOrderAppService().recharges.packages.list();
      const page = unwrapSdkworkAccountListPage<RemoteRechargePackage>(payload);
      return page.items.map(mapRechargePackage);
    },

    async retrievePointsToCashRate() {
      const payload = await getOrderAppService().recharges.settings.retrieve();
      const settings = unwrapSdkworkAccountResource<RemoteRechargeSettings>(payload);
      const rate = settings.basePointsPerCny ?? settings.base_points_per_cny;
      const parsed = toSdkworkAccountNumber(rate, Number.NaN);
      return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
    },

    async createRechargeOrder(input) {
      const payload = await getOrderAppService().recharges.orders.create({
        amount: input.points,
        clientRequestNo: input.requestNo,
        currencyCode: "CNY",
        paymentMethod: normalizeWalletPaymentMethod(input.paymentMethod),
        source: "account-pc-wallet",
      });
      const outcome = unwrapSdkworkAccountResource<RemoteRechargeOrderOutcome>(payload);
      return mapRechargeOutcome(outcome, input);
    },
  };
}
