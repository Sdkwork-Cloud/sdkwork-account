import type { PageInfo } from './page-info';
import type { PointsLotAllocationItem } from './points-lot-allocation-item';

export interface PointsLotAllocationListData {
  items: PointsLotAllocationItem[];
  pageInfo: PageInfo;
}
