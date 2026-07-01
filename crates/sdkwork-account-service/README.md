# sdkwork-account-service

Domain: `commerce`  
Capability: `account`  
Package type: `rust-crate`  
Status: `stable`

Domain rules for wallet accounts, ledger append, points lots (FIFO), billing history queries, and account summary projections. Persistence is implemented in `sdkwork-account-repository-sqlx`; HTTP surfaces live in `sdkwork-routes-account-*-api`.

Machine-readable contract: `specs/component.spec.json`. Canonical standards: `../../../sdkwork-specs/`.

## Public API

- Domain types and query objects under `sdkwork_account_service::*`
- Consumed by repository and route crates; not an HTTP entrypoint

## Verification

```bash
cargo test --manifest-path crates/sdkwork-account-service/Cargo.toml
```
