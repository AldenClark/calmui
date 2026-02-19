use gpui::{InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, div};

use crate::id::ComponentId;
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

#[derive(IntoElement)]
pub struct Text {
    id: ComponentId,
    content: SharedString,
    tone: TextTone,
    size: Size,
    truncate: bool,
    line_clamp: Option<usize>,
    with_ellipsis: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
}

impl Text {
    #[track_caller]
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            id: ComponentId::default(),
            content: content.into(),
            tone: TextTone::Default,
            size: Size::Md,
            truncate: false,
            line_clamp: None,
            with_ellipsis: true,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn tone(mut self, value: TextTone) -> Self {
        self.tone = value;
        self
    }

    pub fn with_size(mut self, value: Size) -> Self {
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

    fn resolved_text_color(&self) -> gpui::Hsla {
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

impl Text {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Text {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let id = self.id.clone();
        let color = self.resolved_text_color();
        let size_preset = self.theme.components.text.sizes.for_size(self.size);
        let mut node = div()
            .id(id)
            .text_color(color)
            .text_size(size_preset.font_size)
            .line_height(size_preset.line_height);

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
                    .max_h(gpui::px(f32::from(size_preset.line_height) * lines as f32));
            }
        }

        gpui::Refineable::refine(gpui::Styled::style(&mut node), &self.style);
        node.child(self.content)
    }
}

impl crate::contracts::ComponentThemeOverridable for Text {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_sized_via_method!(Text);

impl gpui::Styled for Text {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
