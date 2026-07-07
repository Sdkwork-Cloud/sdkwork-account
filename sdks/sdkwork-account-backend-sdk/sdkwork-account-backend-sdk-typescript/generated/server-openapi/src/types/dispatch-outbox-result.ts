import type { OutboxDispatchItem } from './outbox-dispatch-item';

export interface DispatchOutboxResult {
  accepted: boolean;
  dispatchedCount: string;
  pendingLag: string;
  items: OutboxDispatchItem[];
}
