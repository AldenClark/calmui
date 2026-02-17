use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, Component, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, Styled, Window, div, px,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;

use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndicatorPosition {
    TopStart,
    TopCenter,
    TopEnd,
    MiddleStart,
    MiddleEnd,
    BottomStart,
    BottomCenter,
    BottomEnd,
}

pub struct Indicator {
    id: String,
    label: Option<SharedString>,
    dot: bool,
    processing: bool,
    disabled: bool,
    position: IndicatorPosition,
    size_px: f32,
    offset_px: f32,
    color: Option<Hsla>,
    with_border: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    child: Option<SlotRenderer>,
}

impl Indicator {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("indicator"),
            label: None,
            dot: true,
            processing: false,
            disabled: false,
            position: IndicatorPosition::TopEnd,
            size_px: 18.0,
            offset_px: 6.0,
            color: None,
            with_border: true,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            child: None,
        }
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self.dot = false;
        self
    }

    pub fn dot(mut self, value: bool) -> Self {
        self.dot = value;
        self
    }

    pub fn processing(mut self, value: bool) -> Self {
        self.processing = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn position(mut self, value: IndicatorPosition) -> Self {
        self.position = value;
        self
    }

    pub fn size(mut self, value: f32) -> Self {
        self.size_px = value.max(8.0);
        self
    }

    pub fn offset(mut self, value: f32) -> Self {
        self.offset_px = value.max(0.0);
        self
    }

    pub fn color(mut self, value: impl Into<Hsla>) -> Self {
        self.color = Some(value.into());
        self
    }

    pub fn with_border(mut self, value: bool) -> Self {
        self.with_border = value;
        self
    }

    pub fn child(mut self, value: impl IntoElement + 'static) -> Self {
        self.child = Some(Box::new(|| value.into_any_element()));
        self
    }

    fn indicator_host(&self) -> gpui::Div {
        let offset = px(-self.offset_px);
        match self.position {
            IndicatorPosition::TopStart => div().absolute().top(offset).left(offset),
            IndicatorPosition::TopCenter => div()
                .absolute()
                .top(offset)
                .left_0()
                .right_0()
                .flex()
                .justify_center(),
            IndicatorPosition::TopEnd => div().absolute().top(offset).right(offset),
            IndicatorPosition::MiddleStart => div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(offset)
                .flex()
                .items_center(),
            IndicatorPosition::MiddleEnd => div()
                .absolute()
                .top_0()
                .bottom_0()
                .right(offset)
                .flex()
                .items_center(),
            IndicatorPosition::BottomStart => div().absolute().bottom(offset).left(offset),
            IndicatorPosition::BottomCenter => div()
                .absolute()
                .bottom(offset)
                .left_0()
                .right_0()
                .flex()
                .justify_center(),
            IndicatorPosition::BottomEnd => div().absolute().bottom(offset).right(offset),
        }
    }
}

impl WithId for Indicator {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Indicator {
    fn render(mut self, window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let bg = self
            .color
            .as_ref()
            .map(|token| resolve_hsla(&self.theme, token))
            .unwrap_or_else(|| resolve_hsla(&self.theme, &self.theme.semantic.status_error));
        let fg = gpui::white();
        let border = resolve_hsla(&self.theme, &self.theme.semantic.bg_canvas);

        let mut badge = div()
            .id(format!("{}-badge", self.id))
            .h(px(self.size_px))
            .min_w(px(self.size_px))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .px(px((self.size_px * 0.25).max(2.0)))
            .bg(bg)
            .text_color(fg)
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD);

        if self.with_border {
            badge = badge
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(border);
        }

        if self.disabled {
            badge = badge.opacity(0.55);
        }

        if !self.dot {
            badge = badge.child(
                self.label
                    .clone()
                    .unwrap_or_else(|| SharedString::from("1")),
            );
        }

        if self.processing {
            badge = badge.child(
                div()
                    .id(format!("{}-pulse", self.id))
                    .absolute()
                    .size_full()
                    .rounded_full()
                    .border(super::utils::quantized_stroke_px(window, 1.0))
                    .border_color(bg)
                    .with_animation(
                        format!("{}-pulse-anim", self.id),
                        Animation::new(Duration::from_millis(1200))
                            .repeat()
                            .with_easing(gpui::ease_in_out),
                        |this, delta| this.opacity((0.5 - (delta * 0.5)).max(0.0)),
                    ),
            );
        }

        let mut root = div().id(self.id.clone()).relative().flex();
        if let Some(child) = self.child.take() {
            root = root.child(child());
        }

        root.child(self.indicator_host().child(badge))
    }
}

impl IntoElement for Indicator {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Indicator {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Indicator);

impl gpui::Styled for Indicator {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
