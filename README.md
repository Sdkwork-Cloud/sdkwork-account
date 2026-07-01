# sdkwork-account
repository-kind: application

SDKWork commerce **account / wallet** capability repository (domain `commerce`, capability `account`).

- Standards: `../sdkwork-specs/README.md`
- Domain service: `crates/sdkwork-account-service/`
- Repository (sqlx): `crates/sdkwork-account-repository-sqlx/`
- HTTP routers: `crates/sdkwork-routes-account-app-api/`, `crates/sdkwork-routes-account-backend-api/`
- Gateway: `crates/sdkwork-account-standalone-gateway/`
- PC surface: `apps/sdkwork-account-pc/`
- API authorities: `apis/app-api/account/`, `apis/backend-api/account/`

## Quick start

```powershell
pnpm install
pnpm verify
cargo test --workspace
```

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

## Application Roots

- [apps directory index](apps/README.md)
