import {
  getSdkworkOrderService,
  toSdkworkOrderNumber,
  toSdkworkOrderOptionalString,
  unwrapSdkworkOrderPage,
  unwrapSdkworkOrderResource,
  unwrapSdkworkOrderResponse,
  type SdkworkOrderAppService,
} from "@sdkwork/order-service";
import type {
  SdkworkWalletRechargeInput,
  SdkworkWalletRechargePackage,
  SdkworkWalletRechargeResult,
} from "./wallet-service.ts";

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

export interface CreateSdkworkWalletRechargeServiceOptions {
  orderAppService?: SdkworkOrderAppService;
}

export interface SdkworkWalletRechargeService {
  listPackages(): Promise<SdkworkWalletRechargePackage[]>;
  retrievePointsToCashRate(): Promise<number | null>;
  createRechargeOrder(input: SdkworkWalletRechargeInput): Promise<SdkworkWalletRechargeResult>;
}

function readRemotePriceCny(item: RemoteRechargePackage): number {
  const raw = item.priceAmount ?? item.price_amount ?? "0";
  const amount = toSdkworkOrderNumber(raw);
  const currency = (item.currencyCode ?? item.currency_code ?? "CNY").toUpperCase();
  return currency === "CNY" ? amount : amount;
}

function mapRechargePackage(item: RemoteRechargePackage, index: number): SdkworkWalletRechargePackage {
  const idValue = item.id ?? index + 1;
  const numericId = typeof idValue === "number" ? idValue : Number.parseInt(String(idValue), 10);
  const points = toSdkworkOrderNumber(item.points ?? item.grantAmount ?? item.grant_amount);
  const bonus = toSdkworkOrderNumber(item.bonusPoints ?? item.bonus_points);
  const title =
    toSdkworkOrderOptionalString(item.title)
    || (bonus > 0 ? `${points} + ${bonus} bonus points` : `${points} points`);

  return {
    description: toSdkworkOrderOptionalString(item.description),
    id: Number.isFinite(numericId) ? numericId : index + 1,
    points: points + bonus,
    priceCny: readRemotePriceCny(item),
    recommended: Boolean(item.recommended),
    sortWeight: item.sortWeight === null || item.sort_weight === null
      ? null
      : toSdkworkOrderNumber(item.sortWeight ?? item.sort_weight, index),
    title,
  };
}

function mapRechargeOutcome(outcome: RemoteRechargeOrderOutcome, input: SdkworkWalletRechargeInput): SdkworkWalletRechargeResult {
  const status = toSdkworkOrderOptionalString(outcome.status)?.toLowerCase() ?? "pending";
  const normalizedStatus =
    status === "paid" || status === "completed" || status === "success"
      ? "completed"
      : status === "failed" || status === "cancelled"
        ? "failed"
        : "pending";

  return {
    cashAmountCny: toSdkworkOrderNumber(outcome.amount),
    paymentMethod: toSdkworkOrderOptionalString(outcome.paymentMethod ?? outcome.payment_method) ?? input.paymentMethod,
    points: toSdkworkOrderNumber(outcome.points, input.points),
    processedAt: undefined,
    remainingPoints: null,
    requestNo:
      toSdkworkOrderOptionalString(outcome.orderNo ?? outcome.order_no)
      ?? toSdkworkOrderOptionalString(outcome.outTradeNo ?? outcome.out_trade_no)
      ?? input.requestNo,
    status: normalizedStatus,
    transactionId: toSdkworkOrderOptionalString(outcome.outTradeNo ?? outcome.out_trade_no),
  };
}

export function createSdkworkWalletRechargeService(
  options: CreateSdkworkWalletRechargeServiceOptions = {},
): SdkworkWalletRechargeService {
  const getOrderAppService = () => options.orderAppService ?? getSdkworkOrderService();

  return {
    async listPackages() {
      const payload = await getOrderAppService().recharges.packages.list();
      const items = unwrapSdkworkOrderPage<RemoteRechargePackage>(payload);
      return items.map(mapRechargePackage);
    },

    async retrievePointsToCashRate() {
      const payload = await getOrderAppService().recharges.settings.retrieve();
      const settings = unwrapSdkworkOrderResource<RemoteRechargeSettings>(payload);
      const rate = settings.basePointsPerCny ?? settings.base_points_per_cny;
      const parsed = toSdkworkOrderNumber(rate, Number.NaN);
      return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
    },

    async createRechargeOrder(input) {
      const payload = await getOrderAppService().recharges.orders.create({
        amount: input.points,
        clientRequestNo: input.requestNo,
        currencyCode: "CNY",
        source: "account-pc-wallet",
      });
      const outcome = unwrapSdkworkOrderResource<RemoteRechargeOrderOutcome>(payload);
      return mapRechargeOutcome(outcome, input);
    },
  };
}

export const sdkworkWalletRechargeService = createSdkworkWalletRechargeService();
