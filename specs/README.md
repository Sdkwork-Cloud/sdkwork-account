# sdkwork-account Component Specs

Local specification index for the account, wallet, and Token Bank capability. Root SDKWork standards remain authoritative (`../../sdkwork-specs/`).

## Spec Map

| Document | Purpose |
| --- | --- |
| [component.spec.json](./component.spec.json) | Workspace component manifest. |
| [COMMERCE_BOUNDARY_SPEC.md](./COMMERCE_BOUNDARY_SPEC.md) | Ledger ownership, forbidden scope, dependency direction, and commerce capability boundaries. |
| [TOKEN_BANK_ACCOUNT_SPEC.md](./TOKEN_BANK_ACCOUNT_SPEC.md) | Token Bank account taxonomy, exchange-rate, accounting, API naming, database, and AI settlement rules. |
| [commerce-integration.spec.json](./commerce-integration.spec.json) | Machine-readable integration contract and rollout phases. |
| [../apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/specs/README.md](../apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/specs/README.md) | Wallet PC package: UI + SDK injection. |

## Required Account Language

| Product concept | Technical term |
| --- | --- |
| Cash account | `cash` |
| Points account | `points` |
| Token Bank account and AI balance | `token_bank` |
| Token Bank currency code | `TOKEN_BANK` |
| Raw LLM provider tokens | raw usage, not an account asset |

Do not introduce alternate account asset identifiers outside `cash`, `points`, and `token_bank`.

## Related Capability Specs

| Repository | Spec entry |
| --- | --- |
| `sdkwork-order` | Unified orders, checkout, and Token Bank purchase order lifecycle. |
| `sdkwork-payment` | Payment intent, provider refund execution, provider webhook handling, and future provider payout executor boundary. |
| Pricing/metering capability | Raw AI usage and Token Bank conversion snapshots. |

## Verification

```powershell
pnpm verify
cargo test --workspace
node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .
```

Before claiming Token Bank alignment complete, confirm API contracts, SDK facades, PC wallet copy, database design, and service names use `token_bank` rather than forbidden account asset names.
