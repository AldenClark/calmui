use std::rc::Rc;

use gpui::{
    AppContext, ClickEvent, Component, EmptyView, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::control;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

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
pub enum SliderOrientation {
    Horizontal,
    Vertical,
}

pub struct Slider {
    id: String,
    value: f32,
    value_controlled: bool,
    default_value: f32,
    min: f32,
    max: f32,
    step: f32,
    label: Option<SharedString>,
    show_value: bool,
    disabled: bool,
    width_px: f32,
    orientation: SliderOrientation,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl Slider {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("slider"),
            value: 0.0,
            value_controlled: false,
            default_value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            label: None,
            show_value: true,
            disabled: false,
            width_px: 260.0,
            orientation: SliderOrientation::Horizontal,
            variant: Variant::Filled,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
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

    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn width(mut self, width_px: f32) -> Self {
        self.width_px = width_px.max(120.0);
        self
    }

    pub fn orientation(mut self, value: SliderOrientation) -> Self {
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
        Self::normalize_with(self.min, self.max, self.step, raw)
    }

    fn normalize_with(min: f32, max: f32, step: f32, raw: f32) -> f32 {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        let clamped = raw.clamp(min, max);
        let step = step.max(0.001);
        let snapped = ((clamped - min) / step).round() * step + min;
        snapped.clamp(min, max)
    }

    fn resolved_value(&self) -> f32 {
        let controlled = self
            .value_controlled
            .then_some(self.normalize(self.value).to_string());
        let default = self.normalize(self.default_value).to_string();
        let stored = control::text_state(&self.id, "value", controlled, default);
        stored
            .parse::<f32>()
            .ok()
            .map(|value| self.normalize(value))
            .unwrap_or_else(|| self.normalize(self.default_value))
    }

    fn ratio(&self, value: f32) -> f32 {
        let span = (self.max - self.min).max(0.001);
        ((value - self.min) / span).clamp(0.0, 1.0)
    }

    fn segments(&self) -> usize {
        let span = (self.max - self.min).max(self.step.max(0.001));
        ((span / self.step.max(0.001)).round() as usize).clamp(1, 80)
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

    fn filled_color(&self) -> gpui::Hsla {
        let base = resolve_hsla(&self.theme, &self.theme.components.slider.fill_bg);
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.78),
            Variant::Subtle => base.alpha(0.62),
            Variant::Outline => base.alpha(0.88),
            Variant::Ghost => base.alpha(0.55),
        }
    }
}

impl WithId for Slider {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for Slider {
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

impl MotionAware for Slider {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Slider {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.slider;
        let value = self.resolved_value();
        let ratio = self.ratio(value);
        let track_height = self.track_height_px();
        let thumb_size = self.thumb_size_px();
        let track_top = ((thumb_size - track_height) * 0.5).max(0.0);
        let thumb_left = ((self.width_px - thumb_size) * ratio).max(0.0);
        let segment_count = self.segments();
        let display_precision = if self.step < 1.0 { 2 } else { 0 };
        let is_controlled = self.value_controlled;
        let orientation = self.orientation;
        let track_len = self.width_px;
        let on_change = self.on_change.clone();

        if orientation == SliderOrientation::Vertical {
            let track_left = ((thumb_size - track_height) * 0.5).max(0.0);
            let thumb_top = ((track_len - thumb_size) * (1.0 - ratio)).max(0.0);

            let mut track = div()
                .id(format!("{}-track", self.id))
                .absolute()
                .top_0()
                .left(px(track_left))
                .w(px(track_height))
                .h(px(track_len))
                .overflow_hidden()
                .border_1()
                .border_color(resolve_hsla(&self.theme, &tokens.track_bg))
                .bg(resolve_hsla(&self.theme, &tokens.track_bg));
            track = apply_radius(&self.theme, track, self.radius);

            let fill_top = (thumb_top + (thumb_size * 0.5)).clamp(0.0, track_len);
            let fill_height = (track_len - fill_top).max(0.0);
            let mut fill = div()
                .id(format!("{}-fill", self.id))
                .absolute()
                .left(px(track_left))
                .top(px(fill_top))
                .w(px(track_height))
                .h(px(fill_height))
                .bg(self.filled_color());
            fill = apply_radius(&self.theme, fill, self.radius);

            let mut thumb = div()
                .id(format!("{}-thumb", self.id))
                .absolute()
                .top(px(thumb_top))
                .left_0()
                .w(px(thumb_size))
                .h(px(thumb_size))
                .cursor_pointer()
                .border_1()
                .border_color(resolve_hsla(&self.theme, &tokens.thumb_border))
                .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
            thumb = apply_radius(&self.theme, thumb, Radius::Pill);
            if self.disabled {
                thumb = thumb.opacity(0.65);
            }

            let mut rail = div()
                .id(format!("{}-rail", self.id))
                .relative()
                .w(px(thumb_size))
                .h(px(track_len))
                .child(track)
                .child(fill)
                .child(thumb);

            if !self.disabled {
                let id = self.id.clone();
                let min = self.min;
                let max = self.max;
                let step = self.step;
                let on_change = on_change.clone();
                let drag_state = SliderDragState {
                    slider_id: self.id.clone(),
                    min: self.min,
                    max: self.max,
                    step: self.step,
                    controlled: is_controlled,
                };
                let slider_id = self.id.clone();
                let on_change_for_drag = on_change.clone();

                rail = rail
                    .cursor_pointer()
                    .on_click(move |event: &ClickEvent, window, cx| {
                        let local_y = f32::from(event.position().y).clamp(0.0, track_len);
                        let ratio = 1.0 - (local_y / track_len.max(1.0));
                        let raw = min + ((max - min).max(0.001) * ratio);
                        let next = Self::normalize_with(min, max, step, raw);
                        if !is_controlled {
                            control::set_text_state(&id, "value", next.to_string());
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
                        let ratio = 1.0 - (local_y / height);
                        let raw = drag.min + ((drag.max - drag.min).max(0.001) * ratio);
                        let next = Self::normalize_with(drag.min, drag.max, drag.step, raw);

                        if !drag.controlled {
                            control::set_text_state(&slider_id, "value", next.to_string());
                            window.refresh();
                        }
                        if let Some(handler) = on_change_for_drag.as_ref() {
                            (handler)(next, window, cx);
                        }
                    });
            }

            let mut container = v_stack().id(self.id.clone()).gap_1p5().items_center();
            if self.label.is_some() || self.show_value {
                let mut header = v_stack()
                    .items_center()
                    .gap_0p5()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.label));
                if let Some(label) = self.label {
                    header = header.child(label);
                }
                if self.show_value {
                    header = header.child(format!("{value:.display_precision$}"));
                }
                container = container.child(header);
            }

            return container
                .child(rail)
                .with_enter_transition(format!("{}-enter", self.id), self.motion);
        }

        let mut track = h_stack()
            .id(format!("{}-track", self.id))
            .absolute()
            .top(px(track_top))
            .left_0()
            .w(px(self.width_px))
            .h(px(track_height))
            .overflow_hidden()
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.track_bg));
        track = apply_radius(&self.theme, track, self.radius);

        let segment_span = (self.max - self.min) / segment_count as f32;
        let filled_color = self.filled_color();
        let empty_color = resolve_hsla(&self.theme, &tokens.track_bg);
        let segments = (0..segment_count).map(|index| {
            let segment_value = self.min + ((index + 1) as f32 * segment_span);
            let target = self.normalize(segment_value);
            let active = target <= value + (self.step * 0.5);
            let mut segment = div()
                .id(format!("{}-segment-{index}", self.id))
                .flex_1()
                .h(px(track_height))
                .bg(if active { filled_color } else { empty_color });

            if !self.disabled {
                let id = self.id.clone();
                let on_change = on_change.clone();
                segment = segment
                    .cursor_pointer()
                    .on_click(move |_: &ClickEvent, window, cx| {
                        if !is_controlled {
                            control::set_text_state(&id, "value", target.to_string());
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(target, window, cx);
                        }
                    });
            }

            segment.into_any_element()
        });
        track = track.children(segments);

        let mut thumb = div()
            .id(format!("{}-thumb", self.id))
            .absolute()
            .top_0()
            .left(px(thumb_left))
            .w(px(thumb_size))
            .h(px(thumb_size))
            .cursor_pointer()
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.thumb_border))
            .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
        thumb = apply_radius(&self.theme, thumb, Radius::Pill);
        if self.disabled {
            thumb = thumb.opacity(0.65);
        }

        let mut rail = div()
            .id(format!("{}-rail", self.id))
            .relative()
            .w(px(self.width_px))
            .h(px(thumb_size))
            .child(track)
            .child(thumb);

        if !self.disabled {
            let drag_state = SliderDragState {
                slider_id: self.id.clone(),
                min: self.min,
                max: self.max,
                step: self.step,
                controlled: is_controlled,
            };
            let slider_id = self.id.clone();
            let on_change_for_drag = on_change.clone();

            rail = rail
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
                    let ratio = local_x / width;
                    let raw = drag.min + ((drag.max - drag.min).max(0.001) * ratio);
                    let next = Self::normalize_with(drag.min, drag.max, drag.step, raw);

                    if !drag.controlled {
                        control::set_text_state(&slider_id, "value", next.to_string());
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
                header = header.child(format!("{value:.display_precision$}"));
            }
            container = container.child(header);
        }

        container
            .child(rail)
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Slider {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Slider {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Slider);

impl gpui::Styled for Slider {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
