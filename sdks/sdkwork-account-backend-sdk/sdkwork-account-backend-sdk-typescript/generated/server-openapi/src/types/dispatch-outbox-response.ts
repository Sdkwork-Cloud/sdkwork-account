import type { OutboxDispatchItem } from './outbox-dispatch-item';

export interface DispatchOutboxResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
