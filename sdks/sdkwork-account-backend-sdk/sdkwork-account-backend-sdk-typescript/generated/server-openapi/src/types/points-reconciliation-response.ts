import type { PointsReconciliationData } from './points-reconciliation-data';

export interface PointsReconciliationResponse {
  code: 0;
  data: PointsReconciliationData;
  traceId: string;
}
