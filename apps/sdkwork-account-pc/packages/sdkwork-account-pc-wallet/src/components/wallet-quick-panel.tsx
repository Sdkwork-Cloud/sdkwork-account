import { ArrowRight } from "lucide-react";
import { Button } from "@sdkwork/ui-pc-react";
import { useSdkworkWalletIntl } from "../wallet-intl";
import type { SdkworkWalletOverview } from "../wallet-service";

export interface SdkworkWalletQuickPanelProps {
  onOpenPage: () => void;
  onRecharge: () => void;
  onWithdraw: () => void;
  overview: SdkworkWalletOverview;
}

export function SdkworkWalletQuickPanel({
  onOpenPage,
  onRecharge,
  onWithdraw,
  overview,
}: SdkworkWalletQuickPanelProps) {
  const recentTransactions = overview.transactions.slice(0, 4);
  const {
    copy,
    formatAccountLevelSummary,
    formatCurrencyCny,
    formatPoints,
    formatPointsRate,
    formatWalletDelta,
  } = useSdkworkWalletIntl();

  return (
    <div className="w-[20rem] rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel)] shadow-[var(--sdk-shadow-md)]">
      <div className="border-b border-[var(--sdk-color-border-subtle)] px-4 py-4">
        <p className="text-xs text-[var(--sdk-color-text-muted)]">
          {copy.quickPanel.availablePointsLabel}
        </p>
        <p className="mt-1 text-2xl font-semibold tabular-nums tracking-tight text-[var(--sdk-color-text-primary)]">
          {formatPoints(overview.account.availablePoints)}
        </p>
        <p className="mt-1 text-xs text-[var(--sdk-color-text-secondary)]">
          {overview.isAuthenticated
            ? formatAccountLevelSummary(overview.account)
            : copy.quickPanel.signInToUnlock}
        </p>
        <div className="mt-3 flex gap-4 text-xs text-[var(--sdk-color-text-muted)]">
          <span>
            {copy.quickPanel.cashAvailableLabel}: {formatCurrencyCny(overview.account.cashAvailable)}
          </span>
          <span>
            {copy.quickPanel.rateLabel}: {formatPointsRate(overview.pointsToCashRate)}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2 border-b border-[var(--sdk-color-border-subtle)] p-3">
        <Button onClick={onRecharge} size="sm" type="button">
          {copy.actions.recharge}
        </Button>
        <Button onClick={onWithdraw} size="sm" type="button" variant="outline">
          {copy.actions.withdraw}
        </Button>
      </div>

      <div className="p-3">
        <div className="flex items-center justify-between gap-2">
          <p className="text-xs font-medium text-[var(--sdk-color-text-primary)]">
            {copy.quickPanel.recentActivityTitle}
          </p>
          <button
            className="inline-flex items-center gap-1 text-xs text-[var(--sdk-color-brand-primary)] hover:underline"
            onClick={onOpenPage}
            type="button"
          >
            {copy.quickPanel.openCenterAction}
            <ArrowRight className="h-3 w-3" aria-hidden="true" />
          </button>
        </div>

        <div className="mt-2 space-y-1">
          {recentTransactions.length === 0 ? (
            <p className="rounded-[var(--sdk-radius-field)] border border-dashed border-[var(--sdk-color-border-default)] px-3 py-3 text-xs text-[var(--sdk-color-text-secondary)]">
              {copy.quickPanel.noRecentActivity}
            </p>
          ) : recentTransactions.map((transaction) => (
            <div
              className="flex items-center justify-between gap-2 rounded-[var(--sdk-radius-field)] px-2 py-2 hover:bg-[var(--sdk-color-surface-panel-muted)]"
              key={transaction.id}
            >
              <div className="min-w-0 flex-1">
                <p className="truncate text-xs font-medium text-[var(--sdk-color-text-primary)]">
                  {transaction.title}
                </p>
              </div>
              <span className={`shrink-0 text-xs tabular-nums ${transaction.pointsDelta >= 0 ? "text-[var(--sdk-color-state-success)]" : "text-[var(--sdk-color-text-primary)]"}`}>
                {formatWalletDelta(transaction.pointsDelta)}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
