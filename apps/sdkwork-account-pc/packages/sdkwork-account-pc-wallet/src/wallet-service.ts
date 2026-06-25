import {
  getSdkworkAccountService,
  hasSdkworkAccountSession,
  requireSdkworkAccountSession,
  toNullableSdkworkAccountNumber,
  toSdkworkAccountMutationStatus,
  toSdkworkAccountNumber,
  toSdkworkAccountOptionalString,
  unwrapSdkworkAccountResponse,
  type SdkworkAccountAppService,
} from "@sdkwork/account-service";

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
}

export interface SdkworkWalletService {
  getEmptyOverview(): SdkworkWalletOverview;
  getOverview(options?: GetSdkworkWalletOverviewOptions): Promise<SdkworkWalletOverview>;
  rechargePoints(input: SdkworkWalletRechargeInput): Promise<SdkworkWalletRechargeResult>;
  withdrawCash(input: SdkworkWalletWithdrawInput): Promise<SdkworkWalletWithdrawResult>;
}

interface RemoteAccountSummary {
  cashAvailable?: number | string;
  cashFrozen?: number | string;
  hasPayPassword?: boolean;
  pointsAvailable?: number | string;
  pointsFrozen?: number | string;
  tokenAvailable?: number | string;
  tokenFrozen?: number | string;
}

interface RemotePointsAccount {
  availablePoints?: number | string;
  experience?: number | string;
  frozenPoints?: number | string;
  level?: number | string;
  levelName?: string;
  status?: string;
  statusName?: string;
  tokenBalance?: number | string;
  totalEarned?: number | string;
  totalPoints?: number | string;
  totalSpent?: number | string;
}

interface RemoteHistoryItem {
  amount?: number | string;
  createdAt?: string;
  historyId?: string;
  points?: number | string;
  pointsAfter?: number | string;
  pointsBefore?: number | string;
  remarks?: string;
  status?: string;
  statusName?: string;
  transactionId?: string;
  transactionType?: string;
  transactionTypeName?: string;
}

interface RemoteRechargePackage {
  description?: string;
  id?: number | string;
  name?: string;
  pointAmount?: number | string;
  price?: number | string;
  sortWeight?: number | string;
}

interface RemoteRechargeResult {
  cashAmount?: number | string;
  paymentMethod?: string;
  points?: number | string;
  processedAt?: string;
  remainingPoints?: number | string;
  requestNo?: string;
  status?: string;
  transactionId?: string;
}

interface RemoteWithdrawResult {
  amount?: number | string;
  balanceAfter?: number | string;
  createdAt?: string;
  estimatedArrivalTime?: string;
  fromBalanceAfter?: number | string;
  frozenBalance?: number | string;
  processedAt?: string;
  requestNo?: string;
  status?: string;
  transactionId?: string;
  updatedAt?: string;
  channel?: string;
  withdrawMethod?: string;
}

const DEFAULT_HISTORY_PAGE_SIZE = 50;
const SDKWORK_WALLET_REQUEST_NO_PATTERN = /^[A-Za-z0-9_-]{6,64}$/;

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
    transactions: [],
  };
}

function mapAccount(
  summary: RemoteAccountSummary | null | undefined,
  points: RemotePointsAccount | null | undefined,
): SdkworkWalletAccount {
  const availablePoints = toSdkworkAccountNumber(points?.availablePoints, toSdkworkAccountNumber(summary?.pointsAvailable));
  const frozenPoints = toSdkworkAccountNumber(points?.frozenPoints, toSdkworkAccountNumber(summary?.pointsFrozen));

  return {
    availablePoints,
    cashAvailable: toSdkworkAccountNumber(summary?.cashAvailable),
    cashFrozen: toSdkworkAccountNumber(summary?.cashFrozen),
    experience: toNullableSdkworkAccountNumber(points?.experience),
    frozenPoints,
    hasPayPassword: Boolean(summary?.hasPayPassword),
    level: toNullableSdkworkAccountNumber(points?.level),
    levelName: toSdkworkAccountOptionalString(points?.levelName),
    status: toSdkworkAccountOptionalString(points?.status),
    statusName: toSdkworkAccountOptionalString(points?.statusName),
    tokenBalance: toSdkworkAccountNumber(points?.tokenBalance, toSdkworkAccountNumber(summary?.tokenAvailable)),
    totalEarned: toSdkworkAccountNumber(points?.totalEarned),
    totalPoints: toSdkworkAccountNumber(points?.totalPoints, availablePoints + frozenPoints),
    totalSpent: toSdkworkAccountNumber(points?.totalSpent),
  };
}

function resolveTransactionTitle(item: RemoteHistoryItem): string {
  return (
    toSdkworkAccountOptionalString(item.transactionTypeName)
    || toSdkworkAccountOptionalString(item.remarks)
    || "Wallet transaction"
  );
}

function mapTransaction(item: RemoteHistoryItem): SdkworkWalletTransaction {
  return {
    cashAmountCny: toNullableSdkworkAccountNumber(item.amount),
    createdAt: toSdkworkAccountOptionalString(item.createdAt) || new Date(0).toISOString(),
    id:
      toSdkworkAccountOptionalString(item.historyId)
      || toSdkworkAccountOptionalString(item.transactionId)
      || `wallet-transaction-${Date.now()}`,
    pointsAfter: toNullableSdkworkAccountNumber(item.pointsAfter),
    pointsBefore: toNullableSdkworkAccountNumber(item.pointsBefore),
    pointsDelta: toSdkworkAccountNumber(item.points),
    status: toSdkworkAccountOptionalString(item.status),
    statusName: toSdkworkAccountOptionalString(item.statusName),
    title: resolveTransactionTitle(item),
    transactionId: toSdkworkAccountOptionalString(item.transactionId),
    transactionType: toSdkworkAccountOptionalString(item.transactionType),
    transactionTypeName: toSdkworkAccountOptionalString(item.transactionTypeName),
  };
}

function mapRechargePackages(
  rechargePackages: RemoteRechargePackage[],
): SdkworkWalletRechargePackage[] {
  const sorted = [...rechargePackages]
    .map((rechargePackage) => ({
      description: toSdkworkAccountOptionalString(rechargePackage.description),
      id: toSdkworkAccountNumber(rechargePackage.id),
      points: toSdkworkAccountNumber(rechargePackage.pointAmount),
      priceCny: toSdkworkAccountNumber(rechargePackage.price),
      sortWeight: toNullableSdkworkAccountNumber(rechargePackage.sortWeight),
      title: toSdkworkAccountOptionalString(rechargePackage.name) || "Recharge package",
    }))
    .sort((left, right) => (
      (right.sortWeight ?? 0) - (left.sortWeight ?? 0)
      || right.points - left.points
      || left.id - right.id
    ));

  return sorted.map((rechargePackage, index) => ({
    ...rechargePackage,
    recommended: index === 0,
  }));
}

function mapRechargeResult(result: RemoteRechargeResult | null | undefined): SdkworkWalletRechargeResult {
  return {
    cashAmountCny: toNullableSdkworkAccountNumber(result?.cashAmount),
    paymentMethod: toSdkworkAccountOptionalString(result?.paymentMethod),
    points: toSdkworkAccountNumber(result?.points),
    processedAt: toSdkworkAccountOptionalString(result?.processedAt),
    remainingPoints: toNullableSdkworkAccountNumber(result?.remainingPoints),
    requestNo: toSdkworkAccountOptionalString(result?.requestNo),
    status: toSdkworkAccountMutationStatus(toSdkworkAccountOptionalString(result?.status)),
    transactionId: toSdkworkAccountOptionalString(result?.transactionId),
  };
}

function mapWithdrawResult(
  result: RemoteWithdrawResult | null | undefined,
): SdkworkWalletWithdrawResult {
  return {
    amountCny: toNullableSdkworkAccountNumber(result?.amount),
    destinationCode: toSdkworkAccountOptionalString(result?.channel) || toSdkworkAccountOptionalString(result?.withdrawMethod),
    estimatedArrivalTime: toSdkworkAccountOptionalString(result?.estimatedArrivalTime),
    frozenCashAmountCny: toNullableSdkworkAccountNumber(result?.frozenBalance),
    processedAt:
      toSdkworkAccountOptionalString(result?.processedAt)
      || toSdkworkAccountOptionalString(result?.updatedAt)
      || toSdkworkAccountOptionalString(result?.createdAt),
    requestNo: toSdkworkAccountOptionalString(result?.requestNo),
    remainingCashAvailable: toNullableSdkworkAccountNumber(result?.fromBalanceAfter ?? result?.balanceAfter),
    status: toSdkworkAccountMutationStatus(toSdkworkAccountOptionalString(result?.status)),
    transactionId: toSdkworkAccountOptionalString(result?.transactionId),
  };
}

export function createSdkworkWalletService(
  options: CreateSdkworkWalletServiceOptions = {},
): SdkworkWalletService {
  const getaccountAppService = () => options.accountAppService ?? getSdkworkAccountService();

  return {
    getEmptyOverview() {
      return createEmptySdkworkWalletOverview();
    },

    async getOverview(config = {}) {
      if (!hasSdkworkAccountSession()) {
        return createEmptySdkworkWalletOverview();
      }

      const pageSize = config.pageSize ?? DEFAULT_HISTORY_PAGE_SIZE;
      const accountAppService = getaccountAppService();

      const [
        summaryPayload,
        historyPagePayload,
        pointsAccountPayload,
        pointsToCashRatePayload,
        rechargePackagesPayload,
      ] = await Promise.all([
        accountAppService.accounts.current.summary.retrieve(),
        accountAppService.wallet.ledgerEntries.points.list({
              pageNum: 1,
              pageSize,
              sortDirection: "desc",
              sortField: "createdAt",
        }),
        accountAppService.wallet.accounts.points.retrieve(),
        accountAppService.wallet.exchangeRate.retrieve(),
        accountAppService.recharges.packages.list(),
      ]);
      const summary = unwrapSdkworkAccountResponse<RemoteAccountSummary | null>(summaryPayload);
      const historyPage = unwrapSdkworkAccountResponse<{ content?: RemoteHistoryItem[] }>(historyPagePayload);
      const pointsAccount = unwrapSdkworkAccountResponse<RemotePointsAccount | null>(pointsAccountPayload);
      const pointsToCashRate = unwrapSdkworkAccountResponse<number | null>(pointsToCashRatePayload);
      const rechargePackages = unwrapSdkworkAccountResponse<RemoteRechargePackage[]>(rechargePackagesPayload);

      return {
        account: mapAccount(summary, pointsAccount),
        isAuthenticated: true,
        pointsToCashRate: toNullableSdkworkAccountNumber(pointsToCashRate),
        rechargePackages: mapRechargePackages(rechargePackages),
        transactions: (historyPage.content ?? []).map(mapTransaction),
      };
    },

    async rechargePoints(input) {
      requireSdkworkAccountSession("Please sign in to manage wallet balances.");
      const result = unwrapSdkworkAccountResponse<RemoteRechargeResult>(
        await getaccountAppService().recharges.orders.create({
          paymentMethod: toSdkworkAccountOptionalString(input.paymentMethod),
          points: input.points,
          remarks: toSdkworkAccountOptionalString(input.remarks),
          requestNo: toSdkworkAccountOptionalString(input.requestNo),
        }),
        "Failed to recharge points.",
      );

      return mapRechargeResult(result);
    },

    async withdrawCash(input) {
      requireSdkworkAccountSession("Please sign in to manage wallet balances.");

      if (!(input.amountCny > 0)) {
        throw new Error("Withdrawal amount must be greater than zero.");
      }

      const destinationCode = toSdkworkAccountOptionalString(input.destinationCode);
      if (!destinationCode) {
        throw new Error("Select a withdrawal destination before submitting.");
      }

      const accountName = toSdkworkAccountOptionalString(input.accountName);
      if (!accountName) {
        throw new Error("Enter the payout account holder name before submitting.");
      }

      const accountNo = toSdkworkAccountOptionalString(input.accountNo);
      if (!accountNo) {
        throw new Error("Enter the payout account number before submitting.");
      }

      const bankName = toSdkworkAccountOptionalString(input.bankName);
      if (destinationCode === "bank_account" && !bankName) {
        throw new Error("Enter the settlement bank name before submitting.");
      }

      const requestNo = toSdkworkAccountOptionalString(input.requestNo);
      if (requestNo && !SDKWORK_WALLET_REQUEST_NO_PATTERN.test(requestNo)) {
        throw new Error("Request no must use 6-64 letters, numbers, underscores, or hyphens.");
      }

      const result = unwrapSdkworkAccountResponse<RemoteWithdrawResult>(
        await getaccountAppService().wallet.withdrawalTransfers.create({
          accountName,
          accountNo,
          amount: input.amountCny,
          bankName,
          remarks: toSdkworkAccountOptionalString(input.remarks),
          requestNo,
          withdrawMethod: destinationCode,
        }),
        "Failed to withdraw cash.",
      );

      return mapWithdrawResult(result);
    },
  };
}

export const sdkworkWalletService = createSdkworkWalletService();
