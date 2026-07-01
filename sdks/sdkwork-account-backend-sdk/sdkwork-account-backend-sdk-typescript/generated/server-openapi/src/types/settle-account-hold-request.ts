export interface SettleAccountHoldRequest {
  tenantId: string;
  businessType: string;
  transactionNo: string;
  requestNo: string;
  idempotencyKey: string;
}
