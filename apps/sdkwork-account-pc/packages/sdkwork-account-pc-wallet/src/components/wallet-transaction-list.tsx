import { StatusNotice } from "@sdkwork/ui-pc-react";
import { useSdkworkWalletIntl } from "../wallet-intl";
import type { SdkworkWalletTransaction } from "../wallet-service";

export interface SdkworkWalletTransactionListProps {
  transactions: SdkworkWalletTransaction[];
}

export function SdkworkWalletTransactionList({
  transactions,
}: SdkworkWalletTransactionListProps) {
  const {
    copy,
    formatCurrencyCny,
    formatTransactionStatus,
    formatTransactionTimestamp,
    formatWalletDelta,
  } = useSdkworkWalletIntl();

  return (
    <section className="rounded-[1.75rem] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel)] p-5 shadow-[var(--sdk-shadow-sm)]">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h2 className="text-xl font-semibold text-[var(--sdk-color-text-primary)]">{copy.transactionList.title}</h2>
          <p className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
            {copy.transactionList.description}
          </p>
        </div>
      </div>

      {transactions.length === 0 ? (
        <StatusNotice className="mt-5" title={copy.transactionList.emptyTitle}>
          {copy.transactionList.emptyDescription}
        </StatusNotice>
      ) : (
        <div className="mt-5 space-y-3">
          {transactions.map((transaction) => {
            const isPositive = transaction.pointsDelta > 0;

            return (
              <article
                className="rounded-[1.25rem] border border-[var(--sdk-color-border-subtle)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-4"
                key={transaction.id}
              >
                <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div className="min-w-0">
                    <div className="truncate text-sm font-semibold text-[var(--sdk-color-text-primary)]">
                      {transaction.title}
                    </div>
                    <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
                      {transaction.transactionTypeName || transaction.transactionType || copy.transactionList.fallbackType}
                    </div>
                    <div className="mt-2 text-xs text-[var(--sdk-color-text-muted)]">
                      {formatTransactionTimestamp(transaction.createdAt)}
                    </div>
                  </div>

                  <div className="flex flex-col items-start gap-2 text-left sm:items-end sm:text-right">
                    <div className={isPositive ? "text-emerald-500" : "text-[var(--sdk-color-text-primary)]"}>
                      {formatWalletDelta(transaction.pointsDelta)}
                    </div>
                    <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                      {formatCurrencyCny(transaction.cashAmountCny)}
                    </div>
                    <div className="rounded-full border border-[var(--sdk-color-border-default)] px-2.5 py-1 text-[0.65rem] font-semibold uppercase tracking-[0.14em] text-[var(--sdk-color-text-muted)]">
                      {formatTransactionStatus(transaction.status)}
                    </div>
                  </div>
                </div>
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}
