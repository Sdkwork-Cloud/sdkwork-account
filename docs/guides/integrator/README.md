# Integrator Guide

How to consume the account, wallet, and Token Bank capability from app clients and backend integrators.

## Capability Boundaries

| Repository | Role |
| --- | --- |
| `sdkwork-account` | Ledger truth source: cash, points, Token Bank balances, ledger, holds, transfers, billing projection, exchange snapshots, settlement snapshots. |
| `sdkwork-order` | Unified order center, purchase order lifecycle, checkout orchestration, `orders.pay`. |
| `sdkwork-payment` | Payment intent, provider webhooks, refunds, payout on an existing `orderId`. |
| Pricing/metering capability | Raw usage snapshots and conversion from raw AI usage to Token Bank amount. |
| AI runtime capability | LLM, image, video, Agent, workflow, plugin, and model-service execution. |

Account never creates orders, executes payment providers, prices models, meters raw usage, or runs AI workloads. It records account truth after those capabilities call governed account APIs.

## Account Taxonomy

| Account | Technical identifier | Currency code |
| --- | --- | --- |
| Cash | `cash` | `CNY`, `USD`, or another governed fiat code |
| Points | `points` | `POINT` |
| Token Bank | `token_bank` | `TOKEN_BANK` |

Integrations must not introduce alternate AI account asset names; account contracts expose only `token_bank` for Token Bank balance.

Raw LLM tokens, image count, video seconds, GPU seconds, tool calls, workflow steps, and plugin calls are raw usage. They are not account assets.

## API Authorities

| Surface | Prefix | OpenAPI |
| --- | --- | --- |
| App wallet read models | `/app/v3/api/wallet`, `/app/v3/api/billing`, `/app/v3/api/accounts` | `apis/app-api/account/account-app-api.openapi.json` |
| App Token Bank read models | `/app/v3/api/token_bank` | `apis/app-api/account/account-app-api.openapi.json` |
| Backend wallet commands | `/backend/v3/api/wallet` | `apis/backend-api/account/account-backend-api.openapi.json` |
| Backend Token Bank commands | `/backend/v3/api/token_bank` | `apis/backend-api/account/account-backend-api.openapi.json` |

Account does not own recharge or payment execution routes. Integrations call order checkout surfaces directly, and Account PC purchase and withdraw buttons delegate to checkout routes through `onNavigate`.

## Response Envelope

Success: `{ "code": 0, "data": <payload>, "traceId": "<uuid>" }`

Errors: HTTP 4xx/5xx `application/problem+json` with numeric `code` and `traceId`.

Single resources use `data.item`. Lists use `data.items` + `data.pageInfo`.

List pagination:

- Default `page_size` is `20`; max is `200` unless a documented exception exists.
- Offset mode uses `page` + `page_size`.
- Cursor mode uses `cursor` + `page_size`.
- Interactive clients must request one page at a time. Do not download full ledgers and slice locally.

## App Read APIs

Recommended app reads:

- `GET .../accounts/current/summary`
- `GET .../wallet/accounts/cash`
- `GET .../wallet/accounts/points`
- `GET .../wallet/ledger_entries`
- `GET .../wallet/ledger_entries/cash`
- `GET .../wallet/ledger_entries/points`
- `GET .../wallet/points/lots`
- `GET .../wallet/points/summary`
- `GET .../wallet/ledger_entries/{ledgerEntryId}/allocations`
- `GET .../wallet/holds`
- `GET .../wallet/holds/{holdId}`
- `GET .../token_bank/overview`
- `GET .../token_bank/account`
- `GET .../token_bank/ledger_entries`
- `GET .../token_bank/holds`
- `GET .../token_bank/exchange_rates/current`
- `GET .../token_bank/settlements`
- `GET .../billing/history`

## Backend Command APIs

Wallet commands, typically called by order/payment or ops services:

- `POST .../wallet/adjustments`
- `POST .../wallet/adjustments/cash`
- `POST .../wallet/adjustments/points`
- `POST .../wallet/holds`
- `POST .../wallet/holds/{holdId}/settle`
- `POST .../wallet/holds/{holdId}/release`
- `POST .../wallet/holds/expire`
- `POST .../wallet/transfers`
- `POST .../wallet/points/lots/expire`
- `POST .../wallet/points/reconciliation`

Token Bank commands:

- `POST .../token_bank/credits`
- `POST .../token_bank/debits`
- `POST .../token_bank/grants`
- `POST .../token_bank/transfers`
- `POST .../token_bank/holds`
- `POST .../token_bank/holds/{holdId}/settle`
- `POST .../token_bank/holds/{holdId}/release`
- `POST .../token_bank/reversals`
- `POST .../token_bank/exchange_rates`
- `POST .../token_bank/exchange_rates/{rateId}/publish`
- `POST .../token_bank/reconciliation/*`

Every write command requires an idempotency key, trace context, tenant context, and permission-checked caller context.

## Token Bank Purchase Flow

```text
1. App requests Token Bank purchase quote from account app-api.
2. App creates an order through sdkwork-order.
3. User pays through sdkwork-payment checkout.
4. Order fulfillment calls account backend-api after payment success.
5. Account credits token_bank balance and writes:
   - commerce_account_journal
   - commerce_account_journal_line
   - commerce_account_ledger
   - commerce_token_bank_exchange_snapshot
   - commerce_billing_history
   - commerce_outbox_event when required
6. App refreshes wallet and Token Bank read models.
```

## AI Consumption Flow

```text
1. AI runtime asks pricing for estimated Token Bank cost.
2. AI runtime calls account backend-api to create a Token Bank hold.
3. Metering records raw usage.
4. Pricing converts raw usage to final Token Bank amount.
5. AI runtime or orchestrator calls account backend-api to settle or release the hold.
6. Account writes a settlement snapshot and ledger entries.
7. Optional service income credits a SERVICE settlement account.
```

Account stores references such as `jobId`, `usageSnapshotId`, and `pricingSnapshotId`. It does not store raw provider payloads or pricing formulas.

## Idempotency

- Write paths use `commerce_idempotency_record` scopes per operation.
- `COMPLETED` replays the stored outcome.
- Active `LOCKED` maps to HTTP `423`.
- Expired locks are reclaimed automatically.
- Hash mismatch maps to HTTP `409`.
- Concurrent duplicate inserts map to HTTP `423`, not HTTP `500`.

## TypeScript Consumption

| Layer | Package |
| --- | --- |
| Account composed facade | `@sdkwork/account-service` |
| Account app SDK | `@sdkwork/account-app-sdk` |
| Account backend SDK | `@sdkwork/account-backend-sdk` |
| Order purchase and checkout | integrator-owned order-compatible service port |

Account bootstrap:

```typescript
import { bootstrapSdkworkAccountPcSdk } from "@sdkwork/account-pc-core/sdk";

bootstrapSdkworkAccountPcSdk({
  baseUrl: "https://api.example.com",
  authToken: session.authToken,
});
```

Backend ledger integrator:

```typescript
import { bootstrapSdkworkAccountPcBackendSdk } from "@sdkwork/account-pc-core/sdk";

bootstrapSdkworkAccountPcBackendSdk({
  baseUrl: "https://api.example.com",
  accessToken: serviceAccount.accessToken,
});
```

PC wallet purchase/withdraw must not call account APIs for checkout. Pass `onNavigate` plus checkout paths to delegate to payment/order surfaces:

```typescript
<SdkworkWalletPage
  checkoutBasePath="/checkout"
  payoutBasePath="/payments/payout"
  onNavigate={(route) => router.push(route)}
  rechargeFlow="checkout"
  payoutFlow="checkout"
/>
```

After checkout completes, redirect back to the wallet route with `commerceRefresh=1`. `SdkworkWalletPage` strips the query param and refreshes balances when the user returns.

## Verification

```powershell
pnpm verify
node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .
```

See also `docs/architecture/tech/TECH_ARCHITECTURE.md`, `docs/product/prd/PRD.md`, and `specs/TOKEN_BANK_ACCOUNT_SPEC.md`.
