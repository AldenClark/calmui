# Layout Depth Impact Report (v2)

## Scope

This report summarizes the completed flattening batches while keeping the architecture/token upgrades intact.

Flattening-covered components:

- `src/components/input.rs`
- `src/components/textarea.rs`
- `src/components/select.rs`
- `src/components/layers.rs`
- `src/components/modal.rs`
- `src/components/title_bar.rs`
- `src/components/table.rs`
- `src/components/timeline.rs`
- `src/components/layout.rs`
- `src/components/menu.rs`
- `src/components/popover.rs`
- `src/components/tooltip.rs`
- `src/components/hovercard.rs`
- `src/components/tabs.rs`
- `src/components/stepper.rs`

## Metric Delta (Current Additional Batch)

| Component | child_calls | div_calls | canvas_calls | max_chain |
|---|---:|---:|---:|---:|
| table.rs | 42 -> 41 | 19 -> 17 | 2 -> 2 | 4 -> 4 |
| timeline.rs | 16 -> 16 | 8 -> 7 | 0 -> 0 | 6 -> 6 |
| layout.rs | 7 -> 7 | 9 -> 9 | 0 -> 0 | 6 -> 6 |
| menu.rs | 9 -> 9 | 5 -> 4 | 1 -> 1 | 5 -> 5 |
| popover.rs | 5 -> 5 | 3 -> 2 | 0 -> 0 | 5 -> 5 |
| tooltip.rs | 5 -> 5 | 4 -> 3 | 0 -> 0 | 5 -> 5 |
| hovercard.rs | 10 -> 10 | 5 -> 4 | 1 -> 1 | 5 -> 5 |
| tabs.rs | 6 -> 6 | 3 -> 2 | 0 -> 0 | 6 -> 6 |
| stepper.rs | 24 -> 24 | 11 -> 10 | 0 -> 0 | 6 -> 6 |

## What Changed

1. Empty slot/cell placeholders were removed where they only served shape padding.
- `table.rs`: removed empty cell placeholder `div().child("")`; empty-state content now mounts directly without synthetic wrapper.
- `layout.rs`: trailing grid cells are represented by direct empty flex cells, no synthetic nested filler element.

2. Optional title/trigger fallback now renders sparsely.
- `timeline.rs`: title wrapper is only created when title exists.
- `menu.rs` / `popover.rs` / `tooltip.rs` / `hovercard.rs`: default trigger label no longer goes through an extra `div` wrapper.

3. Fallback panel rendering is sparse.
- `tabs.rs`: no-panel fallback now writes directly into the panel node instead of creating an extra fallback wrapper element.

4. Budget guardrails were tightened.
- `test_layout_depth_budget.rs` reduced thresholds for:
  - `hovercard.rs`
  - `layout.rs`
  - `menu.rs`
  - `popover.rs`
  - `stepper.rs`
  - `table.rs`
  - `tabs.rs`
  - `timeline.rs`
  - `tooltip.rs`

5. New anti-regression coverage was added.
- `test_flattening_invariants.rs` now enforces:
  - no empty table-cell placeholders
  - no synthetic grid filler push path
  - no wrapped default trigger fallback text in popup family
  - no stepper optional-label wrapper shell
- `test_component_coverage.rs` ensures every non-test component module is included by both:
  - depth-budget test
  - flattening-invariant test
- `test_behavior_matrix.rs` adds behavior-level render/contract exercises across all render components.

## Functional/Style Impact

Expected behavior changes:

- No public API changes.
- No controlled/uncontrolled state contract changes.
- No keyboard behavior changes in popup/menu/table/tabs flows.

Potential visual micro-shifts:

- Empty-state rows and popup default trigger text lose one wrapper layer, so spacing may be very slightly tighter in highly customized themes that target wrapper nodes by selector.
- Tabs no-panel fallback now inherits panel node typography path directly.

## Validation

- `cargo test --lib` passed (`68 passed, 0 failed`).
- Depth and sparse-rendering guardrails are green.
- Compatible with current gpui dependency in `Cargo.toml` (overlay-related update path verified by compile + tests).
- Full component-by-component depth assessment ledger: `docs/layout-depth-full-evaluation.md`.
