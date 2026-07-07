# Token Bank Account Spec

Status: active
Owner: SDKWork maintainers
Capability: `commerce.account`
Updated: 2026-07-08
Authority: `sdkwork-specs/API_SPEC.md`, `sdkwork-specs/DATABASE_SPEC.md`, `sdkwork-specs/DOMAIN_SPEC.md`, `sdkwork-specs/SECURITY_SPEC.md`

## 1. Purpose

This local spec defines the greenfield account taxonomy and Token Bank rules for `sdkwork-account`.

Token Bank is the single SDKWork account concept for AI-era consumption, income, spending, reservation, settlement, transfer, exchange, and reconciliation. It does not contain any secondary AI account asset class.

## 2. Required Terminology

| Concept | Required term | Technical identifier |
| --- | --- | --- |
| AI account capability | Token Bank | `token_bank` |
| AI account asset | Token Bank | `token_bank` |
| Token Bank currency code | Token Bank | `TOKEN_BANK` |
| Traditional loyalty asset | Points | `points` |
| Fiat balance asset | Cash | `cash` |
| Raw provider usage | Raw usage | Not an account asset |

Forbidden terms for account assets:

- `token`
- `compute_token`
- `compute_credit`
- `llm_token`
- `model_token`
- `auth_token`
- `blockchain_token`

Rules:

- `token_bank` is the only technical identifier for AI account balance.
- `token_bank` is allowed as API namespace, SDK resource namespace, operation prefix, database `asset_code`, and table-name qualifier.
- Naked `token` may appear only in authentication/security contexts that are clearly outside account asset naming.
- Raw model usage must be called raw usage, metering usage, or provider usage. It must not be called account token.

## 3. Account Classification

| Asset code | Currency code | Product meaning | Owner-visible name | Withdrawal | Expiry | Primary source | Primary sink |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cash` | ISO fiat code | Fiat account balance | Cash account | Product-policy dependent | No default expiry | Refunds, settlement, manual adjustment | Withdrawal, payment, reversal |
| `points` | `POINT` | Traditional points and rewards | Points account | No | Yes, through points lots | Rewards, promotions, compensation | Redemption, expiry, reversal |
| `token_bank` | `TOKEN_BANK` | AI consumption and income account | Token Bank | No direct fiat withdrawal by default | Optional policy, not lot-based by default | Fiat exchange, grants, org allocation, service income | AI holds, AI settlement, transfers, burn, reversal |

Rules:

- Points and Token Bank must never be merged in one DTO field, UI label, exchange table, ledger business type, or reconciliation report.
- Raw LLM input/output tokens are metering inputs and must be converted by pricing before account settlement.
- Image, video, Agent, workflow, model-service, and plugin usage must also be converted to Token Bank amounts before account settlement.

## 4. Account Owners And Purposes

Allowed `owner_type` values:

| Owner type | Description |
| --- | --- |
| `USER` | Individual user account. |
| `ORG` | Tenant or organization budget account. |
| `PROJECT` | Project/workspace budget account. |
| `SERVICE` | Model, plugin, application, Agent, workflow, or provider settlement account. |
| `SYSTEM` | Platform reserve, burn, grant, suspense, and reconciliation accounts. |

Allowed `account_purpose` values:

| Purpose | Description |
| --- | --- |
| `GENERAL` | Normal available/frozen/pending account. |
| `BUDGET` | Organization or project allocation pool. |
| `RESERVE` | Token Bank reserve or issuance source. |
| `SETTLEMENT` | Service/provider income settlement account. |
| `BURN` | Consumption sink when no service settlement is due. |
| `SUSPENSE` | Temporary exception and reconciliation account. |

## 5. Amount Rules

All account amounts must use integer smallest units.

| Asset | Smallest unit rule |
| --- | --- |
| `cash` | Fiat minor unit such as fen or cent. |
| `points` | One point unit. |
| `token_bank` | One Token Bank unit. |

Rules:

- SQL account amount columns must use `BIGINT` in Postgres and an equivalent non-rowid integer type in SQLite.
- Floating-point accounting is forbidden.
- Decimal display formatting belongs at the presentation layer.
- Exchange, pricing, settlement, reversal, and reconciliation must use integer arithmetic.
- Rounding must be explicit and deterministic. Default mode: `floor`.
- API `int64` amounts must be serialized as strings at HTTP JSON and TypeScript SDK boundaries.

## 6. Database Rules

Physical database prefix rules:

- Account-owned physical tables must use the registered `acct_` prefix.
- `acct_` means the account/accounting bounded context inside the commerce domain.
- The `commerce.account` capability identity, API package names, and cross-capability boundary names remain unchanged.
- Account-owned greenfield tables must not use the broad `commerce_` prefix.

Required Token Bank tables:

| Table | Requirement |
| --- | --- |
| `acct_account` | Must support `asset_code = token_bank` and `currency_code = TOKEN_BANK`. |
| `acct_ledger_entry` | Must support append-only Token Bank credit, debit, hold, settlement, income, burn, and reversal entries. |
| `acct_hold` | Must support AI budget reservation and final settlement or release. |
| `acct_transfer` | Must support same-asset transfer between Token Bank owner accounts. |
| `acct_token_bank_exchange_rate` | Must store governed fiat-to-Token-Bank exchange rates. |
| `acct_token_bank_exchange_quote` | Must store short-lived purchase quotes. |
| `acct_token_bank_exchange_snapshot` | Must store immutable purchase fulfillment snapshots. |
| `acct_token_bank_settlement_snapshot` | Must store immutable AI spending and service income settlement evidence. |
| `acct_billing_history` | Must project Token Bank income, spending, hold, release, purchase, transfer, and reversal history. |

Database constraints:

- `acct_account.asset_code` must be one of `cash`, `points`, or `token_bank`.
- `token_bank` account rows must use `currency_code = TOKEN_BANK`.
- Account balances and movement amounts must be non-negative integer values.
- Ledger rows must be append-only; reversals must append new rows and reference `reversed_ledger_id`.
- Exchange snapshots and settlement snapshots are immutable after creation.
- Cross-capability references use ids or business numbers; account tables must not own order, payment, pricing, metering, or AI runtime tables.

## 7. Exchange Rate Rules

Token Bank exchange rates convert fiat cash to Token Bank.

Required fields:

| Field | Requirement |
| --- | --- |
| `fromAssetCode` | Must be `cash`. |
| `fromCurrencyCode` | Must be an approved fiat currency code such as `CNY` or `USD`. |
| `toAssetCode` | Must be `token_bank`. |
| `toCurrencyCode` | Must be `TOKEN_BANK`. |
| `rateNumerator` | Token Bank units produced. |
| `rateDenominator` | Fiat minor units. |
| `roundingMode` | Required; default `floor`. |
| `tenantScope` | Global or tenant-specific. |
| `channel` | Web, enterprise, API, admin, campaign, or another governed channel. |
| `effectiveFrom` / `effectiveTo` | Non-overlapping active window per scope/channel/currency. |
| `status` | `draft`, `active`, or `retired`. |

Quote and snapshot rules:

- A purchase quote must reference exactly one active exchange rate.
- A completed purchase must store an immutable exchange snapshot.
- Ledger entries for purchase credit must reference the exchange snapshot.
- Historical snapshots must not change after rate updates.
- Exchange-rate publishing must be audited and permission-gated.

## 8. Token Bank Ledger Operations

Required command families:

| Operation | Purpose |
| --- | --- |
| `token_bank.issue` | Move balance from reserve or create governed issuance. |
| `token_bank.purchase_credit` | Credit after paid fiat purchase with exchange snapshot. |
| `token_bank.grant` | Platform, subscription, compensation, or campaign grant. |
| `token_bank.transfer` | Move between user/org/project/service/system accounts. |
| `token_bank.hold` | Reserve estimated AI budget. |
| `token_bank.settle` | Consume held balance after final metered usage. |
| `token_bank.release` | Release unused or failed-task held balance. |
| `token_bank.debit` | Direct spending when no pre-hold is required. |
| `token_bank.service_income` | Credit provider/service settlement account. |
| `token_bank.burn` | Route consumption to platform burn account. |
| `token_bank.reverse` | Append reverse entry for a previous movement. |

Every mutation must:

- Require an idempotency key.
- Execute in one transaction.
- Update balances with version guards.
- Append journal and ledger rows.
- Record trace id and actor context.
- Emit outbox event when downstream consumers need notification.
- Never update or delete historical ledger rows.

## 9. AI Consumption References

Token Bank settlement entries must record available AI execution evidence references without owning AI runtime data.

Standard evidence references:

| Field | Meaning |
| --- | --- |
| `jobId` | AI task/job identifier. |
| `applicationId` | Calling application or product surface. |
| `modelId` | Model or service identifier. |
| `workflowId` | Workflow identifier when relevant. |
| `pluginId` | Plugin identifier when relevant. |
| `usageSnapshotId` | Metering-owned raw usage snapshot. |
| `pricingSnapshotId` | Pricing-owned conversion snapshot. |
| `quoteId` | Optional pre-run budget quote. |
| `holdId` | Token Bank hold settled or released. |

Account stores references and audit metadata; it must not copy raw provider payloads or own pricing formulas.

## 10. API Naming Rules

Token Bank API paths must use `token_bank`. Alternate AI account asset path names are forbidden.

Standard app-api resources:

- `/app/v3/api/token_bank/overview`
- `/app/v3/api/token_bank/account`
- `/app/v3/api/token_bank/ledger_entries`
- `/app/v3/api/token_bank/holds`
- `/app/v3/api/token_bank/exchange_rates/current`
- `/app/v3/api/token_bank/purchase_quotes`
- `/app/v3/api/token_bank/settlements`

Standard backend-api resources:

- `/backend/v3/api/token_bank/credits`
- `/backend/v3/api/token_bank/debits`
- `/backend/v3/api/token_bank/grants`
- `/backend/v3/api/token_bank/transfers`
- `/backend/v3/api/token_bank/holds`
- `/backend/v3/api/token_bank/holds/{holdId}/settle`
- `/backend/v3/api/token_bank/holds/{holdId}/release`
- `/backend/v3/api/token_bank/reversals`
- `/backend/v3/api/token_bank/exchange_rates`
- `/backend/v3/api/token_bank/exchange_rates/{rateId}/publish`
- `/backend/v3/api/token_bank/reconciliation/*`

SDK resource examples:

```ts
client.tokenBank.overview.retrieve();
client.tokenBank.account.retrieve();
client.tokenBank.ledgerEntries.list(params);
client.tokenBank.holds.create(body);
client.tokenBank.holds.settle(holdId, body);
client.tokenBank.holds.release(holdId, body);
client.tokenBank.transfers.create(body);
client.tokenBank.exchangeRates.current.retrieve(params);
client.tokenBank.purchaseQuotes.create(body);
client.tokenBank.settlements.list(params);
```

## 11. Boundary Rules

Account / Token Bank owns:

- Account identity and balances.
- Journal and ledger truth.
- Holds, transfers, reversals, and reconciliation.
- Exchange-rate configuration, quotes, and immutable exchange snapshots.
- AI settlement snapshots for spending, release, service income, and burn evidence.
- Billing history projection and outbox events.

Account / Token Bank does not own:

- Order lifecycle.
- Payment execution.
- Provider webhooks.
- Model pricing formulas.
- Raw AI usage collection.
- AI task execution.
- Provider cost, margin policy, and subscription catalog.

## 12. Verification

Before declaring Token Bank implementation complete, run:

```powershell
cargo test --workspace
pnpm install
pnpm verify
node ..\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace .
node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .
node ..\sdkwork-specs\tools\check-pagination.mjs --workspace .
node ..\sdkwork-specs\tools\check-app-sdk-consumer-imports.mjs --workspace .
```

Before declaring Token Bank documentation or schema design aligned, scan authored docs, specs, database contracts, APIs, SDK facades, apps, and crates for old operational paths, old SDK resource names, and old Token Bank table prefixes. Forbidden terms may appear only in explicit forbidden-term lists or explanatory warnings.
