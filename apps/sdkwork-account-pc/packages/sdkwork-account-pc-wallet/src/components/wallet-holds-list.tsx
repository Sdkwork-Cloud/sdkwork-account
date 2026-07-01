import { StatusNotice } from "@sdkwork/ui-pc-react";
import { useSdkworkWalletIntl } from "../wallet-intl";
import type { SdkworkWalletHold } from "../wallet-service";

export interface SdkworkWalletHoldsListProps {
  holds: SdkworkWalletHold[];
}

export function SdkworkWalletHoldsList({ holds }: SdkworkWalletHoldsListProps) {
  const {
    copy,
    formatCurrencyCny,
    formatHoldStatus,
    formatPoints,
    formatTransactionTimestamp,
  } = useSdkworkWalletIntl();

  return (
    <section className="rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel)]">
      <div className="border-b border-[var(--sdk-color-border-subtle)] px-5 py-4 sm:px-6">
        <h2 className="text-sm font-semibold text-[var(--sdk-color-text-primary)]">{copy.holdList.title}</h2>
        <p className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
          {copy.holdList.description}
        </p>
      </div>

      {holds.length === 0 ? (
        <div className="px-5 py-6 sm:px-6">
          <StatusNotice title={copy.holdList.emptyTitle}>
            {copy.holdList.emptyDescription}
          </StatusNotice>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[32rem] text-sm">
            <tbody className="divide-y divide-[var(--sdk-color-border-subtle)]">
              {holds.map((hold) => {
                const isPoints = hold.assetType.toLowerCase() === "points";
                const amountLabel = isPoints
                  ? formatPoints(hold.amount)
                  : formatCurrencyCny(hold.amount);

                return (
                  <tr className="hover:bg-[var(--sdk-color-surface-panel-muted)]" key={hold.id}>
                    <td className="px-5 py-3 sm:px-6">
                      <div className="font-medium text-[var(--sdk-color-text-primary)]">
                        {hold.businessType || copy.holdList.fallbackType}
                      </div>
                      <div className="mt-0.5 text-xs text-[var(--sdk-color-text-muted)]">
                        {hold.businessNo || hold.holdId}
                      </div>
                    </td>
                    <td className="px-5 py-3 tabular-nums text-[var(--sdk-color-text-primary)] sm:px-6">
                      {amountLabel}
                    </td>
                    <td className="px-5 py-3 text-[var(--sdk-color-text-secondary)] sm:px-6">
                      {formatHoldStatus(hold.status)}
                    </td>
                    <td className="px-5 py-3 text-right text-xs text-[var(--sdk-color-text-muted)] sm:px-6">
                      {formatTransactionTimestamp(hold.createdAt)}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
