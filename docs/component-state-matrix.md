# 组件状态矩阵（全量扫描）

扫描日期：2026-02-17  
参考基线：Mantine 常见状态模型（`disabled` / `loading` / `error` / `required` / `readOnly` / `opened` / `checked` / `selected` + 交互态 `hover` / `focus` / `active`）

## 1. 状态设置约定（链式 API vs 交互态）

- 应支持链式设置（业务态）：
  - `disabled` / `loading` / `error` / `required` / `read_only`
  - `opened` / `checked` / `value` / `values` / `active` / `visible`
- 不建议链式强设（交互态）：
  - `hover` / `focus` / `pressed`
  - 这类应由事件系统驱动渲染（鼠标、键盘、焦点管理）。

## 2. 组件逐项结论（覆盖全部组件）

### 2.1 表单输入与选择

- `TextInput`：应有 `disabled/read_only/error/required/focus`；已具备，且本轮新增了可直接链式调用的 `label/description/error/required/layout`（无需依赖 `FieldLike` trait 导入）。
- `PasswordInput`：同 `TextInput`；已具备，本轮新增同等链式入口。
- `PinInput`：应有 `disabled/read_only/error/focus`；已补齐，支持只读、错误态边框与错误文案渲染。
- `Textarea`：应有 `disabled/read_only/error/required/focus`；已具备，本轮新增可直接链式 `label/description/error/required/layout`。
- `NumberInput`：应有 `disabled/read_only/error/required/focus`；已具备，本轮新增可直接链式 `label/description/error/required/layout`。
- `Select`：应有 `disabled/error/required/opened` + `option disabled/hover/selected`；已具备，本轮新增可直接链式 `label/description/error/required/layout`。
- `MultiSelect`：同 `Select`；已具备，本轮新增可直接链式 `label/description/error/required/layout`。
- `Checkbox`：应有 `checked/disabled` + `hover/focus` 视觉反馈；已补齐，支持 hover 边框反馈、focus 可视态与键盘触发。
- `Radio`：应有 `checked/disabled` + `hover/focus` 视觉反馈；已补齐，支持 hover 边框反馈、focus 可视态与键盘触发。
- `Switch`：应有 `checked/disabled` + `hover/focus` 视觉反馈；已补齐，支持 hover 边框反馈、focus 可视态与键盘触发。
- `Chip`：应有 `checked/disabled` + `hover/focus` 反馈；已补齐，支持 hover 边框反馈、focus 可视态与键盘触发。
- `ChipGroup` / `CheckboxGroup` / `RadioGroup`：应有 `value(s)` 与 item `disabled`；均已具备。

### 2.2 按钮与触发器

- `Button`：应有 `disabled/loading`；均已具备。
- `ButtonGroup`：应有 `value/default_value` + item `disabled`；已具备。
- `Accordion`：应有 `value/active` + item `disabled`；已具备。
- `ActionIcon`：应有 `disabled/loading`；原缺 `loading`，本轮已新增 `loading` 与 `loading_variant` 链式 API，渲染 Loader 且 loading 时禁点击。
- `Breadcrumbs`：应有 item `disabled/current/hover`；已具备。
- `SegmentedControl`：应有 `value/active` + item `disabled/hover`；已具备。
- `Tabs`：应有 `value/active` + item `disabled/hover`；已具备。
- `Pagination`：应有 `value/active/disabled/hover`；已具备。
- `Tree`：应有 `selected/expanded` + node `disabled/hover`；已具备。
- `Stepper`：应有 `active` + step `disabled/completed`；已具备。
- `Timeline`：应有 `active`；已具备。
- `Rating`：应有 `value/read_only/disabled/active`；已具备。
- `Slider` / `RangeSlider`：应有 `value(s)/disabled/active`；已具备。

### 2.3 弹层与可见性

- `Modal` / `Drawer` / `Popover` / `Tooltip` / `HoverCard` / `Menu`：应有 `opened/default_opened`、可选 `disabled`、关闭策略；已具备。
- `Overlay` / `LoadingOverlay` / `Alert` / `TitleBar`：应有 `visible`；已具备。
- `ModalLayer` / `ToastLayer`：状态由管理器驱动（队列、生命周期）；已具备。

### 2.4 反馈与展示

- `Indicator`：应有 `disabled/processing`；已具备。
- `Progress`：应有 `value/animated/striped`；已具备。
- `Loader`：应有 `variant/size/color`；已具备。
- `Text`：应有 tone（含 error tone）；已具备。
- `Table`：应有 row `hover`、排序/筛选/分页状态；已具备。

### 2.5 结构与纯展示（不要求交互状态）

- `AppShell` / `Sidebar` / `PaneChrome`：布局容器，主要是布局配置与面板开关，已具备。
- `Badge` / `Divider` / `Paper` / `Title` / `Icon` / `Markdown`：纯展示组件，不要求 `disabled/loading/error` 等业务态。
- `Grid` / `SimpleGrid` / `Space` / `Stack` / `ScrollArea`：布局组件，不要求 Mantine 交互态。

## 3. 本轮优化落地清单

- 新增 `ActionIcon`：
  - `loading(bool)`
  - `loading_variant(LoaderVariant)`
  - loading 渲染 Loader，且 loading/disabled 均禁点击
- 新增 `PinInput`：
  - `disabled(bool)` / `read_only(bool)` / `error(...)`
  - 禁用/只读行为约束，错误态边框与错误文案渲染
- 新增输入类“直接链式”状态入口（避免只靠 trait 方法）：
  - `TextInput` / `PasswordInput` / `Textarea` / `NumberInput` / `Select` / `MultiSelect`
  - 均新增：`label` / `description` / `error` / `required` / `layout`
- 新增 check 类 hover + focus 状态渲染：
  - `Checkbox` / `Radio` / `Switch` / `Chip` 补充 hover 边框反馈、focus 可视态与键盘触发
- 新增 check 类细化 token（支持主题精细覆盖）：
  - `Radio`：`border_hover` / `border_focus`
  - `Checkbox`：`border_hover` / `border_focus`
  - `Switch`：`track_hover_border` / `track_focus_border`
  - `Chip`：`border_hover` / `border_focus`
  - 同步补齐 light/dark 默认值、`*Overrides` 与 overrides 链式 API
- 新增交互状态 helper 统一：
  - `components::control` 补充 `focused_state` / `set_focused_state` / `is_activation_key`
  - `Checkbox` / `Radio` / `Switch` / `Chip` / `TextInput` / `Textarea` / `PinInput` 接入，统一 focus 与键盘触发判定

## 4. 收尾状态

- 本轮高优缺口已全部补齐：
  - `Checkbox` / `Radio` / `Switch` / `Chip`：已支持 focus 可视态与键盘触发（`Space` / `Enter`）。
  - `PinInput`：已支持 `read_only` 与 `error` 状态（含行为约束与错误渲染）。
- 本轮建议补充项（原低优先级）也已落地：
  - check 类 token 精细化；
  - 交互 helper 抽象；
  - 关键交互一致性收敛。
