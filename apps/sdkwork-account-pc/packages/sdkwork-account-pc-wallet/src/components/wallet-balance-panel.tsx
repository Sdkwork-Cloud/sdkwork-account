import {
  Banknote,
  Coins,
  Crown,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { Button } from "@sdkwork/ui-pc-react";
import {
  createSdkworkWalletGlassStyle,
  createSdkworkWalletHeroStyle,
  createSdkworkWalletHeroTextStyle,
  createSdkworkWalletToneStyle,
} from "../wallet-appearance";
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
    formatRechargePackageSummary,
  } = useSdkworkWalletIntl();
  const featuredRecharge =
    overview.rechargePackages.find((rechargePackage) => rechargePackage.recommended)
    ?? overview.rechargePackages[0]
    ?? null;
  const primaryHeroTextStyle = createSdkworkWalletHeroTextStyle();
  const mutedHeroTextStyle = createSdkworkWalletHeroTextStyle("muted");
  const subtleHeroTextStyle = createSdkworkWalletHeroTextStyle("subtle");

  return (
    <section
      className="overflow-hidden rounded-[2rem] border border-[color-mix(in_srgb,var(--sdk-color-border-default)_72%,transparent)] px-6 py-7 text-white shadow-[var(--sdk-shadow-lg)]"
      style={createSdkworkWalletHeroStyle()}
    >
      <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div className="text-[0.7rem] font-semibold uppercase tracking-[0.18em]" style={subtleHeroTextStyle}>
            {copy.balancePanel.eyebrow}
          </div>
          <h1 className="mt-3 text-4xl font-semibold tracking-tight" style={primaryHeroTextStyle}>
            {copy.balancePanel.title}
          </h1>
          <p className="mt-3 max-w-2xl text-sm leading-7" style={mutedHeroTextStyle}>
            {copy.balancePanel.description}
          </p>
        </div>
        <div className="flex flex-wrap gap-3">
          <Button onClick={onOpenRecharge} type="button" variant="secondary">
            {copy.balancePanel.primaryAction}
          </Button>
          <Button onClick={onOpenWithdraw} type="button" variant="outline">
            {copy.actions.withdraw}
          </Button>
        </div>
      </div>

      <div className="mt-8 grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(20rem,0.85fr)]">
        <div
          className="rounded-[1.5rem] border p-5 shadow-[var(--sdk-shadow-sm)] backdrop-blur-xl"
          style={createSdkworkWalletGlassStyle("brand", {
            backgroundWeight: 12,
            borderWeight: 26,
          })}
        >
          <div className="flex items-start justify-between gap-4">
            <div>
              <div className="text-sm" style={mutedHeroTextStyle}>{copy.balancePanel.availablePointsLabel}</div>
              <div className="mt-3 text-5xl font-semibold tracking-tight" style={primaryHeroTextStyle}>
                {formatPoints(overview.account.availablePoints)}
              </div>
              <div className="mt-3 text-sm" style={mutedHeroTextStyle}>
                {copy.balancePanel.exchangeRateLabel}: {formatPointsRate(overview.pointsToCashRate)}
              </div>
            </div>
            <div
              className="flex h-14 w-14 items-center justify-center rounded-[1.5rem] border"
              style={createSdkworkWalletToneStyle("brand", {
                backgroundWeight: 14,
                borderWeight: 24,
              })}
            >
              <Coins className="h-6 w-6" />
            </div>
          </div>

          <div className="mt-5 grid gap-3 sm:grid-cols-2">
            <div
              className="rounded-[1rem] border px-4 py-3"
              style={createSdkworkWalletGlassStyle("neutral", {
                backgroundWeight: 10,
                borderWeight: 22,
                surfaceWeight: 80,
              })}
            >
              <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.16em]" style={subtleHeroTextStyle}>
                <Banknote className="h-3.5 w-3.5" />
                {copy.balancePanel.cashAvailableLabel}
              </div>
              <div className="mt-2 text-lg font-semibold" style={primaryHeroTextStyle}>
                {formatCurrencyCny(overview.account.cashAvailable)}
              </div>
            </div>
            <div
              className="rounded-[1rem] border px-4 py-3"
              style={createSdkworkWalletGlassStyle("accent", {
                backgroundWeight: 10,
                borderWeight: 22,
                surfaceWeight: 80,
              })}
            >
              <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.16em]" style={subtleHeroTextStyle}>
                <ShieldCheck className="h-3.5 w-3.5" />
                {copy.balancePanel.payProtectionLabel}
              </div>
              <div className="mt-2 text-lg font-semibold" style={primaryHeroTextStyle}>
                {formatPayProtection(overview.account.hasPayPassword)}
              </div>
            </div>
          </div>
        </div>

        <div
          className="rounded-[1.5rem] border p-5 shadow-[var(--sdk-shadow-sm)] backdrop-blur-xl"
          style={createSdkworkWalletGlassStyle("accent", {
            backgroundWeight: 12,
            borderWeight: 26,
          })}
        >
          <div className="flex items-center gap-3">
            <div
              className="flex h-12 w-12 items-center justify-center rounded-[1rem] border"
              style={createSdkworkWalletToneStyle("accent", {
                backgroundWeight: 18,
                borderWeight: 28,
              })}
            >
              <Crown className="h-5 w-5" />
            </div>
            <div>
              <div className="text-sm" style={mutedHeroTextStyle}>{copy.balancePanel.accountLevelLabel}</div>
              <div className="mt-1 text-xl font-semibold" style={primaryHeroTextStyle}>
                {formatAccountLevelLabel(overview.account)}
              </div>
            </div>
          </div>

          <div
            className="mt-4 rounded-[1rem] border px-4 py-3 text-sm"
            style={createSdkworkWalletGlassStyle("neutral", {
              backgroundWeight: 8,
              borderWeight: 20,
              surfaceWeight: 80,
            })}
          >
            <span style={mutedHeroTextStyle}>
              {formatAccountState(overview.account, overview.isAuthenticated)}
            </span>
          </div>

          <div className="mt-5 grid gap-3">
            <div
              className="rounded-[1rem] border px-4 py-3"
              style={createSdkworkWalletGlassStyle("brand", {
                backgroundWeight: 10,
                borderWeight: 22,
                surfaceWeight: 78,
              })}
            >
              <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.16em]" style={subtleHeroTextStyle}>
                <Sparkles className="h-3.5 w-3.5" />
                {copy.balancePanel.rechargeLaneLabel}
              </div>
              <div className="mt-2 text-sm font-semibold" style={primaryHeroTextStyle}>
                {featuredRecharge ? featuredRecharge.title : copy.balancePanel.noRechargePackagePublished}
              </div>
              <div className="mt-1 text-sm" style={mutedHeroTextStyle}>
                {featuredRecharge
                  ? formatRechargePackageSummary(featuredRecharge)
                  : copy.balancePanel.signInToUnlock}
              </div>
            </div>

            <div
              className="rounded-[1rem] border px-4 py-3"
              style={createSdkworkWalletGlassStyle("accent", {
                backgroundWeight: 10,
                borderWeight: 22,
                surfaceWeight: 78,
              })}
            >
              <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.16em]" style={subtleHeroTextStyle}>
                <Crown className="h-3.5 w-3.5" />
                {copy.balancePanel.accountStatusLabel}
              </div>
              <div className="mt-2 text-sm font-semibold" style={primaryHeroTextStyle}>
                {formatAccountLevelLabel(overview.account)}
              </div>
              <div className="mt-1 text-sm" style={mutedHeroTextStyle}>
                {formatAccountState(overview.account, overview.isAuthenticated)}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
