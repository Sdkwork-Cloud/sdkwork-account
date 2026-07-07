import type { AccountConsumptionItem } from './account-consumption-item';

export interface AccountSummaryItem {
  id: string;
  name: string;
  email: string;
  isVerified: boolean;
  tier: string;
  organization: string;
  availablePoints: string;
  estDaysRemaining: string;
  monthlyPointsConsumed: string;
  consumptionByService?: AccountConsumptionItem[];
  invoiceSettings?: Record<string, unknown>;
  security?: Record<string, unknown>;
  loginLogs?: Record<string, unknown>[];
}
