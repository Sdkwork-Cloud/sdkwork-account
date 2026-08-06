# Account database module

Managed by `sdkwork-database` lifecycle SPI. Table prefix: `acct_`.

`acct_` is the physical database prefix for account/accounting-owned tables. The capability remains `commerce.account`; broad commerce-owned tables such as order-owned `commerce_order` stay outside this module.

## Initialization State

This module is in initialization state for greenfield deployments:

1. Baseline: `database/ddl/baseline/{engine}/0001_account_baseline.sql` contains the full DDL snapshot.
2. Migrations: `database/migrations/{engine}/` is reserved for post-GA incremental schema changes only.
3. Drift: run `pnpm db:drift:check` before release.

## Account Taxonomy

| Account | `asset_code` | `currency_code` |
| --- | --- | --- |
| Cash | `cash` | ISO fiat code such as `CNY` or `USD` |
| Points | `points` | `POINT` |
| Token Bank | `token_bank` | `TOKEN_BANK` |

Database contracts expose only `cash`, `points`, and `token_bank` account asset codes.

## Amount Strategy

All balances, ledger movements, holds, exchange results, and settlement amounts use integer smallest units:

- Postgres: `BIGINT`
- SQLite: `BIGINT` with explicit application-generated ids, not rowid auto allocation
- HTTP/TypeScript SDK boundary: int64 strings

Floating-point account arithmetic is forbidden.

## Token Bank Tables

- `acct_token_bank_exchange_rate`
- `acct_token_bank_exchange_quote`
- `acct_token_bank_exchange_snapshot`
- `acct_token_bank_settlement_snapshot`

These tables support fiat exchange, immutable purchase evidence, AI spending, service income, burn routing, and reconciliation.

## Initialization state

This module is in **initialization state** for greenfield deployments:

1. **Baseline** — `database/ddl/baseline/{engine}/0001_account_baseline.sql` contains the full DDL snapshot.
2. **Migrations** — `database/migrations/{engine}/` is reserved for post-GA incremental schema changes only. It is intentionally empty at initialization.
3. **Drift** — run `pnpm db:drift:check` before release.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
