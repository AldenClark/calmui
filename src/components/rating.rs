use std::rc::Rc;

use gpui::{
    ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::{MotionAware, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::control;
use super::icon::Icon;
use super::primitives::h_stack;
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type ChangeHandler = Rc<dyn Fn(f32, &mut Window, &mut gpui::App)>;

pub struct Rating {
    id: String,
    value: Option<f32>,
    value_controlled: bool,
    default_value: f32,
    max: usize,
    allow_half: bool,
    clearable: bool,
    disabled: bool,
    read_only: bool,
    size: Size,
    radius: Radius,
    variant: Variant,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl Rating {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("rating"),
            value: None,
            value_controlled: false,
            default_value: 0.0,
            max: 5,
            allow_half: true,
            clearable: false,
            disabled: false,
            read_only: false,
            size: Size::Md,
            radius: Radius::Sm,
            variant: Variant::Filled,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = Some(value);
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: f32) -> Self {
        self.default_value = value;
        self
    }

    pub fn max(mut self, value: usize) -> Self {
        self.max = value.max(1);
        self
    }

    pub fn allow_half(mut self, value: bool) -> Self {
        self.allow_half = value;
        self
    }

    pub fn clearable(mut self, value: bool) -> Self {
        self.clearable = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(f32, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_value(&self) -> f32 {
        let max = self.max as f32;
        let controlled = self.value_controlled.then_some(
            self.value
                .unwrap_or(self.default_value)
                .clamp(0.0, max)
                .to_string(),
        );
        let default = self.default_value.clamp(0.0, max).to_string();

        control::text_state(&self.id, "value", controlled, default)
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0)
            .clamp(0.0, max)
    }

    fn icon_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 14.0,
            Size::Sm => 16.0,
            Size::Md => 18.0,
            Size::Lg => 22.0,
            Size::Xl => 26.0,
        }
    }
}

impl WithId for Rating {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for Rating {
    fn variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn radius(mut self, value: Radius) -> Self {
        self.radius = value;
        self
    }
}

impl MotionAware for Rating {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Rating {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.rating;
        let value = self.resolved_value();
        let icon_size = self.icon_size_px();
        let active = resolve_hsla(&self.theme, &tokens.active);
        let inactive = resolve_hsla(&self.theme, &tokens.inactive);

        let stars = (1..=self.max)
            .map(|index| {
                let index_value = index as f32;
                let is_full = value >= index_value;
                let is_half =
                    self.allow_half && value >= (index_value - 0.5) && value < index_value;

                let icon = if is_full {
                    Icon::named_filled("star")
                } else if is_half {
                    Icon::named_outline("star-half")
                } else {
                    Icon::named_outline("star")
                }
                .with_id(format!("{}-star-{index}", self.id))
                .size(icon_size)
                .color(if is_full || is_half { active } else { inactive });

                let mut cell = div()
                    .id(format!("{}-cell-{index}", self.id))
                    .relative()
                    .child(icon)
                    .text_color(if is_full || is_half { active } else { inactive });

                if self.disabled || self.read_only {
                    cell = cell.opacity(0.6).cursor_default();
                } else {
                    let id = self.id.clone();
                    let clearable = self.clearable;
                    let current = value;
                    let value_controlled = self.value_controlled;
                    let on_change = self.on_change.clone();
                    if self.allow_half {
                        let id_for_left = id.clone();
                        let id_for_right = id.clone();
                        let on_change_left = on_change.clone();
                        let on_change_right = on_change.clone();
                        let left_target = (index_value - 0.5).max(0.0);
                        let right_target = index_value;
                        let left_value = if clearable && (current - left_target).abs() < 0.001 {
                            0.0
                        } else {
                            left_target
                        };
                        let right_value = if clearable && (current - right_target).abs() < 0.001 {
                            0.0
                        } else {
                            right_target
                        };

                        cell = cell
                            .cursor_pointer()
                            .child(
                                div()
                                    .id(format!("{}-cell-{index}-left", self.id))
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .w(gpui::px(icon_size * 0.5))
                                    .h(gpui::px(icon_size))
                                    .cursor_pointer()
                                    .on_click(move |_: &ClickEvent, window, cx| {
                                        if !value_controlled {
                                            control::set_text_state(
                                                &id_for_left,
                                                "value",
                                                left_value.to_string(),
                                            );
                                            window.refresh();
                                        }
                                        if let Some(handler) = on_change_left.as_ref() {
                                            (handler)(left_value, window, cx);
                                        }
                                    }),
                            )
                            .child(
                                div()
                                    .id(format!("{}-cell-{index}-right", self.id))
                                    .absolute()
                                    .top_0()
                                    .right_0()
                                    .w(gpui::px(icon_size * 0.5))
                                    .h(gpui::px(icon_size))
                                    .cursor_pointer()
                                    .on_click(move |_: &ClickEvent, window, cx| {
                                        if !value_controlled {
                                            control::set_text_state(
                                                &id_for_right,
                                                "value",
                                                right_value.to_string(),
                                            );
                                            window.refresh();
                                        }
                                        if let Some(handler) = on_change_right.as_ref() {
                                            (handler)(right_value, window, cx);
                                        }
                                    }),
                            );
                    } else {
                        let next_value = if clearable && (current - index_value).abs() < 0.001 {
                            0.0
                        } else {
                            index_value
                        };
                        cell = cell
                            .cursor_pointer()
                            .on_click(move |_: &ClickEvent, window, cx| {
                                if !value_controlled {
                                    control::set_text_state(&id, "value", next_value.to_string());
                                    window.refresh();
                                }
                                if let Some(handler) = on_change.as_ref() {
                                    (handler)(next_value, window, cx);
                                }
                            });
                    }
                }

                cell.into_any_element()
            })
            .collect::<Vec<_>>();

        h_stack()
            .id(self.id.clone())
            .items_center()
            .gap_1()
            .children(stars)
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Rating {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Rating {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
