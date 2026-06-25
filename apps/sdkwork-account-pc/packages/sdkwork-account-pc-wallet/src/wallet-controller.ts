import {
  useMemo,
  useSyncExternalStore,
} from "react";
import {
  createSdkworkWalletService,
  type SdkworkWalletOverview,
  type SdkworkWalletRechargeInput,
  type SdkworkWalletRechargeResult,
  type SdkworkWalletService,
  type SdkworkWalletWithdrawInput,
  type SdkworkWalletWithdrawResult,
} from "./wallet-service";

export interface SdkworkWalletControllerState {
  isBootstrapped: boolean;
  isLoading: boolean;
  isMutating: boolean;
  isRechargeOpen: boolean;
  isWithdrawOpen: boolean;
  lastError?: string;
  overview: SdkworkWalletOverview;
}

export interface SdkworkWalletController {
  bootstrap(): Promise<SdkworkWalletControllerState>;
  closeRecharge(): void;
  closeWithdraw(): void;
  getState(): SdkworkWalletControllerState;
  openRecharge(): void;
  openWithdraw(): void;
  rechargePoints(input: SdkworkWalletRechargeInput): Promise<SdkworkWalletRechargeResult>;
  refresh(): Promise<SdkworkWalletControllerState>;
  service: SdkworkWalletService;
  subscribe(listener: () => void): () => void;
  withdrawCash(input: SdkworkWalletWithdrawInput): Promise<SdkworkWalletWithdrawResult>;
}

export interface CreateSdkworkWalletControllerOptions {
  initialState?: Partial<SdkworkWalletControllerState>;
  service?: Partial<SdkworkWalletService>;
}

export function createSdkworkWalletController(
  options: CreateSdkworkWalletControllerOptions = {},
): SdkworkWalletController {
  const service: SdkworkWalletService = options.service
    ? {
        ...createSdkworkWalletService(),
        ...options.service,
      }
    : createSdkworkWalletService();
  const listeners = new Set<() => void>();
  let state: SdkworkWalletControllerState = {
    isBootstrapped: false,
    isLoading: false,
    isMutating: false,
    isRechargeOpen: false,
    isWithdrawOpen: false,
    overview: service.getEmptyOverview(),
    ...options.initialState,
  };

  function emit(): void {
    listeners.forEach((listener) => listener());
  }

  function setState(
    next:
      | Partial<SdkworkWalletControllerState>
      | ((currentState: SdkworkWalletControllerState) => Partial<SdkworkWalletControllerState>),
  ): void {
    state = {
      ...state,
      ...(typeof next === "function" ? next(state) : next),
    };
    emit();
  }

  async function loadOverview(): Promise<SdkworkWalletOverview> {
    return service.getOverview();
  }

  async function runMutation<TResult>(callback: () => Promise<TResult>): Promise<TResult> {
    setState({
      isMutating: true,
      lastError: undefined,
    });

    try {
      const result = await callback();
      const overview = await loadOverview();
      setState({
        isBootstrapped: true,
        isMutating: false,
        overview,
      });
      return result;
    } catch (error) {
      setState({
        isMutating: false,
        lastError: error instanceof Error ? error.message : "Wallet request failed.",
      });
      throw error;
    }
  }

  return {
    async bootstrap() {
      setState({
        isLoading: true,
        lastError: undefined,
      });

      try {
        const overview = await loadOverview();
        setState({
          isBootstrapped: true,
          isLoading: false,
          overview,
        });
        return state;
      } catch (error) {
        setState({
          isLoading: false,
          lastError: error instanceof Error ? error.message : "Failed to load wallet center.",
        });
        throw error;
      }
    },

    closeRecharge() {
      setState({
        isRechargeOpen: false,
      });
    },

    closeWithdraw() {
      setState({
        isWithdrawOpen: false,
      });
    },

    getState() {
      return state;
    },

    openRecharge() {
      setState({
        isRechargeOpen: true,
      });
    },

    openWithdraw() {
      setState({
        isWithdrawOpen: true,
      });
    },

    async rechargePoints(input) {
      const result = await runMutation(() => service.rechargePoints(input));
      setState({
        isRechargeOpen: false,
      });
      return result;
    },

    async withdrawCash(input) {
      const result = await runMutation(() => service.withdrawCash(input));
      setState({
        isWithdrawOpen: false,
      });
      return result;
    },

    async refresh() {
      const overview = await loadOverview();
      setState({
        isBootstrapped: true,
        isLoading: false,
        overview,
      });
      return state;
    },

    service,

    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}

export function useSdkworkWalletController(
  controller?: SdkworkWalletController,
): SdkworkWalletController {
  return useMemo(() => controller ?? createSdkworkWalletController(), [controller]);
}

export function useSdkworkWalletControllerState(
  controller: SdkworkWalletController,
): SdkworkWalletControllerState {
  return useSyncExternalStore(
    controller.subscribe,
    controller.getState,
    controller.getState,
  );
}
