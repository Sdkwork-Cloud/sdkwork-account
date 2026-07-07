# Account Value Order Orchestration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build production-ready recharge, Token Bank plan purchase, coupon redemption, refund, and withdrawal orchestration with `sdkwork-order` as the business owner, `sdkwork-payment` as the provider executor, and `sdkwork-account` as the ledger truth source.

**Architecture:** Account never creates orders or executes payment channels. Order creates and settles all account-value orders, calls payment for collection/refund/payout execution, and calls account for ledger hold, credit, debit, settlement, release, and reversal. Payment may validate existing orders but must not call account or write account ledger side effects.

**Tech Stack:** Rust services and `sqlx` repositories, SDKWork v3 OpenAPI envelopes, generated TypeScript SDK facades, SDKWork database contracts, SDKWork pagination and SDK import validators.

---

## Scope

This is a cross-repository plan. Execute from each writable repository root:

- `E:\sdkwork-space\sdkwork-account`: account boundary, ledger command contracts, docs.
- `E:\sdkwork-space\sdkwork-order`: PRD, architecture, order subjects, package/plan/coupon/refund/withdrawal APIs, sagas, database, SDKs.
- `E:\sdkwork-space\sdkwork-payment`: refund and payout executor surfaces only; no account dependency.

Current sandbox allows writes only in `sdkwork-account`. Tasks touching `sdkwork-order` or `sdkwork-payment` require a writable run rooted at those repositories.

## File Structure

Account files:

- Modify: `specs/COMMERCE_BOUNDARY_SPEC.md`
- Modify: `specs/commerce-integration.spec.json`
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `tests/static/placeholder.test.mjs`

Order files:

- Create: `specs/ACCOUNT_VALUE_ORDER_SPEC.md`
- Modify: `specs/commerce-recharge.spec.json`
- Modify: `specs/commerce-checkout-topology.spec.json`
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `database/ddl/baseline/{postgres,sqlite}/0001_order_baseline.sql`
- Modify: `database/contract/table-registry.json`
- Modify: `apis/app-api/order/order-app-api.openapi.json`
- Modify: `apis/backend-api/order/order-backend-api.openapi.json`
- Modify: `crates/sdkwork-order-service/src/**`
- Modify: `crates/sdkwork-order-repository-sqlx/src/**`
- Modify: `crates/sdkwork-routes-order-app-api/src/**`
- Modify: `crates/sdkwork-routes-order-backend-api/src/**`
- Modify: `apps/sdkwork-order-common/packages/sdkwork-order-sdk-ports/src/index.ts`
- Regenerate: `sdks/sdkwork-order-app-sdk/**`, `sdks/sdkwork-order-backend-sdk/**`

Payment files:

- Modify: `specs/PAYMENT_EXECUTOR_SPEC.md`
- Modify: `specs/commerce-boundary.spec.json`
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `database/ddl/baseline/{postgres,sqlite}/0001_payment_baseline.sql`
- Modify: `apis/app-api/payment/payment-app-api.openapi.json` or payment OpenAPI authority in the repo
- Modify: `crates/sdkwork-payment-service/src/**`
- Modify: `crates/sdkwork-payment-repository-sqlx/src/**`
- Modify: `crates/sdkwork-routes-payment-app-api/src/**`
- Regenerate: payment SDK families

## Task 1: Account Boundary Contract

**Files:**
- Modify: `tests/static/placeholder.test.mjs`
- Modify: `specs/COMMERCE_BOUNDARY_SPEC.md`
- Modify: `specs/commerce-integration.spec.json`
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Create: `docs/superpowers/plans/2026-07-08-account-value-order-orchestration.md`

- [ ] **Step 1: Write the failing static contract test**

Add a node static test asserting:

```js
assert.match(boundarySpec, /Recharge, coupon redemption, refund, and withdrawal orchestration belong to `sdkwork-order`/);
assert.match(boundarySpec, /`sdkwork-order`\s+-> `sdkwork-payment`/);
assert.match(boundarySpec, /`sdkwork-payment` may only reference `commerce_order` for read-only validation/);
assert.equal(integrationSpec.valueOrderOrchestration.directPaymentToAccountDependencyAllowed, false);
```

- [ ] **Step 2: Run red test**

Run: `pnpm.cmd test:node`

Expected: FAIL because the boundary spec and machine contract do not yet expose the value-order orchestration boundary.

- [ ] **Step 3: Update account docs and machine contract**

Patch account PRD, architecture, commerce boundary spec, and machine contract so:

- `sdkwork-order` owns recharge, coupon redemption, refund request, and withdrawal request orchestration.
- `sdkwork-payment` executes payment, provider refund, and provider payout channels only.
- `sdkwork-account` exposes idempotent ledger, hold, credit, debit, settlement, release, and reversal commands.
- Payment direct dependency on account is forbidden.

- [ ] **Step 4: Run green test**

Run: `pnpm.cmd test:node`

Expected: PASS with all static tests green.

- [ ] **Step 5: Verify account repository**

Run:

```powershell
node E:\sdkwork-space\sdkwork-specs\tools\check-repository-docs-standard.mjs --root E:\sdkwork-space\sdkwork-account
node E:\sdkwork-space\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace E:\sdkwork-space\sdkwork-account
node E:\sdkwork-space\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace E:\sdkwork-space\sdkwork-account
node E:\sdkwork-space\sdkwork-specs\tools\check-pagination.mjs --workspace E:\sdkwork-space\sdkwork-account
node E:\sdkwork-space\sdkwork-specs\tools\check-app-sdk-consumer-imports.mjs --workspace E:\sdkwork-space\sdkwork-account
pnpm.cmd verify
pnpm.cmd test
git diff --check
```

Expected: all pass.

## Task 2: Order Account-Value Spec And Machine Contract

**Writable root:** `E:\sdkwork-space\sdkwork-order`

**Files:**
- Create: `specs/ACCOUNT_VALUE_ORDER_SPEC.md`
- Modify: `specs/commerce-recharge.spec.json`
- Modify: `specs/commerce-checkout-topology.spec.json`
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Test: `tests/static/*.mjs` or existing order static contract test file

- [ ] **Step 1: Write failing static tests**

Assert order machine specs define:

```json
{
  "accountValueOrder": {
    "owner": "sdkwork-order",
    "paymentExecutor": "sdkwork-payment",
    "ledgerExecutor": "sdkwork-account",
    "subjects": [
      "points_recharge",
      "token_bank_recharge",
      "token_bank_plan_purchase",
      "token_bank_plan_renewal",
      "account_recharge_package",
      "coupon_recharge",
      "refund_request",
      "cash_withdrawal"
    ]
  }
}
```

Assert forbidden terms include:

- direct account SQL writes
- payment-owned recharge routes
- payment-to-account ledger writes
- naked token account naming

- [ ] **Step 2: Run red static tests**

Run from `sdkwork-order`: `pnpm.cmd test:node` or the repository's static test command.

Expected: FAIL before specs are updated.

- [ ] **Step 3: Add `ACCOUNT_VALUE_ORDER_SPEC.md`**

Define:

- order subjects and state machines
- package purchase versus Token Bank plan purchase
- coupon redemption and zero-amount or mixed-payment orders
- refund request flow and account pre-hold/reversal
- withdrawal request flow and account cash hold/settlement/release
- idempotency keys and request hash scopes
- payment executor boundaries
- account ledger command boundaries
- SDK and API naming rules

- [ ] **Step 4: Update PRD and architecture**

PRD must state:

- every recharge, coupon redemption, refund, and withdrawal has an order record
- packages and plans are order-owned product/order facts
- account only handles ledger effects
- payment only executes provider channels

Architecture must state:

- `sdkwork-order -> sdkwork-payment`
- `sdkwork-order -> sdkwork-account`
- `sdkwork-payment -X-> sdkwork-account`
- `sdkwork-payment -X-> sdkwork-order service dependency`

- [ ] **Step 5: Run green static tests**

Run: `pnpm.cmd test:node`

Expected: PASS.

## Task 3: Order Database And Domain Model

**Writable root:** `E:\sdkwork-space\sdkwork-order`

**Files:**
- Modify: `database/ddl/baseline/postgres/0001_order_baseline.sql`
- Modify: `database/ddl/baseline/sqlite/0001_order_baseline.sql`
- Modify: `database/contract/table-registry.json`
- Modify: `crates/sdkwork-order-service/src/domain/**`
- Modify: `crates/sdkwork-order-service/src/commands/**`
- Modify: `crates/sdkwork-order-repository-sqlx/src/**`
- Test: order repository and service tests

- [ ] **Step 1: Write failing Rust domain tests**

Add tests requiring:

- valid order subjects listed in Task 2
- `token_bank_recharge` uses `asset_code = token_bank`
- plan billing periods: monthly, quarterly, yearly, continuous_monthly, continuous_yearly
- coupon redemption can be zero-amount or mixed-payment
- refund requires original order reference
- withdrawal requires `asset_code = cash`

- [ ] **Step 2: Run red Rust tests**

Run targeted cargo tests for `sdkwork-order-service`.

Expected: FAIL because domain types and validation are missing.

- [ ] **Step 3: Implement minimal domain types**

Add focused enums and command structs:

- `AccountValueOrderSubject`
- `RechargeTargetAsset`
- `TokenBankPlanPeriod`
- `CreateAccountRechargeOrderCommand`
- `CreateCouponRechargeOrderCommand`
- `CreateOrderRefundRequestCommand`
- `CreateCashWithdrawalRequestCommand`

Use existing SDKWork validation helpers where available.

- [ ] **Step 4: Add SQL schema**

Greenfield target tables:

- `commerce_account_value_package`
- `commerce_token_bank_plan`
- `commerce_order_refund_request`
- `commerce_order_withdrawal_request`

Keep package/plan snapshots copied into `commerce_order_item.sku_snapshot_json` for immutable order evidence.

- [ ] **Step 5: Run database validation**

Run:

```powershell
pnpm.cmd run db:validate
cargo test -p sdkwork-order-service
cargo test -p sdkwork-order-repository-sqlx
```

Expected: PASS.

## Task 4: Order App And Backend API Contracts

**Writable root:** `E:\sdkwork-space\sdkwork-order`

**Files:**
- Modify: `apis/app-api/order/order-app-api.openapi.json`
- Modify: `apis/backend-api/order/order-backend-api.openapi.json`
- Modify: `crates/sdkwork-routes-order-app-api/src/**`
- Modify: `crates/sdkwork-routes-order-backend-api/src/**`
- Modify: `apps/sdkwork-order-common/packages/sdkwork-order-sdk-ports/src/index.ts`

- [ ] **Step 1: Write failing OpenAPI/static tests**

Assert all write operations declare:

- `Idempotency-Key`
- `Sdkwork-Request-Hash`
- `SdkWorkApiResponse`
- `ProblemDetail`
- SDKWork operation pattern resource/action names

- [ ] **Step 2: Add app APIs**

Add or extend resources:

- `recharges.packages.list`
- `recharges.plans.list`
- `recharges.orders.create`
- `recharges.orders.retrieve`
- `orders.refundRequests.create`
- `orders.refundRequests.list`
- `withdrawals.requests.create`
- `withdrawals.requests.retrieve`

- [ ] **Step 3: Add backend APIs**

Add management resources:

- account value package publish/update/retire
- Token Bank plan publish/update/retire
- refund request approve/reject/retry
- withdrawal request approve/reject/retry

- [ ] **Step 4: Run API validators**

Run:

```powershell
node E:\sdkwork-space\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace E:\sdkwork-space\sdkwork-order
node E:\sdkwork-space\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace E:\sdkwork-space\sdkwork-order
node E:\sdkwork-space\sdkwork-specs\tools\check-pagination.mjs --workspace E:\sdkwork-space\sdkwork-order
```

Expected: PASS.

## Task 5: Order Settlement Sagas

**Writable root:** `E:\sdkwork-space\sdkwork-order`

**Files:**
- Modify: `crates/sdkwork-order-service/src/**`
- Modify: `crates/sdkwork-order-service/src/ports/**`
- Modify: `crates/sdkwork-order-integration-account/**`
- Modify: `crates/sdkwork-order-integration-payment/**`
- Test: order service saga tests and integration E2E tests

- [ ] **Step 1: Write failing saga tests**

Cover:

- Token Bank recharge payment success credits `token_bank`.
- Token Bank plan first purchase credits first-cycle grant.
- Token Bank plan renewal credits renewal grant.
- Coupon recharge consumes coupon and credits target account.
- Refund pre-holds or reverses account balance before provider refund.
- Refund success commits account reversal.
- Refund failure releases account hold.
- Withdrawal freezes cash before payout.
- Withdrawal success settles hold and debits cash.
- Withdrawal failure releases hold.

- [ ] **Step 2: Implement ports**

Create clear ports:

- `AccountValueLedgerPort`
- `PaymentRefundExecutorPort`
- `PaymentPayoutExecutorPort`
- `CouponRedemptionPort`

Do not import account repository crates in order repository code.

- [ ] **Step 3: Implement sagas with idempotency**

Standard idempotency key examples:

- `token-bank-recharge:fulfill:{orderId}`
- `token-bank-plan:renewal:{orderId}`
- `coupon-recharge:fulfill:{orderId}`
- `refund-request:account-hold:{refundRequestId}`
- `refund-request:payment-refund:{refundRequestId}`
- `withdrawal:account-hold:{withdrawalRequestId}`
- `withdrawal:payment-payout:{withdrawalRequestId}`

- [ ] **Step 4: Run saga tests**

Run targeted cargo tests, then `cargo test --workspace`.

Expected: PASS.

## Task 6: Payment Refund And Payout Executor Boundary

**Writable root:** `E:\sdkwork-space\sdkwork-payment`

**Files:**
- Modify: `specs/PAYMENT_EXECUTOR_SPEC.md`
- Modify: `specs/commerce-boundary.spec.json`
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `crates/sdkwork-payment-service/src/**`
- Modify: `crates/sdkwork-payment-repository-sqlx/src/**`
- Modify: `crates/sdkwork-routes-payment-app-api/src/**`
- Test: payment service/repository/route tests

- [ ] **Step 1: Write failing boundary tests**

Assert:

- payment has no crate/package dependency on account
- payment has no crate/package dependency on order service
- payment has no recharge routes
- payment refund and payout commands require existing `orderId`

- [ ] **Step 2: Add payout executor if absent**

Add provider payout abstractions parallel to refund:

- `payouts.create`
- `payouts.retrieve`
- `payouts.list`
- provider submission retry
- provider status query

- [ ] **Step 3: Keep account side effects out**

Payment responses expose provider execution state only. Order remains responsible for account hold/release/settlement.

- [ ] **Step 4: Run payment verification**

Run:

```powershell
rg 'sdkwork_account|sdkwork-account|account backend' crates Cargo.toml
cargo test --workspace
pnpm.cmd verify
node E:\sdkwork-space\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace E:\sdkwork-space\sdkwork-payment
node E:\sdkwork-space\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace E:\sdkwork-space\sdkwork-payment
node E:\sdkwork-space\sdkwork-specs\tools\check-pagination.mjs --workspace E:\sdkwork-space\sdkwork-payment
```

Expected: dependency scan has no forbidden hits and all checks pass.

## Task 7: SDK Generation And PC Integration

**Writable roots:** order, payment, then account if wallet integration changes are required.

- [ ] **Step 1: Regenerate OpenAPI and SDKs**

Run each repository's SDK generation scripts. Do not hand-edit generated SDK source.

- [ ] **Step 2: Update composed SDK facades**

App packages must import:

- `@sdkwork/order-app-sdk`
- `@sdkwork/payment-app-sdk`
- `@sdkwork/account-app-sdk`

Forbidden:

- generated transport package names
- raw HTTP wrappers
- account-service exposing `orders.*`, `recharges.*`, `refunds.*`, or `payouts.*`

- [ ] **Step 3: Add PC wallet integration tests**

Wallet recharge/refund/withdrawal UI must delegate to order SDK or host `onNavigate` ports. It must not call payment or account backend mutation APIs directly.

- [ ] **Step 4: Run SDK import validator**

Run for every touched repository:

```powershell
node E:\sdkwork-space\sdkwork-specs\tools\check-app-sdk-consumer-imports.mjs --workspace <repo-root>
```

Expected: PASS.

## Task 8: Cross-Repository Launch Gate

- [ ] **Step 1: Run account verification**

```powershell
cd E:\sdkwork-space\sdkwork-account
pnpm.cmd test
pnpm.cmd verify
cargo clippy --workspace --all-targets -- -D warnings
pnpm.cmd run fmt:rust:check
git diff --check
```

- [ ] **Step 2: Run order verification**

```powershell
cd E:\sdkwork-space\sdkwork-order
pnpm.cmd test
pnpm.cmd verify
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 3: Run payment verification**

```powershell
cd E:\sdkwork-space\sdkwork-payment
pnpm.cmd test
pnpm.cmd verify
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 4: Run SDKWork global validators**

Run API envelope, operation pattern, pagination, docs standard, repo verify, and SDK consumer import validators for every touched repository.

Expected: all pass before claiming production readiness.
