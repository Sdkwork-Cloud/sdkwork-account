# Account Commerce Boundary Spec

Status: active  
Owner: SDKWork maintainers  
Capability: `commerce.account`  
Updated: 2026-06-29

Authority: `sdkwork-specs/MODULE_SPEC.md`, `sdkwork-specs/DOMAIN_SPEC.md`, `sdkwork-specs/API_SPEC.md`

## 1. Purpose

Define what **sdkwork-account** owns and what it must never own, so Order and Payment can evolve without coupling into the ledger foundation.

## 2. Single responsibility

Account is the **ledger truth source**:

- Balances (`commerce_account` by `asset_code`)
- Append-only ledger (`commerce_account_ledger`)
- Points lots (FIFO)
- Holds and transfers
- Billing history projection (read model)
- Backend adjustments (idempotent credits/debits)

## 3. Non-goals (forbidden in this repository)

| Forbidden | Owner |
| --- | --- |
| `commerce_order` create/list/cancel lifecycle | `sdkwork-order` |
| Recharge package CRUD / publish | `sdkwork-order` |
| `recharges.orders.create` | `sdkwork-order` |
| Payment intent, provider webhook, channel config | `sdkwork-payment` |
| Refund execution against provider | `sdkwork-payment` |
| App routes under `/recharges/*` or `/orders/*` | order / payment repos |

## 4. Dependency direction

```text
sdkwork-order   ──backend-api──▶ sdkwork-account   (adjustments, holds)
sdkwork-payment ──via order───▶ sdkwork-order     (pay existing orderId)
sdkwork-account ──✗──▶ order / payment            (no upstream HTTP/RPC)
```

Rules:

- Account **never** imports order or payment crates at repository layer.
- Account **never** reads `commerce_order` or `commerce_recharge_package` tables.
- Fulfillment after payment **must** be triggered by Order saga calling account backend-api, not Payment webhook → account directly.

## 5. API surface (this repo)

| Prefix | Role |
| --- | --- |
| `/app/v3/api/wallet` | Read-only wallet models |
| `/app/v3/api/billing` | Billing history read |
| `/app/v3/api/accounts` | Account summary read |
| `/backend/v3/api/wallet` | Ledger writes: adjustments, holds, transfers |

Envelope: `SdkWorkApiResponse` + `ProblemDetail` per `API_SPEC.md`.

## 6. PC client boundary

| Package | Role |
| --- | --- |
| `@sdkwork/account-pc-wallet` | Wallet UX: balances, holds, ledger, recharge/withdraw **UI** |
| `@sdkwork/account-service` | Facade over `@sdkwork/account-app-sdk` only |

Wallet recharge UI **may** live in account-pc but **must** call `@sdkwork/order-app-sdk` via `@sdkwork/order-service` (see wallet package spec).  
**Must not** add `recharges.*` to `@sdkwork/account-service`.

## 7. Rollout

Track implementation status in [commerce-integration.spec.json](./commerce-integration.spec.json) `implementationPhases`.

## 8. Verification

- `pnpm verify`
- `cargo test --workspace`
- Envelope check per `AGENTS.md`
