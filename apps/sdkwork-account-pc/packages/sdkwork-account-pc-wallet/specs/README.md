# @sdkwork/account-pc-wallet Component Specs

Wallet PC React package: read models from account SDK; recharge uses an injected order-compatible port from the integrator.

## Component

| Field | Value |
| --- | --- |
| Name | `@sdkwork/account-pc-wallet` |
| Domain | `commerce` |
| Capability | `wallet` (account surface) |
| Root | `apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet` |

## Service Split

| Module | SDK or Port | Responsibility |
| --- | --- | --- |
| `wallet-service.ts` | `@sdkwork/account-service` | Overview: balances, holds, ledger |
| `wallet-recharge-service.ts` | order-compatible port | Packages list and create recharge order |
| `wallet-controller.ts` | account service plus injected ports | UI state only |

Do not merge order methods into `wallet-service`; keep commerce creation behind explicit recharge service or order-compatible port injection.

## SDK Dependencies

| Family | Package | Required methods |
| --- | --- | --- |
| Account app | `@sdkwork/account-app-sdk` | `wallet.*`, `billing.*`, `accounts.*`, `tokenBank.*` |
| Order app | integrator-owned order service port | `recharges.packages.list`, `recharges.settings.retrieve`, `recharges.orders.create` |

Bootstrap: `account-pc-core` configures account SDK. Integrators that enable recharge inject an order-compatible service port using the same authenticated session.

## UI Ownership

| Surface | Owner | Data source |
| --- | --- | --- |
| Balance / holds / transactions | this package | account SDK |
| Recharge package grid / dialog | this package | injected order-compatible port |
| Payment cashier | embed or navigate | checkout route owned by order/payment |

## Forbidden

- Raw `fetch` to `/app/v3/api/recharges/*`
- Order SDK imports from account PC wallet package internals
- `rechargePackages` hard-coded in account code instead of supplied by injected commerce port

## Verification

```powershell
pnpm --filter @sdkwork/account-pc-wallet typecheck
pnpm test:vitest
```

See also: [../../../../specs/COMMERCE_BOUNDARY_SPEC.md](../../../../specs/COMMERCE_BOUNDARY_SPEC.md)
