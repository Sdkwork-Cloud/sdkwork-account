export interface CreateTokenBankHoldRequest {
  tenantId: string;
  organizationId?: string;
  ownerUserId: string;
  accountId?: string;
  amount: string;
  businessType: string;
  businessNo: string;
  sourceType: string;
  sourceId: string;
  requestNo: string;
  idempotencyKey: string;
  expiresAt?: string;
}
