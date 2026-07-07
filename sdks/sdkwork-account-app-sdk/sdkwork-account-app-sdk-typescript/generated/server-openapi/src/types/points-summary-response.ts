import type { PointsSummaryData } from './points-summary-data';

export interface PointsSummaryResponse {
  code: 0;
  data: PointsSummaryData;
  traceId: string;
}
