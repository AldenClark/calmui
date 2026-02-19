# Full Component Depth Evaluation (Pass v4)

## v3 Addendum

Additional flattening/optimization changes in this pass:

- `tree.rs`: recursive render path removed (iterative frame-stack traversal).
- `select.rs`: label block deduplicated, option/selected label fallback fixed, wrapper layers reduced.
- `input.rs` / `textarea.rs`: horizontal layout wrapper flattening.

Tightened budget guardrails were updated in:

- `src/components/test_layout_depth_budget.rs`
  - `input.rs`
  - `textarea.rs`
  - `select.rs`
  - `tree.rs`

## v4 Addendum

Additional flattening and budget-tightening updates:

- `app_shell.rs`
- `layers.rs`
- `loader.rs`
- `markdown.rs`
- `title_bar.rs`

New module-level behavior tests added for low-level modules:

- `control.rs`
- `field_variant.rs`
- `interaction_adapter.rs`
- `text_input_actions.rs`
- `toggle.rs`
- `transition.rs`
- `utils.rs`

This file records a full-pass assessment across component modules in `src/components` (test files excluded).

Safe flattening heuristics used in this pass:

- remove empty placeholder nodes (`div().child("")`, synthetic filler elements)
- remove wrapper-only fallback nodes around default trigger text/content
- convert optional wrappers to sparse branches (`Option` + `children`)
- keep API and token behavior unchanged (no rollback)

| Component | Tier | child | div | chain | Result | Notes |
|---|---|---:|---:|---:|---|---|
| `accordion.rs` | P1 | 11 | 4 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `action_icon.rs` | P3 | 1 | 1 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `alert.rs` | P1 | 13 | 9 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `app_shell.rs` | P0 | 29 | 14 | 5 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `badge.rs` | P3 | 3 | 1 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `breadcrumbs.rs` | P3 | 4 | 3 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `button.rs` | P0 | 15 | 7 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `checkbox.rs` | P1 | 12 | 6 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `chip.rs` | P1 | 7 | 2 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `control.rs` | P3 | 0 | 0 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `divider.rs` | P3 | 4 | 8 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `drawer.rs` | P1 | 14 | 14 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `field_variant.rs` | ignore | 0 | 0 | 3 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `hovercard.rs` | P1 | 10 | 4 | 5 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `icon.rs` | P3 | 1 | 1 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `indicator.rs` | P2 | 4 | 11 | 7 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `input.rs` | P0 | 36 | 18 | 6 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `interaction_adapter.rs` | ignore | 0 | 0 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `layers.rs` | P0 | 33 | 27 | 6 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `layout.rs` | P2 | 7 | 9 | 6 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `loader.rs` | P0 | 19 | 15 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `loading_overlay.rs` | P2 | 6 | 4 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `markdown.rs` | P1 | 19 | 13 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `menu.rs` | P1 | 9 | 4 | 5 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `menu_state.rs` | ignore | 0 | 0 | 3 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `mod.rs` | P3 | 0 | 0 | 0 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `modal.rs` | P0 | 16 | 11 | 4 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `number_input.rs` | P1 | 6 | 4 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `overlay.rs` | P3 | 3 | 4 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `pagination.rs` | P2 | 4 | 3 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `paper.rs` | P3 | 1 | 1 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `popover.rs` | P2 | 5 | 2 | 5 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `popup.rs` | P3 | 3 | 2 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `popup_state.rs` | ignore | 0 | 0 | 3 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `progress.rs` | P1 | 11 | 6 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `radio.rs` | P1 | 11 | 6 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `range_slider.rs` | P0 | 22 | 14 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `rating.rs` | P2 | 4 | 3 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `scroll_area.rs` | P3 | 2 | 2 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `segmented_control.rs` | P1 | 6 | 7 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `select.rs` | P0 | 63 | 32 | 6 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `select_state.rs` | ignore | 0 | 0 | 3 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `selection_state.rs` | ignore | 0 | 0 | 1 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `slider.rs` | P0 | 18 | 11 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `slider_axis.rs` | ignore | 0 | 0 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `stepper.rs` | P0 | 24 | 10 | 6 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `switch.rs` | P1 | 11 | 5 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `table.rs` | P0 | 41 | 17 | 4 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `table_state.rs` | ignore | 0 | 0 | 4 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `tabs.rs` | P2 | 6 | 2 | 6 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `text.rs` | P3 | 1 | 1 | 6 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `text_input_actions.rs` | ignore | 0 | 0 | 1 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `text_input_state.rs` | ignore | 0 | 0 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `textarea.rs` | P0 | 30 | 15 | 6 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `timeline.rs` | P1 | 16 | 7 | 6 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `title.rs` | P3 | 3 | 2 | 3 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `title_bar.rs` | P0 | 36 | 27 | 5 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `toggle.rs` | ignore | 0 | 0 | 2 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `tooltip.rs` | P2 | 5 | 3 | 5 | Optimized | Flattening applied in v1/v2 batches with guardrail tests updated. |
| `transition.rs` | ignore | 0 | 0 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `tree.rs` | P1 | 10 | 5 | 5 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `tree_state.rs` | ignore | 0 | 0 | 3 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
| `utils.rs` | ignore | 0 | 0 | 3 | Assessed (No low-risk flattening change in this pass) | No additional safe flattening found without potential visual/behavior coupling. |
