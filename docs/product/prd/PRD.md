# Account PRD

Status: active
Owner: SDKWork maintainers
Application: account
Updated: 2026-06-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` §8

## 1. Background And Problem

Wallet balances, ledger entries, account summaries, and billing history must be append-only, version-guarded, and tenant-isolated.

This repository is a **T1 commerce capability building block**. The `sdkwork-commerce (deleted)` monolith has been dissolved; this repository is self-contained with its own domain logic, persistence, HTTP route builders, API server, and IAM middleware for the **account** capability.

## 2. Target Users

Account holders, finance reviewers, and integrators displaying wallet or billing history.

## 3. Goals And Non-Goals

### Goals

- Own account/billing SQL and wallet/billing HTTP routers.
- Enforce ledger append-only semantics and optimistic balance versioning.

### Non-Goals

- Payment provider execution (payment capability).
- Promotion point exchange rules (promotion capability).

## 4. Scope

- Account summary and security read models.
- Wallet overview, accounts, ledger entries, token balance reads.
- Billing history list.

Primary API prefixes:

- App: `/app/v3/api/wallet`

Migration status: **complete**.

## 5. User Scenarios

- A user views wallet accounts and ledger history scoped to their tenant identity.
- A credit posts through append_ledger_entry with idempotency key replay protection.

## 6. Success Metrics

- Repository tests validate version-guarded balance updates.
- Commerce wallet routes pass via T1 standalone-gateway IAM wrappers.

## 7. Phases

- Phase 1 (complete): SQL + account/billing routers migrated.
- Phase 2 (complete): repository tests use account-local SQLite schema; no commerce storage dev-dep.

## 8. Linked Requirements

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` §8
- Component contract: `specs/component.spec.json` (when present)
- Machine contracts: local `specs/`, future `apis/`, and generated `sdks/`

## 9. Open Questions

- Whether backend wallet admin routes belong in this repo or a dedicated admin standalone-gateway.
