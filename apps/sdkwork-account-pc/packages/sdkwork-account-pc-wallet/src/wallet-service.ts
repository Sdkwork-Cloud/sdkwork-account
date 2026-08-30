import {

  getSdkworkAccountService,

  hasSdkworkAccountSession,

  requireSdkworkAccountSession,

  toNullableSdkworkAccountNumber,

  toSdkworkAccountNumber,

  toSdkworkAccountPointsFromMicro,

  toSdkworkAccountOptionalString,

  unwrapSdkworkAccountListPage,

  unwrapSdkworkAccountResource,

  type SdkworkAccountAppService,

  type SdkworkAccountPageInfo,

} from "@sdkwork/account-service";

import {

  assertWalletCommerceDelegated,

  SdkworkWalletCommerceDelegationError,

} from "./wallet-commerce-delegation.ts";

import {

  createSdkworkWalletRechargeService,
  type SdkworkWalletRechargeOrderService,

  type SdkworkWalletRechargeService,

} from "./wallet-recharge-service.ts";



export { SdkworkWalletCommerceDelegationError } from "./wallet-commerce-delegation.ts";



export interface SdkworkWalletAccount {

  activeLotCount: number | null;

  availablePoints: number;

  cashAvailable: number;

  cashFrozen: number;

  cashPending: number;

  experience: number | null;

  expiringPoints: number | null;

  frozenPoints: number;

  hasPayPassword: boolean;

  level: number | null;

  levelName?: string;

  pointsPending: number;

  status?: string;

  statusName?: string;

  tokenBankAvailable: number;

  tokenBankFrozen: number;

  totalEarned: number | null;

  totalPoints: number;

  totalSpent: number | null;

  unsweptExpiredPoints: number | null;

}



export interface SdkworkWalletTransaction {

  cashAmountCny: number | null;

  createdAt: string;

  id: string;

  pointsAfter: number | null;

  pointsBefore: number | null;

  pointsDelta: number;

  tokenBankDelta: number;

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

  holdPageInfo?: SdkworkAccountPageInfo | null;

  isAuthenticated: boolean;

  pointsToCashRate: number | null;

  rechargePackages: SdkworkWalletRechargePackage[];

  transactionPageInfo?: SdkworkAccountPageInfo | null;

  transactions: SdkworkWalletTransaction[];

}



export interface GetSdkworkWalletOverviewOptions {

  cursor?: string;

  page?: number;

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

  orderAppService?: SdkworkWalletRechargeOrderService;

  rechargeService?: SdkworkWalletRechargeService;

}



export interface SdkworkWalletService {

  getEmptyOverview(): SdkworkWalletOverview;

  getOverview(options?: GetSdkworkWalletOverviewOptions): Promise<SdkworkWalletOverview>;

  loadMoreHolds(page: number): Promise<Pick<SdkworkWalletOverview, "holdPageInfo" | "holds">>;

  loadMoreTransactions(
    cursor: string,
  ): Promise<Pick<SdkworkWalletOverview, "transactionPageInfo" | "transactions">>;

  rechargePoints(input: SdkworkWalletRechargeInput): Promise<SdkworkWalletRechargeResult>;

  withdrawCash(input: SdkworkWalletWithdrawInput): Promise<SdkworkWalletWithdrawResult>;

}



interface RemoteCashAccount {

  availableAmount?: number | string;

  frozenAmount?: number | string;

  pendingAmount?: number | string;

}



interface RemotePointsSummary {

  activeLotCount?: number | string;

  availablePoints?: number | string;

  expiringPoints?: number | string;

  frozenPoints?: number | string;

  monthCreditPoints?: number | string;

  monthDebitPoints?: number | string;

  pendingPoints?: number | string;

  status?: string;

  totalPoints?: number | string;

  unsweptExpiredPoints?: number | string;

}



interface RemoteTokenBankAccount {

  availableAmount?: number | string;

  frozenAmount?: number | string;

}



interface RemotePortfolio {

  cash?: RemoteCashAccount | null;

  points?: RemotePointsSummary | null;

  tokenBank?: RemoteTokenBankAccount | null;

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



const DEFAULT_HISTORY_PAGE_SIZE = 20;

const DEFAULT_HOLDS_PAGE_SIZE = 20;



export function createEmptySdkworkWalletOverview(): SdkworkWalletOverview {

  return {

    account: {

      activeLotCount: null,

      availablePoints: 0,

      cashAvailable: 0,

      cashFrozen: 0,

      cashPending: 0,

      experience: null,

      expiringPoints: null,

      frozenPoints: 0,

      hasPayPassword: false,

      level: null,

      pointsPending: 0,

      tokenBankAvailable: 0,

      tokenBankFrozen: 0,

      totalEarned: null,

      totalPoints: 0,

      totalSpent: null,

      unsweptExpiredPoints: null,

    },

    isAuthenticated: false,

    pointsToCashRate: null,

    rechargePackages: [],

    holds: [],

    holdPageInfo: null,

    transactions: [],

    transactionPageInfo: null,

  };

}



function buildLedgerListQuery(

  config: GetSdkworkWalletOverviewOptions,

  defaultPageSize: number,

): Record<string, number | string> {

  const query: Record<string, number | string> = {

    pageSize: config.pageSize ?? defaultPageSize,

  };



  if (config.page !== undefined) {

    query.page = config.page;

  }



  if (config.cursor) {

    query.cursor = config.cursor;

  }



  return query;

}



function signedAmount(entry: RemoteLedgerEntry): number {

  const amount = toSdkworkAccountNumber(entry.amount);

  const direction = toSdkworkAccountOptionalString(entry.direction)?.toLowerCase();

  return direction === "debit" ? -amount : amount;

}



function mapAccount(portfolio: RemotePortfolio | null | undefined): SdkworkWalletAccount {

  const cash = portfolio?.cash;

  const points = portfolio?.points;

  const tokenBank = portfolio?.tokenBank;

  const availablePoints = toSdkworkAccountPointsFromMicro(points?.availablePoints);

  const frozenPoints = toSdkworkAccountPointsFromMicro(points?.frozenPoints);

  const pendingPoints = toSdkworkAccountPointsFromMicro(points?.pendingPoints);



  return {

    activeLotCount: toNullableSdkworkAccountNumber(points?.activeLotCount),

    availablePoints,

    cashAvailable: toSdkworkAccountNumber(cash?.availableAmount),

    cashFrozen: toSdkworkAccountNumber(cash?.frozenAmount),

    cashPending: toSdkworkAccountNumber(cash?.pendingAmount),

    experience: null,

    expiringPoints: points?.expiringPoints == null ? null : toSdkworkAccountPointsFromMicro(points.expiringPoints),

    frozenPoints,

    hasPayPassword: false,

    level: null,

    pointsPending: pendingPoints,

    status: toSdkworkAccountOptionalString(points?.status),

    tokenBankAvailable: toSdkworkAccountPointsFromMicro(tokenBank?.availableAmount),

    tokenBankFrozen: toSdkworkAccountPointsFromMicro(tokenBank?.frozenAmount),

    totalEarned: points?.monthCreditPoints == null ? null : toSdkworkAccountPointsFromMicro(points.monthCreditPoints),

    totalPoints:
      points?.totalPoints == null
        ? availablePoints + frozenPoints + pendingPoints
        : toSdkworkAccountPointsFromMicro(points.totalPoints),

    totalSpent: points?.monthDebitPoints == null ? null : toSdkworkAccountPointsFromMicro(points.monthDebitPoints),

    unsweptExpiredPoints: points?.unsweptExpiredPoints == null ? null : toSdkworkAccountPointsFromMicro(points.unsweptExpiredPoints),

  };

}



function mapTransaction(entry: RemoteLedgerEntry): SdkworkWalletTransaction {

  const assetType = toSdkworkAccountOptionalString(entry.assetType)?.toLowerCase() ?? "";

  const delta = signedAmount(entry);



  return {

    cashAmountCny: assetType === "cash" ? Math.abs(delta) : null,

    createdAt: toSdkworkAccountOptionalString(entry.createdAt) || new Date(0).toISOString(),

    id: toSdkworkAccountOptionalString(entry.uuid) || toSdkworkAccountOptionalString(entry.id) || `wallet-${Date.now()}`,

    pointsAfter: assetType === "points" ? (entry.balanceAfter == null ? null : toSdkworkAccountPointsFromMicro(entry.balanceAfter)) : null,

    pointsBefore: assetType === "points" ? (entry.balanceBefore == null ? null : toSdkworkAccountPointsFromMicro(entry.balanceBefore)) : null,

    pointsDelta: assetType === "points" ? toSdkworkAccountPointsFromMicro(delta) : 0,

    tokenBankDelta: assetType === "token_bank" ? toSdkworkAccountPointsFromMicro(delta) : 0,

    title: toSdkworkAccountOptionalString(entry.businessType) || "Wallet transaction",

    transactionId: toSdkworkAccountOptionalString(entry.uuid) || toSdkworkAccountOptionalString(entry.id),

    transactionType: toSdkworkAccountOptionalString(entry.businessType),

    transactionTypeName: toSdkworkAccountOptionalString(entry.businessType),

  };

}



function mapHold(entry: RemoteHoldEntry): SdkworkWalletHold {

  const holdId = toSdkworkAccountOptionalString(entry.uuid) || toSdkworkAccountOptionalString(entry.id) || "";

  const holdAssetType = toSdkworkAccountOptionalString(entry.assetType)?.toLowerCase() || "";

  const isMicroHold = holdAssetType === "points" || holdAssetType === "token_bank";



  return {

    id: holdId,

    holdId,

    accountId: toSdkworkAccountOptionalString(entry.accountId) || "",

    assetType: toSdkworkAccountOptionalString(entry.assetType) || "",

    amount: isMicroHold ? toSdkworkAccountPointsFromMicro(entry.amount) : toSdkworkAccountNumber(entry.amount),

    settledAmount: isMicroHold ? toSdkworkAccountPointsFromMicro(entry.settledAmount) : toSdkworkAccountNumber(entry.settledAmount),

    releasedAmount: isMicroHold ? toSdkworkAccountPointsFromMicro(entry.releasedAmount) : toSdkworkAccountNumber(entry.releasedAmount),

    status: toSdkworkAccountOptionalString(entry.status) || "held",

    businessType: toSdkworkAccountOptionalString(entry.businessType) || "",

    businessNo: toSdkworkAccountOptionalString(entry.businessNo) || "",

    createdAt: toSdkworkAccountOptionalString(entry.createdAt) || new Date(0).toISOString(),

    updatedAt: toSdkworkAccountOptionalString(entry.updatedAt) || new Date(0).toISOString(),

  };

}



async function fetchLedgerPage(

  accountAppService: SdkworkAccountAppService,

  query: Record<string, number | string>,

) {

  const ledgerPayload = await accountAppService.wallet.ledgerEntries.list(query);

  const ledgerPage = unwrapSdkworkAccountListPage<RemoteLedgerEntry>(ledgerPayload);



  return {

    transactionPageInfo: ledgerPage.pageInfo,

    transactions: ledgerPage.items.map(mapTransaction),

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



      const ledgerQuery = buildLedgerListQuery(config, DEFAULT_HISTORY_PAGE_SIZE);

      const accountAppService = getAccountAppService();



      const [

        portfolioPayload,

        ledgerPayload,

        holdsPayload,

      ] = await Promise.all([

        accountAppService.wallet.portfolio.list(),

        accountAppService.wallet.ledgerEntries.list(ledgerQuery),

        accountAppService.tokenBank.holds.list({

          page: 1,

          pageSize: DEFAULT_HOLDS_PAGE_SIZE,

        }),

      ]);



      const portfolio = unwrapSdkworkAccountResource<RemotePortfolio>(portfolioPayload);

      const ledgerPage = unwrapSdkworkAccountListPage<RemoteLedgerEntry>(ledgerPayload);

      const holdsPage = unwrapSdkworkAccountListPage<RemoteHoldEntry>(holdsPayload);



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

        account: mapAccount(portfolio),

        holds: holdsPage.items.map(mapHold),

        holdPageInfo: holdsPage.pageInfo,

        isAuthenticated: true,

        pointsToCashRate,

        rechargePackages,

        transactionPageInfo: ledgerPage.pageInfo,

        transactions: ledgerPage.items.map(mapTransaction),

      };

    },



    async loadMoreTransactions(cursor) {

      requireSdkworkAccountSession("Please sign in to manage wallet balances.");

      const ledgerQuery = buildLedgerListQuery(

        { cursor, pageSize: DEFAULT_HISTORY_PAGE_SIZE },

        DEFAULT_HISTORY_PAGE_SIZE,

      );

      return fetchLedgerPage(getAccountAppService(), ledgerQuery);

    },



    async loadMoreHolds(page) {

      requireSdkworkAccountSession("Please sign in to manage wallet balances.");

      const holdsPayload = await getAccountAppService().tokenBank.holds.list({

        page,

        pageSize: DEFAULT_HOLDS_PAGE_SIZE,

      });

      const holdsPage = unwrapSdkworkAccountListPage<RemoteHoldEntry>(holdsPayload);



      return {

        holds: holdsPage.items.map(mapHold),

        holdPageInfo: holdsPage.pageInfo,

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

      assertWalletCommerceDelegated("withdraw", "order");

    },

  };

}



export const sdkworkWalletService = createSdkworkWalletService();


