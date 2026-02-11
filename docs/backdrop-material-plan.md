# Backdrop Material Plan

## Strategy

1. Prefer platform system material whenever available.
2. If unavailable, fallback to renderer backdrop blur.
3. If blur path is unavailable, fallback to tint-only mask.

This policy applies to both full-window and partial overlays.

## Work Breakdown

- [x] Phase 0: define unified overlay material policy in `calmui` (`OverlayMaterialMode`, coverage, system material selection).
- [x] Phase 1: wire full-window overlay consumers (`Modal`, `Drawer`, `ModalLayer`) to system-preferred path.
- [x] Phase 1.5: add capability-aware material routing in `calmui` (`OverlayMaterialCapabilities`) and use conservative system-material detection.
- [x] Phase 1.6: normalize all overlay consumers to explicit policy declarations (`SystemPreferred` / `TintOnly` / fallback).
- [x] Phase 2: add renderer-backed real backdrop blur primitive in `gpui` (`Scene` + renderer passes).
- [x] Phase 3: add platform capability API in `gpui` (window-level / region-level system material availability).
- [x] Phase 4: switch partial overlays to system-region material where supported; fallback to renderer blur elsewhere.
- [ ] Phase 5: add visual regression demos and perf toggles.

## Execution Checklist

### Track A: CalmUI Integration (current sprint)

- [x] Add `OverlayMaterialCapabilities` model and default runtime detection.
- [x] Prevent false-positive “system material available” routing on Linux/FreeBSD by default.
- [x] Keep deterministic fallback chain: system -> renderer blur -> tint-only.
- [x] Set explicit `SystemPreferred` strategy for `LoadingOverlay`.
- [x] Set explicit `TintOnly` strategy for `AppShell` sidebar dismissal mask.
- [x] Audit any remaining implicit overlay usages in demo scenes and convert to explicit strategy.
- [x] Add lightweight debug panel in demo app to toggle overlay material capabilities.
- [x] Add environment overrides for capability forcing:
  - `CALMUI_OVERLAY_WINDOW_SYSTEM`
  - `CALMUI_OVERLAY_REGION_SYSTEM`
  - `CALMUI_OVERLAY_RENDERER_BLUR`
- [x] Add provider-level pluggable capability probe for future gpui patch integration:
  - `CalmProvider::set_overlay_capability_probe_global(...)`
- [x] Add provider override clear API:
  - `CalmProvider::clear_overlay_capabilities_global(...)`
- [x] Capability source precedence finalized:
  - manual override > probe > runtime fallback

### Track B: GPUI Capability API

- [x] CalmUI adapter layer completed (provider-level capability probe + global capability injection).

- [x] Add a public capability query entry in `gpui::Window`:
  - `supports_window_material()`
  - `supports_region_material()`
  - `supports_renderer_backdrop_blur()`
- [x] Implement backend mapping:
  - macOS: window material true, region material false (initial)
  - Windows: window material true, region material false (initial)
  - Linux Wayland: window material compositor-dependent (runtime query)
  - Linux X11 / FreeBSD defaults: false
  - renderer backdrop blur: enabled on macOS/Windows/Linux backends

### Track C: Real Backdrop Blur (heavy patch)

- [x] Add `Backdrop` primitive to `Scene`.
- [x] Add `paint_backdrop(...)` in `Window` API.
- [x] Implement renderer pass on macOS Metal:
  - source copy + masked blur composite
- [x] Implement renderer pass on Windows DirectX:
  - source copy + masked blur composite
- [x] Implement renderer pass on Linux Blade:
  - source copy + masked blur composite
- [x] Hook `OverlayCoverage::Parent` to renderer backdrop fallback path.
- [x] Keep `frosted` stack as final fallback for low-capability platforms.

## Phase 2 Detailed Tasks (gpui Patch)

- [x] `crates/gpui/src/scene.rs`:
  - add `Primitive::Backdrop` and `PrimitiveBatch::Backdrops`.
  - add backdrop instance storage and draw-order batching.
- [x] `crates/gpui/src/window.rs`:
  - add `paint_backdrop(...)` API (similar to `paint_quad`).
  - route overlay/backdrop paint operations into scene primitives.
- [x] `crates/gpui/src/platform/mac/metal_renderer.rs`:
  - add backdrop pass with source copy + gaussian masked composite.
- [x] `crates/gpui/src/platform/windows/directx_renderer.rs`:
  - add backdrop pass with source copy + gaussian masked composite.
- [x] `crates/gpui/src/platform/blade/blade_renderer.rs`:
  - add backdrop pass with source copy + gaussian masked composite.

## Phase 3 Detailed Tasks (Capability API)

- [x] `crates/gpui/src/platform.rs`:
  - add `BackdropCapabilities` model and platform query API.
- [x] `crates/gpui/src/window.rs`:
  - expose window-level capability query on `Window`.
- [x] platform backends:
  - map runtime capability on macOS/Windows/Linux (Wayland/X11 split where needed).

## Current Notes

- Current `calmui` implementation can prioritize system material for window-scoped overlays.
- Current `calmui` implementation now uses conservative capability routing to avoid false-positive system-material paths on unsupported platforms.
- Runtime detection supports Wayland session probing and env-force overrides for fast cross-device validation.
- Region-scoped system material is not yet available via `gpui` API, so partial overlays currently route to renderer backdrop blur (where available) and otherwise to non-system path.
- macOS/Windows/Linux now have renderer backdrop blur via `paint_backdrop(...)` + backend-specific renderer passes.
- Cross-target validation:
  - Linux target (`x86_64-unknown-linux-gnu` with `x11,wayland`) check passed.
  - Windows target (`x86_64-pc-windows-gnu`) check passed.
  - Windows MSVC target check remains environment-limited on current machine (missing `lib.exe`).
- This repo now patches `gpui` to local source at `/Users/ethan/Repo/zed-gpui` via Cargo `[patch]`.
