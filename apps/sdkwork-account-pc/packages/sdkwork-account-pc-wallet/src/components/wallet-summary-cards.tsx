import { useSdkworkWalletIntl } from "../wallet-intl";
import type { SdkworkWalletOverview } from "../wallet-service";

export interface SdkworkWalletSummaryCardsProps {
  overview: SdkworkWalletOverview;
}

export function SdkworkWalletSummaryCards({
  overview,
}: SdkworkWalletSummaryCardsProps) {
  const {
    copy,
    formatAccountLevelLabel,
    formatCurrencyCny,
    formatPoints,
  } = useSdkworkWalletIntl();

  const items = [
    {
      label: copy.summaryCards.cashAvailableLabel,
      value: formatCurrencyCny(overview.account.cashAvailable),
    },
    {
      label: copy.summaryCards.totalEarnedLabel,
      value: formatPoints(overview.account.totalEarned),
    },
    {
      label: copy.summaryCards.totalSpentLabel,
      value: formatPoints(overview.account.totalSpent),
    },
    {
      label: copy.summaryCards.accountLevelLabel,
      value: formatAccountLevelLabel(overview.account),
    },
  ];

  return (
    <section
      aria-label={copy.summaryCards.accountLevelLabel}
      className="rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel)]"
    >
      <dl className="grid grid-cols-2 divide-[var(--sdk-color-border-subtle)] sm:grid-cols-4 sm:divide-x">
        {items.map((item) => (
          <div className="px-5 py-4" key={item.label}>
            <dt className="text-xs text-[var(--sdk-color-text-muted)]">{item.label}</dt>
            <dd className="mt-1 text-lg font-semibold tabular-nums tracking-tight text-[var(--sdk-color-text-primary)]">
              {item.value}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
