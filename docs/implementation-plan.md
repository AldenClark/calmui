# calmui 实施计划（v0.1）

## 1. 目标与边界

- 目标：构建基于 Rust + GPUI 的上层组件库，强调可复用、语义化 token、统一 builder API。
- 边界：组件库不承载业务语义，只提供通用 UI 能力和可组合机制。
- 风格：统一采用 `struct + impl + impl trait`，避免散装工具函数风格。

## 2. 全局硬约束

- 使用 GPUI 仓库最新版本（git dependency），优先复用 `Render` / `RenderOnce` / `Styled` trait。
- 颜色基础值采用主流成熟库的完整默认色板（14 组，10 阶），默认主色为 `blue`。
- 命名保持中性：类型、方法、变量、token key 不包含来源库品牌字样。
- 组件内部禁止硬编码视觉值，必须通过语义 token 解析。
- 状态型组件必须遵循受控 / 非受控双模式，规则见 `docs/component-state-principles.md`。

## 3. 分阶段推进

### 阶段 1：内核与协议（当前阶段）

- `tokens`：原始色板 + 尺寸/圆角/字体/动效基表。
- `theme`：语义 token + 组件 token + 覆盖合并（全局/局部/组件/实例）。
- `style`：variant/size/radius/state/layout 的统一模型。
- `motion`：可替换过渡预设、时长和缓动策略。
- `feedback`：`ToastManager` 生命周期协议和队列策略。
- `overlay`：`ModalManager` 生命周期协议和栈策略。
- `icon`：内置图标包 + 用户注册图标包。
- `provider`：统一注入入口，聚合 theme/motion/icon/manager。

### 阶段 2：基础组件层

- `FieldShell`：统一 `label/description/error/required/layout/slots`。
- `InputCore` + `Loader` + `Icon` 的复用链路。
- 第一批可用组件：`TextInput`、`PasswordInput`、`PinInput`、`Button`、`LoadingOverlay`。

### 阶段 3：浮层和反馈组件落地

- 管理器驱动的 `Modal`、`Toast`、`Drawer`、`Popover`。
- 统一层级、焦点、关闭策略、动画覆盖和无障碍约束。

### 阶段 4：高级组件和布局系统

- 组合型组件与布局组件。
- 扩展库模式（按场景拆包）。

### 阶段 5：稳定化与发布

- API 稳定性分级（stable/beta/experimental）。
- 测试矩阵（行为、状态、主题覆盖、视觉回归）。
- 文档站和示例工程。

## 4. 第一阶段验收标准

- token / theme / style / motion / provider 模块可编译。
- 默认主色为 `blue`，并支持一键切换强调色。
- 主题覆盖支持分层合并和局部覆盖。
- `ToastManager` / `ModalManager` 支持 `open/show`、`update`、`close`、`close_all`。
- 代码组织符合 Rust builder + trait 化风格。

## 5. 风险与处理

- GPUI 为 pre-1.0，存在 API 变动风险：通过 trait 桥接层降低冲击面。
- 跨组件样式一致性风险：通过统一 `style context` 与 `component token` 约束规避。
- 功能扩张导致 API 复杂化风险：坚持默认最小 API、进阶能力渐进暴露。
