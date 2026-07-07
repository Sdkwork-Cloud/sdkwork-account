export interface CreateTokenBankReversalRequest {
  tenantId: string;
  organizationId?: string;
  ownerUserId: string;
  accountId?: string;
  assetType: 'cash' | 'points' | 'token_bank';
  currencyCode?: string;
  direction: 'credit' | 'debit';
  amount: string;
  /** Lowercase snake_case ledger business type (validated by CommerceLedgerBusinessType). */
  businessType: string;
  transactionNo: string;
  requestNo: string;
  idempotencyKey: string;
  /** Optional points lot expiry for credit adjustments. */
  expiresAt?: string;
  /** Optional original ledger entry id when posting a compensating adjustment. */
  reversedLedgerId: string;
}
