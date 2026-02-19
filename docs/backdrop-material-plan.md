# Overlay Material Plan

## Strategy (No GPUI Patch)

1. Use upstream `gpui` from `zed-industries/zed` with pinned `rev`.
2. Keep overlay rendering fully component-driven (`TintOnly` path).
3. Optimize for readability first (modal / drawer / loading masks), not renderer-level backdrop blur.

This policy applies to both full-window and partial overlays.

## Current Behavior

- `OverlayMaterialMode::Auto` / `SystemPreferred` / `RendererBlur` are preserved for API compatibility.
- Runtime rendering is normalized to component-level matte tint (no renderer blur primitive).
- `Modal`, `Drawer`, `ModalLayer`, and `LoadingOverlay` all use readability-focused tint masks.
- `LoadingOverlay` adds a centered readable content plate so transparent content remains legible.

## Theming Guidance (Light + Dark)

- Light theme:
  - stronger dark scrim to suppress busy background details.
  - subtle bright film layer to keep matte feel without washing out foreground.
- Dark theme:
  - deeper base scrim with lower veil lift to avoid crushed contrast.
  - restrained film/border alpha to prevent halo around centered content.

## Validation Checklist

- [x] Remove component dependency on `paint_backdrop(...)`.
- [x] Keep overlays functional without any local `gpui` patch.
- [x] Normalize key overlay consumers to tint path.
- [x] Verify compile against pinned upstream `gpui` revision.
