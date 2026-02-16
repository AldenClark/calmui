use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, backdrop, canvas, div, px,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;

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

pub struct Overlay {
    id: String,
    visible: bool,
    absolute: bool,
    cover_parent: bool,
    _coverage: OverlayCoverage,
    _material_mode: OverlayMaterialMode,
    color: Option<Hsla>,
    opacity: f32,
    blur_strength: f32,
    readability_boost: f32,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
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
            _coverage: OverlayCoverage::Parent,
            _material_mode: OverlayMaterialMode::Auto,
            color: None,
            opacity: 1.0,
            blur_strength: 1.45,
            readability_boost: 0.64,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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
        self._coverage = value;
        self
    }

    pub fn material_mode(mut self, value: OverlayMaterialMode) -> Self {
        self._material_mode = value;
        self
    }

    pub fn restore_window_background(self, _value: bool) -> Self {
        self
    }

    pub fn color(mut self, value: impl Into<Hsla>) -> Self {
        self.color = Some(value.into());
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.opacity = value.clamp(0.0, 1.0);
        self
    }

    pub fn frosted(self, _value: bool) -> Self {
        self
    }

    pub fn blur_strength(mut self, value: f32) -> Self {
        self.blur_strength = value.clamp(0.0, 2.0);
        self
    }

    pub fn readability_boost(mut self, value: f32) -> Self {
        self.readability_boost = value.clamp(0.0, 1.0);
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
    fn render(mut self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(cx);

        if !self.visible {
            return div().into_any_element();
        }

        let token = self
            .color
            .unwrap_or_else(|| self.theme.components.overlay.bg.clone());
        let raw_bg = resolve_hsla(&self.theme, &token);

        let opacity = self.opacity.clamp(0.0, 1.0);
        let blur_strength = self.blur_strength.clamp(0.0, 2.0);
        let readability = self.readability_boost.clamp(0.0, 1.0);
        let neutral_target = match self.theme.color_scheme {
            crate::theme::ColorScheme::Light => gpui::white(),
            crate::theme::ColorScheme::Dark => gpui::black(),
        };

        // Keep overlay component lightweight: it only tunes blur/tint parameters for renderer pass.
        let base_scrim_alpha =
            ((0.07 + (0.15 * readability)) * (0.34 + (0.66 * opacity))).clamp(0.06, 0.24);
        let fallback_scrim = neutral_target.opacity(base_scrim_alpha);

        let blur_radius =
            px((22.0 + (70.0 * blur_strength) + (16.0 * readability)).clamp(22.0, 128.0));
        let tint_base = raw_bg.grayscale().blend(neutral_target.opacity(0.18));
        let backdrop_tint_alpha =
            ((0.03 + (0.08 * blur_strength)) * (0.30 + (0.70 * opacity))).clamp(0.02, 0.16);
        let backdrop_tint = tint_base.opacity(backdrop_tint_alpha);

        let veil_alpha =
            ((0.10 + (0.18 * readability)) * (0.36 + (0.64 * opacity))).clamp(0.08, 0.30);
        let neutral_veil = neutral_target.opacity(veil_alpha);

        let mut root = div().id(self.id.clone()).relative().bg(fallback_scrim);

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
        root = root.child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .bg(neutral_veil),
        );

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

impl crate::contracts::ComponentThemeOverridable for Overlay {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_visible!(Overlay);

impl gpui::Styled for Overlay {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
