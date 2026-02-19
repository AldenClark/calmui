# Layout Depth Impact Report (v4)

## Scope

This pass keeps the tokenization + architecture upgrades, then applies an additional stability-focused optimization batch:

- public API/module surface consolidation (`widgets`, `foundation`, `prelude`)
- high-risk tree rendering path de-recursion
- select/multiselect render flattening and label fallback correctness
- horizontal field-layout wrapper flattening for `input` / `textarea` / `select` / `multiselect`
- additional high-risk shell/layer/title/loading/markdown flattening
- behavior coverage guardrails for all contract-matrix component sets

## Metric Delta (v3 Batch)

Metrics computed with the same line-based budget logic used by `src/components/test_layout_depth_budget.rs`.

| Component | child_calls | div_calls | canvas_calls | max_chain |
|---|---:|---:|---:|---:|
| app_shell.rs | 29 -> 27 | 14 -> 12 | 0 -> 0 | 5 -> 5 |
| layers.rs | 33 -> 30 | 27 -> 24 | 0 -> 0 | 6 -> 6 |
| loader.rs | 19 -> 16 | 15 -> 12 | 0 -> 0 | 6 -> 6 |
| markdown.rs | 19 -> 18 | 13 -> 12 | 0 -> 0 | 4 -> 4 |
| title_bar.rs | 36 -> 34 | 27 -> 25 | 0 -> 0 | 5 -> 5 |
| input.rs | 36 -> 36 | 18 -> 17 | 1 -> 1 | 6 -> 6 |
| textarea.rs | 30 -> 30 | 15 -> 14 | 3 -> 3 | 6 -> 6 |
| select.rs | 63 -> 54 | 32 -> 25 | 2 -> 2 | 6 -> 6 |
| tree.rs | 10 -> 6 | 5 -> 4 | 0 -> 0 | 5 -> 5 |

## What Changed

1. Tree high-risk path was de-recursed.
- `src/components/tree.rs` no longer recursively renders child branches.
- visible node collection is now iterative (explicit frame stack), which lowers stack-depth risk on deep trees.
- keyboard state transitions continue to use the same `tree_state` contract.

2. Select family depth and duplication were reduced.
- `src/components/select.rs` now shares one label-block builder between `Select` and `MultiSelect`.
- per-option render path now uses fallback label text (`value`) when `label` is missing.
- several wrapper-only layout nodes were flattened.

3. Input family horizontal layout wrappers were flattened.
- `src/components/input.rs` and `src/components/textarea.rs` remove one `flex_1` wrapper layer in horizontal layout render branch.

4. Additional high-risk wrappers were flattened in shell/layer/title/loading/markdown.
- `src/components/app_shell.rs` overlay host wrappers flattened.
- `src/components/layers.rs` modal header and root centering wrappers flattened.
- `src/components/title_bar.rs` mac/windows slot wrapper layers flattened.
- `src/components/loader.rs` pulse/bars/oval wrappers flattened.
- `src/components/markdown.rs` quote block wrapper flattened.

5. Depth budgets were tightened for changed hotspots.
- `src/components/test_layout_depth_budget.rs` tightened thresholds for:
  - `app_shell.rs`
  - `layers.rs`
  - `loader.rs`
  - `markdown.rs`
  - `input.rs`
  - `textarea.rs`
  - `select.rs`
  - `table.rs`
  - `title_bar.rs`
  - `tree.rs`

6. Behavior coverage guardrails were expanded.
- `src/components/test_behavior_coverage.rs` enforces alignment between:
  - contract matrix coverage (`test_contract_matrix.rs`)
  - behavior matrix coverage (`test_behavior_matrix.rs`)
- module-local behavior tests were added for low-level modules:
  - `control.rs`
  - `field_variant.rs`
  - `interaction_adapter.rs`
  - `text_input_actions.rs`
  - `toggle.rs`
  - `transition.rs`
  - `utils.rs`

## Functional / Style Impact List

This list enumerates all known externally observable impact from this pass.

1. `Select` (`src/components/select.rs`)
- Functional: options without explicit `label` now display `value` text instead of blank.
- Functional: selected value display also falls back to `value` when `label` is missing.
- Visual: no intentional style-token change.

2. `MultiSelect` (`src/components/select.rs`)
- Functional: selected tags now fall back to option `value` when `label` is missing.
- Visual: tag content wrapper was flattened; no intentional color/spacing token change.

3. `Tree` (`src/components/tree.rs` + `src/components/tree_state.rs`)
- Functional: render/data traversal changed from recursion to iterative frame stack (same API/state contract).
- Visual: line rendering now uses flattened row-level connector rendering; branch-line visuals may differ slightly in deeply nested trees.

4. `TextInput` / `Textarea` horizontal layout
- Structural: wrapper layer removed in horizontal layout branch.
- Expected visual/behavior: unchanged in standard token usage.
- Potential micro impact: custom style selectors targeting the removed wrapper node need adjustment.

5. Public API surface (`src/lib.rs`, `src/widgets/mod.rs`, `src/foundation/mod.rs`, `src/prelude.rs`)
- Additive only: no breaking removal of old `components` path.
- New preferred imports:
  - `calmui::widgets::*` for grouped UI access
  - `calmui::foundation::*` for style/theme/contracts access
  - `calmui::prelude::*` for common one-shot imports

## Validation

- `cargo fmt` passed.
- `cargo test --lib` passed (`93 passed, 0 failed`).
- New regression guardrails are active for:
  - behavior coverage completeness
  - tightened depth budgets
  - public API facade compile smoke tests
