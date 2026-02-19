use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
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

#[derive(IntoElement)]
pub struct Overlay {
    id: ComponentId,
    visible: bool,
    absolute: bool,
    cover_parent: bool,
    coverage: OverlayCoverage,
    material_mode: OverlayMaterialMode,
    restore_window_background: bool,
    frosted: bool,
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
            id: ComponentId::default(),
            visible: true,
            absolute: true,
            cover_parent: true,
            coverage: OverlayCoverage::Parent,
            material_mode: OverlayMaterialMode::Auto,
            restore_window_background: false,
            frosted: true,
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
        self.coverage = value;
        self
    }

    pub fn material_mode(mut self, value: OverlayMaterialMode) -> Self {
        self.material_mode = value;
        self
    }

    pub fn restore_window_background(mut self, value: bool) -> Self {
        self.restore_window_background = value;
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

    pub fn frosted(mut self, value: bool) -> Self {
        self.frosted = value;
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

impl Overlay {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
        let resolved_material = match self.material_mode {
            OverlayMaterialMode::TintOnly => OverlayMaterialMode::TintOnly,
            OverlayMaterialMode::Auto
            | OverlayMaterialMode::SystemPreferred
            | OverlayMaterialMode::RendererBlur => OverlayMaterialMode::TintOnly,
        };
        let use_matte_film =
            self.frosted || !matches!(resolved_material, OverlayMaterialMode::TintOnly);

        let opacity = self.opacity.clamp(0.0, 1.0);
        let blur_strength = self.blur_strength.clamp(0.0, 2.0);
        let readability = self.readability_boost.clamp(0.0, 1.0);
        let (readability_scrim_floor, readability_scrim_span, veil_base, veil_span, film_base) =
            match self.theme.color_scheme {
                crate::theme::ColorScheme::Light => (0.12, 0.26, 0.09, 0.18, 0.03),
                crate::theme::ColorScheme::Dark => (0.18, 0.28, 0.08, 0.14, 0.05),
            };
        let inverse_matte_target = match self.theme.color_scheme {
            crate::theme::ColorScheme::Light => gpui::white(),
            crate::theme::ColorScheme::Dark => gpui::black(),
        };

        // Component-layer matte strategy: increase neutralization and edge diffusion
        // so top-layer content remains readable even on transparent surfaces.
        let matte_strength = ((0.30 + (0.22 * blur_strength) + (0.45 * readability))
            * (0.42 + (0.58 * opacity)))
            .clamp(0.12, 1.0);
        let scrim_alpha =
            (readability_scrim_floor + (readability_scrim_span * matte_strength)).clamp(0.10, 0.56);
        let scrim_color = if self.restore_window_background {
            raw_bg
        } else {
            raw_bg.grayscale()
        };
        let fallback_scrim = scrim_color.opacity(scrim_alpha);

        let veil_alpha =
            ((veil_base + (veil_span * readability)) * (0.45 + (0.55 * opacity))).clamp(0.07, 0.33);
        let neutral_veil = gpui::black().opacity(veil_alpha);
        let matte_film_alpha = if use_matte_film {
            ((film_base + (0.08 * blur_strength)) * (0.55 + (0.45 * opacity))).clamp(0.02, 0.16)
        } else {
            0.0
        };
        let matte_film = raw_bg
            .grayscale()
            .blend(inverse_matte_target.opacity(0.10))
            .opacity(matte_film_alpha);

        let mut root = div().id(self.id.clone()).relative().bg(fallback_scrim);

        if self.cover_parent || matches!(self.coverage, OverlayCoverage::Window) {
            root = root.size_full();
        }

        if self.absolute || matches!(self.coverage, OverlayCoverage::Window) {
            root = root.absolute().top_0().left_0();
        }

        if let Some(handler) = self.on_click {
            root = root.on_click(move |event, window, cx| {
                (handler)(event, window, cx);
            });
        }

        if use_matte_film {
            root = root.child(div().absolute().top_0().left_0().size_full().bg(matte_film));
        }
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

        gpui::Refineable::refine(gpui::Styled::style(&mut root), &self.style);

        root.with_enter_transition(self.id.slot("enter"), self.motion)
            .into_any_element()
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
