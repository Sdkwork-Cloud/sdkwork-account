# SDKWork Commerce Contract Component Specs

Local standards index for `sdkwork_contract_service`. Root SDKWork standards remain authoritative and must not be contradicted here.

## Component

| Field | Value |
| --- | --- |
| Name | `sdkwork-contract-service` |
| Type | `rust-crate` |
| Root | `crates/sdkwork-contract-service` |
| Domain | `commerce` |
| Capability | `contract` |
| Languages | `rust` |
| Status | `stable` |

## Contract Manifest

- [component.spec.json](./component.spec.json) — machine-readable component contract
- Shared commerce domain types, amounts, asset codes, and service errors

## Verification

```bash
cargo test --manifest-path crates/sdkwork-contract-service/Cargo.toml
```
