# Account PRD



Status: active

Owner: SDKWork maintainers

Application: account

Updated: 2026-06-29



## 1. Background And Problem



Wallet balances, ledger entries, account summaries, and billing history must be append-only, version-guarded, tenant-isolated, and aligned with SDKWork L3 database and v3 API envelopes.



`sdkwork-account` is a T1 commerce **ledger** capability: persistence, domain services, HTTP routers, gateway, and a read-focused PC wallet surface.



## 2. Target Users



Account holders, finance reviewers, and integrators displaying wallet or billing history.



## 3. Goals And Non-Goals



### Goals



- Own account/billing SQL and wallet/billing HTTP routers with `SdkWorkApiResponse`.

- Enforce ledger append-only semantics, idempotency replay, and optimistic balance versioning.

- Expose read models on app-api and ledger write commands on backend-api.



### Non-Goals



- **Order headers** (`commerce_order` create/list/cancel) — owned by `sdkwork-order`.

- **Payment provider execution**, refunds, payout — owned by `sdkwork-payment`.

- **Recharge packages and recharge order API** — owned by `sdkwork-order` (not account).

- Promotion point exchange rules — owned by promotion capability.



Recharge and withdrawal are **not** implemented inside account. Every recharge creates a unified `commerce_order` (`subject=points_recharge`) in **sdkwork-order**; payment collects funds; account receives backend adjustments only after payment succeeds.



## 4. Scope



- Account summary read model (app-api).

- **Separate asset read models**: cash account, points account (with lot stats), token account.

- Points ledger list, points lot list, holds list, and wallet overview aggregates.

- Billing history list (app-api).

- Asset-scoped wallet adjustments / ledger append (backend-api).

- Hold lifecycle (create / settle / release) and inter-account transfers (backend-api write; app-api read).



Primary API prefixes:



- App: `/app/v3/api/wallet`, `/app/v3/api/billing`, `/app/v3/api/accounts`

- Backend: `/backend/v3/api/wallet`



### Asset separation principle



Cash and points are **never merged in one HTTP resource**. Each asset has dedicated retrieve/list routes and DTO field names (`availableAmount` vs `availablePoints`). Shared tables remain unified at persistence layer (`commerce_account.asset_code`) for atomic journal integrity.



### Commerce integration principle



| Step | Owner |

| --- | --- |

| Create order (product / virtual / recharge) | `sdkwork-order` (writes `commerce_order`) |

| Collect payment / refund | `sdkwork-payment` |

| Credit or debit ledger | `sdkwork-account` backend-api (`adjustments`, holds, transfers) |

| Display balances / holds / ledger | `sdkwork-account` app-api + PC wallet |



## 5. User Scenarios



- A user views wallet accounts, holds, and ledger history scoped to their tenant identity.

- A backend service posts a credit through `POST /backend/v3/api/wallet/adjustments` with idempotency key replay protection (typically after payment success).

- Checkout creates a hold on account, payment settles or releases based on outcome.

- A user taps **Recharge** in account PC → navigates to payment checkout → payment creates `commerce_order` → on paid, account credits points.



## 6. Success Metrics



- Repository tests validate version-guarded balance updates, idempotency replay, hold lifecycle, and transfers.

- App and backend routes serialize success via `sdkwork-utils-rust` envelopes.

- PC wallet loads overview, holds, and ledger through the v3 account SDK; recharge uses `@sdkwork/order-service`; withdraw delegates to payment payout routes.



## 7. Phases



- **Phase 1 (complete):** L3 schema + repository + app/backend routes + generated TypeScript SDKs + PC bootstrap + holds list UI + commerce delegation for recharge/withdraw.

- **Phase 2 (composition apps):** Mall/shell apps wire `@sdkwork/payment-pc-payment` checkout and post-pay account refresh; payment saga calls account backend adjustments.



## 8. Linked Requirements



Governance: `sdkwork-specs/REQUIREMENTS_SPEC.md` (REQ traceability for this PRD).



- Component contract: `specs/component.spec.json`

- Machine contracts: `apis/`, `database/contract/schema.yaml`

- Related capabilities: `sdkwork-order`, `sdkwork-payment`

- Standards: `sdkwork-specs/API_SPEC.md`, `DATABASE_SPEC.md`, `SUBJECT_ID_SPEC.md`



## 9. Open Questions



- Whether dedicated admin-only list/search routes belong in backend-api before first production tenant onboarding.


