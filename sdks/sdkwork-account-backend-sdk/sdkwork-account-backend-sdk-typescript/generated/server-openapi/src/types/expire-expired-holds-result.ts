export interface ExpireExpiredHoldsResult {
  accepted: boolean;
  replayed: boolean;
  expiredHoldCount: string;
  releasedAmountTotal: string;
}
