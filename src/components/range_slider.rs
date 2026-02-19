use std::rc::Rc;

use gpui::{
    AppContext, ClickEvent, EmptyView, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, canvas, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::Stack;
use super::control;
use super::slider_axis::{self, RailGeometry, SliderAxis};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn((f32, f32), &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RangeThumb {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RangeSliderOrientation {
    Horizontal,
    Vertical,
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

#[derive(IntoElement)]
pub struct RangeSlider {
    id: ComponentId,
    values: Option<(f32, f32)>,
    values_controlled: bool,
    default_values: (f32, f32),
    min: f32,
    max: f32,
    step: f32,
    label: Option<SharedString>,
    show_value: bool,
    disabled: bool,
    width_px: Option<f32>,
    orientation: RangeSliderOrientation,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl RangeSlider {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            values: None,
            values_controlled: false,
            default_values: (20.0, 80.0),
            min: 0.0,
            max: 100.0,
            step: 1.0,
            label: None,
            show_value: true,
            disabled: false,
            width_px: None,
            orientation: RangeSliderOrientation::Horizontal,
            variant: Variant::Filled,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn horizontal() -> Self {
        Self::new().with_orientation(RangeSliderOrientation::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new().with_orientation(RangeSliderOrientation::Vertical)
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
        self.width_px = Some(width_px.max(0.0));
        self
    }

    fn with_orientation(mut self, value: RangeSliderOrientation) -> Self {
        self.orientation = value;
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
        slider_axis::normalize(min, max, step, raw)
    }

    fn normalize_pair_with(min: f32, max: f32, step: f32, left: f32, right: f32) -> (f32, f32) {
        slider_axis::normalize_pair(min, max, step, left, right)
    }

    fn resolved_values(&self) -> (f32, f32) {
        let controlled_left = self.values_controlled.then_some(
            self.values
                .map(|(start, _)| start)
                .unwrap_or(self.default_values.0),
        );
        let controlled_right = self.values_controlled.then_some(
            self.values
                .map(|(_, end)| end)
                .unwrap_or(self.default_values.1),
        );
        let left = control::f32_state(
            &self.id,
            "value-left",
            controlled_left,
            self.default_values.0,
        );
        let right = control::f32_state(
            &self.id,
            "value-right",
            controlled_right,
            self.default_values.1,
        );

        Self::normalize_pair_with(self.min, self.max, self.step, left, right)
    }

    fn state_values(id: &str, fallback: (f32, f32), min: f32, max: f32, step: f32) -> (f32, f32) {
        let left = control::f32_state(id, "value-left", None, fallback.0);
        let right = control::f32_state(id, "value-right", None, fallback.1);
        Self::normalize_pair_with(min, max, step, left, right)
    }

    fn ratio(&self, value: f32) -> f32 {
        slider_axis::ratio(self.min, self.max, value)
    }

    fn rail_geometry(id: &str, fallback_width: f32, fallback_height: f32) -> RailGeometry {
        RailGeometry::from_state(id, fallback_width, fallback_height)
    }

    fn set_values_state(id: &str, values: (f32, f32)) {
        control::set_f32_state(id, "value-left", values.0);
        control::set_f32_state(id, "value-right", values.1);
    }
}

impl RangeSlider {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for RangeSlider {
    fn with_variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn with_radius(mut self, value: Radius) -> Self {
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
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.range_slider;
        let size_preset = tokens.sizes.for_size(self.size);
        let track_len = self
            .width_px
            .unwrap_or_else(|| f32::from(tokens.default_width))
            .max(f32::from(tokens.min_width));
        let values = self.resolved_values();
        let left_ratio = self.ratio(values.0);
        let right_ratio = self.ratio(values.1);
        let track_height = f32::from(size_preset.track_thickness);
        let thumb_size = f32::from(size_preset.thumb_size);
        let track_top = ((thumb_size - track_height) * 0.5).max(0.0);
        let left_thumb_x = ((track_len - thumb_size) * left_ratio).max(0.0);
        let right_thumb_x = ((track_len - thumb_size) * right_ratio).max(0.0);
        let display_precision = if self.step < 1.0 { 2 } else { 0 };
        let is_controlled = self.values_controlled;
        let orientation = self.orientation;

        if orientation == RangeSliderOrientation::Vertical {
            let track_left = ((thumb_size - track_height) * 0.5).max(0.0);
            let left_thumb_y = ((track_len - thumb_size) * (1.0 - left_ratio)).max(0.0);
            let right_thumb_y = ((track_len - thumb_size) * (1.0 - right_ratio)).max(0.0);
            let fill_top = right_thumb_y + (thumb_size * 0.5);
            let fill_bottom = left_thumb_y + (thumb_size * 0.5);
            let fill_height = (fill_bottom - fill_top).max(0.0);

            let mut track = div()
                .id(self.id.slot("track"))
                .absolute()
                .top_0()
                .left(px(track_left))
                .w(px(track_height))
                .h(px(track_len))
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(&self.theme, &tokens.track_bg))
                .bg(resolve_hsla(&self.theme, &tokens.track_bg));
            track = apply_radius(&self.theme, track, self.radius);

            let mut fill = div()
                .id(self.id.slot("range-fill"))
                .absolute()
                .top(px(fill_top))
                .left(px(track_left))
                .w(px(track_height))
                .h(px(fill_height))
                .bg(resolve_hsla(&self.theme, &tokens.range_bg));
            fill = apply_radius(&self.theme, fill, self.radius);

            let mut left_thumb = div()
                .id(self.id.slot("thumb-left"))
                .absolute()
                .top(px(left_thumb_y))
                .left_0()
                .w(px(thumb_size))
                .h(px(thumb_size))
                .cursor_pointer()
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(&self.theme, &tokens.thumb_border))
                .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
            left_thumb = apply_radius(&self.theme, left_thumb, Radius::Pill);

            let mut right_thumb = div()
                .id(self.id.slot("thumb-right"))
                .absolute()
                .top(px(right_thumb_y))
                .left_0()
                .w(px(thumb_size))
                .h(px(thumb_size))
                .cursor_pointer()
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(&self.theme, &tokens.thumb_border))
                .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
            right_thumb = apply_radius(&self.theme, right_thumb, Radius::Pill);

            if !self.disabled {
                let drag_common = |thumb: RangeThumb| RangeSliderDragState {
                    slider_id: self.id.to_string(),
                    thumb,
                    min: self.min,
                    max: self.max,
                    step: self.step,
                    controlled: is_controlled,
                    fallback_left: values.0,
                    fallback_right: values.1,
                };

                let on_change_for_drag_left = self.on_change.clone();
                let on_change_for_drag_right = self.on_change.clone();
                let slider_id_for_drag_left = self.id.to_string();
                let slider_id_for_drag_right = self.id.to_string();

                left_thumb = left_thumb
                    .on_drag(drag_common(RangeThumb::Left), |_drag, _, _, cx| {
                        cx.new(|_| EmptyView)
                    })
                    .on_drag_move::<RangeSliderDragState>(move |event, window, cx| {
                        let drag = event.drag(cx);
                        if drag.slider_id != slider_id_for_drag_left {
                            return;
                        }

                        let geometry = Self::rail_geometry(&drag.slider_id, thumb_size, track_len);
                        let axis = SliderAxis::Vertical;
                        let local_y = axis
                            .local(
                                f32::from(event.event.position.x),
                                f32::from(event.event.position.y),
                                geometry.origin_x,
                                geometry.origin_y,
                            )
                            .clamp(0.0, axis.length(geometry.width, geometry.height));
                        let raw = slider_axis::value_from_local(
                            axis,
                            local_y,
                            axis.length(geometry.width, geometry.height),
                            drag.min,
                            drag.max,
                        );
                        let target = Self::normalize_with(drag.min, drag.max, drag.step, raw);
                        let fallback = (drag.fallback_left, drag.fallback_right);
                        let (_left, right) = Self::state_values(
                            &drag.slider_id,
                            fallback,
                            drag.min,
                            drag.max,
                            drag.step,
                        );
                        let next = (target.min(right), right);

                        if !drag.controlled {
                            Self::set_values_state(&drag.slider_id, next);
                            window.refresh();
                        }
                        if let Some(handler) = on_change_for_drag_left.as_ref() {
                            (handler)(next, window, cx);
                        }
                    });

                right_thumb = right_thumb
                    .on_drag(drag_common(RangeThumb::Right), |_drag, _, _, cx| {
                        cx.new(|_| EmptyView)
                    })
                    .on_drag_move::<RangeSliderDragState>(move |event, window, cx| {
                        let drag = event.drag(cx);
                        if drag.slider_id != slider_id_for_drag_right {
                            return;
                        }

                        let geometry = Self::rail_geometry(&drag.slider_id, thumb_size, track_len);
                        let axis = SliderAxis::Vertical;
                        let local_y = axis
                            .local(
                                f32::from(event.event.position.x),
                                f32::from(event.event.position.y),
                                geometry.origin_x,
                                geometry.origin_y,
                            )
                            .clamp(0.0, axis.length(geometry.width, geometry.height));
                        let raw = slider_axis::value_from_local(
                            axis,
                            local_y,
                            axis.length(geometry.width, geometry.height),
                            drag.min,
                            drag.max,
                        );
                        let target = Self::normalize_with(drag.min, drag.max, drag.step, raw);
                        let fallback = (drag.fallback_left, drag.fallback_right);
                        let (left, _right) = Self::state_values(
                            &drag.slider_id,
                            fallback,
                            drag.min,
                            drag.max,
                            drag.step,
                        );
                        let next = (left, target.max(left));

                        if !drag.controlled {
                            Self::set_values_state(&drag.slider_id, next);
                            window.refresh();
                        }
                        if let Some(handler) = on_change_for_drag_right.as_ref() {
                            (handler)(next, window, cx);
                        }
                    });
            }

            let mut rail = div()
                .id(self.id.slot("rail"))
                .relative()
                .w(px(thumb_size))
                .h(px(track_len))
                .child(track)
                .child(fill)
                .child(
                    canvas(
                        {
                            let id = self.id.clone();
                            move |bounds, _, _cx| {
                                RailGeometry::store(
                                    &id,
                                    f32::from(bounds.origin.x),
                                    f32::from(bounds.origin.y),
                                    f32::from(bounds.size.width),
                                    f32::from(bounds.size.height),
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
                        let geometry = Self::rail_geometry(&id, thumb_size, track_len);
                        let axis = SliderAxis::Vertical;
                        let local_y = axis
                            .local(
                                f32::from(event.position().x),
                                f32::from(event.position().y),
                                geometry.origin_x,
                                geometry.origin_y,
                            )
                            .clamp(0.0, axis.length(geometry.width, geometry.height));
                        let raw = slider_axis::value_from_local(
                            axis,
                            local_y,
                            axis.length(geometry.width, geometry.height),
                            min,
                            max,
                        );
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
            }

            let mut container = Stack::vertical()
                .id(self.id.clone())
                .gap(tokens.header_gap_vertical)
                .items_center();
            if self.label.is_some() || self.show_value {
                let mut header = Stack::vertical()
                    .items_center()
                    .gap(tokens.header_gap_vertical);
                if let Some(label) = self.label {
                    header = header.child(
                        div()
                            .text_size(tokens.label_size)
                            .text_color(resolve_hsla(&self.theme, &tokens.label))
                            .child(label),
                    );
                }
                if self.show_value {
                    header = header.child(
                        div()
                            .text_size(tokens.value_size)
                            .text_color(resolve_hsla(&self.theme, &tokens.value))
                            .child(format!(
                                "{:.display_precision$} - {:.display_precision$}",
                                values.0, values.1
                            )),
                    );
                }
                container = container.child(header);
            }

            return container
                .child(rail)
                .with_enter_transition(self.id.slot("enter"), self.motion);
        }

        let mut track = div()
            .id(self.id.slot("track"))
            .absolute()
            .top(px(track_top))
            .left_0()
            .w(px(track_len))
            .h(px(track_height))
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.track_bg))
            .bg(resolve_hsla(&self.theme, &tokens.track_bg));
        track = apply_radius(&self.theme, track, self.radius);

        let fill_left = left_thumb_x + (thumb_size * 0.5);
        let fill_right = right_thumb_x + (thumb_size * 0.5);
        let fill_width = (fill_right - fill_left).max(0.0);
        let fill = div()
            .id(self.id.slot("range-fill"))
            .absolute()
            .top(px(track_top))
            .left(px(fill_left))
            .w(px(fill_width))
            .h(px(track_height))
            .bg(resolve_hsla(&self.theme, &tokens.range_bg));

        let mut left_thumb = div()
            .id(self.id.slot("thumb-left"))
            .absolute()
            .top_0()
            .left(px(left_thumb_x))
            .w(px(thumb_size))
            .h(px(thumb_size))
            .cursor_pointer()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.thumb_border))
            .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
        left_thumb = apply_radius(&self.theme, left_thumb, Radius::Pill);

        let mut right_thumb = div()
            .id(self.id.slot("thumb-right"))
            .absolute()
            .top_0()
            .left(px(right_thumb_x))
            .w(px(thumb_size))
            .h(px(thumb_size))
            .cursor_pointer()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.thumb_border))
            .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
        right_thumb = apply_radius(&self.theme, right_thumb, Radius::Pill);

        if !self.disabled {
            let drag_common = |thumb: RangeThumb| RangeSliderDragState {
                slider_id: self.id.to_string(),
                thumb,
                min: self.min,
                max: self.max,
                step: self.step,
                controlled: is_controlled,
                fallback_left: values.0,
                fallback_right: values.1,
            };

            let on_change_for_drag_left = self.on_change.clone();
            let on_change_for_drag_right = self.on_change.clone();
            let slider_id_for_drag_left = self.id.to_string();
            let slider_id_for_drag_right = self.id.to_string();

            left_thumb = left_thumb
                .on_drag(drag_common(RangeThumb::Left), |_drag, _, _, cx| {
                    cx.new(|_| EmptyView)
                })
                .on_drag_move::<RangeSliderDragState>(move |event, window, cx| {
                    let drag = event.drag(cx);
                    if drag.slider_id != slider_id_for_drag_left {
                        return;
                    }

                    let geometry = Self::rail_geometry(&drag.slider_id, track_len, thumb_size);
                    let axis = SliderAxis::Horizontal;
                    let local_x = axis
                        .local(
                            f32::from(event.event.position.x),
                            f32::from(event.event.position.y),
                            geometry.origin_x,
                            geometry.origin_y,
                        )
                        .clamp(0.0, axis.length(geometry.width, geometry.height));
                    let raw = slider_axis::value_from_local(
                        axis,
                        local_x,
                        axis.length(geometry.width, geometry.height),
                        drag.min,
                        drag.max,
                    );
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
                    if let Some(handler) = on_change_for_drag_left.as_ref() {
                        (handler)(next, window, cx);
                    }
                });

            right_thumb = right_thumb
                .on_drag(drag_common(RangeThumb::Right), |_drag, _, _, cx| {
                    cx.new(|_| EmptyView)
                })
                .on_drag_move::<RangeSliderDragState>(move |event, window, cx| {
                    let drag = event.drag(cx);
                    if drag.slider_id != slider_id_for_drag_right {
                        return;
                    }

                    let geometry = Self::rail_geometry(&drag.slider_id, track_len, thumb_size);
                    let axis = SliderAxis::Horizontal;
                    let local_x = axis
                        .local(
                            f32::from(event.event.position.x),
                            f32::from(event.event.position.y),
                            geometry.origin_x,
                            geometry.origin_y,
                        )
                        .clamp(0.0, axis.length(geometry.width, geometry.height));
                    let raw = slider_axis::value_from_local(
                        axis,
                        local_x,
                        axis.length(geometry.width, geometry.height),
                        drag.min,
                        drag.max,
                    );
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
                    if let Some(handler) = on_change_for_drag_right.as_ref() {
                        (handler)(next, window, cx);
                    }
                });
        }

        let mut rail = div()
            .id(self.id.slot("rail"))
            .relative()
            .w(px(track_len))
            .h(px(thumb_size))
            .child(track)
            .child(fill)
            .child(
                canvas(
                    {
                        let id = self.id.clone();
                        move |bounds, _, _cx| {
                            RailGeometry::store(
                                &id,
                                f32::from(bounds.origin.x),
                                f32::from(bounds.origin.y),
                                f32::from(bounds.size.width),
                                f32::from(bounds.size.height),
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
                    let geometry = Self::rail_geometry(&id, track_len, thumb_size);
                    let axis = SliderAxis::Horizontal;
                    let local_x = axis
                        .local(
                            f32::from(event.position().x),
                            f32::from(event.position().y),
                            geometry.origin_x,
                            geometry.origin_y,
                        )
                        .clamp(0.0, axis.length(geometry.width, geometry.height));
                    let raw = slider_axis::value_from_local(
                        axis,
                        local_x,
                        axis.length(geometry.width, geometry.height),
                        min,
                        max,
                    );
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
        }

        let mut container = Stack::vertical()
            .id(self.id.clone())
            .gap(tokens.header_gap_vertical);
        if self.label.is_some() || self.show_value {
            let mut header = Stack::horizontal()
                .justify_between()
                .items_center()
                .w(px(track_len))
                .gap(tokens.header_gap_horizontal);

            if let Some(label) = self.label {
                header = header.child(
                    div()
                        .text_size(tokens.label_size)
                        .text_color(resolve_hsla(&self.theme, &tokens.label))
                        .child(label),
                );
            }
            if self.show_value {
                header = header.child(
                    div()
                        .text_size(tokens.value_size)
                        .text_color(resolve_hsla(&self.theme, &tokens.value))
                        .child(format!(
                            "{:.display_precision$} - {:.display_precision$}",
                            values.0, values.1
                        )),
                );
            }
            container = container.child(header);
        }

        container
            .child(rail)
            .with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for RangeSlider {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(RangeSlider);

impl gpui::Styled for RangeSlider {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
