# Account And Token Bank PRD

Status: active
Owner: SDKWork maintainers
Application: account
Updated: 2026-07-08
Specs: `DOCUMENTATION_SPEC.md`, `REQUIREMENTS_SPEC.md`, `API_SPEC.md`, `DATABASE_SPEC.md`

## 1. Background And Problem

AI applications need a durable account foundation that can be explained without a terminology ladder. The account system must support traditional commerce balances and one AI-era account that drives large language models, image generation, video generation, Agents, workflows, plugins, and model services.

`sdkwork-account` is the ledger truth source for this foundation. It owns balances, journaled ledger entries, holds, transfers, billing projections, Token Bank exchange snapshots, AI settlement references, idempotency, outbox events, and reconciliation evidence.

The product language must avoid ambiguous "token" usage:

- **Cash account** uses `cash` for fiat balances and fiat-denominated settlement display.
- **Points account** uses `points` for traditional loyalty points.
- **Token Bank account** uses `token_bank` for AI consumption, income, spending, reservation, settlement, transfer, and reconciliation.
- **Raw model usage** is not an account asset. LLM input tokens, output tokens, image count, video seconds, GPU seconds, tool calls, workflow steps, and plugin calls are metering inputs only.

SDKWork account contracts expose only `token_bank` for AI account balance. Raw model usage remains metering evidence and never becomes an account asset.

## 2. Target Users

- End users who purchase, receive, allocate, and consume Token Bank balances.
- Organization owners who manage AI budgets across users, projects, and workspaces.
- AI runtime, pricing, and metering services that reserve and settle Token Bank balances for model workloads.
- Model, plugin, Agent, workflow, and application providers that receive Token Bank income when marketplace settlement is enabled.
- Finance reviewers who audit fiat-to-Token-Bank exchange, ledger movement, refunds, income, spending, and service settlement.
- Integrators who display wallet, Token Bank, billing history, and AI consumption status through SDKs.
- Operators who publish exchange rates, perform controlled grants, run reconciliation, and investigate account incidents.

## 3. Product Taxonomy

| Account | Technical identifier | Currency code | Product meaning |
| --- | --- | --- | --- |
| Cash account | `cash` | ISO fiat code such as `CNY` or `USD` | Fiat balance, refund balance, withdrawal balance, or settlement display where product policy allows it. |
| Points account | `points` | `POINT` | Traditional points, rewards, redemptions, lots, expiry, and point compensation. |
| Token Bank account | `token_bank` | `TOKEN_BANK` | Unified AI account for model consumption, service income, spending, holds, settlement, transfer, exchange, and reconciliation. |

Rules:

- `token_bank` is both the product capability name and the account asset code.
- `token_bank` is the only AI account identifier exposed by PRD, API, SDK, database, billing, and reconciliation contracts.
- Do not introduce alternate AI account asset identifiers in product copy, APIs, SDKs, database contracts, billing, or reconciliation.
- The word "token" may appear only as part of `token_bank`, `TOKEN_BANK`, Token Bank product copy, or authentication/security contexts that are clearly not account assets.

## 4. Goals And Non-Goals

### Goals

- Provide a complete greenfield account taxonomy: cash account, points account, and Token Bank account.
- Make Token Bank the single account capability for AI consumption, income, spending, exchange, allocation, reservation, settlement, transfer, reversal, and audit.
- Support exchange rates from CNY, USD, and future fiat currencies into Token Bank balances through versioned rate configuration and immutable quote/snapshot records.
- Support AI job lifecycle accounting: budget quote, pre-hold, actual usage settlement, service income split, remainder release, failure release, and reversal.
- Keep all account amounts integer based. Cash uses fiat minor units, points use point units, and Token Bank uses Token Bank units.
- Use append-only journal and ledger semantics for every balance movement.
- Preserve capability boundaries: account records ledger truth; order orchestrates recharge, coupon redemption, refund, withdrawal, and fulfillment lifecycles; payment executes collection, refund, and payout provider channels; pricing converts raw usage to Token Bank amounts; metering records raw usage; AI runtimes execute workloads.
- Expose app-api read models and backend-api command surfaces through SDKWork v3 response envelopes and generated SDKs.

### Non-Goals

- Order lifecycle, cart, checkout, cancellation, recharge package or plan purchase, coupon redemption, refund request, withdrawal request, and fulfillment orchestration. Owner: `sdkwork-order`.
- Payment intent, provider refund execution, provider payout execution, provider webhook ingest, and acquiring channel configuration. Owner: `sdkwork-payment`.
- LLM, image, video, Agent, plugin, model-service, or workflow execution. Owner: the relevant AI runtime capability.
- Raw usage collection and model-specific pricing formulas. Owners: metering and pricing capabilities.
- Provider cost calculation, margin policy, subscription catalog, recharge package publishing, and sales campaign policy. Owners: pricing, billing, order, promotion, or subscription capabilities.
- Treating Token Bank as a login credential, blockchain asset, provider raw LLM token, or public pricing engine.

## 5. Functional Scope

### Token Bank Scope

Token Bank manages one account balance type, `token_bank`, for AI-era value movement:

- Fiat purchase: CNY, USD, and future fiat currencies exchange into Token Bank balance through published exchange rates.
- Platform grant: operations, subscriptions, onboarding, compensation, or campaigns grant Token Bank balance.
- Organization allocation: organization budgets distribute Token Bank balance to projects and users.
- AI reservation: AI tasks create holds before billable work begins.
- AI settlement: completed tasks settle final Token Bank spending after pricing converts raw usage.
- Service income: model services, plugins, Agents, workflows, and applications can receive Token Bank income.
- Release: failed, canceled, or under-budget tasks release unused holds.
- Transfer: allowed owner accounts move Token Bank balance between user, organization, project, service, and system accounts.
- Burn: pure platform consumption can settle to a system burn account instead of a service settlement account.
- Reversal: incorrect movements are corrected through append-only reverse ledger entries.
- Reconciliation: balances, holds, exchange snapshots, settlement snapshots, billing projections, and outbox state are checked against ledger truth.

### Exchange Rate Scope

Exchange rates are first-class account contracts:

- Exchange direction is fiat cash to Token Bank.
- Supported launch currencies are `CNY` and `USD`.
- Rates are configured per fiat currency, tenant scope, channel, effective window, and version.
- Quotes and completed purchases must store immutable exchange-rate snapshots.
- Later rate changes must never alter historical orders, ledger entries, billing history, or reconciliation evidence.
- Rounding is deterministic and documented on the rate. The default rounding mode is `floor`.
- All exchange calculation uses integer arithmetic: fiat minor units to Token Bank units.

### Wallet And Billing Scope

- Account summary read model.
- Account summary uses explicit traditional points fields: `availablePoints`, `monthlyPointsConsumed`, and `consumptionByService[].pointsConsumed`.
- Asset-specific read models for cash, points, and Token Bank.
- Points lots and lot allocation audit for traditional points.
- Token Bank balance, holds, income summary, spending summary, exchange quote, purchase snapshot, AI settlement snapshot, and ledger history.
- Holds list and detail across assets.
- Billing history list populated from ledger append and Token Bank settlement events.
- Backend reconciliation for account balances, points lots, Token Bank exchange snapshots, held-vs-settled AI jobs, and service income settlement.

## 6. User Scenarios

1. **Buy Token Bank balance with CNY**
   - A user requests a CNY purchase quote.
   - Token Bank returns the active CNY-to-Token-Bank rate and quote snapshot.
   - Order creates a `token_bank_recharge` order or an account recharge package order.
   - Payment collects CNY.
   - Order fulfillment calls account backend.
   - Account credits `token_bank` and records the exchange snapshot in ledger and billing history.

2. **Buy Token Bank balance with USD**
   - The same flow uses the active USD exchange-rate configuration.
   - The ledger stores fiat currency, fiat minor amount, rate numerator, rate denominator, rounding mode, and resulting Token Bank units.

3. **Run an LLM task**
   - AI runtime obtains a pricing estimate from pricing.
   - Token Bank creates a hold for the estimated budget.
   - Metering records raw input/output model usage.
   - Pricing converts raw usage to a final Token Bank amount.
   - Account settles the hold, records the AI settlement snapshot, and releases any unused remainder.

4. **Generate an image or video**
   - Raw usage may include image count, dimensions, video duration, resolution, model tier, or GPU time.
   - Pricing converts the usage to a Token Bank amount.
   - Account records only Token Bank hold, settlement, and optional service income.

5. **Allocate enterprise budget**
   - An organization buys or receives Token Bank balance.
   - An admin transfers balance from the organization budget account to project or user accounts.
   - Project AI workloads consume from the assigned budget account.

6. **Settle service income**
   - A plugin, model service, Agent, workflow, or application earns income for completed work.
   - Settlement debits the user/project account and credits a service settlement account.
   - Later conversion, withdrawal, revenue share, or payout workflow is outside this repository unless explicitly integrated through order/payment capabilities.

7. **Release failed work**
   - A task fails before billable completion.
   - The hold is released.
   - Billing history shows reservation and release audit without a consumption charge.

8. **Reverse incorrect settlement**
   - A completed charge is found incorrect.
   - Account appends a reversal entry with `reversedLedgerId`.
   - Historical ledger entries remain immutable.

9. **Refund a Token Bank recharge**
   - Order creates a `refund_request` tied to the original paid recharge order.
   - Order calls account to hold or reverse the refundable Token Bank amount before provider refund execution.
   - Payment executes the provider refund against the existing order payment.
   - Order commits account reversal on refund success or releases the account hold on provider failure.

10. **Withdraw cash balance**
    - Order creates a `cash_withdrawal` request and runs approval, risk, and state transitions.
    - Account freezes the requested `cash` balance.
    - Payment executes provider payout after order approval.
    - Order settles the account hold on payout success or releases it on payout failure.

## 7. Success Metrics

- Product, API, SDK, database, and billing documentation use `token_bank` as the only AI account identifier.
- No authored account contract exposes any alternate AI account asset identifier.
- Every balance mutation is traceable through journal, ledger, idempotency record, and optional outbox event.
- Fiat-to-Token-Bank purchases preserve immutable exchange-rate snapshots.
- AI jobs cannot start when required Token Bank holds fail for insufficient balance.
- Completed AI jobs settle actual Token Bank spending and release unused held balance.
- Service income and platform burn routing are explicit and auditable.
- Organization/project/user allocation flows are auditable.
- Reconciliation jobs can detect mismatches in balances, holds, exchange snapshots, settlement snapshots, billing projections, and outbox events.
- Generated SDKs expose resource names that read like business capabilities: `tokenBank.account.*`, `tokenBank.holds.*`, `tokenBank.exchangeRates.*`, and `wallet.points.*`.

## 8. Phases

### Phase 1 - Greenfield Account Foundation

- Define account taxonomy: `cash`, `points`, `token_bank`.
- Build integer-based account, journal, ledger, hold, transfer, idempotency, outbox, and billing projection.
- Expose wallet read models and backend ledger command APIs.
- Preserve points-specific lot and expiry behavior.

### Phase 2 - Token Bank Exchange

- Add exchange-rate configuration for CNY/USD to Token Bank.
- Add purchase quote and immutable exchange snapshot records.
- Integrate order/payment fulfillment into Token Bank credit commands.
- Add admin exchange-rate publish workflow and validation.

### Phase 3 - AI Consumption Accounting

- Add Token Bank hold, settle, release, direct debit, transfer, reversal, service income, and burn flows.
- Store usage and pricing snapshot references on settlement snapshots.
- Add spending and income summaries by user, organization, project, service, model, application, and time range.

### Phase 4 - Organization And Marketplace Flow

- Add organization/project budget accounts.
- Add service settlement accounts for model, plugin, Agent, workflow, and application providers.
- Add transfers, service revenue settlement, burn routing, reconciliation reports, and operator incident workflows.

## 9. Linked Requirements

- Local component contract: `specs/component.spec.json`
- Local Token Bank spec: `specs/TOKEN_BANK_ACCOUNT_SPEC.md`
- Commerce boundary spec: `specs/COMMERCE_BOUNDARY_SPEC.md`
- Machine integration contract: `specs/commerce-integration.spec.json`
- API authorities: `apis/app-api/account/`, `apis/backend-api/account/`
- Database contract: `database/contract/schema.yaml`
- Database baseline: `database/ddl/baseline/{postgres,sqlite}/0001_account_baseline.sql`
- Global standards: `sdkwork-specs/API_SPEC.md`, `DATABASE_SPEC.md`, `DOCUMENTATION_SPEC.md`, `REQUIREMENTS_SPEC.md`, `SUBJECT_ID_SPEC.md`

## 10. Product Decisions

- Token Bank is a single concept. It does not manage a second AI account asset.
- `token_bank` is the account asset code, API namespace, SDK resource namespace, operation prefix, and database classification value for AI account balance.
- `TOKEN_BANK` is the currency code for Token Bank account rows.
- Raw model tokens remain raw usage and must be converted by pricing before account settlement.
- Token Bank may support purchase, grant, income, spending, transfer, hold, settlement, release, burn, and reversal, but all of those are operations on the same `token_bank` account balance.

## 11. Resolved Ownership Decisions

- Pricing ownership: a dedicated pricing capability owns public rate cards, model pricing formulas, raw-usage conversion, and `pricingSnapshotId`. Account receives the final Token Bank amount and stores pricing snapshot references only.
- Metering ownership: a metering capability owns raw usage collection for LLM, image, video, Agent, workflow, plugin, and model-service activity. Account stores `usageSnapshotId` references and never copies raw provider payloads.
- Exchange-rate publishing: account owns Token Bank exchange-rate storage and audit. Global backend-admin operators may publish global rates, and tenant backend-admin operators may publish tenant-scoped rates when permissions allow it. Every publish action is idempotent, versioned, and audited.
- Purchase reversal: the default account behavior is append-only Token Bank reversal or account compensation. Fiat refunds and provider settlement refunds are orchestrated by order/payment; account only records the ledger effect after the owning capability calls the backend API.
- Marketplace settlement scope: the first Token Bank marketplace release includes service settlement accounts, service income ledger entries, settlement snapshots, and burn routing. Revenue share policy, tax, payout execution, and fiat withdrawal remain outside this repository.
- Account value order ownership: `sdkwork-order` is the only business orchestrator for recharge packages, Token Bank plan purchase or renewal, coupon redemption, refund requests, and withdrawal requests. `sdkwork-payment` executes provider collection, refund, and payout only; it must not call account ledger APIs directly. `sdkwork-account` exposes idempotent ledger, hold, settlement, and reversal commands consumed by order.
