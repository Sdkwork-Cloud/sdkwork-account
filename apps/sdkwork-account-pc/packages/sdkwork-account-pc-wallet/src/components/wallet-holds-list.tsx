import { Button, StatusNotice } from "@sdkwork/ui-pc-react";
import { useSdkworkWalletIntl } from "../wallet-intl";
import type { SdkworkWalletHold } from "../wallet-service";

export interface SdkworkWalletHoldsListProps {
  hasMore?: boolean;
  holds: SdkworkWalletHold[];
  isLoadingMore?: boolean;
  onLoadMore?: () => void;
}

export function SdkworkWalletHoldsList({
  hasMore = false,
  holds,
  isLoadingMore = false,
  onLoadMore,
}: SdkworkWalletHoldsListProps) {
  const {
    copy,
    formatCurrencyCny,
    formatHoldStatus,
    formatPoints,
    formatTokenBank,
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
                const normalizedAssetType = hold.assetType.toLowerCase();
                const assetLabel = normalizedAssetType === "points"
                  ? copy.holdList.pointsAsset
                  : normalizedAssetType === "token_bank"
                    ? copy.holdList.tokenBankAsset
                    : normalizedAssetType === "cash"
                      ? copy.holdList.cashAsset
                      : copy.holdList.unknownAsset;
                const amountLabel = normalizedAssetType === "points"
                  ? formatPoints(hold.amount)
                  : normalizedAssetType === "token_bank"
                    ? formatTokenBank(hold.amount)
                    : formatCurrencyCny(hold.amount);

                return (
                  <tr className="hover:bg-[var(--sdk-color-surface-panel-muted)]" key={hold.id}>
                    <td className="px-5 py-3 sm:px-6">
                      <div className="font-medium text-[var(--sdk-color-text-primary)]">
                        {hold.businessType || copy.holdList.fallbackType}
                      </div>
                      <div className="mt-0.5 text-xs text-[var(--sdk-color-text-muted)]">
                        {assetLabel} - {hold.businessNo || hold.holdId}
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

      {hasMore && onLoadMore ? (
        <div className="border-t border-[var(--sdk-color-border-subtle)] px-5 py-4 sm:px-6">
          <Button
            loading={isLoadingMore}
            onClick={onLoadMore}
            size="sm"
            type="button"
            variant="outline"
          >
            {copy.holdList.loadMore}
          </Button>
        </div>
      ) : null}
    </section>
  );
}
