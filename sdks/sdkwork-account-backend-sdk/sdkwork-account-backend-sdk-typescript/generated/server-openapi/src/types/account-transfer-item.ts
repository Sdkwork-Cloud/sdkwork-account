export interface AccountTransferItem {
  id: string;
  uuid: string;
  tenantId: string;
  organizationId?: string;
  fromAccountId: string;
  toAccountId: string;
  ownerUserId: string;
  assetType: string;
  amount: string;
  status: string;
  businessType: string;
  businessNo: string;
  requestNo: string;
  idempotencyKey: string;
  journalId: string;
  traceId: string;
  createdAt: string;
}
