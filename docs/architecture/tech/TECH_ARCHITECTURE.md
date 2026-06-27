# Account Technical Architecture
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

Status: active
Owner: SDKWork maintainers
Updated: 2026-06-24

## 1. Architecture Overview

Describe the repository/application architecture.


## Capability stack

`sdkwork-account` owns the full **account / wallet** capability:

| Layer | Path |
| --- | --- |
| Domain (Rust) | `crates/sdkwork-commerce-account-service/` (wallet domain; migrating naming) |
| HTTP routers | `crates/sdkwork-routes-account-*-api/` |
| API server | `crates/sdkwork-account-standalone-gateway/` |
| PC client | `apps/sdkwork-account-pc/` |
| Client facade | `packages/common/account/sdkwork-account-service/` |

## PC surface

```text
apps/sdkwork-account-pc/
  packages/sdkwork-account-pc-core/
  packages/sdkwork-account-pc-shell/
  packages/sdkwork-account-pc-wallet/    ← migrated from sdkwork-commerce-pc
```

Composition apps consume `@sdkwork/account-pc-wallet` via workspace paths — not a central commerce PC repo.

## API ownership

- App API prefix: `/app/v3/api/wallet`
- Backend API prefix: `/backend/v3/api/wallet`
- Table prefix: `commerce_` (commerce domain)

## Verification

```powershell
cd E:\sdkwork-space\sdkwork-account
pnpm verify
```

## Related docs

- Commerce repository dissolution: `../../sdkwork-specs/MIGRATION_SPEC.md` §8
