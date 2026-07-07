export interface CreateTokenBankTransferRequest {
  tenantId: string;
  organizationId?: string;
  fromAccountId: string;
  toAccountId: string;
  ownerUserId: string;
  amount: string;
  businessType: string;
  businessNo: string;
  requestNo: string;
  idempotencyKey: string;
}
