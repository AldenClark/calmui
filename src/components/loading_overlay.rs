use gpui::{
    AnyElement, Component, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, div,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::Size;

use super::loader::{Loader, LoaderElement, LoaderVariant};
use super::overlay::{Overlay, OverlayMaterialMode};
use super::primitives::v_stack;
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type LoaderRenderer = Box<dyn FnOnce(Size, Hsla, String) -> AnyElement>;

pub struct LoadingOverlay {
    id: String,
    visible: bool,
    label: Option<SharedString>,
    variant: LoaderVariant,
    size: Size,
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
            id: stable_auto_id("loading-overlay"),
            visible: true,
            label: None,
            variant: LoaderVariant::Dots,
            size: Size::Md,
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

    pub fn size(mut self, value: Size) -> Self {
        self.size = value;
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
                .size(size)
                .color(color)
                .into_any_element()
        }));
        self
    }
}

impl WithId for LoadingOverlay {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl MotionAware for LoadingOverlay {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for LoadingOverlay {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let mut root = div().id(self.id.clone()).relative().w_full();

        if let Some(content) = self.content.take() {
            root = root.child(content());
        }

        if !self.visible {
            return root.into_any_element();
        }

        let tokens = &self.theme.components.loading_overlay;
        let loader_color = tokens.loader_color.clone();
        let loader = if let Some(renderer) = self.loader.take() {
            renderer(
                self.size,
                loader_color.clone(),
                format!("{}-loader", self.id),
            )
        } else {
            Loader::new()
                .with_id(format!("{}-loader", self.id))
                .variant(self.variant)
                .size(self.size)
                .color(loader_color)
                .into_any_element()
        };

        let mut content = v_stack()
            .items_center()
            .gap_2()
            .child(loader)
            .text_color(resolve_hsla(&self.theme, &tokens.label));

        if let Some(label) = self.label {
            content = content.child(div().text_sm().child(label));
        }

        let overlay = Overlay::new()
            .with_id(format!("{}-mask", self.id))
            .material_mode(OverlayMaterialMode::Auto)
            .motion(self.motion)
            .color(tokens.bg.clone())
            .content(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(content),
            );

        root.child(overlay).into_any_element()
    }
}

impl IntoElement for LoadingOverlay {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for LoadingOverlay {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_visible!(LoadingOverlay);

impl gpui::Styled for LoadingOverlay {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
