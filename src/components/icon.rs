use gpui::{
    Component, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div, px,
    svg,
};

use crate::icon::{IconRegistry, IconSource};
use crate::{contracts::WithId, id::stable_auto_id};

use super::utils::resolve_hsla;

#[derive(Clone)]
enum IconColor {
    Token(Hsla),
    Raw(gpui::Hsla),
}

pub struct Icon {
    id: String,
    source: IconSource,
    size: f32,
    color: Option<IconColor>,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    registry: IconRegistry,
}

impl Icon {
    #[track_caller]
    pub fn new(source: IconSource) -> Self {
        Self {
            id: stable_auto_id("icon"),
            source,
            size: 16.0,
            color: None,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            registry: IconRegistry::new(),
        }
    }

    #[track_caller]
    pub fn named(name: impl Into<String>) -> Self {
        Self::new(IconSource::named(name))
    }

    pub fn source(mut self, source: IconSource) -> Self {
        self.source = source;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(8.0);
        self
    }

    pub fn color_token(mut self, token: impl Into<Hsla>) -> Self {
        self.color = Some(IconColor::Token(token.into()));
        self
    }

    pub fn color(mut self, value: gpui::Hsla) -> Self {
        self.color = Some(IconColor::Raw(value));
        self
    }

    pub fn registry(mut self, registry: IconRegistry) -> Self {
        self.registry = registry;
        self
    }

    fn resolve_color(&self) -> Option<gpui::Hsla> {
        match &self.color {
            Some(IconColor::Token(token)) => Some(resolve_hsla(&self.theme, token)),
            Some(IconColor::Raw(value)) => Some(*value),
            None => None,
        }
    }
}

impl WithId for Icon {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Icon {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let color = self.resolve_color();
        if let Some(path) = self.registry.resolve(&self.source) {
            let mut icon = svg()
                .external_path(path.to_string_lossy().to_string())
                .w(px(self.size))
                .h(px(self.size))
                .id(self.id);
            if let Some(color) = color {
                icon = icon.text_color(color);
            }
            return icon.into_any_element();
        }

        let mut fallback = div()
            .id(self.id)
            .w(px(self.size))
            .h(px(self.size))
            .child("?");
        if let Some(color) = color {
            fallback = fallback.text_color(color);
        }
        fallback.into_any_element()
    }
}

impl IntoElement for Icon {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Icon {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Icon {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
