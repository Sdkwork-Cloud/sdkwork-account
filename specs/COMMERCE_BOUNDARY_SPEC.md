# Account Commerce Boundary Spec

Status: active
Owner: SDKWork maintainers
Capability: `commerce.account`
Updated: 2026-07-08

Authority: `sdkwork-specs/MODULE_SPEC.md`, `sdkwork-specs/DOMAIN_SPEC.md`, `sdkwork-specs/API_SPEC.md`

## 1. Purpose

Define what `sdkwork-account` owns and what it must never own, so order, payment, pricing, metering, and AI runtime capabilities can evolve without coupling into the ledger foundation.

## 2. Single Responsibility

Account is the ledger truth source for:

- Balances by owner, asset, currency, and account purpose.
- Append-only journal and ledger entries.
- Traditional points lots and lot allocations.
- Token Bank balances, holds, exchange snapshots, AI settlement snapshots, income, spending, and reversal references.
- Holds, releases, settlements, reversals, and transfers.
- Billing history projection.
- Idempotency records and transactional outbox events.
- Backend adjustments and governed account commands.

## 3. Account Boundary

| Account | Code | Owned by account | Not owned by account |
| --- | --- | --- | --- |
| Cash | `cash` | Balance, ledger, holds, transfers, display history. | Payment provider execution, payout settlement, acquiring channel config. |
| Points | `points` | Traditional points balance, lots, expiry, allocation audit. | Promotion campaign rules and product package publishing. |
| Token Bank | `token_bank` | AI account balance, exchange snapshots, holds, AI spending, service income, burn, transfer, reversal, reconciliation. | Model pricing formulas, raw AI metering, AI execution, provider cost policy. |

`token_bank` is the only valid AI account asset. Alternate AI account asset identifiers are not valid account assets.

Database naming boundary:

- Account-owned physical tables must use `acct_`.
- `acct_` is the physical bounded-context prefix for account/accounting tables; the capability remains `commerce.account`.
- `commerce_order` remains order-owned and outside account storage.
- Account integrations must use approved APIs and events, not direct SQL against order, payment, pricing, metering, or AI runtime tables.

## 4. Non-Goals

| Forbidden | Owner |
| --- | --- |
| `commerce_order` create/list/cancel lifecycle | `sdkwork-order` |
| Recharge or purchase package CRUD / publish | `sdkwork-order` or catalog/subscription capability |
| Recharge, coupon redemption, refund, and withdrawal orchestration | `sdkwork-order` |
| Payment intent, provider refund, provider payout, provider webhook ingest, and channel config | `sdkwork-payment` |
| Model execution for LLM/image/video/Agent/workflow/plugin | AI runtime capability |
| Raw usage collection | metering capability |
| Model-specific price tables and conversion formulas | pricing capability |
| Provider cost and margin policy | billing/pricing/finance capability |
| App routes under `/recharges/*`, `/orders/*`, or payment checkout paths | order/payment repos |

## 5. Dependency Direction

```text
`sdkwork-order`    -> `sdkwork-account` backend-api   (value-order fulfillment, holds, reversals)
`sdkwork-order`    -> `sdkwork-payment` executor      (payment, refund, payout channel execution)
`sdkwork-payment`  -> `commerce_order` read-only SQL  (existing order validation only)
pricing/metering   -> `sdkwork-account` backend-api   (Token Bank settlement inputs)
AI runtime         -> `sdkwork-account` backend-api   (hold/settle/release through governed service)
`sdkwork-account`  -> no upstream repository tables
```

Rules:

- Recharge, coupon redemption, refund, and withdrawal orchestration belong to `sdkwork-order`.
- Account must never import order, payment, pricing, metering, or AI runtime crates at repository layer.
- Account must never read `commerce_order`, payment provider, pricing table, or raw AI usage tables.
- Fulfillment after payment, coupon redemption, refund reversal, and withdrawal settlement must be triggered by order fulfillment calling account backend-api.
- `sdkwork-payment` may only reference `commerce_order` for read-only validation; it must not call account backend-api, import account crates, or write account ledger side effects.
- AI consumption must pass pricing and metering snapshot references into account; account must not calculate model prices.

## 6. API Surface

| Prefix | Role |
| --- | --- |
| `/app/v3/api/wallet` | Cash, points, holds, ledger, and wallet read models. |
| `/app/v3/api/token_bank` | Token Bank account, exchange quote, holds, settlement, ledger, income, and spending read models. |
| `/app/v3/api/billing` | Billing history read model. |
| `/app/v3/api/accounts` | Account summary read model. |
| `/backend/v3/api/wallet` | Cash/points ledger writes, holds, transfers, and operational commands. |
| `/backend/v3/api/token_bank` | Token Bank credit, debit, grant, exchange, hold, settle, release, transfer, reverse, and reconciliation commands. |

Envelope: `SdkWorkApiResponse` + `ProblemDetail` per `API_SPEC.md`.

## 7. PC Client Boundary

| Package | Role |
| --- | --- |
| `@sdkwork/account-pc-wallet` | Wallet and Token Bank UX: balances, holds, ledger, purchase/checkout navigation, withdraw UI delegation. |
| `@sdkwork/account-service` | Facade over `@sdkwork/account-app-sdk` and account backend SDK surfaces only. |

Rules:

- Wallet purchase/recharge UI may live in account PC, but order creation and payment checkout must call order/payment SDK surfaces through approved service packages.
- Account service must not add `recharges.*`, `orders.*`, payment provider execution, pricing formula, metering collection, or raw AI execution calls.
- Token Bank UI must label the account as Token Bank and must not introduce compute token, compute credit, or naked token account wording.

## 8. Rollout

Track implementation status in `specs/commerce-integration.spec.json`.

## 9. Verification

- `pnpm verify`
- `cargo test --workspace`
- API envelope, operation pattern, pagination, and SDK import checks per `AGENTS.md`
- Forbidden account naming scan per `specs/TOKEN_BANK_ACCOUNT_SPEC.md`
