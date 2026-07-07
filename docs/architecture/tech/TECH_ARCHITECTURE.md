# Account And Token Bank Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-08
Specs: `API_SPEC.md`, `ARCHITECTURE_DECISION_SPEC.md`, `DATABASE_SPEC.md`, `DOCUMENTATION_SPEC.md`, `SDK_SPEC.md`, `SECURITY_SPEC.md`, `PAGINATION_SPEC.md`

## 1. Architecture Overview

`sdkwork-account` is the commerce account and Token Bank ledger capability. It owns the durable financial truth for cash, points, and Token Bank balances. It provides account persistence, double-entry journal writes, append-only ledger entries, holds, transfers, exchange-rate snapshots, AI settlement snapshots, billing history projections, HTTP route crates, generated SDK families, and the PC wallet surface.

Token Bank is a single concept in this repository:

```text
Token Bank product capability = token_bank account asset = token_bank API namespace
```

There is no second AI account asset beside `token_bank`. Raw provider usage, including LLM input/output tokens, remains metering data outside the account ledger.

Account does not create orders, execute payments, price models, collect raw usage, or run AI workloads. Those responsibilities remain in order, payment, pricing, metering, and AI runtime capabilities.

## 2. Technology Choices

| Concern | Choice |
| --- | --- |
| Persistence | SQL L3 schema under `database/`, with Postgres as production target and SQLite for local/test parity. |
| Amount model | Integer smallest units in SQL `BIGINT`; no floating-point or scale-fixed decimal account balances. |
| Domain core | Rust domain and ports in `crates/sdkwork-account-service/`. |
| Repository | `sqlx` repositories in `crates/sdkwork-account-repository-sqlx/`. |
| HTTP routes | Rust app-api and backend-api route crates under `crates/sdkwork-routes-account-*-api/`. |
| Gateway | `crates/sdkwork-account-gateway-assembly/` and `crates/sdkwork-account-standalone-gateway/`. |
| SDKs | Generated TypeScript SDK families under `sdks/sdkwork-account-app-sdk/` and `sdks/sdkwork-account-backend-sdk/`. |
| PC client | `apps/sdkwork-account-pc/` consuming composed SDK facades through account service packages. |
| API profile | SDKWork v3 envelopes, `ProblemDetail`, standard pagination, and generated SDK resource methods. |

## 3. System Boundaries And Modules

```text
Order
  owns purchase orders and fulfillment orchestration
  calls account when payment has succeeded

Payment
  owns payment intents, provider webhooks, refunds, payouts
  never writes account ledger directly

Pricing
  owns raw-usage to Token Bank amount conversion
  passes pricingSnapshotId and final Token Bank amount to account settlement

Metering
  owns raw usage snapshots
  passes usageSnapshotId to pricing and account settlement evidence

AI Runtime
  owns LLM, image, video, Agent, workflow, plugin, and model execution
  asks account to hold, settle, or release Token Bank balance

Account / Token Bank
  owns accounts, balances, journal, ledger, holds, transfers, exchange snapshots,
  settlement snapshots, billing projections, idempotency, outbox, and reconciliation
```

Dependency direction:

```text
order/payment/pricing/metering/ai-runtime --> account backend-api
account app-api --> users and frontend read models
account repository --> account-owned tables only
```

Account must not import order, payment, pricing, metering, or AI runtime repositories or read their tables.

## 4. Directory And Package Layout

| Layer | Path |
| --- | --- |
| Product PRD | `docs/product/prd/PRD.md` |
| Technical architecture | `docs/architecture/tech/TECH_ARCHITECTURE.md` |
| Local specs | `specs/` |
| API contracts | `apis/app-api/account/`, `apis/backend-api/account/` |
| Database contract | `database/contract/schema.yaml` |
| Database baselines | `database/ddl/baseline/postgres/`, `database/ddl/baseline/sqlite/` |
| Domain and ports | `crates/sdkwork-account-service/` |
| Repository | `crates/sdkwork-account-repository-sqlx/` |
| Route crates | `crates/sdkwork-routes-account-app-api/`, `crates/sdkwork-routes-account-backend-api/` |
| Gateway | `crates/sdkwork-account-gateway-assembly/`, `crates/sdkwork-account-standalone-gateway/` |
| SDK families | `sdks/sdkwork-account-app-sdk/`, `sdks/sdkwork-account-backend-sdk/` |
| PC app | `apps/sdkwork-account-pc/` |
| Shared account service facade | `apps/sdkwork-account-common/packages/sdkwork-account-service/` |

## 5. Data Ownership

### Account Taxonomy

| Account | `asset_code` | `currency_code` | Balance semantics |
| --- | --- | --- | --- |
| Cash | `cash` | ISO fiat code, for example `CNY` or `USD` | Fiat minor units used for account display, refunds, settlement display, or withdrawal policy. |
| Points | `points` | `POINT` | Traditional integer points with optional lot expiry and allocation audit. |
| Token Bank | `token_bank` | `TOKEN_BANK` | AI account balance stored in integer Token Bank units. |

Rules:

- `token_bank` is the only AI account asset code.
- Alternate AI account asset codes are forbidden in authored account contracts.
- Raw model tokens are usage metrics, not ledger assets.

### Account Owners

| `owner_type` | Purpose |
| --- | --- |
| `USER` | Individual wallet and Token Bank balance. |
| `ORG` | Enterprise account and budget pool. |
| `PROJECT` | Project/workspace budget account. |
| `SERVICE` | Model, plugin, application, Agent, workflow, or provider settlement account. |
| `SYSTEM` | Platform reserve, grant, burn, suspense, and reconciliation accounts. |

### Account Purposes

| `account_purpose` | Purpose |
| --- | --- |
| `GENERAL` | Normal spendable account. |
| `BUDGET` | Organization or project allocation pool. |
| `RESERVE` | Platform reserve and issuance account. |
| `SETTLEMENT` | Service/provider income settlement account. |
| `BURN` | Platform consumption sink. |
| `SUSPENSE` | Temporary exception holding account for reconciliation workflows. |

### Amount Representation

All money-like and account-balance values are integer based:

- Cash stores fiat minor units, such as cents or fen.
- Points store integer point units.
- Token Bank stores integer Token Bank units.
- API `int64` values are serialized as strings at HTTP JSON and TypeScript SDK boundaries.
- Floating-point arithmetic is forbidden in account, exchange, quote, settlement, reversal, and reconciliation paths.

## 6. Database Design

The database is in greenfield initialization state, so the baseline DDL is the source of truth. Post-GA changes must use migrations. The schema must remain aligned across:

- `database/contract/schema.yaml`
- `database/contract/table-registry.json`
- `database/ddl/baseline/postgres/0001_account_baseline.sql`
- `database/ddl/baseline/sqlite/0001_account_baseline.sql`

Physical table names use the registered `acct_` prefix for account/accounting-owned tables. The repository still belongs to the `commerce.account` capability; `acct_` is only the database bounded-context prefix. Cross-capability tables such as order-owned `commerce_order` keep their owning capability prefix and must not be read or written by this repository.

### Core Tables

| Table | Role |
| --- | --- |
| `acct_account` | Account identity, owner, asset, purpose, and available/frozen/pending balances. |
| `acct_journal` | Atomic accounting transaction header. |
| `acct_journal_line` | Double-entry journal lines. |
| `acct_ledger_entry` | Append-only user-visible and operator-visible ledger. |
| `acct_hold` | Available-to-frozen reservation and settlement lifecycle. |
| `acct_transfer` | Same-asset account transfer record. |
| `acct_points_lot` | Points lot inventory and expiry. |
| `acct_points_lot_allocation` | Points lot debit allocation audit. |
| `acct_token_bank_exchange_rate` | Published fiat-to-Token-Bank rate configuration. |
| `acct_token_bank_exchange_quote` | Purchase quote generated from a rate. |
| `acct_token_bank_exchange_snapshot` | Immutable exchange snapshot attached to purchase fulfillment and ledger entries. |
| `acct_token_bank_settlement_snapshot` | Immutable AI spending/income evidence for hold settlement, direct debit, service income, and burn routing. |
| `acct_idempotency_record` | Command replay, conflict, and in-flight lock state. |
| `acct_outbox_event` | Transactional domain events. |
| `acct_billing_history` | User-visible account and Token Bank billing projection. |

### Standard Field Rules

- Runtime business ids use application-generated Snowflake `BIGINT`; SQLite must not use `INTEGER PRIMARY KEY` rowid allocation.
- Public ids use `uuid VARCHAR(64)` or `TEXT` in SQLite.
- Account amount columns are `BIGINT NOT NULL DEFAULT 0`.
- Ledger and snapshot amounts are `BIGINT NOT NULL` with non-negative checks for absolute movement amounts.
- Status fields that cross APIs use stable string enums; internal established status fields may use `INTEGER` only when already owned by this capability.
- JSON is `JSONB` in Postgres and `TEXT` in SQLite with application-level schema validation.
- Cross-capability references are ids or external business numbers only; no cross-repository foreign keys.
- High-frequency query fields such as tenant, owner, account, asset, status, created time, business number, and idempotency key must be independent columns, not JSON-only fields.

### Account Constraints

`acct_account` must enforce:

- Unique account boundary: `(tenant_id, organization_id, owner_type, owner_id, asset_code, currency_code, account_purpose)`.
- Valid account assets: `cash`, `points`, `token_bank`.
- Currency consistency:
  - `cash` requires a fiat `currency_code`.
  - `points` requires `POINT`.
  - `token_bank` requires `TOKEN_BANK`.
- Non-negative `available_amount`, `frozen_amount`, and `pending_amount`.
- Optimistic concurrency through `version`.

### Exchange Tables

`acct_token_bank_exchange_rate` stores governed rate configuration:

| Field group | Required columns |
| --- | --- |
| Identity | `id`, `uuid`, `tenant_id`, `rate_no` |
| Direction | `from_asset_code = cash`, `from_currency_code`, `to_asset_code = token_bank`, `to_currency_code = TOKEN_BANK` |
| Calculation | `rate_numerator`, `rate_denominator`, `rounding_mode` |
| Scope | `tenant_scope`, `channel`, `effective_from`, `effective_to`, `status` |
| Governance | `published_by`, `published_at`, `retired_at`, `version`, `created_at`, `updated_at` |

`acct_token_bank_exchange_quote` stores a short-lived purchase quote. It records the exact fiat amount, Token Bank amount, rate id, owner, account, expiry, status, idempotency key, and trace id.

`acct_token_bank_exchange_snapshot` stores immutable purchase fulfillment evidence. It copies rate fields into the snapshot so later rate changes cannot alter historical ledger, billing, or reconciliation facts.

### AI Settlement Table

`acct_token_bank_settlement_snapshot` stores one immutable settlement evidence row for AI spending and service income:

| Field group | Required columns |
| --- | --- |
| Identity | `id`, `uuid`, `tenant_id`, `settlement_no` |
| Scope | `organization_id`, `owner_type`, `owner_id`, `account_id`, `service_account_id` |
| AI references | `job_id`, `application_id`, `model_id`, `workflow_id`, `plugin_id`, `usage_snapshot_id`, `pricing_snapshot_id` |
| Amounts | `estimated_amount`, `settled_amount`, `released_amount`, `service_income_amount`, `platform_burn_amount` |
| Ledger references | `hold_id`, `journal_id`, `debit_ledger_id`, `credit_ledger_id` |
| Governance | `settlement_mode`, `status`, `trace_id`, `created_at` |

Account stores references and final amounts. It does not store raw provider payloads or pricing formulas.

### Index Strategy

Required indexes:

- `acct_account`: tenant/owner/asset lookup and unique account boundary.
- `acct_ledger_entry`: tenant/account/created, tenant/business_no, tenant/request_no, tenant/source.
- `acct_hold`: tenant/account/status, tenant/source, tenant/expires/status.
- `acct_transfer`: tenant/from/created and tenant/to/created.
- `acct_points_lot`: tenant/account/expires for FEFO allocation.
- `acct_token_bank_exchange_rate`: tenant/from_currency/channel/status/effective time.
- `acct_token_bank_exchange_quote`: tenant/owner/status/created and tenant/quote_no.
- `acct_token_bank_exchange_snapshot`: tenant/order/payment and tenant/account/created.
- `acct_token_bank_settlement_snapshot`: tenant/account/created, tenant/job, tenant/service_account/created.
- `acct_billing_history`: tenant/owner/occurred and tenant/source.
- `acct_outbox_event`: status/next_retry_at.

List APIs must page at SQL level using these indexes and must not load unbounded rows into process memory.

## 7. Token Bank Exchange Architecture

Exchange rates convert fiat minor units to Token Bank units.

Required rate fields:

| Field | Meaning |
| --- | --- |
| `rateId` | Stable rate identifier. |
| `fromAssetCode` | `cash`. |
| `fromCurrencyCode` | `CNY`, `USD`, or future ISO currency. |
| `toAssetCode` | `token_bank`. |
| `toCurrencyCode` | `TOKEN_BANK`. |
| `rateNumerator` | Token Bank units produced. |
| `rateDenominator` | Fiat minor units consumed. |
| `roundingMode` | Default `floor`. |
| `channel` | Web, enterprise, admin, API, or campaign channel. |
| `tenantScope` | Global or tenant-specific scope. |
| `effectiveFrom` / `effectiveTo` | Versioned validity window. |
| `status` | `draft`, `active`, `retired`. |
| `version` | Optimistic lock version. |

Quote creation uses one active rate and returns a deterministic result. Purchase fulfillment stores an immutable exchange snapshot with rate fields, fiat amount, Token Bank amount, order id, payment id, and trace id.

## 8. AI Consumption Accounting

AI consumption is a three-stage accounting workflow:

1. **Budget quote**
   - Pricing estimates the maximum Token Bank cost from requested model/task parameters.
   - Account does not calculate provider prices.

2. **Hold**
   - Token Bank creates a hold for the estimated budget.
   - Available decreases and frozen increases.
   - Insufficient balance rejects task start.

3. **Settlement**
   - Metering records raw usage.
   - Pricing converts raw usage to a final Token Bank amount.
   - Account records a settlement snapshot.
   - Account settles the hold, debits the consuming account, credits a service settlement or burn account when required, and releases unused remainder.

Failed or canceled tasks release the hold. Completed-but-wrong tasks use reversal ledger entries; historical records are never updated in place.

## 9. API, SDK, And Data Ownership

### App API

Standard app-api resource groups:

- `/app/v3/api/wallet/accounts/cash`
- `/app/v3/api/wallet/accounts/points`
- `/app/v3/api/token_bank/overview`
- `/app/v3/api/token_bank/account`
- `/app/v3/api/token_bank/ledger_entries`
- `/app/v3/api/token_bank/holds`
- `/app/v3/api/token_bank/exchange_rates/current`
- `/app/v3/api/token_bank/purchase_quotes`
- `/app/v3/api/token_bank/settlements`
- `/app/v3/api/billing/history`
- `/app/v3/api/accounts/current/summary`

The account summary response uses explicit points fields for traditional points statistics: `availablePoints`, `monthlyPointsConsumed`, and `consumptionByService[].pointsConsumed`. Token Bank read models stay under `/app/v3/api/token_bank/*`.

### Backend API

Standard backend-api resource groups:

- `/backend/v3/api/wallet/adjustments/cash`
- `/backend/v3/api/wallet/adjustments/points`
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

All APIs use `SdkWorkApiResponse` success envelopes and `ProblemDetail` errors. List APIs must be paginated at the store. SDK consumers must use composed SDK packages, not generator transport names.

### SDK Method Shape

SDK facades must read as business resources:

```ts
account.wallet.points.account.retrieve();
account.wallet.points.ledgerEntries.list(params);
account.tokenBank.overview.retrieve();
account.tokenBank.account.retrieve();
account.tokenBank.ledgerEntries.list(params);
account.tokenBank.holds.create(body);
account.tokenBank.holds.settle(holdId, body);
account.tokenBank.holds.release(holdId, body);
account.tokenBank.transfers.create(body);
account.tokenBank.reversals.create(body);
account.tokenBank.exchangeRates.current.retrieve(params);
account.tokenBank.purchaseQuotes.create(body);
account.tokenBank.settlements.list(params);
```

## 10. Security, Privacy, And Observability

- Protected app-api and backend-api routes require SDKWork dual-token context.
- Backend commands validate tenant, owner, organization/project scope, and caller permission through `WebRequestContext`.
- Idempotency is required for every command that changes balance, rate state, quote state, settlement state, or reversal state.
- Rate publishing requires backend-admin authority and audit metadata.
- Exchange-rate snapshots, settlement snapshots, and ledger entries are immutable audit records.
- Sensitive payment details must be referenced through order/payment ids, not copied into account logs.
- Raw provider payloads must not be copied into account tables.
- Observability emits structured logs, metrics, traces, and outbox events for every command outcome.

Key metrics:

- Balance mutation success/failure count by operation and asset.
- Idempotency replay/conflict/locked counts.
- Hold created/settled/released amounts by asset.
- Token Bank purchase volume by fiat currency and channel.
- Token Bank spending and income volume by model/application/service/project.
- Reconciliation mismatch counts.
- Outbox pending lag.

## 11. Deployment And Runtime Topology

Standalone and cloud deployments expose the same API contracts and SDK method shapes.

```text
PC / H5 / Mobile
  -> account app-api SDK
  -> wallet and Token Bank read models

Backend services
  -> account backend-api SDK
  -> adjustments, holds, transfers, exchange rates, reconciliation

Order fulfillment
  -> account backend-api
  -> Token Bank credit after fiat payment success

AI runtime
  -> pricing/metering
  -> account backend-api hold/settle/release
```

Runtime configuration separates:

- Account API base URL.
- Payment checkout/payout base URL.
- Pricing/metering service base URL.
- Token Bank default fiat currencies.
- Exchange-rate admin feature flags.

## 12. Architecture Decision Index

Active architecture decisions:

- Adopt `token_bank` as the only AI account asset code and API namespace.
- Forbid alternate AI account asset naming.
- Store account amounts as integer smallest units in SQL `BIGINT`.
- Model fiat-to-Token-Bank exchange through versioned rates, quotes, and immutable snapshots.
- Keep pricing/metering outside account and pass snapshot references into settlement.
- Support multi-owner Token Bank accounts: user, org, project, service, and system.
- Use semantic Token Bank API resources instead of generic wallet-only routes for AI workflows.

## 13. Verification

Required verification after implementation changes:

```powershell
cargo test --workspace
pnpm install
pnpm verify
node ..\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace .
node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .
node ..\sdkwork-specs\tools\check-pagination.mjs --workspace .
node ..\sdkwork-specs\tools\check-app-sdk-consumer-imports.mjs --workspace .
```

Documentation and schema design changes should at minimum run:

```powershell
node ..\sdkwork-specs\tools\check-repository-docs-standard.mjs --root .
node -e "for (const f of ['specs/component.spec.json','specs/commerce-integration.spec.json','database/contract/table-registry.json','database/database.manifest.json']) { JSON.parse(require('fs').readFileSync(f,'utf8')); console.log(f + ' OK'); }"
```

## 14. Related Docs

- Product PRD: `docs/product/prd/PRD.md`
- Token Bank account spec: `specs/TOKEN_BANK_ACCOUNT_SPEC.md`
- Commerce boundary spec: `specs/COMMERCE_BOUNDARY_SPEC.md`
- Integration machine contract: `specs/commerce-integration.spec.json`
- Database contract: `database/contract/schema.yaml`
- Global standards: `sdkwork-specs/API_SPEC.md`, `DATABASE_SPEC.md`, `SDK_SPEC.md`, `SECURITY_SPEC.md`, `PAGINATION_SPEC.md`
