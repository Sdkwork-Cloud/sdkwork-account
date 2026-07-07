export interface CreateTokenBankLedgerMutationRequest {
  tenantId: string;
  organizationId?: string;
  ownerUserId: string;
  accountId?: string;
  currencyCode?: string;
  amount: string;
  /** Lowercase snake_case ledger business type (validated by CommerceLedgerBusinessType). */
  businessType: string;
  transactionNo: string;
  requestNo: string;
  idempotencyKey: string;
  /** Optional points lot expiry for credit adjustments. */
  expiresAt?: string;
  /** Optional original ledger entry id when posting a compensating adjustment. */
  reversedLedgerId?: string;
}
