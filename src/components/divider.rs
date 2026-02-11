use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, px,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;

use super::utils::resolve_hsla;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DividerOrientation {
    Horizontal,
    Vertical,
}

pub struct Divider {
    id: String,
    orientation: DividerOrientation,
    label: Option<SharedString>,
    theme: crate::theme::LocalTheme,
}

impl Divider {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("divider"),
            orientation: DividerOrientation::Horizontal,
            label: None,
            theme: crate::theme::LocalTheme::default(),
        }
    }

    pub fn orientation(mut self, orientation: DividerOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl WithId for Divider {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Divider {
    fn render(mut self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.divider;
        let line = resolve_hsla(&self.theme, &tokens.line);
        let label_color = resolve_hsla(&self.theme, &tokens.label);

        match self.orientation {
            DividerOrientation::Vertical => div().id(self.id).w(px(1.0)).h_full().bg(line),
            DividerOrientation::Horizontal => {
                if let Some(label) = self.label {
                    div()
                        .id(self.id)
                        .w_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(div().flex_1().h(px(1.0)).bg(line))
                        .child(div().text_xs().text_color(label_color).child(label))
                        .child(div().flex_1().h(px(1.0)).bg(line))
                } else {
                    div().id(self.id).w_full().h(px(1.0)).bg(line)
                }
            }
        }
    }
}

impl IntoElement for Divider {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Divider {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
