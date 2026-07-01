export interface CreateWalletAdjustmentRequest {
  tenantId: string;
  organizationId?: string;
  ownerUserId: string;
  accountId?: string;
  assetType: 'cash' | 'points' | 'token';
  currencyCode?: string;
  direction: 'credit' | 'debit';
  amount: string;
  businessType: string;
  transactionNo: string;
  requestNo: string;
  idempotencyKey: string;
}
