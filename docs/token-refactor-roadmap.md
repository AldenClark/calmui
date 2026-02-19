# Token Refactor Roadmap (v1)

## 1. Scope and Principles

- Keep `variant` and `size` as semantic shortcuts.
- Every semantic value maps to explicit theme tokens.
- For composite components, expose only stable and reusable internal slots.
- Prefer token control over ad-hoc `Styled` overrides.

## 2. Implemented in This Pass

### 2.1 API simplification

- `Divider`: replace `new + orientation` with direct constructors.
  - `Divider::horizontal()`
  - `Divider::vertical()`
- Label-required constructors changed to optional-label builders:
  - `Button::new()` + `.label(...)`
  - `Badge::new()` + `.label(...)`
  - `Chip::new()` + `.label(...)`
  - `Checkbox::new()` + `.label(...)`
  - `Radio::new()` + `.label(...)`

### 2.2 Input-family token expansion

Expanded tokens and override API for:

- `input`
- `select`
- `textarea`
- `number_input`

Added dimensions include:

- Typography: `label_size`, `label_weight`, `description_size`, `error_size`
- Layout: `layout_gap_vertical`, `layout_gap_horizontal`, `horizontal_label_width`
- Slots: `slot_gap`, `slot_min_width`, `slot_fg` (for input)
- Select popup/tag: option paddings/sizes, dropdown paddings/gap/max-height, tag paddings/sizes/max-width, icon size
- NumberInput controls: control width/height/icon-size/gap
- Size presets: shared `FieldSizeScale` (`xs/sm/md/lg/xl`) with
  - `font_size`
  - `line_height`
  - `padding_x`
  - `padding_y`
  - `caret_height`

### 2.3 Event adapter convergence

Added shared key helpers in `control` and reused in components:

- `is_plain_keystroke`
- `is_activation_keystroke`
- `is_escape_keystroke`
- `step_direction_from_vertical_key`

Adopted by:

- `toggle`
- `modal`
- `layers` (modal layer)
- `number_input`

## 3. Full-library scan result (remaining extraction)

Remaining high-hardcode areas (priority):

1. `layers`, `table`, `accordion`, `stepper`, `tree`, `title_bar`, `loader`
2. `progress`, `tabs`, `segmented_control`, `range_slider`, `slider`, `timeline`
3. `app_shell`, `drawer`, `alert`, `breadcrumbs`, `switch`, `pagination`

Suggested next token families:

1. Container metrics tokens
   - panel paddings, section gaps, header/footer heights
2. Option/list row metrics tokens
   - row height, row paddings, icon/check widths
3. Navigation/control metrics tokens
   - tab/segment/pagination item heights and inline spacing
4. Data-display metrics tokens
   - table header/row/cell paddings and caption typography

## 4. Migration strategy

1. Expand token structs and override API.
2. Replace component hardcoded values with token lookups.
3. Keep behavior parity with snapshot tests and interaction tests.
4. Repeat by priority cluster until hardcoded metrics are removed from composite components.
