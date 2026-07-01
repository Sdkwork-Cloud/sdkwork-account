# @sdkwork/account-pc-wallet Component Specs

Wallet PC React package: read models from account SDK, recharge flow from order SDK.

## Component

| Field | Value |
| --- | --- |
| Name | `@sdkwork/account-pc-wallet` |
| Domain | `commerce` |
| Capability | `wallet` (account surface) |
| Root | `apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet` |

## Service split (required)

| Module | SDK | Responsibility |
| --- | --- | --- |
| `wallet-service.ts` | `@sdkwork/account-service` | Overview: balances, holds, ledger |
| `wallet-recharge-service.ts` (planned) | `@sdkwork/order-service` | Packages list, create recharge order, pay |
| `wallet-controller.ts` | both via injection | UI state only |

Do not merge order methods into `wallet-service` without a separate file and explicit order service injection.

## SDK dependencies

| Family | Package | Required methods |
| --- | --- | --- |
| Account app | `@sdkwork/account-app-sdk` | `wallet.*`, `billing.*`, `accounts.*` |
| Order app | `@sdkwork/order-app-sdk` | `recharges.*`, `orders.pay` |

Bootstrap: `account-pc-core` must configure both token providers (same session).

## UI ownership

| Surface | Owner | Data source |
| --- | --- | --- |
| Balance / holds / transactions | this package | account SDK |
| Recharge package grid / dialog | this package | order SDK |
| Payment cashier (QR / poll) | embed or navigate | order `orders.pay` → payment |

## Forbidden

- Raw `fetch` to `/app/v3/api/recharges/*`
- Local DTO copies of order recharge types (map from generated SDK)
- `rechargePackages` hard-coded or empty without order SDK call

## Verification

```powershell
pnpm --filter @sdkwork/account-pc-wallet typecheck
pnpm test:vitest
```

See also: [../../../../specs/COMMERCE_BOUNDARY_SPEC.md](../../../../specs/COMMERCE_BOUNDARY_SPEC.md)
