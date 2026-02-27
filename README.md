# calmui

English | [简体中文](./README.zh-CN.md)

`calmui` is a Rust UI component library built on top of GPUI.

## Status

`calmui` is currently intended mainly for internal PushGo usage:

- Scope: PushGo-related projects and repositories
- Maturity: not broadly validated with large-scale external production workloads

If you plan to use this outside PushGo, treat it as early-stage software and validate carefully in your environment.

## Scope

- Reusable UI components: `src/components`
- Theme/token/contract layers: `src/theme`, `src/tokens.rs`, `src/contracts.rs`
- Form model and derive macro: `src/form`, `crates/calmui_form_derive`
- Optional i18n capability: feature flag `i18n`

## Toolchain

- Rust stable (edition 2024)
- GPUI dependency pinned by commit in `Cargo.toml`

## Cargo Features

- `i18n`: enables locale detection support via `sys-locale`
- `extend-icon`: enables icon extension-related capability

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## License

[MIT License](./LICENSE)
