import type { PageInfo } from './page-info';
import type { PointsLotItem } from './points-lot-item';

export interface PointsLotListData {
  items: PointsLotItem[];
  pageInfo: PageInfo;
}
