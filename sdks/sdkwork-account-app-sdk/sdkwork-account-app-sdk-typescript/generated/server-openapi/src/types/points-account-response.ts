import type { PointsAccountItem } from './points-account-item';

export interface PointsAccountResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
