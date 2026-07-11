# Account Value Orders Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement production-grade order-owned account value orchestration for Token Bank recharge, package/plan/coupon recharge, refund review execution, and withdrawal review execution while preserving account/payment capability boundaries.

**Architecture:** `sdkwork-order-service` owns command models, request state machines, idempotency keys, and ports. SQLx repositories persist order-owned packages, plans, refund requests, and withdrawal requests; integration crates implement account ledger and payment executor ports; route crates expose SDKWork v3 app/backend APIs and inject runtime ports from `sdkwork-order-service-host`.

**Tech Stack:** Rust service/repository/route crates, SQLx SQLite/Postgres, SDKWork web framework envelopes, order/payment/account integration ports, OpenAPI/TypeScript SDK generation.

---

### Task 1: Backend Runtime Port Wiring

**Files:**
- Modify: `crates/sdkwork-order-service-host/src/lib.rs`
- Modify: `crates/sdkwork-routes-order-backend-api/src/routes.rs`
- Test: `crates/sdkwork-routes-order-backend-api/tests/backend_openapi_routes.rs`

- [ ] Add payment refund executor and future payout executor boundary ports to `OrderServiceHost`.
- [ ] Expose cloned getters for backend route assembly.
- [ ] Wire backend admin commerce router with the `*_with_*_and_ports` constructors.
- [ ] Run targeted backend route tests.

### Task 2: Payment Executor Integration

**Files:**
- Create: `crates/sdkwork-order-integration-payment/`
- Modify: `Cargo.toml`
- Modify: `crates/sdkwork-order-service-host/src/lib.rs`
- Test: `crates/sdkwork-order-integration-payment/tests/*`

- [ ] Add a payment-owned refund executor adapter consumed by order through `PaymentRefundExecutorPort`.
- [ ] Keep payout fail-closed until `sdkwork-payment` exposes a concrete provider payout executor contract.
- [ ] Ensure order does not call PSP SDKs or write payment SQL directly.

### Task 3: Account HTTP Hold Integration

**Files:**
- Modify: `crates/sdkwork-order-integration-account/src/http_adapter.rs`
- Test: `crates/sdkwork-order-integration-account/tests/*`

- [ ] Route `Credit`, `Debit`, and `Reversal` through account adjustment APIs.
- [ ] Route `Hold`, `HoldSettle`, and `HoldRelease` through account hold APIs.
- [ ] Map account response envelope `data.item` into `AccountValueLedgerOutcome`.

### Task 4: API, Specs, And Documentation Alignment

**Files:**
- Modify: `crates/sdkwork-routes-order-app-api/src/recharge_router.rs`
- Modify: `scripts/openapi/materialize-recharges-openapi.mjs`
- Modify: `apis/**/order-*.openapi.json`
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `specs/ACCOUNT_VALUE_ORDER_SPEC.md`
- Modify: `specs/RECHARGE_ORDER_SPEC.md`

- [ ] Use explicit account-side fields (`accountAmount`, `accountUnitCode`) and provider-side fields (`providerAmount`, `providerCurrencyCode`) for refund/withdrawal commands.
- [ ] Keep Token Bank canonical asset code `token_bank`.
- [ ] Remove stale `compute_credit`, `compute_token`, naked `token`, and legacy phase wording.

### Task 5: Verification

- [ ] Run `cargo fmt`.
- [ ] Run targeted Rust tests for service, repository, account integration, payment integration, and backend route crates.
- [ ] Run `cargo test --workspace`.
- [ ] Run SDKWork API, pagination, SDK consumer import, layering, and Rust composition checks.
- [ ] Run `pnpm.cmd verify` if TypeScript/OpenAPI surfaces changed.
