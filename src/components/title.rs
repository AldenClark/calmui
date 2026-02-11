use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    div,
};

use crate::contracts::{ThemeScoped, WithId};
use crate::id::stable_auto_id;
use crate::theme::Theme;

use super::primitives::v_stack;
use super::utils::resolve_hsla;

pub struct Title {
    id: String,
    text: SharedString,
    subtitle: Option<SharedString>,
    order: u8,
    theme: Theme,
}

impl Title {
    #[track_caller]
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("title"),
            text: text.into(),
            subtitle: None,
            order: 2,
            theme: Theme::default(),
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
}

impl WithId for Title {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl ThemeScoped for Title {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for Title {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let tokens = &self.theme.components.title;
        let mut headline = div()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(resolve_hsla(&self.theme, &tokens.fg));

        headline = match self.order {
            1 => headline.text_3xl(),
            2 => headline.text_2xl(),
            3 => headline.text_xl(),
            4 => headline.text_lg(),
            5 => headline.text_base(),
            _ => headline.text_sm(),
        };

        let mut root = v_stack()
            .id(self.id)
            .gap_1()
            .child(headline.child(self.text));
        if let Some(subtitle) = self.subtitle {
            root = root.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.subtitle))
                    .child(subtitle),
            );
        }

        root
    }
}

impl IntoElement for Title {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}
