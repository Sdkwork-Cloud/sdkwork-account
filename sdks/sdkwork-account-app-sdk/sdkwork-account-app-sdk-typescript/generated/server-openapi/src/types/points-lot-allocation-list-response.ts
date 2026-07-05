import type { PageInfo } from './page-info';
import type { PointsLotAllocationItem } from './points-lot-allocation-item';

export interface PointsLotAllocationListResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
