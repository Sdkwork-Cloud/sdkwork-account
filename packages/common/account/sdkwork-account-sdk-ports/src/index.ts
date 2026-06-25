export const APP_ACCOUNT_METHOD_TREE = {
  accounts: {
    current: {
      summary: { retrieve: true },
    },
  },
  recharges: {
    packages: { list: true },
    settings: { retrieve: true },
    orders: {
      create: true,
      retrieve: true,
      cancel: true,
    },
  },
  wallet: {
    overview: { retrieve: true },
    accounts: {
      list: true,
      retrieve: true,
      overview: { retrieve: true },
      points: { retrieve: true },
      tokens: { retrieve: true },
    },
    ledgerEntries: {
      list: true,
      retrieve: true,
      points: { list: true },
    },
    exchangeRate: { retrieve: true },
    exchangeRules: { list: true },
    points: {
      exchangeRules: { list: true },
    },
    tokens: { retrieve: true },
    holds: {
      create: true,
      releases: { create: true },
      settlements: { create: true },
    },
    pointExchanges: {
      create: true,
      retrieve: true,
    },
    pointTransfers: { create: true },
    requests: { retrieve: true },
    adjustments: { create: true },
    topupTransfers: { create: true },
    transactions: {
      list: true,
      retrieve: true,
    },
    withdrawalTransfers: { create: true },
  },
} as const;

export type AccountRequestParams = Record<string, unknown>;
export type AccountSdkResponse<T> = Promise<
  T | { code?: number | string; data?: T; message?: string; msg?: string }
>;
export type AccountSdkMethod = (...args: any[]) => AccountSdkResponse<any>;

type MethodTree = {
  readonly [key: string]: true | MethodTree;
};

export type ClientFromMethodTree<TTree extends MethodTree> = {
  readonly [TKey in keyof TTree]: TTree[TKey] extends true
    ? AccountSdkMethod
    : TTree[TKey] extends MethodTree
      ? ClientFromMethodTree<TTree[TKey]>
      : never;
};

export type AccountAppSdkClient = {
  commerce: ClientFromMethodTree<typeof APP_ACCOUNT_METHOD_TREE>;
};
