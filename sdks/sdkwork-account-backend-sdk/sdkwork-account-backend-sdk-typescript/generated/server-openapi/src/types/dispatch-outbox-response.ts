import type { DispatchOutboxData } from './dispatch-outbox-data';

export interface DispatchOutboxResponse {
  code: 0;
  data: DispatchOutboxData;
  traceId: string;
}
