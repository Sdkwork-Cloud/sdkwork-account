import { Button } from "@sdkwork/ui-pc-react";
import { useSdkworkWalletIntl } from "../wallet-intl";
import type { SdkworkWalletOverview } from "../wallet-service";

export interface SdkworkWalletBalancePanelProps {
  onOpenRecharge: () => void;
  onOpenWithdraw: () => void;
  overview: SdkworkWalletOverview;
}

export function SdkworkWalletBalancePanel({
  onOpenRecharge,
  onOpenWithdraw,
  overview,
}: SdkworkWalletBalancePanelProps) {
  const {
    copy,
    formatAccountLevelLabel,
    formatAccountState,
    formatCurrencyCny,
    formatPayProtection,
    formatPoints,
    formatPointsRate,
  } = useSdkworkWalletIntl();

  const metrics = [
    {
      label: copy.balancePanel.cashAvailableLabel,
      value: formatCurrencyCny(overview.account.cashAvailable),
    },
    {
      label: copy.balancePanel.payProtectionLabel,
      value: formatPayProtection(overview.account.hasPayPassword),
    },
    {
      label: copy.balancePanel.accountLevelLabel,
      value: formatAccountLevelLabel(overview.account),
    },
    {
      label: copy.balancePanel.exchangeRateLabel,
      value: formatPointsRate(overview.pointsToCashRate),
    },
  ];

  return (
    <section className="overflow-hidden rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel)]">
      <div className="flex flex-col gap-4 border-b border-[var(--sdk-color-border-subtle)] px-5 py-5 sm:flex-row sm:items-start sm:justify-between sm:px-6">
        <div className="min-w-0">
          <h1 className="text-lg font-semibold tracking-tight text-[var(--sdk-color-text-primary)]">
            {copy.balancePanel.title}
          </h1>
          <p className="mt-1 max-w-xl text-sm text-[var(--sdk-color-text-secondary)]">
            {copy.balancePanel.description}
          </p>
          <p className="mt-2 text-xs text-[var(--sdk-color-text-muted)]">
            {formatAccountState(overview.account, overview.isAuthenticated)}
          </p>
        </div>
        <div className="flex shrink-0 flex-wrap gap-2">
          <Button onClick={onOpenRecharge} type="button">
            {copy.balancePanel.primaryAction}
          </Button>
          <Button onClick={onOpenWithdraw} type="button" variant="outline">
            {copy.actions.withdraw}
          </Button>
        </div>
      </div>

      <div className="px-5 py-6 sm:px-6">
        <p className="text-sm text-[var(--sdk-color-text-secondary)]">
          {copy.balancePanel.availablePointsLabel}
        </p>
        <p className="mt-1 text-4xl font-semibold tabular-nums tracking-tight text-[var(--sdk-color-text-primary)]">
          {formatPoints(overview.account.availablePoints)}
        </p>
      </div>

      <dl className="grid grid-cols-2 border-t border-[var(--sdk-color-border-subtle)] sm:grid-cols-4">
        {metrics.map((metric) => (
          <div
            className="border-[var(--sdk-color-border-subtle)] px-5 py-4 sm:border-l sm:first:border-l-0 [&:nth-child(2)]:border-l-0 sm:[&:nth-child(2)]:border-l"
            key={metric.label}
          >
            <dt className="text-xs text-[var(--sdk-color-text-muted)]">{metric.label}</dt>
            <dd className="mt-1 text-sm font-medium tabular-nums text-[var(--sdk-color-text-primary)]">
              {metric.value}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
