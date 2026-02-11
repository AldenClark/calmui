# 组件状态原则（受控 / 非受控）

## 1. 目标

- 所有“有交互状态”的组件，默认同时支持受控和非受控。
- 非受控模式开箱可用，不强依赖外部回调。
- 受控模式保持 Rust builder 风格与语义清晰性。

## 2. 统一状态模型

- 受控输入：
  - `value(...)` / `checked(...)` / `opened(...)` / `values(...)`
- 非受控初始值：
  - `default_value(...)` / `default_checked(...)` / `default_opened(...)` / `default_values(...)`
- 状态变更回调：
  - `on_change(...)` / `on_open_change(...)`

约定：

- 受控值优先级最高；
- 未提供受控值时，组件读取内部状态；
- 内部状态不存在时，使用 `default_*` 初始化。

## 3. 行为约定

- 非受控模式：
  - 组件内部先写状态，再 `window.refresh()`，最后触发回调（如存在）。
- 受控模式：
  - 组件不写内部状态，只触发回调，请求外部更新。
- 受控模式下若无回调：
  - 组件表现为只读状态（这是预期行为）。

## 4. 存储约定

- 内部状态统一使用 `components::control`。
- 状态键格式：`{component_id}::{slot}`，例如：
  - `text-input::value`
  - `select::opened`
  - `checkbox-group::values`
- 每个组件实例必须有稳定 `id`，支持通过 `WithId::with_id(...)` 覆盖。

## 5. 新组件接入清单

- 为每个可交互状态字段提供 `state + default_state + callback` 三件套 API。
- 在 `render` 前解析出 `resolved_*`，渲染只消费 `resolved_*`。
- 事件分发中严格执行“受控不落库、非受控落库”的分支。
- 禁止散装状态函数暴露到组件外部，统一走 `struct + impl + trait` builder 风格。

## 6. 已落地组件（第一批）

- `TextInput / PasswordInput / PinInput`
- `Switch / Checkbox / Radio / Chip`
- `Select / MultiSelect`
- `Popover / Modal / Tooltip / HoverCard`
- `ButtonGroup`
