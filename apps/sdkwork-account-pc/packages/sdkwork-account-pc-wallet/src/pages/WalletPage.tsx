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
import { createSdkworkWalletBackdropStyle } from "../wallet-appearance";
import {
  SdkworkWalletIntlProvider,
  useSdkworkWalletIntl,
} from "../wallet-intl";
import { SdkworkWalletBalancePanel } from "../components/wallet-balance-panel";
import { SdkworkWalletRechargeDialog } from "../components/wallet-recharge-dialog";
import { SdkworkWalletSummaryCards } from "../components/wallet-summary-cards";
import { SdkworkWalletTransactionList } from "../components/wallet-transaction-list";
import { SdkworkWalletWithdrawDialog } from "../components/wallet-withdraw-dialog";
import {
  navigateWalletRechargeCheckout,
  resolveWalletRechargeFlow,
  type SdkworkWalletRechargeFlow,
} from "../wallet-checkout-navigation";

export interface SdkworkWalletPageProps {
  checkoutBasePath?: string;
  controller?: SdkworkWalletController;
  locale?: string | null;
  messages?: SdkworkWalletMessagesOverrides;
  onNavigate?: (route: string) => void;
  rechargeFlow?: SdkworkWalletRechargeFlow;
}

interface SdkworkWalletPageContentProps {
  checkoutBasePath?: string;
  controller?: SdkworkWalletController;
  onNavigate?: (route: string) => void;
  rechargeFlow?: SdkworkWalletRechargeFlow;
}

function SdkworkWalletPageContent({
  checkoutBasePath,
  controller: controllerProp,
  onNavigate,
  rechargeFlow,
}: SdkworkWalletPageContentProps) {
  const controller = useSdkworkWalletController(controllerProp);
  const state = useSdkworkWalletControllerState(controller);
  const { copy } = useSdkworkWalletIntl();
  const resolvedRechargeFlow = resolveWalletRechargeFlow(rechargeFlow, onNavigate);
  const featuredRechargePackage =
    state.overview.rechargePackages.find((rechargePackage) => rechargePackage.recommended)
    ?? state.overview.rechargePackages[0]
    ?? null;

  useEffect(() => {
    if (!state.isBootstrapped && !state.isLoading) {
      void controller.bootstrap();
    }
  }, [controller, state.isBootstrapped, state.isLoading]);

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

  return (
    <div className="relative h-full overflow-y-auto">
      <div
        className="pointer-events-none absolute inset-x-0 top-0 h-72"
        style={createSdkworkWalletBackdropStyle()}
      />

      <div className="relative px-4 py-4 sm:px-5 sm:py-5">
        <div className="mx-auto max-w-[88rem] space-y-5">
          <SdkworkWalletBalancePanel
            onOpenRecharge={openWalletRecharge}
            onOpenWithdraw={() => controller.openWithdraw()}
            overview={state.overview}
          />

          <SdkworkWalletSummaryCards overview={state.overview} />

          {state.isLoading && !state.isBootstrapped ? <LoadingBlock label={copy.page.loading} /> : null}

          {state.lastError ? (
            <StatusNotice title={copy.page.errorTitle} tone="danger">
              {state.lastError}
            </StatusNotice>
          ) : null}

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
          onOpenChange={(open) => {
            if (!open) {
              controller.closeWithdraw();
            }
          }}
          open={state.isWithdrawOpen}
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
