# Token Bank Implementation Alignment Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align the account implementation, API contracts, database contracts, SDK surfaces, PC wallet service, and documentation to the single `token_bank` account concept.

**Architecture:** Account remains the commerce ledger truth source. Token Bank is represented by one account asset code (`token_bank`) and one currency code (`TOKEN_BANK`) for AI consumption, income, spending, reservation, settlement, transfer, exchange, and reconciliation. Raw model/provider token usage is metering data outside this repository and must not become an account asset.

**Tech Stack:** Rust service crates, SQLx repository crates, Rust route crates, OpenAPI 3.1 SDKWork v3 contracts, generated TypeScript SDKs, React/TypeScript PC wallet packages, SDKWork validation tools.

---

## Scope

- Replace any account asset naming that still exposes an alternate AI account identifier.
- Preserve authentication/security token wording where it clearly refers to SDKWork credentials.
- Keep account as ledger truth source only; do not implement order creation, payment provider execution, payout settlement, model execution, model metering, or AI pricing formulas here.
- Use integer smallest-unit amounts for account ledger values; no decimal account amount storage.
- Keep generated SDK output derived from OpenAPI and generator inputs; do not hand-edit generated transport files.

## Files

- Modify: `crates/sdkwork-contract-service/src/lib.rs`
- Modify: `crates/sdkwork-contract-service/tests/commerce_core_standard.rs`
- Modify: `crates/sdkwork-account-service/src/domain/mod.rs`
- Modify: `crates/sdkwork-account-service/src/service/mod.rs`
- Modify: `crates/sdkwork-account-service/tests/account_standard.rs`
- Modify: `crates/sdkwork-account-repository-sqlx/src/store/mod.rs`
- Modify: `crates/sdkwork-account-repository-sqlx/src/postgres_account.rs`
- Modify: `crates/sdkwork-account-repository-sqlx/src/sqlite_account.rs`
- Modify: `crates/sdkwork-account-repository-sqlx/src/postgres_hold.rs`
- Modify: `crates/sdkwork-account-repository-sqlx/src/sqlite_hold.rs`
- Modify: `crates/sdkwork-routes-account-app-api/src/account_router.rs`
- Modify: `crates/sdkwork-routes-account-backend-api/src/wallet_router.rs`
- Modify: `crates/sdkwork-routes-account-backend-api/src/hold_router.rs`
- Modify: `apis/app-api/account/account-app-api.openapi.json`
- Modify: `apis/backend-api/account/account-backend-api.openapi.json`
- Modify: `apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/src/wallet-service.ts`
- Modify: `apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/src/components/wallet-balance-panel.tsx`
- Modify: `apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/tests/wallet.service.test.ts`
- Regenerate when feasible: `sdks/**/generated/**`

## Task 1: Core Contract Types

- [x] Step 1: Update `crates/sdkwork-contract-service/tests/commerce_core_standard.rs` to assert `CommerceAccountAssetType::TokenBank.as_str() == "token_bank"` and reject decimal `CommerceMoney` values such as `"19.90"`.
- [x] Step 2: Run `cargo test -p sdkwork-contract-service commerce_core_standard -- --nocapture`; expected result before implementation is failure from missing `TokenBank` and decimal validation drift.
- [x] Step 3: Use the `TokenBank` AI account enum variant in `crates/sdkwork-contract-service/src/lib.rs`; return `token_bank` from `as_str`.
- [x] Step 4: Change `CommerceMoney::new` semantics to non-negative integer smallest-unit strings.
- [x] Step 5: Rename ledger business constants to Token Bank wording.
- [x] Step 6: Re-run `cargo test -p sdkwork-contract-service commerce_core_standard -- --nocapture`; expected result is pass.

## Task 2: Account Service Model And Service Contract

- [x] Step 1: Update `crates/sdkwork-account-service/tests/account_standard.rs` to expect `summary.token_bank` and Token Bank operation names.
- [x] Step 2: Run `cargo test -p sdkwork-account-service account_standard -- --nocapture`; expected result before implementation is failure from old field/operation names.
- [x] Step 3: Rename the AI account summary field to `token_bank` in `crates/sdkwork-account-service/src/domain/mod.rs`.
- [x] Step 4: Update `account_service_contract()` in `crates/sdkwork-account-service/src/service/mod.rs` to expose Token Bank operations.
- [x] Step 5: Re-run `cargo test -p sdkwork-account-service account_standard -- --nocapture`; expected result is pass.

## Task 3: SQLx Repository Mapping

- [x] Step 1: Add or update repository tests covering `token_bank` asset and `TOKEN_BANK` currency mapping where local test support exists.
- [x] Step 2: Run the narrow repository test or `cargo test -p sdkwork-account-repository-sqlx`; expected result before implementation may fail on old mappings.
- [x] Step 3: Update repository asset-code parsers and currency mappings to accept only `token_bank` for the AI account asset.
- [x] Step 4: Reject forbidden AI account asset aliases because this pre-launch application has no production consumers.
- [x] Step 5: Re-run `cargo test -p sdkwork-account-repository-sqlx`; expected result is pass or a concrete follow-up failure to debug systematically.

## Task 4: Rust Route Contracts

- [x] Step 1: Update route tests or compile-time expectations for app and backend route crates to use Token Bank paths and `CommerceAccountAssetType::TokenBank`.
- [x] Step 2: Run `cargo test -p sdkwork-routes-account-app-api -p sdkwork-routes-account-backend-api`; expected result before implementation may fail on old route names.
- [x] Step 3: Replace app AI-account token route naming with Token Bank route naming.
- [x] Step 4: Replace backend AI-account token adjustment route naming with Token Bank credit/debit/grant/reversal route naming.
- [x] Step 5: Reject forbidden AI account asset aliases in route parsers.
- [x] Step 6: Re-run route crate tests; expected result is pass or a concrete follow-up failure to debug systematically.

## Task 5: OpenAPI And SDK Generation

- [x] Step 1: Update app and backend OpenAPI contracts to remove forbidden AI account token asset enum values and resource names.
- [x] Step 2: Add or rename schemas so Token Bank response types use `TokenBank`, `tokenBank`, and `token_bank` where each naming layer requires it.
- [x] Step 3: Run `node ..\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace .` and `node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .`; expected result is pass after contract edits.
- [x] Step 4: Run `pnpm.cmd run sdk:generate` if the repository generator setup is present and functional.
- [x] Step 5: Do not hand-edit generated SDK transport files if generation fails; document the exact generator failure and fix the generator input or script.

## Task 6: PC Wallet Service And UI

- [x] Step 1: Update PC service tests to expect Token Bank names and account fields.
- [x] Step 2: Run the narrow PC wallet test command from package scripts.
- [x] Step 3: Update service mapping and UI labels to display Token Bank without introducing forbidden AI account asset wording.
- [x] Step 4: Ensure services consume composed SDK packages only.
- [x] Step 5: Re-run the narrow PC wallet tests and `node ..\sdkwork-specs\tools\check-app-sdk-consumer-imports.mjs --workspace .`.

## Task 7: Final Verification Loop

- [x] Step 1: Run `pnpm.cmd run db:validate`.
- [x] Step 2: Run `node ..\sdkwork-specs\tools\check-repository-docs-standard.mjs --root .`.
- [x] Step 3: Run `node ..\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace .`.
- [x] Step 4: Run `node ..\sdkwork-specs\tools\check-api-response-envelope.mjs --workspace .`.
- [x] Step 5: Run `node ..\sdkwork-specs\tools\check-pagination.mjs --workspace .`.
- [x] Step 6: Run `node ..\sdkwork-specs\tools\check-app-sdk-consumer-imports.mjs --workspace .`.
- [x] Step 7: Run `cargo test --workspace`.
- [x] Step 8: Run `pnpm.cmd verify`.
- [x] Step 9: Run a targeted scan for forbidden account asset wording and classify any remaining hits as either valid auth/security references or documentation warnings.

## Acceptance

- [x] `token_bank` is the only AI account asset code in authored source contracts and implementation.
- [x] `TOKEN_BANK` is the only Token Bank currency code.
- [x] `cash`, `points`, and `token_bank` are the only account asset categories.
- [x] Raw provider/model token usage is never treated as an account asset.
- [x] API inputs/outputs follow SDKWork v3 envelopes and operation semantics.
- [x] List/search APIs remain paginated and bounded.
- [x] PC wallet wording is clear and does not expose duplicated Token Bank concepts.
- [x] Verification commands above pass or have documented external/tooling blockers with exact evidence.

## Execution Evidence

- `pnpm.cmd run check:app-composition` passes after adding PC core permission composition for `sdkwork-account-app-sdk` and `sdkwork-iam-app-sdk`.
- `pnpm.cmd verify` passes, including TypeScript typecheck, Vitest, `cargo test --workspace`, app composition, pagination, and API envelope checks.
- `pnpm.cmd run db:validate` passes.
- `node ..\sdkwork-specs\tools\check-repository-docs-standard.mjs --root .` passes.
- `node ..\sdkwork-specs\tools\check-api-operation-patterns.mjs --workspace .` passes.
- `node ..\sdkwork-specs\tools\check-app-sdk-consumer-imports.mjs --workspace .` passes.
- `apps/sdkwork-account-pc/packages/sdkwork-account-pc-core/tests/account-service-envelope.test.ts` verifies account-service accepts only SDKWork v3 numeric success envelopes and rejects non-v3 `code` / `msg` / bare list payload shapes.
- `pnpm.cmd verify` passes after the 2026-07-08 envelope hardening loop with 12 Vitest files and 27 tests.
- Account summary fields use explicit points wording and integer-string amounts: `availablePoints`, `monthlyPointsConsumed`, and `consumptionByService[].pointsConsumed`; `availableCredits` and `monthlyConsumption` are rejected by static OpenAPI tests.
- `pnpm.cmd test:node`, `cargo test -p sdkwork-account-service account_summary_snapshot_uses_points_terms_and_integer_strings -- --nocapture`, and `cargo test -p sdkwork-routes-account-app-api -- --nocapture` pass after the account-summary naming hardening loop.
- `pnpm.cmd run sdk:generate:app` regenerated the app SDK from OpenAPI, followed by app SDK generated output `publish-core` check and build.
- `pnpm.cmd install --lockfile-only --frozen-lockfile` passes. Full `pnpm.cmd install --frozen-lockfile` reached the 300 second environment timeout after reporting an up-to-date lockfile and reused packages; `pnpm.cmd verify` and `pnpm.cmd build` were rerun afterward and pass.
- Forbidden Token Bank SDK/resource alias scan returns no matches in runtime, API, UI, or SDK facade resource names.
- Remaining alternate asset-code mentions are confined to `specs/TOKEN_BANK_ACCOUNT_SPEC.md`, `specs/commerce-integration.spec.json`, and `database/contract/schema.yaml` forbidden-code guardrails.
