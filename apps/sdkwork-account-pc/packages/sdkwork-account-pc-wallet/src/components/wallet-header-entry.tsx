import {
  Suspense,
  useEffect,
  useRef,
  useState,
  type ComponentType,
} from "react";
import {
  Coins,
  ShieldCheck,
} from "lucide-react";
import type { SdkworkWalletController } from "../wallet-controller";
import {
  useSdkworkWalletController,
  useSdkworkWalletControllerState,
} from "../wallet-controller";
import { createSdkworkWalletToneStyle } from "../wallet-appearance";
import { useSdkworkWalletIntl } from "../wallet-intl";
import { SdkworkWalletQuickPanel, type SdkworkWalletQuickPanelProps } from "./wallet-quick-panel";
import { SdkworkWalletRechargeDialog } from "./wallet-recharge-dialog";
import { SdkworkWalletWithdrawDialog } from "./wallet-withdraw-dialog";
import {
  navigateWalletRechargeCheckout,
  resolveWalletRechargeFlow,
  type SdkworkWalletRechargeFlow,
} from "../wallet-checkout-navigation";

export interface SdkworkWalletHeaderEntryProps {
  accountLabel?: string;
  checkoutBasePath?: string;
  controller?: SdkworkWalletController;
  onNavigate?: (route: string) => void;
  onOpenPage?: () => void;
  quickPanelClassName?: string;
  QuickPanel?: ComponentType<SdkworkWalletQuickPanelProps>;
  rechargeFlow?: SdkworkWalletRechargeFlow;
}

export function SdkworkWalletHeaderEntry({
  accountLabel,
  checkoutBasePath,
  controller: controllerProp,
  onNavigate,
  onOpenPage,
  quickPanelClassName,
  QuickPanel: QuickPanelComponent,
  rechargeFlow,
}: SdkworkWalletHeaderEntryProps) {
  const controller = useSdkworkWalletController(controllerProp);
  const state = useSdkworkWalletControllerState(controller);
  const [isPanelOpen, setIsPanelOpen] = useState(false);
  const entryRef = useRef<HTMLDivElement>(null);
  const PanelComponent = QuickPanelComponent ?? SdkworkWalletQuickPanel;
  const {
    copy,
    formatAccountLevelSummary,
    formatPoints,
  } = useSdkworkWalletIntl();
  const resolvedRechargeFlow = resolveWalletRechargeFlow(rechargeFlow, onNavigate);
  const featuredRechargePackage =
    state.overview.rechargePackages.find((rechargePackage) => rechargePackage.recommended)
    ?? state.overview.rechargePackages[0]
    ?? null;

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

  useEffect(() => {
    if (!state.isBootstrapped && !state.isLoading && !state.lastError) {
      void controller.bootstrap().catch(() => undefined);
    }
  }, [controller, state.isBootstrapped, state.isLoading, state.lastError]);

  useEffect(() => {
    if (!isPanelOpen) {
      return undefined;
    }

    const handlePointerDown = (event: PointerEvent) => {
      if (entryRef.current?.contains(event.target as Node)) {
        return;
      }

      setIsPanelOpen(false);
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsPanelOpen(false);
      }
    };

    document.addEventListener("pointerdown", handlePointerDown, true);
    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isPanelOpen]);

  return (
    <div className="relative flex items-center gap-2" ref={entryRef}>
      <button
        className="inline-flex h-9 items-center gap-2 rounded-[1rem] border px-3 text-sm font-medium"
        onClick={() => {
          onOpenPage?.();
        }}
        style={createSdkworkWalletToneStyle("accent", {
          backgroundWeight: 12,
          borderWeight: 24,
        })}
        type="button"
      >
        <ShieldCheck className="h-4 w-4" />
        {accountLabel ?? formatAccountLevelSummary(state.overview.account)}
      </button>

      <button
        aria-expanded={isPanelOpen}
        aria-haspopup="dialog"
        aria-label={copy.headerEntry.balanceAriaLabel}
        className="inline-flex h-9 items-center gap-2 rounded-[1rem] border border-[var(--sdk-color-border-subtle)] bg-[var(--sdk-color-surface-panel-muted)] px-3 text-sm font-medium text-[var(--sdk-color-text-primary)]"
        onClick={() => setIsPanelOpen((current) => !current)}
        type="button"
      >
        <Coins className="h-4 w-4" />
        {formatPoints(state.overview.account.availablePoints)} {copy.headerEntry.pointsSuffix}
      </button>

      {isPanelOpen ? (
        <div
          className={quickPanelClassName ?? "absolute right-0 top-[calc(100%+0.75rem)] z-50"}
          role="dialog"
          aria-label={copy.headerEntry.balanceAriaLabel}
        >
          <Suspense fallback={null}>
            <PanelComponent
              onOpenPage={() => {
                setIsPanelOpen(false);
                onOpenPage?.();
              }}
              onRecharge={() => {
                setIsPanelOpen(false);
                openWalletRecharge();
              }}
              onWithdraw={() => {
                setIsPanelOpen(false);
                controller.openWithdraw();
              }}
              overview={state.overview}
            />
          </Suspense>
        </div>
      ) : null}

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
  );
}
