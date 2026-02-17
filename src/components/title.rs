use gpui::{
    Component, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels, Refineable,
    RenderOnce, SharedString, Styled, Window, div,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;

use super::Stack;

#[derive(Debug)]
pub struct Title {
    id: String,
    text: SharedString,
    subtitle: Option<SharedString>,
    order: u8,
    font_size: Option<Pixels>,
    line_height: Option<Pixels>,
    font_weight: Option<FontWeight>,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
}

impl Title {
    #[track_caller]
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("title"),
            text: text.into(),
            subtitle: None,
            order: 2,
            font_size: None,
            line_height: None,
            font_weight: None,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn subtitle(mut self, value: impl Into<SharedString>) -> Self {
        self.subtitle = Some(value.into());
        self
    }

    pub fn order(mut self, value: u8) -> Self {
        self.order = value.clamp(1, 6);
        self
    }

    pub fn font_size(mut self, value: impl Into<Pixels>) -> Self {
        self.font_size = Some(value.into());
        self
    }

    pub fn line_height(mut self, value: impl Into<Pixels>) -> Self {
        self.line_height = Some(value.into());
        self
    }

    pub fn font_weight(mut self, value: FontWeight) -> Self {
        self.font_weight = Some(value);
        self
    }
}

impl WithId for Title {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Title {
    fn render(mut self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(cx);
        let tokens = &self.theme.components.title;
        let base_level = tokens.level(self.order);
        let headline_size = self.font_size.unwrap_or(base_level.font_size);
        let headline_line_height = self.line_height.unwrap_or(base_level.line_height);
        let headline_weight = self.font_weight.unwrap_or(base_level.weight);

        let headline = div()
            .text_size(headline_size)
            .line_height(headline_line_height)
            .font_weight(headline_weight)
            .text_color(tokens.fg);

        let mut root = Stack::vertical()
            .id(self.id)
            .gap(tokens.gap)
            .child(headline.child(self.text));
        if let Some(subtitle) = self.subtitle {
            root = root.child(
                div()
                    .text_size(tokens.subtitle_size)
                    .line_height(tokens.subtitle_line_height)
                    .font_weight(tokens.subtitle_weight)
                    .text_color(tokens.subtitle)
                    .child(subtitle),
            );
        }
        root.style().refine(&self.style);

        root
    }
}

impl IntoElement for Title {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl Styled for Title {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl crate::contracts::ComponentThemeOverridable for Title {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
