export const APP_ACCOUNT_METHOD_TREE = {
  accounts: {
    current: {
      summary: { retrieve: true },
    },
  },
  billing: {
    history: { list: true },
  },
  wallet: {
    overview: { retrieve: true },
    portfolio: { list: true },
    accounts: {
      list: true,
      cash: { retrieve: true },
      points: { retrieve: true },
    },
    ledgerEntries: {
      list: true,
      retrieve: true,
      cash: { list: true },
      points: { list: true },
      allocations: { list: true },
    },
    points: {
      summary: { retrieve: true },
      lots: { list: true },
    },
    holds: {
      list: true,
      retrieve: true,
    },
  },
  tokenBank: {
    account: { retrieve: true },
    overview: { retrieve: true },
    ledgerEntries: {
      list: true,
    },
    holds: {
      list: true,
    },
  },
} as const;

export const BACKEND_ACCOUNT_METHOD_TREE = {
  wallet: {
    health: {
      retrieve: true,
    },
    outbox: {
      dispatch: true,
    },
    adjustments: {
      create: true,
      cash: { create: true },
      points: { create: true },
    },
    holds: {
      create: true,
      settle: true,
      release: true,
      expire: true,
    },
    points: {
      reconciliation: true,
      lots: {
        expire: true,
      },
    },
    transfers: {
      create: true,
    },
  },
  tokenBank: {
    credits: { create: true },
    debits: { create: true },
    grants: { create: true },
    reversals: { create: true },
    holds: {
      create: true,
      settle: true,
      release: true,
    },
    transfers: {
      create: true,
    },
  },
} as const;

export type AccountRequestParams = Record<string, unknown>;
export type AccountSdkResponse<T> = Promise<
  T | { code: number; data: T; traceId?: string }
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

export type AccountBackendSdkClient = {
  commerce: ClientFromMethodTree<typeof BACKEND_ACCOUNT_METHOD_TREE>;
};
