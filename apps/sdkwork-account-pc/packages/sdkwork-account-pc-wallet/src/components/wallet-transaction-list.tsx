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
    <section className="rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel)]">
      <div className="border-b border-[var(--sdk-color-border-subtle)] px-5 py-4 sm:px-6">
        <h2 className="text-sm font-semibold text-[var(--sdk-color-text-primary)]">
          {copy.transactionList.title}
        </h2>
        <p className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
          {copy.transactionList.description}
        </p>
      </div>

      {transactions.length === 0 ? (
        <div className="px-5 py-6 sm:px-6">
          <StatusNotice title={copy.transactionList.emptyTitle}>
            {copy.transactionList.emptyDescription}
          </StatusNotice>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[36rem] text-sm">
            <thead>
              <tr className="border-b border-[var(--sdk-color-border-subtle)] text-left text-xs text-[var(--sdk-color-text-muted)]">
                <th className="px-5 py-3 font-medium sm:px-6">{copy.transactionList.columnDescription}</th>
                <th className="px-5 py-3 font-medium sm:px-6">{copy.transactionList.columnPoints}</th>
                <th className="px-5 py-3 font-medium sm:px-6">{copy.transactionList.columnAmount}</th>
                <th className="px-5 py-3 font-medium sm:px-6">{copy.transactionList.columnStatus}</th>
                <th className="px-5 py-3 text-right font-medium sm:px-6">{copy.transactionList.columnTime}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--sdk-color-border-subtle)]">
              {transactions.map((transaction) => {
                const isPositive = transaction.pointsDelta > 0;

                return (
                  <tr className="hover:bg-[var(--sdk-color-surface-panel-muted)]" key={transaction.id}>
                    <td className="px-5 py-3 sm:px-6">
                      <div className="font-medium text-[var(--sdk-color-text-primary)]">
                        {transaction.title}
                      </div>
                      <div className="mt-0.5 text-xs text-[var(--sdk-color-text-muted)]">
                        {transaction.transactionTypeName || transaction.transactionType || copy.transactionList.fallbackType}
                      </div>
                    </td>
                    <td className={`px-5 py-3 tabular-nums sm:px-6 ${isPositive ? "text-[var(--sdk-color-state-success)]" : "text-[var(--sdk-color-text-primary)]"}`}>
                      {formatWalletDelta(transaction.pointsDelta)}
                    </td>
                    <td className="px-5 py-3 tabular-nums text-[var(--sdk-color-text-secondary)] sm:px-6">
                      {formatCurrencyCny(transaction.cashAmountCny)}
                    </td>
                    <td className="px-5 py-3 text-[var(--sdk-color-text-secondary)] sm:px-6">
                      {formatTransactionStatus(transaction.status)}
                    </td>
                    <td className="px-5 py-3 text-right text-xs text-[var(--sdk-color-text-muted)] sm:px-6">
                      {formatTransactionTimestamp(transaction.createdAt)}
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
