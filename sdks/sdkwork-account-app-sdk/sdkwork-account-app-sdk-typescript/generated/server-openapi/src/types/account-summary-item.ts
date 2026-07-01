export interface AccountSummaryItem {
  id: string;
  name: string;
  email: string;
  isVerified: boolean;
  tier: string;
  organization: string;
  availableCredits: number;
  estDaysRemaining: string;
  monthlyConsumption: number;
  consumptionByService?: Record<string, unknown>[];
  invoiceSettings?: Record<string, unknown>;
  security?: Record<string, unknown>;
  loginLogs?: Record<string, unknown>[];
}
