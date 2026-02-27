use std::rc::Rc;

use gpui::InteractiveElement;
use gpui::StatefulInteractiveElement;
use gpui::{
    AppContext, Bounds, ClickEvent, Corners, EmptyView, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, canvas, div, fill, point, px, size,
};

use crate::contracts::{FieldLike, MotionAware};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};
use crate::theme::SemanticRadiusToken;

use super::Stack;
use super::control;
use super::slider_axis::{self, SliderAxis};
use super::utils::{apply_radius, quantized_stroke_px, resolve_hsla, resolve_radius, snap_px};

type ChangeHandler = Rc<dyn Fn(f32, &mut Window, &mut gpui::App)>;

#[derive(Clone)]
struct SliderDragState {
    slider_id: String,
    min: f32,
    max: f32,
    step: f32,
    controlled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SliderOrientation {
    Horizontal,
    Vertical,
}

#[derive(IntoElement)]
pub struct Slider {
    pub(crate) id: ComponentId,
    value: f32,
    value_controlled: bool,
    default_value: f32,
    min: f32,
    max: f32,
    step: f32,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    show_value: bool,
    disabled: bool,
    width_px: Option<f32>,
    orientation: SliderOrientation,
    variant: Variant,
    size: Size,
    radius: Radius,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl Slider {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            value: 0.0,
            value_controlled: false,
            default_value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            label: None,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            show_value: true,
            disabled: false,
            width_px: None,
            orientation: SliderOrientation::Horizontal,
            variant: Variant::Filled,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn horizontal() -> Self {
        Self::new().with_orientation(SliderOrientation::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new().with_orientation(SliderOrientation::Vertical)
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: f32) -> Self {
        self.default_value = value;
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        self.min = min;
        self.max = max;
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        if self.min > self.max {
            self.max = self.min;
        }
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        if self.max < self.min {
            self.min = self.max;
        }
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

    pub fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn error(mut self, value: impl Into<SharedString>) -> Self {
        self.error = Some(value.into());
        self
    }

    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }

    pub fn layout(mut self, value: FieldLayout) -> Self {
        self.layout = value;
        self
    }

    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }
    pub fn width(mut self, width_px: f32) -> Self {
        self.width_px = Some(width_px.max(0.0));
        self
    }

    fn with_orientation(mut self, value: SliderOrientation) -> Self {
        self.orientation = value;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(f32, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn normalize(&self, raw: f32) -> f32 {
        slider_axis::normalize(self.min, self.max, self.step, raw)
    }

    fn resolved_value(&self) -> f32 {
        let controlled = self.value_controlled.then_some(self.normalize(self.value));
        let default = self.normalize(self.default_value);
        self.normalize(control::f32_state(&self.id, "value", controlled, default))
    }

    fn ratio(&self, value: f32) -> f32 {
        slider_axis::ratio(self.min, self.max, value)
    }

    fn segments(&self) -> usize {
        let span = (self.max - self.min).max(self.step.max(0.001));
        ((span / self.step.max(0.001)).round() as usize).clamp(1, 80)
    }

    fn filled_color(&self) -> gpui::Hsla {
        let base = resolve_hsla(&self.theme, self.theme.components.slider.fill_bg);
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.78),
            Variant::Subtle => base.alpha(0.62),
            Variant::Outline => base.alpha(0.88),
            Variant::Ghost => base.alpha(0.55),
        }
    }
}

impl Slider {}

crate::impl_variant_size_radius_via_methods!(Slider, variant, size, radius);

impl MotionAware for Slider {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Slider {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.slider;
        let size_preset = tokens.sizes.for_size(self.size);
        let track_len = self
            .width_px
            .unwrap_or_else(|| f32::from(tokens.default_width))
            .max(f32::from(tokens.min_width));
        let value = self.resolved_value();
        let ratio = self.ratio(value);
        let track_height = f32::from(size_preset.track_thickness);
        let thumb_size = f32::from(size_preset.thumb_size);
        let track_top = ((thumb_size - track_height) * 0.5).max(0.0);
        let thumb_left =
            slider_axis::thumb_offset(SliderAxis::Horizontal, track_len, thumb_size, ratio);
        let segment_count = self.segments();
        let track_color = resolve_hsla(&self.theme, tokens.track_bg);
        let fill_color = self.filled_color();
        let tick_color = resolve_hsla(&self.theme, tokens.thumb_border).alpha(0.35);
        let tick_thickness = f32::from(quantized_stroke_px(window, 1.0));
        let track_corner = Corners::all(resolve_radius(
            &self.theme,
            SemanticRadiusToken::from(self.radius),
        ));
        let display_precision = if self.step < 1.0 { 2 } else { 0 };
        let is_controlled = self.value_controlled;
        let orientation = self.orientation;
        let on_change = self.on_change.clone();
        let label_text = self.label.clone().map(|label| {
            if self.required {
                SharedString::from(format!("{label} *"))
            } else {
                label
            }
        });
        let description = self.description.clone();
        let error = self.error.clone();
        let has_meta =
            label_text.is_some() || self.show_value || description.is_some() || error.is_some();

        if orientation == SliderOrientation::Vertical {
            let track_left = ((thumb_size - track_height) * 0.5).max(0.0);
            let thumb_top =
                slider_axis::thumb_offset(SliderAxis::Vertical, track_len, thumb_size, ratio);
            let fill_top = (thumb_top + (thumb_size * 0.5)).clamp(0.0, track_len);
            let fill_height = (track_len - fill_top).max(0.0);
            let track_layer = canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let track_bounds = Bounds::new(
                        point(bounds.origin.x + px(track_left), bounds.origin.y),
                        size(px(track_height), px(track_len)),
                    );
                    window.paint_quad(fill(track_bounds, track_color).corner_radii(track_corner));

                    if fill_height > 0.0 {
                        let fill_bounds = Bounds::new(
                            point(
                                bounds.origin.x + px(track_left),
                                bounds.origin.y + px(fill_top),
                            ),
                            size(px(track_height), px(fill_height)),
                        );
                        window.paint_quad(fill(fill_bounds, fill_color).corner_radii(track_corner));
                    }

                    if segment_count > 1 && tick_thickness > 0.0 {
                        for index in 1..segment_count {
                            let y = track_len * (index as f32 / segment_count as f32);
                            let tick_top = f32::from(snap_px(window, y - (tick_thickness * 0.5)));
                            let tick_bounds = Bounds::new(
                                point(
                                    bounds.origin.x + px(track_left),
                                    bounds.origin.y + px(tick_top),
                                ),
                                size(px(track_height), px(tick_thickness)),
                            );
                            window.paint_quad(fill(tick_bounds, tick_color));
                        }
                    }
                },
            )
            .absolute()
            .size_full();

            let mut thumb = div()
                .id(self.id.slot("thumb"))
                .absolute()
                .top(px(thumb_top))
                .left_0()
                .w(px(thumb_size))
                .h(px(thumb_size))
                .cursor_pointer()
                .border(quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(&self.theme, tokens.thumb_border))
                .bg(resolve_hsla(&self.theme, tokens.thumb_bg));
            thumb = apply_radius(&self.theme, thumb, Radius::Pill);
            if self.disabled {
                thumb = thumb.opacity(0.65);
            }

            let mut rail = div()
                .id(self.id.slot("rail"))
                .relative()
                .w(px(thumb_size))
                .h(px(track_len))
                .child(track_layer)
                .child(thumb);

            if !self.disabled {
                let id = self.id.clone();
                let min = self.min;
                let max = self.max;
                let step = self.step;
                let on_change = on_change.clone();
                let drag_state = SliderDragState {
                    slider_id: self.id.to_string(),
                    min: self.min,
                    max: self.max,
                    step: self.step,
                    controlled: is_controlled,
                };
                let slider_id = self.id.to_string();
                let on_change_for_drag = on_change.clone();

                rail = rail
                    .cursor_pointer()
                    .on_click(move |event: &ClickEvent, window, cx| {
                        let local_y = f32::from(event.position().y).clamp(0.0, track_len);
                        let raw = slider_axis::value_from_local(
                            SliderAxis::Vertical,
                            local_y,
                            track_len,
                            min,
                            max,
                        );
                        let next = slider_axis::normalize(min, max, step, raw);
                        if !is_controlled {
                            control::set_f32_state(&id, "value", next);
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(next, window, cx);
                        }
                    })
                    .on_drag(drag_state, |_drag, _, _, cx| cx.new(|_| EmptyView))
                    .on_drag_move::<SliderDragState>(move |event, window, cx| {
                        let drag = event.drag(cx);
                        if drag.slider_id != slider_id {
                            return;
                        }
                        let bounds = event.bounds;
                        let height = f32::from(bounds.size.height).max(1.0);
                        let local_y = (f32::from(event.event.position.y)
                            - f32::from(bounds.origin.y))
                        .clamp(0.0, height);
                        let raw = slider_axis::value_from_local(
                            SliderAxis::Vertical,
                            local_y,
                            height,
                            drag.min,
                            drag.max,
                        );
                        let next = slider_axis::normalize(drag.min, drag.max, drag.step, raw);

                        if !drag.controlled {
                            control::set_f32_state(&slider_id, "value", next);
                            window.refresh();
                        }
                        if let Some(handler) = on_change_for_drag.as_ref() {
                            (handler)(next, window, cx);
                        }
                    });
            }

            let mut meta = Stack::vertical()
                .gap(tokens.header_gap_vertical)
                .items_center();
            if label_text.is_some() || self.show_value {
                let mut header = Stack::vertical()
                    .items_center()
                    .gap(tokens.header_gap_vertical);
                if let Some(label) = label_text.clone() {
                    header = header.child(
                        div()
                            .text_size(tokens.label_size)
                            .text_color(resolve_hsla(&self.theme, tokens.label))
                            .child(label),
                    );
                }
                if self.show_value {
                    header = header.child(
                        div()
                            .text_size(tokens.value_size)
                            .text_color(resolve_hsla(&self.theme, tokens.value))
                            .child(format!("{value:.display_precision$}")),
                    );
                }
                meta = meta.child(header);
            }
            if let Some(description) = description.clone() {
                meta = meta.child(
                    div()
                        .text_size(tokens.label_size)
                        .text_color(resolve_hsla(&self.theme, tokens.label).alpha(0.78))
                        .child(description),
                );
            }
            if let Some(error) = error.clone() {
                meta = meta.child(
                    div()
                        .text_size(tokens.label_size)
                        .text_color(resolve_hsla(&self.theme, self.theme.semantic.status_error))
                        .child(error),
                );
            }

            return match self.layout {
                FieldLayout::Vertical => {
                    let mut container = Stack::vertical()
                        .id(self.id.clone())
                        .gap(tokens.header_gap_vertical)
                        .items_center();
                    if has_meta {
                        container = container.child(meta);
                    }
                    container
                        .child(rail)
                        .with_enter_transition(self.id.slot("enter"), self.motion)
                        .into_any_element()
                }
                FieldLayout::Horizontal => {
                    let mut container = Stack::horizontal()
                        .id(self.id.clone())
                        .items_start()
                        .gap(tokens.header_gap_horizontal);
                    if has_meta {
                        container = container.child(meta);
                    }
                    container
                        .child(rail)
                        .with_enter_transition(self.id.slot("enter"), self.motion)
                        .into_any_element()
                }
            };
        }

        let fill_width = (track_len * ratio).clamp(0.0, track_len);
        let track_layer = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let track_bounds = Bounds::new(
                    point(bounds.origin.x, bounds.origin.y + px(track_top)),
                    size(px(track_len), px(track_height)),
                );
                window.paint_quad(fill(track_bounds, track_color).corner_radii(track_corner));

                if fill_width > 0.0 {
                    let fill_bounds = Bounds::new(
                        point(bounds.origin.x, bounds.origin.y + px(track_top)),
                        size(px(fill_width), px(track_height)),
                    );
                    window.paint_quad(fill(fill_bounds, fill_color).corner_radii(track_corner));
                }

                if segment_count > 1 && tick_thickness > 0.0 {
                    for index in 1..segment_count {
                        let x = track_len * (index as f32 / segment_count as f32);
                        let tick_left = f32::from(snap_px(window, x - (tick_thickness * 0.5)));
                        let tick_bounds = Bounds::new(
                            point(
                                bounds.origin.x + px(tick_left),
                                bounds.origin.y + px(track_top),
                            ),
                            size(px(tick_thickness), px(track_height)),
                        );
                        window.paint_quad(fill(tick_bounds, tick_color));
                    }
                }
            },
        )
        .absolute()
        .size_full();

        let mut thumb = div()
            .id(self.id.slot("thumb"))
            .absolute()
            .top_0()
            .left(px(thumb_left))
            .w(px(thumb_size))
            .h(px(thumb_size))
            .cursor_pointer()
            .border(quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, tokens.thumb_border))
            .bg(resolve_hsla(&self.theme, tokens.thumb_bg));
        thumb = apply_radius(&self.theme, thumb, Radius::Pill);
        if self.disabled {
            thumb = thumb.opacity(0.65);
        }

        let mut rail = div()
            .id(self.id.slot("rail"))
            .relative()
            .w(px(track_len))
            .h(px(thumb_size))
            .child(track_layer)
            .child(thumb);

        if !self.disabled {
            let drag_state = SliderDragState {
                slider_id: self.id.to_string(),
                min: self.min,
                max: self.max,
                step: self.step,
                controlled: is_controlled,
            };
            let slider_id = self.id.to_string();
            let on_change_for_drag = on_change.clone();
            let id = self.id.clone();
            let min = self.min;
            let max = self.max;
            let step = self.step;
            let on_change_for_click = on_change.clone();

            rail = rail
                .cursor_pointer()
                .on_click(move |event: &ClickEvent, window, cx| {
                    let local_x = f32::from(event.position().x).clamp(0.0, track_len);
                    let raw = slider_axis::value_from_local(
                        SliderAxis::Horizontal,
                        local_x,
                        track_len,
                        min,
                        max,
                    );
                    let next = slider_axis::normalize(min, max, step, raw);
                    if !is_controlled {
                        control::set_f32_state(&id, "value", next);
                        window.refresh();
                    }
                    if let Some(handler) = on_change_for_click.as_ref() {
                        (handler)(next, window, cx);
                    }
                })
                .on_drag(drag_state, |_drag, _, _, cx| cx.new(|_| EmptyView))
                .on_drag_move::<SliderDragState>(move |event, window, cx| {
                    let drag = event.drag(cx);
                    if drag.slider_id != slider_id {
                        return;
                    }

                    let bounds = event.bounds;
                    let width = f32::from(bounds.size.width).max(1.0);
                    let local_x = (f32::from(event.event.position.x) - f32::from(bounds.origin.x))
                        .clamp(0.0, width);
                    let raw = slider_axis::value_from_local(
                        SliderAxis::Horizontal,
                        local_x,
                        width,
                        drag.min,
                        drag.max,
                    );
                    let next = slider_axis::normalize(drag.min, drag.max, drag.step, raw);

                    if !drag.controlled {
                        control::set_f32_state(&slider_id, "value", next);
                        window.refresh();
                    }
                    if let Some(handler) = on_change_for_drag.as_ref() {
                        (handler)(next, window, cx);
                    }
                });
        }

        let mut meta = Stack::vertical().gap(tokens.header_gap_vertical);
        if label_text.is_some() || self.show_value {
            let mut header = Stack::horizontal()
                .justify_between()
                .items_center()
                .w(px(track_len))
                .gap(tokens.header_gap_horizontal);
            if let Some(label) = label_text {
                header = header.child(
                    div()
                        .text_size(tokens.label_size)
                        .text_color(resolve_hsla(&self.theme, tokens.label))
                        .child(label),
                );
            }
            if self.show_value {
                header = header.child(
                    div()
                        .text_size(tokens.value_size)
                        .text_color(resolve_hsla(&self.theme, tokens.value))
                        .child(format!("{value:.display_precision$}")),
                );
            }
            meta = meta.child(header);
        }
        if let Some(description) = description {
            meta = meta.child(
                div()
                    .text_size(tokens.label_size)
                    .text_color(resolve_hsla(&self.theme, tokens.label).alpha(0.78))
                    .child(description),
            );
        }
        if let Some(error) = error {
            meta = meta.child(
                div()
                    .text_size(tokens.label_size)
                    .text_color(resolve_hsla(&self.theme, self.theme.semantic.status_error))
                    .child(error),
            );
        }

        match self.layout {
            FieldLayout::Vertical => {
                let mut container = Stack::vertical()
                    .id(self.id.clone())
                    .gap(tokens.header_gap_vertical);
                if has_meta {
                    container = container.child(meta);
                }
                container
                    .child(rail)
                    .with_enter_transition(self.id.slot("enter"), self.motion)
                    .into_any_element()
            }
            FieldLayout::Horizontal => {
                let mut container = Stack::horizontal()
                    .id(self.id.clone())
                    .items_start()
                    .gap(tokens.header_gap_horizontal);
                if has_meta {
                    container = container.child(meta);
                }
                container
                    .child(rail)
                    .with_enter_transition(self.id.slot("enter"), self.motion)
                    .into_any_element()
            }
        }
    }
}

crate::impl_disableable!(Slider, |this, value| this.disabled = value);

impl FieldLike for Slider {
    fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }

    fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.description = Some(value.into());
        self
    }

    fn error(mut self, value: impl Into<SharedString>) -> Self {
        self.error = Some(value.into());
        self
    }

    fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }

    fn layout(mut self, value: FieldLayout) -> Self {
        self.layout = value;
        self
    }
}
