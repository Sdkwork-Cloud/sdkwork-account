import type { PointsSummaryItem } from './points-summary-item';

export interface PointsSummaryResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
