# calmui

[English](./README.md) | 简体中文

`calmui` 是一个基于 GPUI 的 Rust UI 组件库。

## 当前状态

`calmui` 当前主要用于 PushGo 生态内部项目：

- 使用范围：PushGo 相关仓库与项目自用
- 成熟度：尚未经过大面积、长周期的外部生产场景验证

如果用于 PushGo 之外的生产环境，请按早期软件处理，并先完成充分验证。

## 项目范围

- 可复用 UI 组件：`src/components`
- 主题 / 设计 Token / 契约层：`src/theme`、`src/tokens.rs`、`src/contracts.rs`
- 表单模型与派生宏：`src/form`、`crates/calmui_form_derive`
- 可选 i18n 能力：`i18n` feature

## 工具链

- Rust stable（edition 2024）
- `Cargo.toml` 中固定 commit 的 GPUI 依赖

## Cargo Feature

- `i18n`：通过 `sys-locale` 启用运行时语言环境识别
- `extend-icon`：启用图标扩展相关能力

## 基本验证

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## 许可证

[MIT License](./LICENSE)
