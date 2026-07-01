import { useEffect } from "react";
import {
  LoadingBlock,
  StatusNotice,
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
  navigateWalletWithdrawPayout,
  type SdkworkWalletPayoutFlow,
  resolveWalletPayoutFlow,
} from "../wallet-payout-navigation";

export interface SdkworkWalletPageProps {
  checkoutBasePath?: string;
  controller?: SdkworkWalletController;
  locale?: string | null;
  messages?: SdkworkWalletMessagesOverrides;
  onNavigate?: (route: string) => void;
  payoutBasePath?: string;
  payoutFlow?: SdkworkWalletPayoutFlow;
  rechargeFlow?: SdkworkWalletRechargeFlow;
}

interface SdkworkWalletPageContentProps {
  checkoutBasePath?: string;
  controller?: SdkworkWalletController;
  onNavigate?: (route: string) => void;
  payoutBasePath?: string;
  payoutFlow?: SdkworkWalletPayoutFlow;
  rechargeFlow?: SdkworkWalletRechargeFlow;
}

function SdkworkWalletPageContent({
  checkoutBasePath,
  controller: controllerProp,
  onNavigate,
  payoutBasePath,
  payoutFlow,
  rechargeFlow,
}: SdkworkWalletPageContentProps) {
  const controller = useSdkworkWalletController(controllerProp);
  const state = useSdkworkWalletControllerState(controller);
  const { copy } = useSdkworkWalletIntl();
  const resolvedRechargeFlow = resolveWalletRechargeFlow(rechargeFlow, onNavigate);
  const resolvedPayoutFlow = resolveWalletPayoutFlow(payoutFlow, onNavigate);
  const featuredRechargePackage =
    state.overview.rechargePackages.find((rechargePackage) => rechargePackage.recommended)
    ?? state.overview.rechargePackages[0]
    ?? null;

  useEffect(() => {
    if (!state.isBootstrapped && !state.isLoading && !state.lastError) {
      void controller.bootstrap().catch(() => undefined);
    }
  }, [controller, state.isBootstrapped, state.isLoading, state.lastError]);

  function openWalletRecharge() {
    if (
      resolvedRechargeFlow === "checkout"
      && onNavigate
      && featuredRechargePackage
      && navigateWalletRechargeCheckout({
        checkoutBasePath,
        onNavigate,
        package: featuredRechargePackage,
      })
    ) {
      return;
    }

    controller.openRecharge();
  }

  function openWalletWithdraw() {
    if (
      resolvedPayoutFlow === "checkout"
      && onNavigate
      && navigateWalletWithdrawPayout({
        onNavigate,
        payoutBasePath,
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
              {state.lastError}
            </StatusNotice>
          ) : null}

          <SdkworkWalletHoldsList holds={state.overview.holds} />

          <SdkworkWalletTransactionList transactions={state.overview.transactions} />
        </div>

        <SdkworkWalletRechargeDialog
          checkoutBasePath={checkoutBasePath}
          controller={controller}
          onNavigate={onNavigate}
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
          payoutBasePath={payoutBasePath}
          payoutFlow={resolvedPayoutFlow}
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
