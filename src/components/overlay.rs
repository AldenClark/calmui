use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, WindowBackgroundAppearance, backdrop,
    canvas, div, px,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::provider::CalmProvider;

use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type OverlayContent = Box<dyn FnOnce() -> AnyElement>;
type OverlayClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayMaterialMode {
    Auto,
    SystemPreferred,
    RendererBlur,
    TintOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayCoverage {
    Parent,
    Window,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlaySystemMaterial {
    Blurred,
    MicaBackdrop,
    MicaAltBackdrop,
    Transparent,
}

impl OverlaySystemMaterial {
    fn to_window_background(self) -> WindowBackgroundAppearance {
        match self {
            Self::Blurred => WindowBackgroundAppearance::Blurred,
            Self::MicaBackdrop => WindowBackgroundAppearance::MicaBackdrop,
            Self::MicaAltBackdrop => WindowBackgroundAppearance::MicaAltBackdrop,
            Self::Transparent => WindowBackgroundAppearance::Transparent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ResolvedMaterialPath {
    System(WindowBackgroundAppearance),
    RendererBlur,
    TintOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OverlayMaterialCapabilities {
    pub window_system: bool,
    pub region_system: bool,
    pub renderer_blur: bool,
}

impl OverlayMaterialCapabilities {
    pub fn from_window(window: &Window) -> Self {
        Self {
            window_system: window.supports_window_material(),
            region_system: window.supports_region_material(),
            renderer_blur: window.supports_renderer_backdrop_blur(),
        }
    }

    fn parse_env_bool(value: &str) -> Option<bool> {
        match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        }
    }

    fn env_bool(name: &str) -> Option<bool> {
        std::env::var(name)
            .ok()
            .and_then(|value| Self::parse_env_bool(&value))
    }

    pub fn detect() -> Self {
        Self {
            // Conservative default:
            // - macOS / Windows: mature window-level materials
            // - Linux / FreeBSD: compositor-dependent and often unavailable
            window_system: cfg!(target_os = "macos") || cfg!(target_os = "windows"),
            // GPUI currently exposes no region-scoped system material API.
            region_system: false,
            // Renderer-level backdrop blur is implemented on macOS/Windows/Linux paths.
            renderer_blur: cfg!(any(
                target_os = "macos",
                target_os = "windows",
                target_os = "linux",
                target_os = "freebsd"
            )),
        }
    }

    pub fn with_env_overrides(mut self) -> Self {
        if let Some(value) = Self::env_bool("CALMUI_OVERLAY_WINDOW_SYSTEM") {
            self.window_system = value;
        }
        if let Some(value) = Self::env_bool("CALMUI_OVERLAY_REGION_SYSTEM") {
            self.region_system = value;
        }
        if let Some(value) = Self::env_bool("CALMUI_OVERLAY_RENDERER_BLUR") {
            self.renderer_blur = value;
        }
        self
    }

    pub fn detect_runtime() -> Self {
        #[allow(unused_mut)]
        let mut caps = Self::detect();

        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            let wayland_display = std::env::var_os("WAYLAND_DISPLAY").is_some();
            let xdg_session_type = std::env::var("XDG_SESSION_TYPE")
                .ok()
                .map(|value| value.to_ascii_lowercase());
            let is_wayland_session =
                wayland_display || matches!(xdg_session_type.as_deref(), Some("wayland"));
            if is_wayland_session {
                // Many Wayland compositors expose blur-capable paths; keep this optimistic and
                // allow explicit override below when needed.
                caps.window_system = true;
            }
        }

        caps.with_env_overrides()
    }
}

impl Default for OverlayMaterialCapabilities {
    fn default() -> Self {
        Self::detect_runtime()
    }
}

pub struct Overlay {
    id: String,
    visible: bool,
    absolute: bool,
    cover_parent: bool,
    coverage: OverlayCoverage,
    material_mode: OverlayMaterialMode,
    system_material: OverlaySystemMaterial,
    restore_window_background: bool,
    material_capabilities: Option<OverlayMaterialCapabilities>,
    color: Option<Hsla>,
    opacity: f32,
    frosted: bool,
    blur_strength: f32,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    content: Option<OverlayContent>,
    on_click: Option<OverlayClickHandler>,
}

impl Overlay {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("overlay"),
            visible: true,
            absolute: true,
            cover_parent: true,
            coverage: OverlayCoverage::Parent,
            material_mode: OverlayMaterialMode::Auto,
            system_material: OverlaySystemMaterial::Blurred,
            restore_window_background: true,
            material_capabilities: None,
            color: None,
            opacity: 1.0,
            frosted: true,
            blur_strength: 1.3,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            content: None,
            on_click: None,
        }
    }

    pub fn visible(mut self, value: bool) -> Self {
        self.visible = value;
        self
    }

    pub fn absolute(mut self, value: bool) -> Self {
        self.absolute = value;
        self
    }

    pub fn cover_parent(mut self, value: bool) -> Self {
        self.cover_parent = value;
        self
    }

    pub fn coverage(mut self, value: OverlayCoverage) -> Self {
        self.coverage = value;
        self
    }

    pub fn material_mode(mut self, value: OverlayMaterialMode) -> Self {
        self.material_mode = value;
        self
    }

    pub fn system_material(mut self, value: OverlaySystemMaterial) -> Self {
        self.system_material = value;
        self
    }

    pub fn restore_window_background(mut self, value: bool) -> Self {
        self.restore_window_background = value;
        self
    }

    pub fn material_capabilities(mut self, value: OverlayMaterialCapabilities) -> Self {
        self.material_capabilities = Some(value);
        self
    }

    fn resolve_material_path(
        &self,
        capabilities: OverlayMaterialCapabilities,
    ) -> ResolvedMaterialPath {
        match self.material_mode {
            OverlayMaterialMode::TintOnly => ResolvedMaterialPath::TintOnly,
            OverlayMaterialMode::RendererBlur => {
                if capabilities.renderer_blur {
                    ResolvedMaterialPath::RendererBlur
                } else {
                    ResolvedMaterialPath::TintOnly
                }
            }
            OverlayMaterialMode::SystemPreferred | OverlayMaterialMode::Auto => {
                let supports_system = match self.coverage {
                    OverlayCoverage::Window => capabilities.window_system,
                    OverlayCoverage::Parent => capabilities.region_system,
                };

                if supports_system && self.coverage == OverlayCoverage::Window {
                    ResolvedMaterialPath::System(self.system_material.to_window_background())
                } else if self.frosted && capabilities.renderer_blur {
                    ResolvedMaterialPath::RendererBlur
                } else {
                    ResolvedMaterialPath::TintOnly
                }
            }
        }
    }

    fn maybe_apply_window_background(
        &self,
        window: &mut Window,
        material: WindowBackgroundAppearance,
    ) {
        if self.coverage == OverlayCoverage::Window {
            window.set_background_appearance(material);
        }
    }

    fn maybe_restore_window_background(&self, window: &mut Window) {
        if self.restore_window_background && self.coverage == OverlayCoverage::Window {
            window.set_background_appearance(WindowBackgroundAppearance::Opaque);
        }
    }

    pub fn color(mut self, value: impl Into<Hsla>) -> Self {
        self.color = Some(value.into());
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.opacity = value.clamp(0.0, 1.0);
        self
    }

    pub fn frosted(mut self, value: bool) -> Self {
        self.frosted = value;
        self
    }

    pub fn blur_strength(mut self, value: f32) -> Self {
        self.blur_strength = value.clamp(0.0, 2.0);
        self
    }

    pub fn content(mut self, content: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl WithId for Overlay {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl MotionAware for Overlay {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Overlay {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let window_caps = OverlayMaterialCapabilities::from_window(_window);
        let static_caps = OverlayMaterialCapabilities::detect();
        let runtime_caps = OverlayMaterialCapabilities {
            window_system: window_caps.window_system || static_caps.window_system,
            region_system: window_caps.region_system || static_caps.region_system,
            renderer_blur: window_caps.renderer_blur || static_caps.renderer_blur,
        }
        .with_env_overrides();
        let capabilities = self
            .material_capabilities
            .unwrap_or_else(|| CalmProvider::overlay_capabilities_for(_window, _cx, runtime_caps));
        let material_path = self.resolve_material_path(capabilities);

        if !self.visible {
            self.maybe_restore_window_background(_window);
            return div().into_any_element();
        }

        let (use_backdrop_blur, fallback_frosted, effective_opacity) = match material_path {
            ResolvedMaterialPath::System(material) => {
                self.maybe_apply_window_background(_window, material);
                // Keep stronger tint for readability while still letting OS material show through.
                (false, false, (self.opacity * 0.68).clamp(0.0, 1.0))
            }
            ResolvedMaterialPath::RendererBlur => {
                self.maybe_restore_window_background(_window);
                (true, false, self.opacity)
            }
            ResolvedMaterialPath::TintOnly => {
                self.maybe_restore_window_background(_window);
                (false, self.frosted, self.opacity)
            }
        };

        let token = self
            .color
            .unwrap_or_else(|| self.theme.components.overlay.bg.clone());
        let raw_bg = resolve_hsla(&self.theme, &token);
        let raw_alpha = raw_bg.a;
        let blended_alpha = if use_backdrop_blur {
            let backdrop_base = (0.26 + (0.12 * self.blur_strength)).clamp(0.24, 0.42);
            ((raw_alpha * 0.35) + (backdrop_base * 0.65)) * effective_opacity
        } else if fallback_frosted {
            let frosted_base = (0.78 + (0.12 * self.blur_strength)).clamp(0.74, 0.94);
            ((raw_alpha * 0.24) + (frosted_base * 0.76)) * effective_opacity
        } else {
            raw_alpha * effective_opacity
        }
        .clamp(0.0, 1.0);
        let bg = raw_bg.opacity(blended_alpha);

        let mut root = div().id(self.id.clone()).relative().bg(bg);

        if self.cover_parent {
            root = root.size_full();
        }

        if self.absolute {
            root = root.absolute().top_0().left_0();
        }

        if let Some(handler) = self.on_click {
            root = root.on_click(move |event, window, cx| {
                (handler)(event, window, cx);
            });
        }

        if use_backdrop_blur {
            let blur_radius = px((8.0 + (26.0 * self.blur_strength)).clamp(8.0, 48.0));
            let backdrop_tint_alpha = (0.14 + (0.10 * self.blur_strength)).clamp(0.12, 0.30);
            let backdrop_tint =
                raw_bg.opacity((backdrop_tint_alpha * effective_opacity).clamp(0.0, 1.0));
            root = root.child(
                canvas(
                    move |bounds, _, _| bounds,
                    move |bounds, _, window, _cx| {
                        window.paint_backdrop(backdrop(
                            bounds,
                            gpui::Corners::default(),
                            blur_radius,
                            backdrop_tint,
                        ));
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            );
        }

        if fallback_frosted {
            let haze_alpha = (0.10 + (0.18 * self.blur_strength)).clamp(0.08, 0.34);
            let depth_alpha = (0.06 + (0.10 * self.blur_strength)).clamp(0.05, 0.20);
            let edge_alpha = (0.03 + (0.06 * self.blur_strength)).clamp(0.02, 0.12);
            let noise_alpha = (0.008 + (0.012 * self.blur_strength)).clamp(0.006, 0.022);

            let haze = if bg.l <= 0.5 {
                gpui::white().opacity(haze_alpha)
            } else {
                gpui::black().opacity(haze_alpha)
            };
            let depth = if bg.l <= 0.5 {
                gpui::black().opacity(depth_alpha)
            } else {
                gpui::white().opacity(depth_alpha)
            };
            let edge = if bg.l <= 0.5 {
                gpui::white().opacity(edge_alpha)
            } else {
                gpui::black().opacity(edge_alpha)
            };
            let grain_hi = if bg.l <= 0.5 {
                gpui::white().opacity(noise_alpha)
            } else {
                gpui::black().opacity(noise_alpha)
            };
            let grain_lo = if bg.l <= 0.5 {
                gpui::black().opacity(noise_alpha * 0.85)
            } else {
                gpui::white().opacity(noise_alpha * 0.85)
            };

            let highlight_base = if bg.l <= 0.5 {
                (0.12 + (0.06 * self.blur_strength)).clamp(0.10, 0.20)
            } else {
                (0.08 + (0.04 * self.blur_strength)).clamp(0.06, 0.14)
            };
            let highlight_color = gpui::white();
            let mut highlight_layer = div()
                .id(format!("{}-frost-highlight", self.id))
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h(px(28.0));
            for i in 0..10_u32 {
                let y = (i as f32) * 3.0;
                let alpha = (highlight_base * (1.0 - (i as f32) / 10.0)).max(0.0);
                let band = div()
                    .absolute()
                    .top(px(y))
                    .left_0()
                    .w_full()
                    .h(px(3.0))
                    .bg(highlight_color.opacity(alpha));
                highlight_layer = highlight_layer.child(band);
            }

            let mut noise_layer = div()
                .id(format!("{}-frost-noise", self.id))
                .absolute()
                .top_0()
                .left_0()
                .size_full();
            for i in 0..360_u32 {
                let x = ((i * 53 + 17) % 1440) as f32;
                let y = ((i * 97 + 29) % 960) as f32;

                let (w, h) = match i % 13 {
                    0 => (2.0, 1.0),
                    1 => (1.0, 2.0),
                    2 => (2.0, 2.0),
                    _ => (1.0, 1.0),
                };
                let tone = match i % 5 {
                    0 => grain_hi,
                    1 => grain_lo,
                    2 => grain_hi.opacity((grain_hi.a * 0.8).clamp(0.0, 1.0)),
                    _ => grain_lo.opacity((grain_lo.a * 0.9).clamp(0.0, 1.0)),
                };
                let grain = div()
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .w(px(w))
                    .h(px(h))
                    .bg(tone);
                noise_layer = noise_layer.child(grain);
            }

            root = root
                .child(
                    div()
                        .id(format!("{}-frost-core", self.id))
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(haze),
                )
                .child(
                    div()
                        .id(format!("{}-frost-depth", self.id))
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(depth),
                )
                .child(highlight_layer)
                .child(noise_layer)
                .child(
                    div()
                        .id(format!("{}-frost-edge", self.id))
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(edge),
                );
        }

        if let Some(content) = self.content {
            root = root.child(content());
        }

        root.with_enter_transition(format!("{}-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl IntoElement for Overlay {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Overlay {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
