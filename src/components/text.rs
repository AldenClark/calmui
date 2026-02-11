use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    div,
};

use crate::contracts::{ThemeScoped, WithId};
use crate::id::stable_auto_id;
use crate::style::Size;
use crate::theme::Theme;

use super::utils::resolve_hsla;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextTone {
    Default,
    Secondary,
    Muted,
    Accent,
    Success,
    Warning,
    Error,
}

pub struct Text {
    id: String,
    content: SharedString,
    tone: TextTone,
    size: Size,
    truncate: bool,
    theme: Theme,
}

impl Text {
    #[track_caller]
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("text"),
            content: content.into(),
            tone: TextTone::Default,
            size: Size::Md,
            truncate: false,
            theme: Theme::default(),
        }
    }

    pub fn tone(mut self, value: TextTone) -> Self {
        self.tone = value;
        self
    }

    pub fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    pub fn truncate(mut self, value: bool) -> Self {
        self.truncate = value;
        self
    }

    fn text_color(&self) -> gpui::Hsla {
        let tokens = &self.theme.components.text;
        let token = match self.tone {
            TextTone::Default => &tokens.fg,
            TextTone::Secondary => &tokens.secondary,
            TextTone::Muted => &tokens.muted,
            TextTone::Accent => &tokens.accent,
            TextTone::Success => &tokens.success,
            TextTone::Warning => &tokens.warning,
            TextTone::Error => &tokens.error,
        };
        resolve_hsla(&self.theme, token)
    }
}

impl WithId for Text {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl ThemeScoped for Text {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let id = self.id.clone();
        let color = self.text_color();
        let mut node = div().id(id).text_color(color);

        node = match self.size {
            Size::Xs => node.text_xs(),
            Size::Sm => node.text_sm(),
            Size::Md => node.text_base(),
            Size::Lg => node.text_lg(),
            Size::Xl => node.text_xl(),
        };

        if self.truncate {
            node = node.truncate();
        }

        node.child(self.content)
    }
}

impl IntoElement for Text {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}
