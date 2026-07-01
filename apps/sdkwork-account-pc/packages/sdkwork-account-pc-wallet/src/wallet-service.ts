import {
  getSdkworkAccountService,
  hasSdkworkAccountSession,
  requireSdkworkAccountSession,
  toNullableSdkworkAccountNumber,
  toSdkworkAccountNumber,
  toSdkworkAccountOptionalString,
  unwrapSdkworkAccountPage,
  unwrapSdkworkAccountResource,
  type SdkworkAccountAppService,
} from "@sdkwork/account-service";
import type { SdkworkOrderAppService } from "@sdkwork/order-service";
import {
  assertWalletCommerceDelegated,
  SdkworkWalletCommerceDelegationError,
} from "./wallet-commerce-delegation.ts";
import {
  createSdkworkWalletRechargeService,
  type SdkworkWalletRechargeService,
} from "./wallet-recharge-service.ts";

export { SdkworkWalletCommerceDelegationError } from "./wallet-commerce-delegation.ts";

export interface SdkworkWalletAccount {
  availablePoints: number;
  cashAvailable: number;
  cashFrozen: number;
  experience: number | null;
  frozenPoints: number;
  hasPayPassword: boolean;
  level: number | null;
  levelName?: string;
  status?: string;
  statusName?: string;
  tokenBalance: number;
  totalEarned: number;
  totalPoints: number;
  totalSpent: number;
}

export interface SdkworkWalletTransaction {
  cashAmountCny: number | null;
  createdAt: string;
  id: string;
  pointsAfter: number | null;
  pointsBefore: number | null;
  pointsDelta: number;
  status?: string;
  statusName?: string;
  title: string;
  transactionId?: string;
  transactionType?: string;
  transactionTypeName?: string;
}

export interface SdkworkWalletHold {
  id: string;
  holdId: string;
  accountId: string;
  assetType: string;
  amount: number;
  settledAmount: number;
  releasedAmount: number;
  status: string;
  businessType: string;
  businessNo: string;
  createdAt: string;
  updatedAt: string;
}

export interface SdkworkWalletRechargePackage {
  description?: string;
  id: number;
  points: number;
  priceCny: number;
  recommended: boolean;
  sortWeight: number | null;
  title: string;
}

export interface SdkworkWalletOverview {
  account: SdkworkWalletAccount;
  holds: SdkworkWalletHold[];
  isAuthenticated: boolean;
  pointsToCashRate: number | null;
  rechargePackages: SdkworkWalletRechargePackage[];
  transactions: SdkworkWalletTransaction[];
}

export interface GetSdkworkWalletOverviewOptions {
  pageSize?: number;
}

export interface SdkworkWalletRechargeInput {
  paymentMethod?: string;
  points: number;
  remarks?: string;
  requestNo?: string;
}

export interface SdkworkWalletRechargeResult {
  cashAmountCny: number | null;
  paymentMethod?: string;
  points: number;
  processedAt?: string;
  remainingPoints: number | null;
  requestNo?: string;
  status: "completed" | "failed" | "pending";
  transactionId?: string;
}

export interface SdkworkWalletWithdrawInput {
  accountName: string;
  accountNo: string;
  amountCny: number;
  bankName?: string;
  destinationCode: string;
  remarks?: string;
  requestNo?: string;
}

export interface SdkworkWalletWithdrawResult {
  amountCny: number | null;
  destinationCode?: string;
  estimatedArrivalTime?: string;
  frozenCashAmountCny: number | null;
  processedAt?: string;
  requestNo?: string;
  remainingCashAvailable: number | null;
  status: "completed" | "failed" | "pending";
  transactionId?: string;
}

export interface CreateSdkworkWalletServiceOptions {
  accountAppService?: SdkworkAccountAppService;
  orderAppService?: SdkworkOrderAppService;
  rechargeService?: SdkworkWalletRechargeService;
}

export interface SdkworkWalletService {
  getEmptyOverview(): SdkworkWalletOverview;
  getOverview(options?: GetSdkworkWalletOverviewOptions): Promise<SdkworkWalletOverview>;
  rechargePoints(input: SdkworkWalletRechargeInput): Promise<SdkworkWalletRechargeResult>;
  withdrawCash(input: SdkworkWalletWithdrawInput): Promise<SdkworkWalletWithdrawResult>;
}

interface RemoteCashAccount {
  availableAmount?: number | string;
  frozenAmount?: number | string;
  pendingAmount?: number | string;
}

interface RemotePointsAccount {
  availablePoints?: number | string;
  frozenPoints?: number | string;
  pendingPoints?: number | string;
  status?: string;
  totalPoints?: number | string;
}

interface RemoteTokenAccount {
  availableAmount?: number | string;
  frozenAmount?: number | string;
}

interface RemoteLedgerEntry {
  amount?: number | string;
  assetType?: string;
  balanceAfter?: number | string;
  balanceBefore?: number | string;
  businessType?: string;
  createdAt?: string;
  direction?: string;
  id?: string;
  uuid?: string;
}

interface RemoteHoldEntry {
  id?: string;
  uuid?: string;
  accountId?: string;
  assetType?: string;
  amount?: number | string;
  settledAmount?: number | string;
  releasedAmount?: number | string;
  status?: string;
  businessType?: string;
  businessNo?: string;
  createdAt?: string;
  updatedAt?: string;
}

const DEFAULT_HISTORY_PAGE_SIZE = 50;
const DEFAULT_HOLDS_PAGE_SIZE = 20;

export function createEmptySdkworkWalletOverview(): SdkworkWalletOverview {
  return {
    account: {
      availablePoints: 0,
      cashAvailable: 0,
      cashFrozen: 0,
      experience: null,
      frozenPoints: 0,
      hasPayPassword: false,
      level: null,
      tokenBalance: 0,
      totalEarned: 0,
      totalPoints: 0,
      totalSpent: 0,
    },
    isAuthenticated: false,
    pointsToCashRate: null,
    rechargePackages: [],
    holds: [],
    transactions: [],
  };
}

function signedAmount(entry: RemoteLedgerEntry): number {
  const amount = toSdkworkAccountNumber(entry.amount);
  const direction = toSdkworkAccountOptionalString(entry.direction)?.toLowerCase();
  return direction === "debit" ? -amount : amount;
}

function mapAccount(
  cash: RemoteCashAccount | null | undefined,
  points: RemotePointsAccount | null | undefined,
  token: RemoteTokenAccount | null | undefined,
): SdkworkWalletAccount {
  const availablePoints = toSdkworkAccountNumber(points?.availablePoints);
  const frozenPoints = toSdkworkAccountNumber(points?.frozenPoints);
  const pendingPoints = toSdkworkAccountNumber(points?.pendingPoints);

  return {
    availablePoints,
    cashAvailable: toSdkworkAccountNumber(cash?.availableAmount),
    cashFrozen: toSdkworkAccountNumber(cash?.frozenAmount),
    experience: null,
    frozenPoints,
    hasPayPassword: false,
    level: null,
    status: toSdkworkAccountOptionalString(points?.status),
    tokenBalance: toSdkworkAccountNumber(token?.availableAmount),
    totalEarned: 0,
    totalPoints: toSdkworkAccountNumber(points?.totalPoints, availablePoints + frozenPoints + pendingPoints),
    totalSpent: 0,
  };
}

function mapTransaction(entry: RemoteLedgerEntry): SdkworkWalletTransaction {
  const assetType = toSdkworkAccountOptionalString(entry.assetType)?.toLowerCase() ?? "";
  const delta = signedAmount(entry);

  return {
    cashAmountCny: assetType === "cash" ? Math.abs(delta) : null,
    createdAt: toSdkworkAccountOptionalString(entry.createdAt) || new Date(0).toISOString(),
    id: toSdkworkAccountOptionalString(entry.uuid) || toSdkworkAccountOptionalString(entry.id) || `wallet-${Date.now()}`,
    pointsAfter: assetType === "points" ? toNullableSdkworkAccountNumber(entry.balanceAfter) : null,
    pointsBefore: assetType === "points" ? toNullableSdkworkAccountNumber(entry.balanceBefore) : null,
    pointsDelta: assetType === "points" ? delta : 0,
    title: toSdkworkAccountOptionalString(entry.businessType) || "Wallet transaction",
    transactionId: toSdkworkAccountOptionalString(entry.uuid) || toSdkworkAccountOptionalString(entry.id),
    transactionType: toSdkworkAccountOptionalString(entry.businessType),
    transactionTypeName: toSdkworkAccountOptionalString(entry.businessType),
  };
}

function mapHold(entry: RemoteHoldEntry): SdkworkWalletHold {
  const holdId = toSdkworkAccountOptionalString(entry.uuid) || toSdkworkAccountOptionalString(entry.id) || "";

  return {
    id: holdId,
    holdId,
    accountId: toSdkworkAccountOptionalString(entry.accountId) || "",
    assetType: toSdkworkAccountOptionalString(entry.assetType) || "",
    amount: toSdkworkAccountNumber(entry.amount),
    settledAmount: toSdkworkAccountNumber(entry.settledAmount),
    releasedAmount: toSdkworkAccountNumber(entry.releasedAmount),
    status: toSdkworkAccountOptionalString(entry.status) || "held",
    businessType: toSdkworkAccountOptionalString(entry.businessType) || "",
    businessNo: toSdkworkAccountOptionalString(entry.businessNo) || "",
    createdAt: toSdkworkAccountOptionalString(entry.createdAt) || new Date(0).toISOString(),
    updatedAt: toSdkworkAccountOptionalString(entry.updatedAt) || new Date(0).toISOString(),
  };
}

export function createSdkworkWalletService(
  options: CreateSdkworkWalletServiceOptions = {},
): SdkworkWalletService {
  const getAccountAppService = () => options.accountAppService ?? getSdkworkAccountService();
  const rechargeService =
    options.rechargeService
    ?? (options.orderAppService
      ? createSdkworkWalletRechargeService({ orderAppService: options.orderAppService })
      : undefined);

  return {
    getEmptyOverview() {
      return createEmptySdkworkWalletOverview();
    },

    async getOverview(config = {}) {
      if (!hasSdkworkAccountSession()) {
        return createEmptySdkworkWalletOverview();
      }

      const pageSize = config.pageSize ?? DEFAULT_HISTORY_PAGE_SIZE;
      const accountAppService = getAccountAppService();

      const [cashPayload, pointsPayload, tokenPayload, ledgerPayload, holdsPayload] = await Promise.all([
        accountAppService.wallet.accounts.cash.retrieve(),
        accountAppService.wallet.accounts.points.retrieve(),
        accountAppService.wallet.accounts.tokens.retrieve(),
        accountAppService.wallet.ledgerEntries.points.list({
          page: 1,
          pageSize,
        }),
        accountAppService.wallet.holds.list({
          page: 1,
          pageSize: DEFAULT_HOLDS_PAGE_SIZE,
        }),
      ]);

      const cash = unwrapSdkworkAccountResource<RemoteCashAccount>(cashPayload);
      const points = unwrapSdkworkAccountResource<RemotePointsAccount>(pointsPayload);
      const token = unwrapSdkworkAccountResource<RemoteTokenAccount>(tokenPayload);
      const ledgerEntries = unwrapSdkworkAccountPage<RemoteLedgerEntry>(ledgerPayload);
      const holds = unwrapSdkworkAccountPage<RemoteHoldEntry>(holdsPayload);

      let rechargePackages: SdkworkWalletRechargePackage[] = [];
      let pointsToCashRate: number | null = null;
      if (rechargeService) {
        try {
          [rechargePackages, pointsToCashRate] = await Promise.all([
            rechargeService.listPackages(),
            rechargeService.retrievePointsToCashRate(),
          ]);
        } catch {
          rechargePackages = [];
          pointsToCashRate = null;
        }
      }

      return {
        account: mapAccount(cash, points, token),
        holds: holds.map(mapHold),
        isAuthenticated: true,
        pointsToCashRate,
        rechargePackages,
        transactions: ledgerEntries.map(mapTransaction),
      };
    },

    async rechargePoints(input) {
      requireSdkworkAccountSession("Please sign in to manage wallet balances.");
      if (rechargeService) {
        return rechargeService.createRechargeOrder(input);
      }
      assertWalletCommerceDelegated("recharge", "order");
    },

    async withdrawCash() {
      requireSdkworkAccountSession("Please sign in to manage wallet balances.");
      assertWalletCommerceDelegated("withdraw", "payment");
    },
  };
}

export const sdkworkWalletService = createSdkworkWalletService();
