use std::rc::Rc;

use gpui::{
    AppContext, ClickEvent, Component, EmptyView, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, canvas, div, px,
};

use crate::contracts::{MotionAware, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::control;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn((f32, f32), &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RangeThumb {
    Left,
    Right,
}

#[derive(Clone)]
struct RangeSliderDragState {
    slider_id: String,
    thumb: RangeThumb,
    min: f32,
    max: f32,
    step: f32,
    controlled: bool,
    fallback_left: f32,
    fallback_right: f32,
}

pub struct RangeSlider {
    id: String,
    values: Option<(f32, f32)>,
    values_controlled: bool,
    default_values: (f32, f32),
    min: f32,
    max: f32,
    step: f32,
    label: Option<SharedString>,
    show_value: bool,
    disabled: bool,
    width_px: f32,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl RangeSlider {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("range-slider"),
            values: None,
            values_controlled: false,
            default_values: (20.0, 80.0),
            min: 0.0,
            max: 100.0,
            step: 1.0,
            label: None,
            show_value: true,
            disabled: false,
            width_px: 260.0,
            variant: Variant::Filled,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn values(mut self, start: f32, end: f32) -> Self {
        self.values = Some((start, end));
        self.values_controlled = true;
        self
    }

    pub fn clear_values(mut self) -> Self {
        self.values = None;
        self.values_controlled = true;
        self
    }

    pub fn default_values(mut self, start: f32, end: f32) -> Self {
        self.default_values = (start, end);
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step.max(0.001);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn width(mut self, width_px: f32) -> Self {
        self.width_px = width_px.max(140.0);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn((f32, f32), &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn normalize_with(min: f32, max: f32, step: f32, raw: f32) -> f32 {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        let step = step.max(0.001);
        let clamped = raw.clamp(min, max);
        let snapped = ((clamped - min) / step).round() * step + min;
        snapped.clamp(min, max)
    }

    fn normalize_pair_with(min: f32, max: f32, step: f32, left: f32, right: f32) -> (f32, f32) {
        let mut left = Self::normalize_with(min, max, step, left);
        let mut right = Self::normalize_with(min, max, step, right);
        if left > right {
            std::mem::swap(&mut left, &mut right);
        }
        (left, right)
    }

    fn resolved_values(&self) -> (f32, f32) {
        let controlled = if self.values_controlled {
            self.values
                .map(|(start, end)| vec![start.to_string(), end.to_string()])
        } else {
            None
        };
        let default = vec![
            self.default_values.0.to_string(),
            self.default_values.1.to_string(),
        ];
        let stored = control::list_state(&self.id, "values", controlled, default);

        let left = stored
            .first()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(self.default_values.0);
        let right = stored
            .get(1)
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(self.default_values.1);

        Self::normalize_pair_with(self.min, self.max, self.step, left, right)
    }

    fn state_values(id: &str, fallback: (f32, f32), min: f32, max: f32, step: f32) -> (f32, f32) {
        let stored = control::list_state(
            id,
            "values",
            None,
            vec![fallback.0.to_string(), fallback.1.to_string()],
        );
        let left = stored
            .first()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(fallback.0);
        let right = stored
            .get(1)
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(fallback.1);
        Self::normalize_pair_with(min, max, step, left, right)
    }

    fn ratio(&self, value: f32) -> f32 {
        let span = (self.max - self.min).max(0.001);
        ((value - self.min) / span).clamp(0.0, 1.0)
    }

    fn track_height_px(&self) -> f32 {
        match self.size {
            Size::Xs => 4.0,
            Size::Sm => 5.0,
            Size::Md => 6.0,
            Size::Lg => 8.0,
            Size::Xl => 10.0,
        }
    }

    fn thumb_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 12.0,
            Size::Sm => 14.0,
            Size::Md => 16.0,
            Size::Lg => 20.0,
            Size::Xl => 24.0,
        }
    }

    fn rail_origin_and_width(id: &str, fallback_width: f32) -> (f32, f32) {
        let origin_x = control::text_state(id, "rail-origin-x", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let width = control::text_state(id, "rail-width", None, fallback_width.to_string())
            .parse::<f32>()
            .ok()
            .filter(|width| *width > 1.0)
            .unwrap_or(fallback_width);
        (origin_x, width)
    }

    fn set_values_state(id: &str, values: (f32, f32)) {
        control::set_list_state(
            id,
            "values",
            vec![values.0.to_string(), values.1.to_string()],
        );
    }
}

impl WithId for RangeSlider {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for RangeSlider {
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

impl MotionAware for RangeSlider {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for RangeSlider {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.range_slider;
        let values = self.resolved_values();
        let left_ratio = self.ratio(values.0);
        let right_ratio = self.ratio(values.1);
        let track_height = self.track_height_px();
        let thumb_size = self.thumb_size_px();
        let track_top = ((thumb_size - track_height) * 0.5).max(0.0);
        let left_thumb_x = ((self.width_px - thumb_size) * left_ratio).max(0.0);
        let right_thumb_x = ((self.width_px - thumb_size) * right_ratio).max(0.0);
        let display_precision = if self.step < 1.0 { 2 } else { 0 };
        let is_controlled = self.values_controlled;

        let mut track = div()
            .id(format!("{}-track", self.id))
            .absolute()
            .top(px(track_top))
            .left_0()
            .w(px(self.width_px))
            .h(px(track_height))
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.track_bg))
            .bg(resolve_hsla(&self.theme, &tokens.track_bg));
        track = apply_radius(track, self.radius);

        let fill_left = left_thumb_x + (thumb_size * 0.5);
        let fill_right = right_thumb_x + (thumb_size * 0.5);
        let fill_width = (fill_right - fill_left).max(0.0);
        let fill = div()
            .id(format!("{}-range-fill", self.id))
            .absolute()
            .top_0()
            .left(px(fill_left))
            .w(px(fill_width))
            .h(px(track_height))
            .bg(resolve_hsla(&self.theme, &tokens.range_bg));

        let mut left_thumb = div()
            .id(format!("{}-thumb-left", self.id))
            .absolute()
            .top_0()
            .left(px(left_thumb_x))
            .w(px(thumb_size))
            .h(px(thumb_size))
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.thumb_border))
            .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
        left_thumb = apply_radius(left_thumb, Radius::Pill);

        let mut right_thumb = div()
            .id(format!("{}-thumb-right", self.id))
            .absolute()
            .top_0()
            .left(px(right_thumb_x))
            .w(px(thumb_size))
            .h(px(thumb_size))
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.thumb_border))
            .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
        right_thumb = apply_radius(right_thumb, Radius::Pill);

        let mut rail = div()
            .id(format!("{}-rail", self.id))
            .relative()
            .w(px(self.width_px))
            .h(px(thumb_size))
            .child(track)
            .child(fill)
            .child(
                canvas(
                    {
                        let id = self.id.clone();
                        move |bounds, _, _cx| {
                            control::set_text_state(
                                &id,
                                "rail-origin-x",
                                f32::from(bounds.origin.x).to_string(),
                            );
                            control::set_text_state(
                                &id,
                                "rail-width",
                                f32::from(bounds.size.width).to_string(),
                            );
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(left_thumb)
            .child(right_thumb);

        if self.disabled {
            rail = rail.opacity(0.65);
        } else {
            let id = self.id.clone();
            let min = self.min;
            let max = self.max;
            let step = self.step;
            let fallback = values;
            let on_change = self.on_change.clone();

            rail = rail
                .cursor_pointer()
                .on_click(move |event: &ClickEvent, window, cx| {
                    let (origin_x, width) = Self::rail_origin_and_width(&id, 260.0);
                    let local_x = (f32::from(event.position().x) - origin_x).clamp(0.0, width);
                    let ratio = local_x / width.max(1.0);
                    let raw = min + ((max - min).max(0.001) * ratio);
                    let target = Self::normalize_with(min, max, step, raw);

                    let (left, right) = Self::state_values(&id, fallback, min, max, step);
                    let next = if (target - left).abs() <= (target - right).abs() {
                        (target.min(right), right)
                    } else {
                        (left, target.max(left))
                    };

                    if !is_controlled {
                        Self::set_values_state(&id, next);
                        window.refresh();
                    }
                    if let Some(handler) = on_change.as_ref() {
                        (handler)(next, window, cx);
                    }
                });

            let drag_common = |thumb: RangeThumb| RangeSliderDragState {
                slider_id: self.id.clone(),
                thumb,
                min: self.min,
                max: self.max,
                step: self.step,
                controlled: is_controlled,
                fallback_left: values.0,
                fallback_right: values.1,
            };

            let on_change_for_drag = self.on_change.clone();
            let slider_id_for_drag = self.id.clone();

            rail = rail
                .on_drag(drag_common(RangeThumb::Left), |_drag, _, _, cx| {
                    cx.new(|_| EmptyView)
                })
                .on_drag(drag_common(RangeThumb::Right), |_drag, _, _, cx| {
                    cx.new(|_| EmptyView)
                })
                .on_drag_move::<RangeSliderDragState>(move |event, window, cx| {
                    let drag = event.drag(cx);
                    if drag.slider_id != slider_id_for_drag {
                        return;
                    }

                    let (origin_x, width) = Self::rail_origin_and_width(&drag.slider_id, 260.0);
                    let local_x =
                        (f32::from(event.event.position.x) - origin_x).clamp(0.0, width.max(1.0));
                    let ratio = local_x / width.max(1.0);
                    let raw = drag.min + ((drag.max - drag.min).max(0.001) * ratio);
                    let target = Self::normalize_with(drag.min, drag.max, drag.step, raw);

                    let fallback = (drag.fallback_left, drag.fallback_right);
                    let (left, right) = Self::state_values(
                        &drag.slider_id,
                        fallback,
                        drag.min,
                        drag.max,
                        drag.step,
                    );

                    let next = match drag.thumb {
                        RangeThumb::Left => (target.min(right), right),
                        RangeThumb::Right => (left, target.max(left)),
                    };

                    if !drag.controlled {
                        Self::set_values_state(&drag.slider_id, next);
                        window.refresh();
                    }
                    if let Some(handler) = on_change_for_drag.as_ref() {
                        (handler)(next, window, cx);
                    }
                });
        }

        let mut container = v_stack().id(self.id.clone()).gap_1p5();
        if self.label.is_some() || self.show_value {
            let mut header = h_stack()
                .justify_between()
                .items_center()
                .w(px(self.width_px))
                .text_sm()
                .text_color(resolve_hsla(&self.theme, &tokens.label));

            if let Some(label) = self.label {
                header = header.child(label);
            }
            if self.show_value {
                header = header.child(format!(
                    "{:.display_precision$} - {:.display_precision$}",
                    values.0, values.1
                ));
            }
            container = container.child(header);
        }

        container
            .child(rail)
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for RangeSlider {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for RangeSlider {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
