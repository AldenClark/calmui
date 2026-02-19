# Behavior Test Coverage Matrix

Behavior test source: `src/components/test_behavior_matrix.rs`.

## Covered Render Components (all)

- Accordion
- ActionIcon
- Alert
- AppShell
- Badge
- Breadcrumbs
- Button
- ButtonGroup
- Checkbox
- CheckboxGroup
- Chip
- ChipGroup
- Divider
- Drawer
- Grid
- HoverCard
- Icon
- Indicator
- Loader
- LoadingOverlay
- Markdown
- Menu
- Modal
- ModalLayer
- MultiSelect
- NumberInput
- Overlay
- Pagination
- Paper
- Popover
- Progress
- Radio
- RadioGroup
- RangeSlider
- Rating
- ScrollArea
- SegmentedControl
- Select
- Sidebar
- SimpleGrid
- Slider
- Space
- Stepper
- Switch
- Table
- Tabs
- Text
- TextInput
- PasswordInput
- PinInput
- Textarea
- Timeline
- Title
- TitleBar
- ToastLayer
- Tooltip
- Tree

## Coverage Dimensions

- `Disableable` behavior toggles (`disabled(true/false)`) for disableable components.
- `Openable` behavior toggles (`opened(true/false)`) for openable components.
- `Visible` behavior toggles (`visible(true/false)`) for visible components.
- `FieldLike` behavior contract (`label/description/error/required/layout`) for field components.
- Variant-size-radius behavior matrix for style-configurable components.
- End-to-end render scenarios for all render components, including overlay/layer manager-backed components.
