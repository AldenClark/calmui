use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    div,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;
use crate::style::Size;

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
    line_clamp: Option<usize>,
    with_ellipsis: bool,
    theme: crate::theme::LocalTheme,
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
            line_clamp: None,
            with_ellipsis: true,
            theme: crate::theme::LocalTheme::default(),
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

    pub fn line_clamp(mut self, value: usize) -> Self {
        self.line_clamp = Some(value.max(1));
        self
    }

    pub fn with_ellipsis(mut self, value: bool) -> Self {
        self.with_ellipsis = value;
        self
    }

    fn line_height_px(&self) -> f32 {
        match self.size {
            Size::Xs => 14.0,
            Size::Sm => 16.0,
            Size::Md => 18.0,
            Size::Lg => 22.0,
            Size::Xl => 26.0,
        }
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

impl RenderOnce for Text {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
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
            if self.with_ellipsis {
                node = node.truncate();
            } else {
                node = node.overflow_hidden().whitespace_nowrap();
            }
        }

        if let Some(lines) = self.line_clamp {
            if self.with_ellipsis {
                node = node.line_clamp(lines);
            } else {
                node = node
                    .overflow_hidden()
                    .max_h(gpui::px(self.line_height_px() * lines as f32));
            }
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

impl crate::contracts::ComponentThemePatchable for Text {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
