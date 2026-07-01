# sdkwork-account Component Specs

Local specification index for the account / wallet capability. Root SDKWork standards remain authoritative (`../../sdkwork-specs/`).

## Spec map

| Document | Purpose |
| --- | --- |
| [component.spec.json](./component.spec.json) | Workspace component manifest |
| [COMMERCE_BOUNDARY_SPEC.md](./COMMERCE_BOUNDARY_SPEC.md) | Ledger ownership, forbidden scope, dependency rules |
| [commerce-integration.spec.json](./commerce-integration.spec.json) | Machine-readable integration contract + rollout phases |
| [../apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/specs/README.md](../apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/specs/README.md) | Wallet PC package: UI + SDK injection |

## Related capability specs (sibling repositories)

| Repository | Spec entry |
| --- | --- |
| `sdkwork-order` | `specs/RECHARGE_ORDER_SPEC.md` — unified orders + recharge |
| `sdkwork-payment` | `specs/PAYMENT_EXECUTOR_SPEC.md` — payment intent / refund only |

## Verification

```powershell
pnpm verify
node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .
```

Before claiming commerce alignment complete, confirm wallet PC `sdkDependencies` match [commerce-integration.spec.json](./commerce-integration.spec.json).
