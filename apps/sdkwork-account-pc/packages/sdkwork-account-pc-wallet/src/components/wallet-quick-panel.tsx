import {
  ArrowRight,
  Banknote,
  Coins,
  Sparkles,
} from "lucide-react";
import { Button } from "@sdkwork/ui-pc-react";
import {
  createSdkworkWalletGlassStyle,
  createSdkworkWalletHeroStyle,
  createSdkworkWalletHeroTextStyle,
  createSdkworkWalletPanelStyle,
  createSdkworkWalletToneStyle,
} from "../wallet-appearance";
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
  const featuredRechargePackage =
    overview.rechargePackages.find((rechargePackage) => rechargePackage.recommended)
    ?? overview.rechargePackages[0]
    ?? null;
  const recentTransactions = overview.transactions.slice(0, 4);
  const {
    copy,
    formatAccountLevelLabel,
    formatAccountLevelSummary,
    formatCurrencyCny,
    formatPoints,
    formatPointsRate,
    formatRechargePackageSummary,
    formatWalletDelta,
  } = useSdkworkWalletIntl();
  const primaryHeroTextStyle = createSdkworkWalletHeroTextStyle();
  const mutedHeroTextStyle = createSdkworkWalletHeroTextStyle("muted");
  const subtleHeroTextStyle = createSdkworkWalletHeroTextStyle("subtle");

  return (
    <div
      className="w-[24rem] rounded-[1.5rem] border border-[var(--sdk-color-border-default)] p-4 shadow-[var(--sdk-shadow-lg)]"
      style={createSdkworkWalletPanelStyle("neutral", {
        backgroundWeight: 4,
        borderWeight: 18,
        surfaceWeight: 94,
      })}
    >
      <div
        className="rounded-[1.25rem] border border-[color-mix(in_srgb,var(--sdk-color-border-default)_72%,transparent)] p-4 text-white shadow-[var(--sdk-shadow-sm)]"
        style={createSdkworkWalletHeroStyle()}
      >
        <div className="flex items-start justify-between gap-3">
          <div>
            <div className="text-[0.7rem] font-semibold uppercase tracking-[0.18em]" style={subtleHeroTextStyle}>
              {copy.quickPanel.availablePointsLabel}
            </div>
            <div className="mt-2 text-3xl font-semibold tracking-tight" style={primaryHeroTextStyle}>
              {formatPoints(overview.account.availablePoints)}
            </div>
            <div className="mt-1 text-sm" style={mutedHeroTextStyle}>
              {overview.isAuthenticated
                ? formatAccountLevelSummary(overview.account)
                : copy.quickPanel.signInToUnlock}
            </div>
          </div>
          <div
            className="flex h-11 w-11 items-center justify-center rounded-[1rem] border"
            style={createSdkworkWalletToneStyle("brand", {
              backgroundWeight: 14,
              borderWeight: 24,
            })}
          >
            <Coins className="h-5 w-5" />
          </div>
        </div>

        <div className="mt-4 grid grid-cols-2 gap-3">
          <div
            className="rounded-[1rem] border px-3 py-2"
            style={createSdkworkWalletGlassStyle("neutral", {
              backgroundWeight: 10,
              borderWeight: 22,
              surfaceWeight: 78,
            })}
          >
            <div className="flex items-center gap-2 text-[0.65rem] uppercase tracking-[0.16em]" style={subtleHeroTextStyle}>
              <Banknote className="h-3.5 w-3.5" />
              {copy.quickPanel.cashAvailableLabel}
            </div>
            <div className="mt-1 text-sm font-semibold" style={primaryHeroTextStyle}>
              {formatCurrencyCny(overview.account.cashAvailable)}
            </div>
          </div>
          <div
            className="rounded-[1rem] border px-3 py-2"
            style={createSdkworkWalletGlassStyle("accent", {
              backgroundWeight: 10,
              borderWeight: 22,
              surfaceWeight: 78,
            })}
          >
            <div className="flex items-center gap-2 text-[0.65rem] uppercase tracking-[0.16em]" style={subtleHeroTextStyle}>
              <Sparkles className="h-3.5 w-3.5" />
              {copy.quickPanel.currentAccountLabel}
            </div>
            <div className="mt-1 text-sm font-semibold" style={primaryHeroTextStyle}>
              {formatAccountLevelLabel(overview.account)}
            </div>
          </div>
        </div>

        <div className="mt-4 text-xs" style={subtleHeroTextStyle}>
          {copy.quickPanel.rateLabel}: {formatPointsRate(overview.pointsToCashRate)}
        </div>
      </div>

      <div className="mt-4 grid grid-cols-2 gap-3">
        <Button className="justify-start" onClick={onRecharge} size="sm" type="button" variant="secondary">
          <Coins className="h-4 w-4" />
          {copy.actions.recharge}
        </Button>
        <Button className="justify-start" onClick={onWithdraw} size="sm" type="button" variant="outline">
          <Banknote className="h-4 w-4" />
          {copy.actions.withdraw}
        </Button>
      </div>

      <div
        className="mt-5 rounded-[1.1rem] border px-4 py-3"
        style={createSdkworkWalletPanelStyle("accent", {
          backgroundWeight: 8,
          borderWeight: 24,
          surfaceWeight: 90,
        })}
      >
        <div className="text-xs font-semibold uppercase tracking-[0.16em] text-[var(--sdk-color-text-muted)]">
          {copy.quickPanel.rechargeLaneLabel}
        </div>
        <div className="mt-2 text-sm font-semibold text-[var(--sdk-color-text-primary)]">
          {featuredRechargePackage ? featuredRechargePackage.title : copy.balancePanel.noRechargePackagePublished}
        </div>
        <div className="mt-1 text-xs text-[var(--sdk-color-text-secondary)]">
          {featuredRechargePackage
            ? formatRechargePackageSummary(featuredRechargePackage)
            : copy.quickPanel.noRechargePackageDescription}
        </div>
      </div>

      <div className="mt-5">
        <div className="flex items-center justify-between gap-3">
          <div className="text-sm font-semibold text-[var(--sdk-color-text-primary)]">
            {copy.quickPanel.recentActivityTitle}
          </div>
          <button
            className="inline-flex items-center gap-1 text-xs font-semibold text-[var(--sdk-color-brand-primary)]"
            onClick={onOpenPage}
            type="button"
          >
            {copy.quickPanel.openCenterAction}
            <ArrowRight className="h-3.5 w-3.5" />
          </button>
        </div>

        <div className="mt-3 space-y-3">
          {recentTransactions.length === 0 ? (
            <div className="rounded-[1rem] border border-dashed border-[var(--sdk-color-border-default)] px-4 py-4 text-sm text-[var(--sdk-color-text-secondary)]">
              {copy.quickPanel.noRecentActivity}
            </div>
          ) : recentTransactions.map((transaction) => (
            <div
              className="rounded-[1rem] border border-[var(--sdk-color-border-subtle)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3"
              key={transaction.id}
            >
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="truncate text-sm font-semibold text-[var(--sdk-color-text-primary)]">
                    {transaction.title}
                  </div>
                  <div className="mt-1 text-xs text-[var(--sdk-color-text-secondary)]">
                    {transaction.transactionTypeName || transaction.transactionType || copy.transactionList.fallbackType}
                  </div>
                </div>
                <div className={transaction.pointsDelta >= 0 ? "text-emerald-500" : "text-[var(--sdk-color-text-primary)]"}>
                  {formatWalletDelta(transaction.pointsDelta)}
                </div>
              </div>
              <div className="mt-2 text-[0.7rem] text-[var(--sdk-color-text-muted)]">
                {formatCurrencyCny(transaction.cashAmountCny)}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
