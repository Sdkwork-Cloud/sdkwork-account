# Account PRD

Status: active
Owner: SDKWork maintainers
Application: account
Updated: 2026-06-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Platform split alignment (commerce T0): `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md`

## 1. Background And Problem

Wallet balances, ledger entries, account summaries, and billing history must be append-only, version-guarded, and tenant-isolated.

This repository is a **T1 commerce capability building block**. `sdkwork-commerce` remains the T0 composition layer (gateway, IAM wrappers, composed SDK). This repository owns domain logic, persistence, and HTTP route builders for the **account** capability.

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
- Commerce wallet routes pass via thin IAM wrappers.

## 7. Phases

- Phase 1 (complete): SQL + account/billing routers migrated.
- Phase 2 (complete): repository tests use account-local SQLite schema; no commerce storage dev-dep.

## 8. Linked Requirements

- Commerce capability split alignment: `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md`
- Component contract: `specs/component.spec.json` (when present)
- Machine contracts: local `specs/`, future `apis/`, and generated `sdks/`

## 9. Open Questions

- Whether backend wallet admin routes belong in this repo or commerce T0 only.
