export interface OutboxDispatchItem {
  id: string;
  uuid: string;
  tenantId: string;
  aggregateType: string;
  aggregateId: string;
  eventType: string;
  eventVersion: number;
  eventKey: string;
  payload: string;
  createdAt: string;
}
