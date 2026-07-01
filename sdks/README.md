# SDKWork Account SDK Families

HTTP SDK generation workspace for `sdkwork-account`.

| Family | Authority | Package |
| --- | --- | --- |
| `sdkwork-account-app-sdk` | `sdkwork-account.app` | `@sdkwork/account-app-sdk` |
| `sdkwork-account-backend-sdk` | `sdkwork-account.backend` | `@sdkwork/account-backend-sdk` |

Canonical API authorities live under `apis/`. Each SDK family copies or derives from those OpenAPI documents under `openapi/`.

Generate TypeScript transport SDKs with `@sdkwork/sdk-generator` / `sdkgen`:

```powershell
pnpm sdk:generate
```

PC bootstrap uses `@sdkwork/account-app-sdk` through `@sdkwork/account-service` (`bootstrapSdkworkAccountPcSdk`).

Recharge and product checkout use `sdkwork-payment` / `sdkwork-order` respectively; account backend-api receives ledger adjustments after payment succeeds.
