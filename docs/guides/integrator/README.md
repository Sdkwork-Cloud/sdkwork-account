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

## Asset model

Cash, points, and token are **separate account rows** (`asset_code`: `cash` | `points` | `token`). App routes expose asset-scoped read models:

- `GET .../wallet/accounts/cash` — cash balance DTO
- `GET .../wallet/accounts/points` — points balance + lot stats (`activeLotCount`, `expiringPoints`)
- `GET .../wallet/accounts/tokens` — token account row
- `GET .../wallet/ledger_entries/cash|points` — asset-filtered ledger
- `GET .../wallet/points/lots` — points lot list (FIFO consumption source)
- `GET .../wallet/holds` — hold list (optional `accountId`, `assetType`, `status`)
- `GET .../wallet/holds/{holdId}` — hold detail

Backend ledger writes (typically called by payment/order sagas):

- `POST .../wallet/adjustments` — generic (`assetType` in body)
- `POST .../wallet/adjustments/cash|points|tokens` — asset forced by path
- `POST .../wallet/holds` — reserve balance (`available` → `frozen`)
- `POST .../wallet/holds/{holdId}/settle` — capture hold
- `POST .../wallet/holds/{holdId}/release` — cancel hold
- `POST .../wallet/transfers` — atomic inter-account transfer

Required adjustment fields: `tenantId`, `ownerUserId`, `direction`, `amount`, `businessType`, `transactionNo`, `requestNo`, `idempotencyKey`.

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

## Verification

```powershell
pnpm verify
node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .
```

See also `docs/architecture/tech/TECH_ARCHITECTURE.md` and `docs/product/prd/PRD.md`.
