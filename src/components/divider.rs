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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DividerLabelPosition {
    Start,
    Center,
    End,
}

pub struct Divider {
    id: String,
    orientation: DividerOrientation,
    label: Option<SharedString>,
    label_position: DividerLabelPosition,
    theme: crate::theme::LocalTheme,
}

impl Divider {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("divider"),
            orientation: DividerOrientation::Horizontal,
            label: None,
            label_position: DividerLabelPosition::Center,
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

    pub fn label_position(mut self, value: DividerLabelPosition) -> Self {
        self.label_position = value;
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
                    let left_flex = match self.label_position {
                        DividerLabelPosition::Start => 0.0,
                        DividerLabelPosition::Center => 1.0,
                        DividerLabelPosition::End => 1.0,
                    };
                    let right_flex = match self.label_position {
                        DividerLabelPosition::Start => 1.0,
                        DividerLabelPosition::Center => 1.0,
                        DividerLabelPosition::End => 0.0,
                    };

                    let left_line = if left_flex == 0.0 {
                        div().w(px(16.0)).h(px(1.0)).bg(line)
                    } else {
                        div().flex_1().h(px(1.0)).bg(line)
                    };
                    let right_line = if right_flex == 0.0 {
                        div().w(px(16.0)).h(px(1.0)).bg(line)
                    } else {
                        div().flex_1().h(px(1.0)).bg(line)
                    };

                    div()
                        .id(self.id)
                        .w_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(left_line)
                        .child(div().text_xs().text_color(label_color).child(label))
                        .child(right_line)
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
