import { useEffect, useMemo, useState } from "react";
import { Check } from "lucide-react";
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
      <DialogContent className="w-[min(92vw,40rem)] gap-0 overflow-hidden p-0">
        <DialogHeader className="border-b border-[var(--sdk-color-border-subtle)] px-6 py-5">
          <DialogTitle>{copy.rechargeDialog.title}</DialogTitle>
          <DialogDescription>{copy.rechargeDialog.description}</DialogDescription>
        </DialogHeader>

        <div className="space-y-5 px-6 py-5">
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
            <div className="space-y-2">
              <p className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                {copy.rechargeDialog.packageGridLabel}
              </p>
              <div className="grid gap-2 sm:grid-cols-2">
                {rechargePackages.map((rechargePackage) => {
                  const isSelected = selectedPoints === rechargePackage.points && !customPoints;

                  return (
                    <button
                      className={`rounded-[var(--sdk-radius-field)] border px-4 py-3 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--sdk-color-border-focus)] ${
                        isSelected
                          ? "border-[var(--sdk-color-brand-primary)] bg-[var(--sdk-color-brand-primary-soft)]"
                          : "border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel)] hover:bg-[var(--sdk-color-surface-panel-muted)]"
                      }`}
                      key={rechargePackage.id}
                      onClick={() => {
                        setSelectedPoints(rechargePackage.points);
                        setCustomPoints("");
                      }}
                      type="button"
                    >
                      <div className="flex items-start justify-between gap-2">
                        <div>
                          <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                            {rechargePackage.title}
                          </div>
                          <div className="mt-1 text-lg font-semibold tabular-nums text-[var(--sdk-color-text-primary)]">
                            {formatPoints(rechargePackage.points)}
                          </div>
                          <div className="mt-1 text-xs text-[var(--sdk-color-text-secondary)]">
                            {formatRechargePackageSummary(rechargePackage)}
                          </div>
                        </div>
                        {rechargePackage.recommended ? (
                          <span className="rounded-full bg-[var(--sdk-color-brand-primary-soft)] px-2 py-0.5 text-[0.65rem] font-medium text-[var(--sdk-color-brand-primary)]">
                            {copy.rechargeDialog.recommendedBadge}
                          </span>
                        ) : null}
                      </div>
                      {isSelected ? (
                        <div className="mt-2 inline-flex items-center gap-1 text-xs font-medium text-[var(--sdk-color-brand-primary)]">
                          <Check className="h-3 w-3" aria-hidden="true" />
                          {copy.rechargeDialog.selectedLabel}
                        </div>
                      ) : null}
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          <div className="space-y-2">
            <label className="text-sm font-medium text-[var(--sdk-color-text-primary)]" htmlFor="sdkwork-wallet-custom-value">
              {copy.rechargeDialog.customAmountLabel}
            </label>
            <Input
              className="h-10"
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

          {!usesCheckoutFlow ? (
            <div className="space-y-2">
              <p className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                {copy.rechargeDialog.paymentMethodLabel}
              </p>
              <div className="flex flex-wrap gap-2">
                {PAYMENT_METHODS.map((method) => {
                  const isSelected = paymentMethod === method;

                  return (
                    <button
                      className={`rounded-[var(--sdk-radius-pill)] border px-3 py-1.5 text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--sdk-color-border-focus)] ${
                        isSelected
                          ? "border-[var(--sdk-color-brand-primary)] bg-[var(--sdk-color-brand-primary-soft)] text-[var(--sdk-color-brand-primary)]"
                          : "border-[var(--sdk-color-border-default)] text-[var(--sdk-color-text-secondary)] hover:bg-[var(--sdk-color-surface-panel-muted)]"
                      }`}
                      key={method}
                      onClick={() => setPaymentMethod(method)}
                      type="button"
                    >
                      {formatPaymentMethod(method)}
                    </button>
                  );
                })}
              </div>
            </div>
          ) : (
            <p className="rounded-[var(--sdk-radius-field)] border border-dashed border-[var(--sdk-color-border-default)] px-4 py-3 text-sm text-[var(--sdk-color-text-secondary)]">
              {copy.rechargeDialog.checkoutFlowDescription}
            </p>
          )}

          <div className="rounded-[var(--sdk-radius-field)] border border-[var(--sdk-color-border-subtle)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3">
            <div className="flex items-center justify-between gap-3 text-sm">
              <span className="text-[var(--sdk-color-text-secondary)]">{copy.rechargeDialog.rateLabel}</span>
              <span className="font-medium text-[var(--sdk-color-text-primary)]">
                {formatPointsRate(state.overview.pointsToCashRate)}
              </span>
            </div>
            <div className="mt-2 flex items-center justify-between gap-3">
              <span className="text-sm text-[var(--sdk-color-text-secondary)]">
                {copy.rechargeDialog.estimatedPriceLabel}
              </span>
              <span className="text-lg font-semibold tabular-nums text-[var(--sdk-color-text-primary)]">
                {formatCurrencyCny(payableAmount)}
              </span>
            </div>
            <p className="mt-2 text-2xl font-semibold tabular-nums text-[var(--sdk-color-text-primary)]">
              {formatPoints(effectivePoints || 0)}
            </p>
          </div>
        </div>

        <DialogFooter className="border-t border-[var(--sdk-color-border-subtle)] px-6 py-4 sm:justify-end">
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
