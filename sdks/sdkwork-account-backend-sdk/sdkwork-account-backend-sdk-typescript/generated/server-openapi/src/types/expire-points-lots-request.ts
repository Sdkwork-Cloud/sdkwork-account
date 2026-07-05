export interface ExpirePointsLotsRequest {
  tenantId: string;
  organizationId?: string;
  /** Optional scope filter; when set only lots owned by this user are swept. */
  ownerUserId?: string;
  /** Optional scope filter; when set only lots for this account are swept. */
  accountId?: string;
  requestNo: string;
  idempotencyKey: string;
}
