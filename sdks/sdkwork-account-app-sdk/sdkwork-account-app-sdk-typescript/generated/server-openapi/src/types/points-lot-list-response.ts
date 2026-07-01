import type { PageInfo } from './page-info';
import type { PointsLotItem } from './points-lot-item';

export interface PointsLotListResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
