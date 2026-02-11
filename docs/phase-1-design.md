# 第一阶段设计方案（内核层）

## 1. Token 架构

分四层：

1. Primitive Tokens：原始值（色板、尺寸、圆角、字号、动效时长）。
2. Semantic Tokens：语义值（文本/背景/边框/状态）。
3. Component Tokens：组件语义值（button/input/modal/toast）。
4. Resolved Tokens：运行时解析结果（受 variant/state/theme 影响）。

关键点：

- 组件只读取第 3 层，不直接引用第 1 层。
- primary color 默认 `blue`，支持通过 theme API 快速切换。

## 2. Theme 覆盖模型

覆盖优先级（从低到高）：

1. Global Theme
2. Nested Theme Scope
3. Component Override
4. Instance Props

实现策略：

- 使用 `ThemePatch` 深合并。
- 标量值直接覆盖，结构体按字段级覆盖。
- palette 支持按颜色组局部覆盖（如仅覆盖 `blue` 10 阶）。

## 3. Style 统一协议

统一上下文：

- `variant`
- `size`
- `radius`
- `state`
- `field layout`

收益：

- 全组件共享同一组风格维度。
- 避免组件之间命名和行为分叉。

## 4. Motion 统一协议

默认能力：

- 预设：fade/scale/slide/pop。
- 参数：duration/delay/easing。
- 策略：full/reduced/none。

目标：

- 可被组件内建动画复用。
- 用户可在实例级覆盖动画预设和时长。

## 5. 全局管理器设计

### ToastManager

- 生命周期：`show -> update -> dismiss -> dismiss_all`
- 队列：按位置分桶，支持 `max_visible`
- 每条可配置：类型、位置、自动关闭、动画

### ModalManager

- 生命周期：`open -> update -> close -> close_all`
- 数据结构：栈模型
- 每条可配置：类型、关闭策略、层级、动画

## 6. Provider 注入模型

`CalmProvider` 聚合：

- Theme
- MotionConfig
- IconRegistry
- ToastManager
- ModalManager

该模型为后续组件树提供统一配置入口。

## 7. GPUI 对接策略

- 使用 trait 桥接层承接 `Render` / `RenderOnce` / `Styled`。
- 在组件层优先复用 GPUI 能力，而非重造渲染协议。
- 对 pre-1.0 变动保持隔离，降低未来迁移成本。
