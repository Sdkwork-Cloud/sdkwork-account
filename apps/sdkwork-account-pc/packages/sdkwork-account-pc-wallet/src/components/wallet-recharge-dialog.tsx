import {
  useEffect,
  useMemo,
  useState,
} from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  StatusNotice,
} from "@sdkwork/ui-pc-react";
import { createSdkworkWalletPanelStyle } from "../wallet-appearance";
import type { SdkworkWalletController } from "../wallet-controller";
import { useSdkworkWalletControllerState } from "../wallet-controller";
import { useSdkworkWalletIntl } from "../wallet-intl";
import {
  navigateWalletRechargeCheckout,
  type SdkworkWalletRechargeFlow,
} from "../wallet-checkout-navigation";

export interface SdkworkWalletRechargeDialogProps {
  checkoutBasePath?: string;
  controller: SdkworkWalletController;
  onNavigate?: (route: string) => void;
  onOpenChange?: (open: boolean) => void;
  open: boolean;
  rechargeFlow?: SdkworkWalletRechargeFlow;
}

const PAYMENT_METHODS = ["WECHAT", "ALIPAY", "BANKCARD"] as const;

function sanitizeNumber(value: string): string {
  return value.replaceAll(/\D+/g, "").slice(0, 7);
}

export function SdkworkWalletRechargeDialog({
  checkoutBasePath,
  controller,
  onNavigate,
  onOpenChange,
  open,
  rechargeFlow = "direct",
}: SdkworkWalletRechargeDialogProps) {
  const state = useSdkworkWalletControllerState(controller);
  const [selectedPoints, setSelectedPoints] = useState<number>(0);
  const [customPoints, setCustomPoints] = useState("");
  const [paymentMethod, setPaymentMethod] = useState<(typeof PAYMENT_METHODS)[number]>("WECHAT");
  const {
    copy,
    formatCurrencyCny,
    formatPaymentMethod,
    formatPoints,
    formatPointsRate,
    formatRechargePackageSummary,
  } = useSdkworkWalletIntl();
  const rechargePackages = state.overview.rechargePackages;
  const usesCheckoutFlow = rechargeFlow === "checkout" && Boolean(onNavigate);
  const effectivePoints = selectedPoints || Number.parseInt(customPoints || "0", 10) || 0;
  const payableAmount = state.overview.pointsToCashRate
    ? Number((effectivePoints / state.overview.pointsToCashRate).toFixed(2))
    : null;

  useEffect(() => {
    if (!open) {
      return;
    }

    const defaultPoints =
      rechargePackages.find((rechargePackage) => rechargePackage.recommended)?.points
      ?? rechargePackages[0]?.points
      ?? 0;
    setSelectedPoints(defaultPoints);
    setCustomPoints("");
    setPaymentMethod("WECHAT");
  }, [open, rechargePackages]);

  const canSubmit = useMemo(
    () => state.overview.isAuthenticated && effectivePoints > 0 && !state.isMutating,
    [effectivePoints, state.isMutating, state.overview.isAuthenticated],
  );

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="w-[min(92vw,52rem)]">
        <DialogHeader>
          <DialogTitle>{copy.rechargeDialog.title}</DialogTitle>
          <DialogDescription>
            {copy.rechargeDialog.description}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5">
          <div
            className="rounded-[1.25rem] border p-4"
            style={createSdkworkWalletPanelStyle("brand", {
              backgroundWeight: 8,
              borderWeight: 24,
              surfaceWeight: 92,
            })}
          >
            <div className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
              {copy.rechargeDialog.selectionEyebrow}
            </div>
            <div className="mt-3 text-3xl font-semibold text-[var(--sdk-color-text-primary)]">
              {formatPoints(effectivePoints || 0)}
            </div>
            <div className="mt-2 text-sm text-[var(--sdk-color-text-secondary)]">
              {copy.rechargeDialog.rateLabel}: {formatPointsRate(state.overview.pointsToCashRate)}
            </div>
            <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
              {copy.rechargeDialog.estimatedPriceLabel}: {formatCurrencyCny(payableAmount)}
            </div>
          </div>

          {!state.overview.isAuthenticated ? (
            <StatusNotice title={copy.rechargeDialog.signInRequiredTitle} tone="warning">
              {copy.rechargeDialog.signInRequiredDescription}
            </StatusNotice>
          ) : null}

          {rechargePackages.length === 0 ? (
            <StatusNotice title={copy.rechargeDialog.noPackagesTitle}>
              {copy.rechargeDialog.noPackagesDescription}
            </StatusNotice>
          ) : (
            <div className="grid gap-3 sm:grid-cols-3">
              {rechargePackages.map((rechargePackage) => (
                <Button
                  className="h-auto min-h-20 flex-col items-start gap-1 px-4 py-3 text-left"
                  key={rechargePackage.id}
                  onClick={() => {
                    setSelectedPoints(rechargePackage.points);
                    setCustomPoints("");
                  }}
                  type="button"
                  variant={selectedPoints === rechargePackage.points ? "secondary" : "outline"}
                >
                  <span className="text-sm font-semibold">{rechargePackage.title}</span>
                  <span className="text-xs font-normal text-[var(--sdk-color-text-secondary)]">
                    {formatRechargePackageSummary(rechargePackage)}
                  </span>
                </Button>
              ))}
            </div>
          )}

          <div className="space-y-2">
            <label className="text-sm font-medium text-[var(--sdk-color-text-primary)]" htmlFor="sdkwork-wallet-custom-value">
              {copy.rechargeDialog.customAmountLabel}
            </label>
            <Input
              id="sdkwork-wallet-custom-value"
              inputMode="numeric"
              onChange={(event) => {
                setSelectedPoints(0);
                setCustomPoints(sanitizeNumber(event.target.value));
              }}
              onFocus={() => setSelectedPoints(0)}
              placeholder={copy.rechargeDialog.customAmountPlaceholder}
              value={customPoints}
            />
          </div>

          <div className="space-y-3">
            {!usesCheckoutFlow ? (
              <>
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                  {copy.rechargeDialog.paymentMethodLabel}
                </div>
                <div className="grid gap-3 sm:grid-cols-3">
                  {PAYMENT_METHODS.map((method) => (
                    <Button
                      key={method}
                      onClick={() => setPaymentMethod(method)}
                      type="button"
                      variant={paymentMethod === method ? "secondary" : "outline"}
                    >
                      {formatPaymentMethod(method)}
                    </Button>
                  ))}
                </div>
              </>
            ) : (
              <div className="rounded-[1.25rem] border border-dashed border-[var(--sdk-color-border-default)] px-4 py-4 text-sm text-[var(--sdk-color-text-secondary)]">
                {copy.rechargeDialog.checkoutFlowDescription}
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button onClick={() => onOpenChange?.(false)} type="button" variant="ghost">
            {copy.actions.cancel}
          </Button>
          <Button
            disabled={!canSubmit}
            loading={state.isMutating}
            onClick={() => {
              if (usesCheckoutFlow && onNavigate) {
                const selectedPackage = rechargePackages.find((rechargePackage) => rechargePackage.points === effectivePoints) ?? null;
                if (
                  navigateWalletRechargeCheckout({
                    checkoutBasePath,
                    onNavigate,
                    ...(selectedPackage ? { package: selectedPackage } : {
                      points: effectivePoints,
                      pointsToCashRate: state.overview.pointsToCashRate,
                    }),
                  })
                ) {
                  onOpenChange?.(false);
                }
                return;
              }

              void controller.rechargePoints({
                paymentMethod,
                points: effectivePoints,
              });
            }}
            type="button"
          >
            {usesCheckoutFlow ? copy.actions.continueToCheckout : copy.actions.confirmRecharge}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
