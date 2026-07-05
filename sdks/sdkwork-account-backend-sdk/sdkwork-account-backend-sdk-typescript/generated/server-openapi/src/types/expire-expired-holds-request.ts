export interface ExpireExpiredHoldsRequest {
  tenantId: string;
  organizationId?: string;
  ownerUserId?: string;
  accountId?: string;
  requestNo: string;
  idempotencyKey: string;
}
