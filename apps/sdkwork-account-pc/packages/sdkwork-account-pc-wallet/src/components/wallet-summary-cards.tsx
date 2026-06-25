import {
  ArrowDownCircle,
  Banknote,
  Coins,
  Crown,
} from "lucide-react";
import { StatCard } from "@sdkwork/ui-pc-react";
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

  return (
    <div className="grid gap-5 md:grid-cols-2 xl:grid-cols-4">
      <StatCard
        icon={<Banknote className="h-5 w-5" />}
        label={copy.summaryCards.cashAvailableLabel}
        value={formatCurrencyCny(overview.account.cashAvailable)}
      />
      <StatCard
        icon={<Coins className="h-5 w-5" />}
        label={copy.summaryCards.totalEarnedLabel}
        value={formatPoints(overview.account.totalEarned)}
      />
      <StatCard
        change="-"
        changeTone="danger"
        icon={<ArrowDownCircle className="h-5 w-5" />}
        label={copy.summaryCards.totalSpentLabel}
        value={formatPoints(overview.account.totalSpent)}
      />
      <StatCard
        icon={<Crown className="h-5 w-5" />}
        label={copy.summaryCards.accountLevelLabel}
        value={formatAccountLevelLabel(overview.account)}
      />
    </div>
  );
}
