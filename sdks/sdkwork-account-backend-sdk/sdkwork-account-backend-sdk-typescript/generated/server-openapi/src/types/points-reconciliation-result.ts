import type { PointsReconciliationMismatchItem } from './points-reconciliation-mismatch-item';

export interface PointsReconciliationResult {
  checkedAccountCount: string;
  mismatchCount: string;
  mismatches: PointsReconciliationMismatchItem[];
}
