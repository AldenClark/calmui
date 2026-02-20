use gpui::{
    AnyElement, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, div, px,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::Size;

use super::Stack;
use super::loader::{Loader, LoaderElement, LoaderVariant};
use super::overlay::{Overlay, OverlayMaterialMode};
use super::utils::{quantized_stroke_px, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type LoaderRenderer = Box<dyn FnOnce(Size, Hsla, ElementId) -> AnyElement>;

#[derive(IntoElement)]
pub struct LoadingOverlay {
    id: ComponentId,
    visible: bool,
    label: Option<SharedString>,
    variant: LoaderVariant,
    size: Size,
    overlay_opacity: f32,
    overlay_blur_strength: f32,
    overlay_readability_boost: f32,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    content: Option<SlotRenderer>,
    loader: Option<LoaderRenderer>,
}

impl LoadingOverlay {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            visible: true,
            label: None,
            variant: LoaderVariant::Dots,
            size: Size::Md,
            overlay_opacity: 0.98,
            overlay_blur_strength: 1.6,
            overlay_readability_boost: 0.92,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            content: None,
            loader: None,
        }
    }

    pub fn visible(mut self, value: bool) -> Self {
        self.visible = value;
        self
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }

    pub fn variant(mut self, value: LoaderVariant) -> Self {
        self.variant = value;
        self
    }

    pub fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    pub fn overlay_opacity(mut self, value: f32) -> Self {
        self.overlay_opacity = value.clamp(0.0, 1.0);
        self
    }

    pub fn overlay_blur_strength(mut self, value: f32) -> Self {
        self.overlay_blur_strength = value.clamp(0.0, 2.0);
        self
    }

    pub fn overlay_readability_boost(mut self, value: f32) -> Self {
        self.overlay_readability_boost = value.clamp(0.0, 1.0);
        self
    }

    pub fn content(mut self, content: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn loader<L>(mut self, loader: L) -> Self
    where
        L: LoaderElement,
    {
        self.loader = Some(Box::new(move |size, color, id| {
            loader
                .with_id(id)
                .with_size(size)
                .color(color)
                .into_any_element()
        }));
        self
    }
}

impl LoadingOverlay {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl MotionAware for LoadingOverlay {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for LoadingOverlay {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let mut root = div().id(self.id.clone()).relative().w_full();

        if let Some(content) = self.content.take() {
            root = root.child(content());
        }

        if !self.visible {
            return root;
        }

        let tokens = &self.theme.components.loading_overlay;
        let loader_color = tokens.loader_color.clone();
        let loader = if let Some(renderer) = self.loader.take() {
            renderer(self.size, loader_color.clone(), self.id.slot("loader"))
        } else {
            Loader::new()
                .with_id(self.id.slot("loader"))
                .variant(self.variant)
                .with_size(self.size)
                .color(loader_color)
                .into_any_element()
        };

        let mut content = Stack::vertical()
            .items_center()
            .gap(tokens.content_gap)
            .child(loader)
            .text_color(resolve_hsla(&self.theme, &tokens.label));

        if let Some(label) = self.label {
            content = content.child(div().text_size(tokens.label_size).child(label));
        }

        let (content_panel_bg, content_panel_border) = match self.theme.color_scheme {
            crate::theme::ColorScheme::Light => {
                (gpui::black().opacity(0.34), gpui::white().opacity(0.20))
            }
            crate::theme::ColorScheme::Dark => {
                (gpui::black().opacity(0.48), gpui::white().opacity(0.16))
            }
        };

        let overlay = Overlay::new()
            .with_id(self.id.slot("mask"))
            .material_mode(OverlayMaterialMode::TintOnly)
            .frosted(false)
            .motion(self.motion)
            .color(tokens.bg.clone())
            .opacity(self.overlay_opacity)
            .blur_strength(self.overlay_blur_strength)
            .readability_boost((self.overlay_readability_boost + 0.08).clamp(0.0, 1.0))
            .content(
                div()
                    .size_full()
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .id(self.id.slot("content-panel"))
                            .rounded(px(14.0))
                            .bg(content_panel_bg)
                            .border(quantized_stroke_px(window, 1.0))
                            .border_color(content_panel_border)
                            .shadow_sm()
                            .px(px(20.0))
                            .py(px(14.0))
                            .child(content),
                    ),
            );

        root.child(overlay)
    }
}

impl crate::contracts::ComponentThemeOverridable for LoadingOverlay {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_sized_via_method!(LoadingOverlay);

crate::impl_visible!(LoadingOverlay);

impl gpui::Styled for LoadingOverlay {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
