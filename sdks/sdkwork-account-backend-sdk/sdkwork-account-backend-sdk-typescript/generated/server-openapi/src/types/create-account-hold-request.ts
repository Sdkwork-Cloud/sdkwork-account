export interface CreateAccountHoldRequest {
  tenantId: string;
  organizationId?: string;
  ownerUserId: string;
  accountId?: string;
  assetType: 'cash' | 'points' | 'token_bank';
  amount: string;
  businessType: string;
  businessNo: string;
  sourceType: string;
  sourceId: string;
  requestNo: string;
  idempotencyKey: string;
  expiresAt?: string;
}
