# SDKWork Account Component Specs

Local standards index for `sdkwork_account_service`. Root SDKWork standards remain authoritative and must not be contradicted here.

## Component

| Field | Value |
| --- | --- |
| Name | `sdkwork-account-service` |
| Type | `rust-crate` |
| Root | `crates/sdkwork-account-service` |
| Domain | `commerce` |
| Capability | `account` |
| Languages | `rust` |
| Status | `stable` |

## Contract Manifest

- [component.spec.json](./component.spec.json) — machine-readable component contract
- Consumers integrate through public exports and declared runtime entrypoints only

## Verification

```bash
cargo test --manifest-path crates/sdkwork-account-service/Cargo.toml
```
