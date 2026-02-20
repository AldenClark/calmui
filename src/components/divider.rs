use gpui::{
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::id::ComponentId;

use super::utils::{hairline_px, resolve_hsla};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DividerAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DividerLabelPosition {
    Start,
    Center,
    End,
}

#[derive(IntoElement)]
pub struct Divider {
    id: ComponentId,
    axis: DividerAxis,
    label: Option<SharedString>,
    label_position: DividerLabelPosition,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
}

impl Divider {
    #[track_caller]
    pub fn horizontal() -> Self {
        Self {
            id: ComponentId::default(),
            axis: DividerAxis::Horizontal,
            label: None,
            label_position: DividerLabelPosition::Center,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    #[track_caller]
    pub fn vertical() -> Self {
        Self {
            axis: DividerAxis::Vertical,
            ..Self::horizontal()
        }
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

impl Divider {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Divider {
    fn render(mut self, window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.divider;
        let line = resolve_hsla(&self.theme, &tokens.line);
        let label_color = resolve_hsla(&self.theme, &tokens.label);
        let line_thickness = hairline_px(window);

        match self.axis {
            DividerAxis::Vertical => div()
                .id(self.id)
                .w(line_thickness)
                .h_full()
                .flex_none()
                .bg(line),
            DividerAxis::Horizontal => {
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
                        div().w(tokens.edge_span).h(line_thickness).bg(line)
                    } else {
                        div().flex_1().h(line_thickness).bg(line)
                    };
                    let right_line = if right_flex == 0.0 {
                        div().w(tokens.edge_span).h(line_thickness).bg(line)
                    } else {
                        div().flex_1().h(line_thickness).bg(line)
                    };

                    div()
                        .id(self.id)
                        .w_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(tokens.label_gap)
                        .child(left_line)
                        .child(
                            div()
                                .text_size(tokens.label_size)
                                .text_color(label_color)
                                .child(label),
                        )
                        .child(right_line)
                } else {
                    div().id(self.id).w_full().h(line_thickness).bg(line)
                }
            }
        }
    }
}

impl crate::contracts::ComponentThemeOverridable for Divider {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Divider {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
