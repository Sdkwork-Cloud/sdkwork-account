import type { AccountConsumptionItem } from './account-consumption-item';
import type { AccountInvoiceSettingsItem } from './account-invoice-settings-item';
import type { AccountLoginLogItem } from './account-login-log-item';
import type { AccountSecuritySummaryItem } from './account-security-summary-item';

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
  invoiceSettings?: AccountInvoiceSettingsItem;
  security?: AccountSecuritySummaryItem;
  loginLogs?: AccountLoginLogItem[];
}
