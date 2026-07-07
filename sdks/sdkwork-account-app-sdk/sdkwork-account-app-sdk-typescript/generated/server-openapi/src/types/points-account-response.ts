import type { PointsAccountData } from './points-account-data';

export interface PointsAccountResponse {
  code: 0;
  data: PointsAccountData;
  traceId: string;
}
