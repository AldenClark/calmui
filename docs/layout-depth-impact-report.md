# Layout Depth Impact Report (Current Batch)

## Scope

This report covers the current flattening batch applied without rolling back architecture/token refactors.

Touched components for depth flattening:

- `src/components/input.rs`
- `src/components/textarea.rs`
- `src/components/select.rs`
- `src/components/layers.rs`
- `src/components/modal.rs`

## Metric Delta

Baseline source: local scan snapshot taken before this batch (`/tmp/calmui_depth_scan.csv`).

| Component | child_calls | div_calls | canvas_calls | max_chain |
|---|---:|---:|---:|---:|
| input.rs | 36 -> 36 | 19 -> 18 | 1 -> 1 | 6 -> 6 |
| textarea.rs | 30 -> 30 | 16 -> 15 | 3 -> 3 | 6 -> 6 |
| select.rs | 65 -> 63 | 34 -> 32 | 2 -> 2 | 6 -> 6 |
| layers.rs | 33 -> 33 | 28 -> 27 | 0 -> 0 | 6 -> 6 |
| modal.rs | 16 -> 16 | 12 -> 11 | 0 -> 0 | 4 -> 4 |

## What Changed

1. Optional label block rendering is now sparse.
- `input` / `textarea` / `select` / `multi-select` no longer render an empty label container when `label/description/error` are all absent.
- Horizontal layout only mounts label-width container when label block exists.

2. Optional close action rendering is now sparse.
- `modal` and `layers::ModalLayer` no longer allocate an empty close-action node when close button is disabled.

3. No token API rollback.
- All token fields and theming entry points remain intact.
- The refactor only changes node materialization strategy (presence/absence of optional wrapper nodes).

## Functional/Style Impact Assessment

Expected behavior impact:

- No API contract changes.
- No controlled/uncontrolled state behavior changes.
- No keyboard interaction changes.

Potential visual impact (needs manual smoke check):

- Slight alignment differences in field horizontal layout when label block is omitted.
- Modal/header spacing can look subtly tighter when close button is hidden (empty placeholder removed).

## Validation Status

- `cargo test --lib` passed.
- New guardrail tests are active:
  - contract matrix
  - state logic suite
  - layout depth budget suite
