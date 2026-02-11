use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::theme::ColorValue;

use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type OverlayContent = Box<dyn FnOnce() -> AnyElement>;
type OverlayClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>;

pub struct Overlay {
    id: String,
    visible: bool,
    absolute: bool,
    cover_parent: bool,
    color: Option<ColorValue>,
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

    pub fn color(mut self, value: ColorValue) -> Self {
        self.color = Some(value);
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
        if !self.visible {
            return div().into_any_element();
        }

        let token = self
            .color
            .unwrap_or_else(|| self.theme.components.overlay.bg.clone());
        let raw_bg = resolve_hsla(&self.theme, &token);
        let raw_alpha = raw_bg.a;
        let blended_alpha = if self.frosted {
            // In frosted mode, stay more *transparent* than a typical overlay. We then
            // recover readability with highlight/depth/grain passes below.
            //
            // The previous mix skewed too opaque ("grey mask" look). This keeps the
            // base tint lighter, and lets the glass layers do the work.
            let frosted_base = (0.46 + (0.14 * self.blur_strength)).clamp(0.40, 0.72);
            ((raw_alpha * 0.42) + (frosted_base * 0.58)) * self.opacity
        } else {
            raw_alpha * self.opacity
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

        if self.frosted {
            // GPUI has no per-element backdrop filter yet. Approximate blur/glass by
            // stacking haze + depth + low-frequency "fog" + grain + edge/highlight passes.
            //
            // Notes:
            // - Avoid directional textures (grid/scanlines) because they read as "overlay UI",
            //   not material.
            // - Use denser, irregular grain to simulate acrylic/matte glass.
            // - Use a top highlight to fake reflected light.
            let haze_alpha = (0.10 + (0.18 * self.blur_strength)).clamp(0.08, 0.34);
            let depth_alpha = (0.06 + (0.10 * self.blur_strength)).clamp(0.05, 0.20);
            let edge_alpha = (0.03 + (0.06 * self.blur_strength)).clamp(0.02, 0.12);
            let fog_alpha = (0.018 + (0.028 * self.blur_strength)).clamp(0.012, 0.06);
            let noise_alpha = (0.015 + (0.035 * self.blur_strength)).clamp(0.012, 0.075);

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
            let texture_hi = if bg.l <= 0.5 {
                gpui::white().opacity(fog_alpha)
            } else {
                gpui::black().opacity(fog_alpha)
            };
            let texture_lo = if bg.l <= 0.5 {
                gpui::black().opacity(fog_alpha * 0.75)
            } else {
                gpui::white().opacity(fog_alpha * 0.75)
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

            // Low-frequency "fog" blobs: gives a subtle diffusion feel without blur.
            // Keep it non-directional so it doesn't read as UI grid.
            let mut fog_layer = div()
                .id(format!("{}-frost-fog", self.id))
                .absolute()
                .top_0()
                .left_0()
                .size_full();
            for i in 0..18_u32 {
                // Deterministic pseudo-random placement/sizing (no RNG dependency).
                let x = ((i * 173 + 41) % 1440) as f32;
                let y = ((i * 137 + 29) % 960) as f32;
                let w = (140.0 + (((i * 97 + 11) % 140) as f32)).min(280.0);
                let h = (110.0 + (((i * 83 + 23) % 130) as f32)).min(260.0);
                let tone = if i % 2 == 0 { texture_hi } else { texture_lo };

                // Soft blobs: large rounded rects at very low alpha.
                // (We avoid heavy rounding helpers to stay compatible with older gpui style APIs.)
                let blob = div()
                    .absolute()
                    .left(px(x - (w * 0.5)))
                    .top(px(y - (h * 0.5)))
                    .w(px(w))
                    .h(px(h))
                    .rounded_full()
                    .bg(tone);

                fog_layer = fog_layer.child(blob);
            }

            // Top highlight: fakes reflected light. Implemented as stacked thin bands
            // so we don't depend on gradient APIs.
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
            // Denser micro-grain (avoid round dots; use tiny rectangles).
            // This reads much closer to acrylic/matte glass than sparse circles.
            for i in 0..900_u32 {
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
                .child(fog_layer)
                .child(highlight_layer)
                .child(noise_layer)
                .child(
                    div()
                        .id(format!("{}-frost-edge", self.id))
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(edge)
                        .border_1()
                        .border_color(edge),
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
