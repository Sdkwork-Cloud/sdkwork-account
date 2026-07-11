import { useEffect } from "react";
import {
  LoadingBlock,
  StatusNotice,
  Button,
} from "@sdkwork/ui-pc-react";
import type { SdkworkWalletMessagesOverrides } from "../wallet-copy";
import type { SdkworkWalletController } from "../wallet-controller";
import {
  useSdkworkWalletController,
  useSdkworkWalletControllerState,
} from "../wallet-controller";
import {
  SdkworkWalletIntlProvider,
  useSdkworkWalletIntl,
} from "../wallet-intl";
import { SdkworkWalletBalancePanel } from "../components/wallet-balance-panel";
import { SdkworkWalletRechargeDialog } from "../components/wallet-recharge-dialog";
import { SdkworkWalletSummaryCards } from "../components/wallet-summary-cards";
import { SdkworkWalletTransactionList } from "../components/wallet-transaction-list";
import { SdkworkWalletHoldsList } from "../components/wallet-holds-list";
import { SdkworkWalletWithdrawDialog } from "../components/wallet-withdraw-dialog";
import {
  navigateWalletRechargeCheckout,
  resolveWalletRechargeFlow,
  type SdkworkWalletRechargeFlow,
} from "../wallet-checkout-navigation";
import {
  shouldRefreshWalletAfterCommerceReturn,
  stripWalletCommerceReturnParams,
} from "../wallet-commerce-return";
import {
  navigateWalletWithdrawalRequest,
  type SdkworkWalletWithdrawalFlow,
  resolveWalletWithdrawalFlow,
} from "../wallet-withdrawal-navigation";

export interface SdkworkWalletPageProps {
  checkoutBasePath?: string;
  controller?: SdkworkWalletController;
  locale?: string | null;
  messages?: SdkworkWalletMessagesOverrides;
  onNavigate?: (route: string) => void;
  rechargeFlow?: SdkworkWalletRechargeFlow;
  /** Post-checkout redirect target; defaults to current pathname or `/wallet`. */
  walletReturnPath?: string;
  withdrawalFlow?: SdkworkWalletWithdrawalFlow;
  withdrawalRequestBasePath?: string;
}

interface SdkworkWalletPageContentProps {
  checkoutBasePath?: string;
  controller?: SdkworkWalletController;
  onNavigate?: (route: string) => void;
  rechargeFlow?: SdkworkWalletRechargeFlow;
  walletReturnPath?: string;
  withdrawalFlow?: SdkworkWalletWithdrawalFlow;
  withdrawalRequestBasePath?: string;
}

function resolveWalletReturnPath(walletReturnPath: string | undefined): string | undefined {
  const explicit = walletReturnPath?.trim();
  if (explicit) {
    return explicit;
  }
  if (typeof window === "undefined") {
    return "/wallet";
  }
  return window.location.pathname || "/wallet";
}

function SdkworkWalletPageContent({
  checkoutBasePath,
  controller: controllerProp,
  onNavigate,
  rechargeFlow,
  walletReturnPath,
  withdrawalFlow,
  withdrawalRequestBasePath,
}: SdkworkWalletPageContentProps) {
  const controller = useSdkworkWalletController(controllerProp);
  const state = useSdkworkWalletControllerState(controller);
  const { copy } = useSdkworkWalletIntl();
  const resolvedRechargeFlow = resolveWalletRechargeFlow(rechargeFlow, onNavigate);
  const resolvedWithdrawalFlow = resolveWalletWithdrawalFlow(withdrawalFlow, onNavigate);
  const resolvedWalletReturnPath = onNavigate ? resolveWalletReturnPath(walletReturnPath) : undefined;
  const featuredRechargePackage =
    state.overview.rechargePackages.find((rechargePackage) => rechargePackage.recommended)
    ?? state.overview.rechargePackages[0]
    ?? null;

  useEffect(() => {
    if (!state.isBootstrapped && !state.isLoading && !state.lastError) {
      void controller.bootstrap().catch(() => undefined);
    }
  }, [controller, state.isBootstrapped, state.isLoading, state.lastError]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    function refreshAfterCommerceReturn(): void {
      const params = new URLSearchParams(window.location.search);
      if (!shouldRefreshWalletAfterCommerceReturn(params)) {
        return;
      }

      const cleaned = stripWalletCommerceReturnParams(params);
      const nextUrl = cleaned.toString()
        ? `${window.location.pathname}?${cleaned.toString()}`
        : window.location.pathname;
      window.history.replaceState(null, "", nextUrl);

      if (state.isBootstrapped) {
        void controller.refresh().catch(() => undefined);
      }
    }

    refreshAfterCommerceReturn();

    function handlePageShow(event: PageTransitionEvent): void {
      if (event.persisted) {
        refreshAfterCommerceReturn();
      }
    }

    window.addEventListener("pageshow", handlePageShow);
    return () => {
      window.removeEventListener("pageshow", handlePageShow);
    };
  }, [controller, state.isBootstrapped]);

  function openWalletRecharge() {
    if (
      resolvedRechargeFlow === "checkout"
      && onNavigate
      && featuredRechargePackage
      && navigateWalletRechargeCheckout({
        checkoutBasePath,
        onNavigate,
        package: featuredRechargePackage,
        walletReturnPath: resolvedWalletReturnPath,
      })
    ) {
      return;
    }

    controller.openRecharge();
  }

  function openWalletWithdraw() {
    if (
      resolvedWithdrawalFlow === "checkout"
      && onNavigate
      && navigateWalletWithdrawalRequest({
        onNavigate,
        withdrawalRequestBasePath,
      })
    ) {
      return;
    }

    controller.openWithdraw();
  }

  return (
    <div className="h-full overflow-y-auto">
      <div className="px-4 py-4 sm:px-5 sm:py-5">
        <div className="mx-auto max-w-5xl space-y-4">
          <SdkworkWalletBalancePanel
            onOpenRecharge={openWalletRecharge}
            onOpenWithdraw={openWalletWithdraw}
            overview={state.overview}
          />

          <SdkworkWalletSummaryCards overview={state.overview} />

          {state.isLoading && !state.isBootstrapped ? <LoadingBlock label={copy.page.loading} /> : null}

          {state.lastError ? (
            <StatusNotice title={copy.page.errorTitle} tone="danger">
              <div className="space-y-3">
                <p>{state.lastError}</p>
                <Button
                  loading={state.isLoading}
                  onClick={() => {
                    void controller.bootstrap();
                  }}
                  size="sm"
                  type="button"
                  variant="outline"
                >
                  {copy.actions.retry}
                </Button>
              </div>
            </StatusNotice>
          ) : null}

          <SdkworkWalletHoldsList
            hasMore={Boolean(state.overview.holdPageInfo?.hasMore)}
            holds={state.overview.holds}
            isLoadingMore={state.isLoadingMore}
            onLoadMore={() => {
              void controller.loadMoreHolds();
            }}
          />

          <SdkworkWalletTransactionList
            hasMore={Boolean(state.overview.transactionPageInfo?.hasMore)}
            isLoadingMore={state.isLoadingMore}
            onLoadMore={() => {
              void controller.loadMoreTransactions();
            }}
            transactions={state.overview.transactions}
          />
        </div>

        <SdkworkWalletRechargeDialog
          checkoutBasePath={checkoutBasePath}
          controller={controller}
          onNavigate={onNavigate}
          walletReturnPath={resolvedWalletReturnPath}
          onOpenChange={(open) => {
            if (!open) {
              controller.closeRecharge();
            }
          }}
          open={state.isRechargeOpen}
          rechargeFlow={resolvedRechargeFlow}
        />
        <SdkworkWalletWithdrawDialog
          controller={controller}
          onNavigate={onNavigate}
          onOpenChange={(open) => {
            if (!open) {
              controller.closeWithdraw();
            }
          }}
          open={state.isWithdrawOpen}
          withdrawalFlow={resolvedWithdrawalFlow}
          withdrawalRequestBasePath={withdrawalRequestBasePath}
        />
      </div>
    </div>
  );
}

export function SdkworkWalletPage({
  locale,
  messages,
  ...props
}: SdkworkWalletPageProps) {
  const content = <SdkworkWalletPageContent {...props} />;

  if (locale || messages) {
    return (
      <SdkworkWalletIntlProvider locale={locale} messages={messages}>
        {content}
      </SdkworkWalletIntlProvider>
    );
  }

  return content;
}
