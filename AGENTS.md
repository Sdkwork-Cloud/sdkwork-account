# Repository Guidelines

## SDKWORK Soul

Read `../sdkwork-specs/SOUL.md` before executing tasks in this root.

## Capability Identity

- Domain: `commerce`
- Capability: `account`
- PC surface: `apps/sdkwork-account-pc/`
- Table prefix: `commerce_`
- App API prefix: `/app/v3/api/wallet`
- Backend API prefix: `/backend/v3/api/wallet`

## Verification

```bash
cargo test --workspace
pnpm install && pnpm verify
```

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)
