# Account Value Order Workflows Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete sdkwork-order account value workflows for Token Bank recharge, package recharge, coupon recharge, refund requests, and cash withdrawals while keeping account as ledger truth and payment as provider executor.

**Architecture:** Order owns commercial evidence, request state, API routes, and saga orchestration. Account effects go through `AccountValueLedgerPort`; provider refund effects go through the payment executor port, while provider payout stays behind a future boundary port and fails closed until `sdkwork-payment` exposes a concrete executor. SQL repositories only persist order-owned state.

**Tech Stack:** Rust service/domain/ports, SQLx PostgreSQL/SQLite repositories, Axum route crates, OpenAPI/TypeScript SDK generation, SDKWork v3 API envelopes.

---

### Task 1: Refund And Withdrawal Execution Saga

**Files:**
- Modify: `crates/sdkwork-order-service/src/ports/account_value.rs`
- Modify: `crates/sdkwork-order-service/src/service/mod.rs`
- Create: `crates/sdkwork-order-service/src/service/account_value_request_execution.rs`
- Test: `crates/sdkwork-order-service/tests/account_value_request_execution_standard.rs`

- [ ] **Step 1: Write failing tests** for refund approve/retry and withdrawal approve/retry calling account ledger and payment ports in the required order.
- [ ] **Step 2: Run narrow service test** with `cargo test -p sdkwork-order-service account_value_request_execution_standard`.
- [ ] **Step 3: Implement service ports and execution functions** with idempotent account hold/reversal and provider execution requests.
- [ ] **Step 4: Re-run narrow service test** until green.

### Task 2: Backend Route Wiring

**Files:**
- Modify: `crates/sdkwork-routes-order-backend-api/src/backend_commerce_admin_router.rs`
- Modify: `crates/sdkwork-routes-order-backend-api/src/routes.rs`

- [ ] **Step 1: Add route-level tests or compile checks** proving backend refund/withdrawal review can receive ledger/refund and future payout boundary ports.
- [ ] **Step 2: Wire backend review routes** through service execution instead of repository-only status updates.
- [ ] **Step 3: Keep repository layer as state persistence only** and do not add account/payment calls to SQLx crates.

### Task 3: Repository State Transition Alignment

**Files:**
- Modify: `crates/sdkwork-order-repository-sqlx/src/sqlite_account_value.rs`
- Modify: `crates/sdkwork-order-repository-sqlx/src/postgres_account_value.rs`
- Modify: `database/ddl/baseline/sqlite/0001_order_baseline.sql`
- Modify: `database/ddl/baseline/postgres/0001_order_baseline.sql`

- [ ] **Step 1: Add tests if state fields or provider references are missing.**
- [ ] **Step 2: Persist state transitions and provider execution references idempotently.**
- [ ] **Step 3: Keep SQLite/PostgreSQL parity and SQL-level pagination.**

### Task 4: Contract And Documentation Alignment

**Files:**
- Modify: `apis/app-api/order/order-app-api.openapi.json`
- Modify: `apis/backend-api/order/order-backend-api.openapi.json`
- Modify: `scripts/openapi/materialize-recharges-openapi.mjs`
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `specs/ACCOUNT_VALUE_ORDER_SPEC.md`
- Modify: `specs/RECHARGE_ORDER_SPEC.md`

- [ ] **Step 1: Update docs from contracted workflow to implemented workflow only after code and tests pass.**
- [ ] **Step 2: Regenerate SDKs if OpenAPI changes are required.**
- [ ] **Step 3: Remove stale naming and historical wording.**

### Task 5: Verification

- [ ] **Step 1:** `cargo test -p sdkwork-order-service account_value`
- [ ] **Step 2:** `cargo test --workspace`
- [ ] **Step 3:** `pnpm.cmd test:node`
- [ ] **Step 4:** `pnpm.cmd db:validate`
- [ ] **Step 5:** `pnpm.cmd verify`
- [ ] **Step 6:** SDKWork validators for API envelope, operation patterns, pagination, and SDK consumer imports.
