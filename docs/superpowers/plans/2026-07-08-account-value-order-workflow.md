# Account Value Order Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the order-owned account value workflow for Token Bank recharge, plan purchase/renewal, account recharge packages, coupon recharge, refund requests, and cash withdrawal requests.

**Architecture:** `sdkwork-order` owns commercial evidence and workflow state, calls `sdkwork-payment` for provider execution, and calls `sdkwork-account` through ledger ports only. Existing points recharge routes and repositories are extended into a generic account value order path without moving ledger truth into order.

**Tech Stack:** Rust service/route/repository crates, SQLx SQLite/PostgreSQL stores, SDKWork v3 OpenAPI envelopes, SDKWork pagination, SDKWork database contract manifests.

---

### Task 1: Contract Coverage

**Files:**
- Modify: `tests/static/openapi-contract.test.mjs`
- Modify: `crates/sdkwork-routes-order-app-api/src/http_route_manifest.rs`
- Modify: `crates/sdkwork-routes-order-backend-api/src/http_route_manifest.rs`
- Modify: `apis/app-api/order/order-app-api.openapi.json`
- Modify: `apis/backend-api/order/order-backend-api.openapi.json`

- [ ] Add failing static assertions for app account value paths and backend management paths.
- [ ] Run `pnpm.cmd test:node` and confirm the new assertions fail before implementation.
- [ ] Add route manifest entries and OpenAPI paths with SDKWork v3 envelopes and write-command headers.
- [ ] Run `pnpm.cmd test:node` and confirm the assertions pass.

### Task 2: App Recharge Workflow

**Files:**
- Modify: `crates/sdkwork-routes-order-app-api/src/recharge_router.rs`
- Modify: `crates/sdkwork-order-repository-sqlx/src/sqlite_recharge.rs`
- Modify: `crates/sdkwork-order-repository-sqlx/src/postgres_recharge.rs`
- Modify: `crates/sdkwork-order-service/src/commands/account_value.rs`
- Test: `crates/sdkwork-order-service/tests/account_value_order_standard.rs`
- Test: `crates/sdkwork-order-repository-sqlx/tests/recharge_boundary_standard.rs`

- [ ] Add failing tests for Token Bank subject parsing, package/plan/coupon request validation, and account value order snapshot fields.
- [ ] Extend recharge request mapping to accept `subject`, `targetAsset`, `grantAmount`, `packageId`, `planCode`, `planPeriod`, and `couponCode`.
- [ ] Persist account value orders as `commerce_order` records with immutable item snapshots and payment attempt callback metadata.
- [ ] Keep points recharge compatibility as the `points_recharge` default path.
- [ ] Run focused service, route, and repository tests.

### Task 3: Refund And Withdrawal Workflow

**Files:**
- Modify: `crates/sdkwork-order-service/src/commands/account_value.rs`
- Modify: `crates/sdkwork-order-service/src/ports/account_value.rs`
- Modify: `crates/sdkwork-routes-order-app-api/src/recharge_router.rs` or new focused router
- Modify: `crates/sdkwork-order-repository-sqlx/src/sqlite_recharge.rs`
- Modify: `crates/sdkwork-order-repository-sqlx/src/postgres_recharge.rs`
- Modify: `crates/sdkwork-routes-order-backend-api/src/backend_commerce_admin_router.rs`

- [ ] Add failing tests for creating refund request and withdrawal request records with idempotency.
- [ ] Add SQL-backed request create/list/retrieve/update operations with tenant and owner filters.
- [ ] Wire app user routes for create/list/retrieve.
- [ ] Wire backend operator routes for list/approve/reject/retry.
- [ ] Keep provider refund execution behind the payment executor port; keep provider payout behind a future boundary port and fail-closed until `sdkwork-payment` exposes a concrete executor.

### Task 4: Documentation And Specs

**Files:**
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `specs/ACCOUNT_VALUE_ORDER_SPEC.md`
- Modify: `specs/RECHARGE_ORDER_SPEC.md`
- Modify: `specs/commerce-recharge.spec.json`

- [ ] Replace residual points-only wording where account value workflows are implemented.
- [ ] Document exact subject names, owner boundaries, route ownership, and table ownership.
- [ ] Keep any still-unimplemented external fulfillment scenarios explicitly scoped outside account value workflow.

### Task 5: Verification

**Commands:**
- `pnpm.cmd test:node`
- `pnpm.cmd db:validate`
- `cargo test -p sdkwork-order-service`
- `cargo test -p sdkwork-order-repository-sqlx --no-run`
- `cargo test -p sdkwork-routes-order-app-api --no-run`
- `cargo test -p sdkwork-routes-order-backend-api --no-run`
- `node ..\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace .`
- `node ..\sdkwork-specs\tools\check-pagination.mjs --workspace .`
- `node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .`
- `node ..\sdkwork-specs\tools\check-app-sdk-consumer-imports.mjs --workspace .`
- `git diff --check`
