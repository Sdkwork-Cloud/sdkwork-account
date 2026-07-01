export interface PointsAccountItem {
  accountId: string;
  accountUuid: string;
  tenantId: string;
  organizationId?: string;
  ownerUserId: string;
  availablePoints: string;
  frozenPoints: string;
  pendingPoints: string;
  totalPoints: string;
  activeLotCount: string;
  expiringPoints: string;
  status: string;
  version: string;
}
