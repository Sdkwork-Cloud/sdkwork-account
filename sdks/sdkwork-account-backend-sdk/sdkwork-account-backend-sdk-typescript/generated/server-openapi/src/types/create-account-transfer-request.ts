export interface CreateAccountTransferRequest {
  tenantId: string;
  organizationId?: string;
  fromAccountId: string;
  toAccountId: string;
  ownerUserId: string;
  assetType: 'cash' | 'points' | 'token_bank';
  amount: string;
  businessType: string;
  businessNo: string;
  requestNo: string;
  idempotencyKey: string;
}
