# Account Technical Architecture



Status: active

Owner: SDKWork maintainers

Updated: 2026-06-29



## 1. Architecture Overview



`sdkwork-account` is the commerce **ledger / wallet read model** capability. It owns L3 account persistence, domain services, HTTP route crates, standalone gateway, and the PC wallet surface.



It does **not** own order headers, payment intents, or provider webhooks. Those live in `sdkwork-order` and `sdkwork-payment`.



## Capability stack



| Layer | Path |

| --- | --- |

| Database contract + migrations | `database/` |

| Domain + ports | `crates/sdkwork-account-service/` |

| Repository (sqlx) | `crates/sdkwork-account-repository-sqlx/` |

| App + backend HTTP routers | `crates/sdkwork-routes-account-*-api/` |

| Gateway assembly | `crates/sdkwork-account-gateway-assembly/` |

| API server | `crates/sdkwork-account-standalone-gateway/` |

| PC client | `apps/sdkwork-account-pc/` |

| Client facade | `apps/sdkwork-account-common/packages/sdkwork-account-service/` |

| Generated SDK families | `sdks/sdkwork-account-app-sdk/`, `sdks/sdkwork-account-backend-sdk/` |



## Commerce dependency graph



```text

                    ┌─────────────────┐

                    │  sdkwork-order  │  commerce_order (product, virtual, checkout)

                    └────────┬────────┘

                             │ orders.pay

                             ▼

┌──────────────┐     ┌─────────────────┐     ┌──────────────────┐

│ account PC   │────▶│ sdkwork-payment │────▶│ sdkwork-account  │

│ (read + nav) │ nav │ recharge/refund │ adj │ ledger backend   │

└──────────────┘     │ payout          │     └──────────────────┘

                     └─────────────────┘

```



- **Recharge:** order creates `commerce_order` (`subject=points_recharge`), then `orders.pay` creates payment intent; payment collects; order fulfillment saga credits account via backend adjustments.

- **Product / virtual pay:** order creates header → payment collects → account settle/adjust.

- **Refund:** payment refund references `order_id`; account reversal via idempotent backend adjustment.



## Data model (L3)



Greenfield schema (`commerce_*` tables owned by account capability). **Cash、积分、Token 在持久层按 `asset_code` 分账户行存储，在 API 层按资源拆分，不混用 DTO。**



| Asset | `asset_code` | 余额语义 | 批次 |

| --- | --- | --- | --- |

| Cash | `cash` | 法币 `available` / `frozen` / `pending` | — |

| Points | `points` | 整数积分余额 | `commerce_points_lot` FIFO |

| Token | `token` | 整数 Token 余额 | — |



- `commerce_account` — 按资产分行的钱包账户 + 乐观锁版本

- `commerce_account_ledger` — 只追加流水，含 `balance_before` / `balance_after`

- `commerce_account_journal` + `commerce_account_journal_line` — 复式记账基础

- `commerce_points_lot` — 积分批次（入账创建 lot，出账 FIFO 扣减）

- `commerce_account_hold` / `commerce_account_transfer` — 预扣（available→frozen）与账户间转账

- `commerce_idempotency_record` — 写操作幂等

- `commerce_outbox_event` — 领域 outbox（ledger append 写入 `account.ledger_appended`）

- `commerce_billing_history` — 用户可见账单投影



`commerce_order` and payment tables are **not** owned by this repository (see `sdkwork-order`, `sdkwork-payment`).



Identifiers: internal `BIGINT` snowflake IDs + external `uuid VARCHAR(64)` for API surfaces. Subject scope uses `tenant_id` / `owner_id` / `organization_id` as `BIGINT` (`organization_id = 0` for tenant-level).



## API ownership



- App API (read-only wallet + billing): `/app/v3/api/wallet`, `/app/v3/api/billing`, `/app/v3/api/accounts`

- Backend API (ledger writes + ops): `/backend/v3/api/wallet`

- Success envelope: `SdkWorkApiResponse` `{ code: 0, data, traceId }` per `API_SPEC.md`

- Errors: HTTP 4xx/5xx `application/problem+json` with numeric `code` + `traceId`



Implemented app routes:



- `GET /app/v3/api/accounts/current/summary`

- `GET /app/v3/api/wallet/overview`

- `GET /app/v3/api/wallet/accounts` (all assets, optional `assetType`)

- `GET /app/v3/api/wallet/accounts/cash`

- `GET /app/v3/api/wallet/accounts/points` (includes lot stats: `activeLotCount`, `expiringPoints`)

- `GET /app/v3/api/wallet/accounts/tokens`

- `GET /app/v3/api/wallet/ledger_entries` (all assets, optional `assetType`)

- `GET /app/v3/api/wallet/ledger_entries/cash`

- `GET /app/v3/api/wallet/ledger_entries/points`

- `GET /app/v3/api/wallet/points/lots`

- `GET /app/v3/api/wallet/holds` (optional `accountId`, `assetType`, `status`)

- `GET /app/v3/api/wallet/holds/{holdId}`

- `GET /app/v3/api/wallet/ledger_entries/{ledgerEntryId}`

- `GET /app/v3/api/billing/history`



Implemented backend routes:



- `GET /backend/v3/api/wallet/health`

- `POST /backend/v3/api/wallet/adjustments` (generic; `assetType` required in body)

- `POST /backend/v3/api/wallet/adjustments/cash`

- `POST /backend/v3/api/wallet/adjustments/points` (writes ledger + points lot FIFO)

- `POST /backend/v3/api/wallet/adjustments/tokens`

- `POST /backend/v3/api/wallet/holds`

- `POST /backend/v3/api/wallet/holds/{holdId}/settle`

- `POST /backend/v3/api/wallet/holds/{holdId}/release`

- `POST /backend/v3/api/wallet/transfers`



## PC surface



```text

apps/sdkwork-account-pc/

  packages/sdkwork-account-pc-core/

  packages/sdkwork-account-pc-shell/     ← SDK bootstrap + theme + commerce navigation

  packages/sdkwork-account-pc-wallet/    ← balances, holds, ledger; recharge/withdraw delegate

```



Wallet UI consumes `@sdkwork/account-app-sdk` through `@sdkwork/account-service`.



- **Read path:** `bootstrapSdkworkAccountPcSdk()` → `wallet.*` app-api methods.

- **Recharge / withdraw:** recharge packages and `recharges.orders.create` via `@sdkwork/order-service`; payment checkout for collection (`rechargeFlow` / `payoutFlow` = `checkout` + `onNavigate`). Withdraw and payout settlement remain in `sdkwork-payment`.

- **Backend integrators:** `bootstrapSdkworkAccountPcBackendSdk()` for adjustments, holds, transfers.



Environment (shell):



- `VITE_SDKWORK_ACCOUNT_API_BASE` — account gateway origin

- `VITE_SDKWORK_PAYMENT_CHECKOUT_BASE` — recharge checkout path (default `/checkout`)

- `VITE_SDKWORK_PAYMENT_PAYOUT_BASE` — payout path (default `/payments/payout`)



## Verification



```powershell

cd E:\sdkwork-space\sdkwork-account

pnpm verify

cargo test --workspace

node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .

pnpm db:validate

```



## Related docs



Governance: `sdkwork-specs/ARCHITECTURE_DECISION_SPEC.md` (ADR index: `docs/architecture/decisions/`).



- Product scope: `docs/product/prd/PRD.md`

- **Module specs:** `specs/README.md`, `specs/COMMERCE_BOUNDARY_SPEC.md`

- Integrator guide: `docs/guides/integrator/README.md`

- Database contract: `database/contract/schema.yaml`

- API authorities: `apis/app-api/account/`, `apis/backend-api/account/`


