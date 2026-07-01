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
    accounts: {
      list: true,
      cash: { retrieve: true },
      points: { retrieve: true },
      tokens: { retrieve: true },
    },
    ledgerEntries: {
      list: true,
      retrieve: true,
      cash: { list: true },
      points: { list: true },
    },
    points: {
      lots: { list: true },
    },
    holds: {
      list: true,
      retrieve: true,
    },
    tokens: { retrieve: true },
  },
} as const;

export const BACKEND_ACCOUNT_METHOD_TREE = {
  wallet: {
    adjustments: {
      create: true,
      cash: { create: true },
      points: { create: true },
      tokens: { create: true },
    },
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
  T | { code?: number | string; data?: T; message?: string; msg?: string; traceId?: string }
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
