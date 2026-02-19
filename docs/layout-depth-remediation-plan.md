# Layout Depth Remediation Plan (v1)

## Goal

Keep the new architecture + tokenization model, while removing layout-depth regressions that can trigger recursive layout stack pressure in complex pages.

## Constraints

- Do not roll back the token refactor.
- Keep public builder APIs and component semantics stable.
- Make depth control enforceable in CI, not only ad-hoc by manual reviews.
- Must remain compatible with current `gpui` dependency:
  - `https://github.com/zed-industries/zed.git`
  - `rev = bc31ad4a8c61f1eb2f3daf28a892fcd147b08185`

## Delivery Tracks

1. Test and guardrail track (implemented)
- `src/components/test_contract_matrix.rs`
  - Compile-time contract matrix across render/styled/theming/behavior traits.
- `src/components/test_component_smoke.rs`
  - Smoke rendering coverage for exported component families (primitives/forms/overlay/layout/data-heavy).
- `src/components/test_state_logic.rs`
  - Deterministic state-machine tests for popup/menu/select/table/tree/input/slider logic.
- `src/components/test_behavior_matrix.rs`
  - Behavior-level matrix that exercises render-time behavior toggles across all render components.
- `src/components/test_layout_depth_budget.rs`
  - Depth budget gate for all component files.
- `src/components/test_flattening_invariants.rs`
  - Enforces sparse-rendering invariants across all component sources.
- `src/components/test_component_coverage.rs`
  - Ensures every non-test component module is covered by both depth budgets and flattening invariants.
- `docs/component-depth-matrix.csv`
  - Current component depth baseline and priority tiering.
- `docs/layout-depth-full-evaluation.md`
  - Full-pass file-by-file assessment (optimized vs. no-safe-change-in-pass).
- `docs/behavior-test-coverage.md`
  - Behavior-level coverage ledger for all render components.

2. Architecture flattening track (current batch implemented)
- Remove empty placeholder nodes for optional regions (`label/description/error/close action`).
- Keep existing token surfaces, but render optional slots only when data exists.
- Prefer `children(option)` over synthetic empty `div` placeholders.

3. Component-by-component flattening track
- P0 batch (must stabilize first):
  - `select.rs`, `input.rs`, `textarea.rs`, `app_shell.rs`, `table.rs`, `title_bar.rs`, `layers.rs`, `range_slider.rs`, `stepper.rs`, `slider.rs`, `button.rs`, `loader.rs`, `modal.rs`
- P1 batch:
  - `markdown.rs`, `timeline.rs`, `checkbox.rs`, `segmented_control.rs`, `drawer.rs`, `accordion.rs`, `radio.rs`, `alert.rs`, `tree.rs`, `menu.rs`, `switch.rs`, `number_input.rs`, `progress.rs`, `chip.rs`, `hovercard.rs`
- P2/P3 hardening:
  - Remaining components in `docs/component-depth-matrix.csv`

4. Impact accounting track (implemented for current batch)
- For each batch, publish:
  - metric delta (`child/div/canvas/max_chain`)
  - style deltas (spacing/click area/focus ring/overlay coverage)
  - behavior deltas (open/close, keyboard interaction, controlled/uncontrolled rules)

## Batch Acceptance Criteria

Every batch must pass:

1. `cargo test --lib`
2. no public API break
3. no token override regression
4. depth budget gate green
5. impact report updated

## Current Status

- Test foundation: done
- Guardrails wired into `cargo test --lib`: done
- High-risk sparse-rendering batch (`input/textarea/select/modal/layers/title_bar`): done
- Additional flattening batch (`table/timeline/layout/menu/popover/tooltip/hovercard/tabs`): done
- Additional stepper flattening pass (`stepper` optional label wrapper removal): done
- Updated impact report and depth matrix: done
- Behavior matrix coverage for all render components: done

## Risk Notes

- Largest risk is visual micro-shifts caused by removing wrapper nodes.
- Highest-risk surfaces are modal/titlebar/shell overlays and input focus rings.
- New `overlay` behavior should be validated in stacked scenarios (`Overlay + ModalLayer + AppShell`), because matte/frosted layering now differs from older versions.
- Popup-family default trigger rendering changed from wrapped fallback nodes to direct text fallback; theme overrides that relied on wrapper selectors should be smoke-tested.
