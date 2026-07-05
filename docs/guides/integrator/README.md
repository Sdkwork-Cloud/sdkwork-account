# Integrator Guide

How to consume the account / wallet capability from app clients and backend integrators.

## Capability boundaries

| Repository | Role |
| --- | --- |
| `sdkwork-account` | Ledger truth source: balances, ledger, holds, transfers, billing projection |
| `sdkwork-order` | Unified order center, recharge packages, `recharges.orders.create`, `orders.pay` orchestration |
| `sdkwork-payment` | Payment intent, provider webhooks, refunds, payout (on existing `orderId`) |

Every recharge creates a **commerce order** in **sdkwork-order** (`subject=points_recharge`). Payment intent is created by **`orders.pay`** (payment repository). Account credits points only via backend adjustments after payment succeeds and order fulfillment saga runs.

## API authorities

| Surface | Prefix | OpenAPI |
| --- | --- | --- |
| App (read-only) | `/app/v3/api/wallet`, `/app/v3/api/billing`, `/app/v3/api/accounts` | `apis/app-api/account/account-app-api.openapi.json` |
| Backend (ledger writes) | `/backend/v3/api/wallet` | `apis/backend-api/account/account-backend-api.openapi.json` |

Deprecated `/app/v3/api/recharges/*` on payment forwards to order via `SDKWORK_ORDER_APP_API_ORIGIN` (default `http://127.0.0.1:18093`). Prefer calling order directly.  
Product checkout: `sdkwork-order` app-api (`checkout.sessions.*`, `orders.pay`).

## Response envelope

Success: `{ "code": 0, "data": <payload>, "traceId": "<uuid>" }`  
Errors: HTTP 4xx/5xx `application/problem+json` with numeric `code` and `traceId`.

Single resources use `data.item`. Lists use `data.items` + `data.pageInfo`.

List pagination (`PAGINATION_SPEC.md`):

- Default `pageSize` is **20** (max **200**).
- Offset mode: `page` + `pageSize`; response includes `totalItems`, `totalPages`, `hasMore`.
- Cursor mode: numeric `cursor` (row offset) or RFC3339 keyset cursor on ledger lists; response includes `nextCursor` + `hasMore`. Never combine cursor with SQL `OFFSET` on the same request.

Idempotency:

- Write paths use `commerce_idempotency_record` scopes per operation.
- `COMPLETED` → replay stored outcome; `LOCKED` → HTTP **423** (`42301`); `FAILED` → reclaim lock on retry; hash mismatch → HTTP **409**.

Account summary (`GET .../accounts/current/summary`):

- Wallet fields (`availableCredits`, monthly consumption, service breakdown) are computed from billing projection.
- Profile fields (`name`, `email`, `isVerified`, `tier`) are enriched from `IamAppContext` (`display_name`, `email`, `email_verified`, `standard_role_codes`) when the IAM session injector is active.

Health:

- `GET /backend/v3/api/wallet/health` returns `SdkWorkApiResponse` with database probe (`ready` / `degraded`), `database` (`up` / `down`), and `outboxPendingLag`.

Outbox relay:

- `POST /backend/v3/api/wallet/outbox/dispatch` — optional `batchSize` (default **100**, max **200**). Atomically marks pending `commerce_outbox_event` rows as `PUBLISHED` and returns payloads for cron/worker forwarding. Integrators should publish returned events to their bus and rely on consumer idempotency (`eventKey`).

## Asset model

Cash, points, and token are **separate account rows** (`asset_code`: `cash` | `points` | `token`). App routes expose asset-scoped read models:

- `GET .../wallet/accounts/cash` — cash balance DTO
- `GET .../wallet/accounts/points` — points balance + lot stats (`activeLotCount`, `expiringPoints`)
- `GET .../wallet/accounts/tokens` — token account row
- `GET .../wallet/ledger_entries/cash|points` — asset-filtered ledger
- `GET .../wallet/points/lots` — points lot list (FEFO consumption source)
- `GET .../wallet/points/summary` — one-screen points summary (`unsweptExpiredPoints`, month credit/debit, lot stats)
- `GET .../wallet/ledger_entries/{ledgerEntryId}/allocations` — lot allocation audit for a debit ledger entry (capped at max page size **200** per ledger entry)
- `GET .../wallet/holds` — hold list (optional `accountId`, `assetType`, `status`)
- `GET .../wallet/holds/{holdId}` — hold detail

Backend ledger writes (typically called by payment/order sagas):

- `POST .../wallet/adjustments` — generic (`assetType` in body)
- `POST .../wallet/adjustments/cash|points|tokens` — asset forced by path
- `POST .../wallet/holds` — reserve balance (`available` → `frozen`)
- `POST .../wallet/holds/{holdId}/settle` — capture hold
- `POST .../wallet/holds/{holdId}/release` — cancel hold
- `POST .../wallet/holds/expire` — sweep expired holds (idempotent; restores frozen → available)
- `POST .../wallet/transfers` — atomic inter-account transfer
- `POST .../wallet/points/lots/expire` — sweep expired points lots (idempotent; debits balance + writes allocation rows)
- `POST .../wallet/points/reconciliation` — ops integrity check: lot remaining vs account available

Points expire sweep fields: `tenantId`, `requestNo`, `idempotencyKey`; optional `organizationId`, `ownerUserId`, `accountId`.

Hold expire sweep uses the same scope fields as points expire.

Domain outbox (`commerce_outbox_event`): all write paths emit typed events (`account.ledger_appended`, `account.hold_*`, `account.transfer_completed`, `account.points_lots_expired`) in the same transaction for downstream consumers.

Required adjustment fields: `tenantId`, `ownerUserId`, `direction`, `amount`, `businessType`, `transactionNo`, `requestNo`, `idempotencyKey`.

Points-specific optional fields: `expiresAt` (credit lot TTL), `reversedLedgerId` (compensating entry link).

`businessType` must be lowercase snake_case (see `CommerceLedgerBusinessType` in `sdkwork-contract-service`).

Write-path guards (repository layer):

- Positive amount required for adjustments, holds, and transfers.
- `accountId` must belong to `ownerUserId` and `organizationId` when explicitly supplied.
- Points debit fails when lot remaining is insufficient (FEFO + allocation audit).
- Points transfer moves lots on both debit and credit legs.
- Hold settle/release rejects expired holds.
- Transfer requires `from_account` owner to match `ownerUserId`; cross-user P2P to `to_account` is allowed within the same organization.

Billing projection: each ledger append writes `commerce_billing_history` in the same transaction.

## End-to-end recharge flow

```text
1. App: sdkwork-order `recharges.orders.create`
      → commerce_order + order_item (points in sku_snapshot_json)
      → orchestrates `orders.pay` → payment_intent + payment_attempt (Payment repo)
2. User pays via payment provider
3. Payment backend: `POST .../payments/owner-orders/{orderId}/confirmations`
      → marks payment attempt succeeded (Payment repo)
      → calls order backend saga (Payment → Order boundary only)
4. Order backend: `POST .../orders/{orderId}/points-recharge/fulfillments`
      → marks order payment success + credits account via `AccountPointsCreditPort`
5. Account backend: `POST .../wallet/adjustments/points` (idempotent, HTTP adapter default)
6. App: sdkwork-account wallet.* read APIs refresh balances / ledger
```

Service env (cross-repo):

| Variable | Purpose |
| --- | --- |
| `SDKWORK_ORDER_BACKEND_API_ORIGIN` | Payment → order saga (default `http://127.0.0.1:18093`) |
| `SDKWORK_PAYMENT_ORDER_SERVICE_AUTH_TOKEN` | Bearer for payment → order backend |
| `SDKWORK_ACCOUNT_BACKEND_API_ORIGIN` | Order → account adjustments (default `http://127.0.0.1:18095`) |
| `SDKWORK_ORDER_ACCOUNT_SERVICE_AUTH_TOKEN` | Bearer for order → account backend |
| `SDKWORK_ORDER_ACCOUNT_LEDGER_ADAPTER` | `http` (default) or `store` for in-process ledger |

## TypeScript consumption

| Layer | Package |
| --- | --- |
| Account composed facade | `@sdkwork/account-service` |
| Account app SDK | `@sdkwork/account-app-sdk` (`pnpm sdk:generate:app`) |
| Account backend SDK | `@sdkwork/account-backend-sdk` (`pnpm sdk:generate:backend`) |
| Order recharge (separate repo) | `@sdkwork/order-app-sdk` via `@sdkwork/order-service` |

Account bootstrap (read models):

```typescript
import { bootstrapSdkworkAccountPcSdk } from "@sdkwork/account-pc-core/sdk";

bootstrapSdkworkAccountPcSdk({
  baseUrl: "https://api.example.com",
  authToken: session.authToken,
});
```

Backend ledger integrator (payment saga, ops):

```typescript
import { bootstrapSdkworkAccountPcBackendSdk } from "@sdkwork/account-pc-core/sdk";

bootstrapSdkworkAccountPcBackendSdk({
  baseUrl: "https://api.example.com",
  accessToken: serviceAccount.accessToken,
});
```

PC wallet recharge/withdraw **must not** call account APIs for checkout. Pass `onNavigate` + `rechargeFlow="checkout"` / `payoutFlow="checkout"` to delegate to payment surfaces:

```typescript
<SdkworkWalletPage
  checkoutBasePath="/checkout"
  payoutBasePath="/payments/payout"
  onNavigate={(route) => router.push(route)}
  rechargeFlow="checkout"
  payoutFlow="checkout"
/>
```

After payment or payout checkout completes, redirect back to the wallet route with `commerceRefresh=1` (or `payment=success` / `orderStatus=paid`). `SdkworkWalletPage` strips those query params and refreshes balances when the user returns in-app or via bfcache. Checkout navigation from wallet also forwards `returnUrl` (built via `createWalletCommerceReturnUrl`) so payment surfaces can redirect without hardcoding wallet paths.

## Verification

```powershell
pnpm verify
node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .
```

See also `docs/architecture/tech/TECH_ARCHITECTURE.md` and `docs/product/prd/PRD.md`.
