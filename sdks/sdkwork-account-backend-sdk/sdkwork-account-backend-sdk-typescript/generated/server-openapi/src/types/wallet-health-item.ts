export interface WalletHealthItem {
  status: 'ready' | 'degraded';
  database: 'up' | 'down';
  outboxPendingLag: string;
}
