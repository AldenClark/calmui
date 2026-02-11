use std::{rc::Rc, str::FromStr, time::Duration};

use gpui::{
    Animation, AnimationExt, AnyElement, ClickEvent, Component, FocusHandle, InteractiveElement,
    IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, px,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::contracts::{FieldLike, MotionAware, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};

use super::control;
use super::icon::Icon;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::{apply_input_size, apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn(f64, &mut Window, &mut gpui::App)>;
type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

const CARET_BLINK_TOGGLE_MS: u64 = 680;
const CARET_BLINK_CYCLE_MS: u64 = CARET_BLINK_TOGGLE_MS * 2;

pub struct NumberInput {
    id: String,
    value: Option<f64>,
    value_controlled: bool,
    default_value: f64,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    precision: Option<usize>,
    placeholder: Option<SharedString>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    left_slot: Option<SlotRenderer>,
    right_slot: Option<SlotRenderer>,
    controls: bool,
    disabled: bool,
    read_only: bool,
    max_length: Option<usize>,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    focus_handle: Option<FocusHandle>,
    on_change: Option<ChangeHandler>,
}

impl NumberInput {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("number-input"),
            value: None,
            value_controlled: false,
            default_value: 0.0,
            min: None,
            max: None,
            step: 1.0,
            precision: None,
            placeholder: None,
            label: None,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            left_slot: None,
            right_slot: None,
            controls: true,
            disabled: false,
            read_only: false,
            max_length: None,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Sm,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            focus_handle: None,
            on_change: None,
        }
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: f64) -> Self {
        self.default_value = value;
        self
    }

    pub fn min(mut self, value: f64) -> Self {
        self.min = Some(value);
        if let Some(max) = self.max {
            if max < value {
                self.max = Some(value);
            }
        }
        self
    }

    pub fn max(mut self, value: f64) -> Self {
        self.max = Some(value);
        if let Some(min) = self.min {
            if min > value {
                self.min = Some(value);
            }
        }
        self
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        if min <= max {
            self.min = Some(min);
            self.max = Some(max);
        } else {
            self.min = Some(max);
            self.max = Some(min);
        }
        self
    }

    pub fn step(mut self, value: f64) -> Self {
        self.step = value.abs().max(0.000_001);
        self
    }

    pub fn precision(mut self, value: usize) -> Self {
        self.precision = Some(value.min(8));
        self
    }

    pub fn placeholder(mut self, value: impl Into<SharedString>) -> Self {
        self.placeholder = Some(value.into());
        self
    }

    pub fn left_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.left_slot = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn right_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.right_slot = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn controls(mut self, value: bool) -> Self {
        self.controls = value;
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

    pub fn max_length(mut self, value: usize) -> Self {
        self.max_length = Some(value.max(1));
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(f64, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn decimal_from_f64(value: f64) -> Decimal {
        if !value.is_finite() {
            return Decimal::ZERO;
        }
        Decimal::from_str(&format!("{value:.18}")).unwrap_or(Decimal::ZERO)
    }

    fn clamp_decimal(&self, value: Decimal) -> Decimal {
        let mut next = value;
        if let Some(min) = self.min {
            next = next.max(Self::decimal_from_f64(min));
        }
        if let Some(max) = self.max {
            next = next.min(Self::decimal_from_f64(max));
        }
        next
    }

    fn decimals_from_step(step: Decimal) -> usize {
        let text = step.normalize().to_string();
        text.split('.').nth(1).map(|part| part.len()).unwrap_or(0)
    }

    fn format_decimal_value(value: Decimal, precision: Option<usize>, step: Decimal) -> String {
        let precision = precision.unwrap_or_else(|| Self::decimals_from_step(step));
        let rounded = value.round_dp(precision as u32).normalize();
        let text = rounded.to_string();
        if text == "-0" { "0".to_string() } else { text }
    }

    fn is_incomplete_number(text: &str) -> bool {
        let trimmed = text.trim();
        trimmed.is_empty() || trimmed == "-" || trimmed == "." || trimmed == "-."
    }

    fn parse_number(text: &str) -> Option<Decimal> {
        if Self::is_incomplete_number(text) {
            return None;
        }
        Decimal::from_str(text.trim()).ok()
    }

    fn resolved_text(&self) -> String {
        let step = Self::decimal_from_f64(self.step.abs().max(0.000_001));
        let controlled = self
            .value_controlled
            .then_some(self.value.unwrap_or(self.default_value))
            .map(Self::decimal_from_f64)
            .map(|value| self.clamp_decimal(value))
            .map(|value| Self::format_decimal_value(value, self.precision, step));

        let default = Self::format_decimal_value(
            self.clamp_decimal(Self::decimal_from_f64(self.default_value)),
            self.precision,
            step,
        );

        control::text_state(&self.id, "value-text", controlled, default)
    }

    fn with_text_update(
        current: &str,
        event: &KeyDownEvent,
        max_length: Option<usize>,
    ) -> Option<String> {
        let key = event.keystroke.key.as_str();

        if key == "backspace" {
            let mut next = current.to_string();
            next.pop();
            return Some(next);
        }

        if key == "enter" {
            return None;
        }

        let has_modifier = event.keystroke.modifiers.control
            || event.keystroke.modifiers.platform
            || event.keystroke.modifiers.function;
        if has_modifier {
            return None;
        }

        let inserted = event
            .keystroke
            .key_char
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| {
                if key.chars().count() == 1 {
                    Some(key.to_string())
                } else {
                    None
                }
            })?;

        let mut next = current.to_string();
        for ch in inserted.chars() {
            if ch.is_ascii_digit() {
                next.push(ch);
            } else if ch == '-' {
                if next.is_empty() {
                    next.push(ch);
                }
            } else if ch == '.' {
                if !next.contains('.') {
                    if next.is_empty() || next == "-" {
                        next.push('0');
                    }
                    next.push('.');
                }
            }
        }

        if let Some(max_length) = max_length {
            if next.chars().count() > max_length {
                next = next.chars().take(max_length).collect();
            }
        }

        Some(next)
    }

    fn stepped_value_text_for(
        current_text: &str,
        direction: f64,
        step: f64,
        min: Option<f64>,
        max: Option<f64>,
        precision: Option<usize>,
        default_value: f64,
    ) -> (String, f64) {
        let min_decimal = min.map(Self::decimal_from_f64);
        let max_decimal = max.map(Self::decimal_from_f64);
        let clamp = |mut value: Decimal| {
            if let Some(min) = min_decimal {
                value = value.max(min);
            }
            if let Some(max) = max_decimal {
                value = value.min(max);
            }
            value
        };

        let current = Self::parse_number(current_text)
            .map(clamp)
            .unwrap_or_else(|| clamp(Self::decimal_from_f64(default_value)));

        let step_decimal = Self::decimal_from_f64(step.abs().max(0.000_001));
        let base = min_decimal.unwrap_or(Decimal::ZERO);
        let delta = if direction < 0.0 {
            -step_decimal
        } else {
            step_decimal
        };
        let raw_next = current + delta;
        let stepped = (((raw_next - base) / step_decimal).round() * step_decimal) + base;
        let clamped = clamp(stepped);
        let formatted = Self::format_decimal_value(clamped, precision, step_decimal);
        let as_f64 = clamped.to_f64().unwrap_or(default_value);
        (formatted, as_f64)
    }

    fn caret_height_px(&self) -> f32 {
        match self.size {
            Size::Xs => 13.0,
            Size::Sm => 15.0,
            Size::Md => 17.0,
            Size::Lg => 19.0,
            Size::Xl => 21.0,
        }
    }

    fn render_label_block(&self) -> AnyElement {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return div().into_any_element();
        }

        let tokens = &self.theme.components.number_input;
        let mut block = v_stack().gap_1();

        if let Some(label) = &self.label {
            let mut label_row = h_stack().gap_1().child(
                div()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(resolve_hsla(&self.theme, &tokens.label))
                    .child(label.clone()),
            );

            if self.required {
                label_row = label_row.child(
                    div()
                        .text_color(resolve_hsla(&self.theme, &self.theme.semantic.status_error))
                        .child("*"),
                );
            }

            block = block.child(label_row);
        }

        if let Some(description) = &self.description {
            block = block.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.description))
                    .child(description.clone()),
            );
        }

        if let Some(error) = &self.error {
            block = block.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.error))
                    .child(error.clone()),
            );
        }

        block.into_any_element()
    }

    fn render_input_box(&mut self, window: &Window) -> AnyElement {
        let tokens = &self.theme.components.number_input;
        let current_text = self.resolved_text();
        let is_focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|focus_handle| focus_handle.is_focused(window));

        let mut input = div()
            .id(format!("{}-box", self.id))
            .focusable()
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .text_color(resolve_hsla(&self.theme, &tokens.fg))
            .border_1();

        input = apply_input_size(input, self.size);
        input = apply_radius(input, self.radius);

        let border = if self.error.is_some() {
            resolve_hsla(&self.theme, &tokens.border_error)
        } else if is_focused {
            resolve_hsla(&self.theme, &tokens.border_focus)
        } else {
            resolve_hsla(&self.theme, &tokens.border)
        };
        input = input.border_color(border);

        if self.disabled {
            input = input.cursor_default().opacity(0.55);
        } else {
            input = input.cursor_text();
        }

        if let Some(focus_handle) = &self.focus_handle {
            let handle_for_click = focus_handle.clone();
            input = input
                .track_focus(focus_handle)
                .on_click(move |_, window, cx| {
                    window.focus(&handle_for_click, cx);
                });
        }

        if !self.disabled && !self.read_only {
            let on_change = self.on_change.clone();
            let value_controlled = self.value_controlled;
            let input_id = self.id.clone();
            let max_length = self.max_length;
            let min = self.min;
            let max = self.max;
            let current_text_for_input = current_text.clone();
            input = input.on_key_down(move |event, window, cx| {
                if let Some(next) =
                    Self::with_text_update(&current_text_for_input, event, max_length)
                {
                    if !value_controlled {
                        control::set_text_state(&input_id, "value-text", next.clone());
                        window.refresh();
                    }

                    if let Some(parsed) = Self::parse_number(&next) {
                        let mut clamped = parsed;
                        if let Some(min) = min {
                            clamped = clamped.max(Self::decimal_from_f64(min));
                        }
                        if let Some(max) = max {
                            clamped = clamped.min(Self::decimal_from_f64(max));
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(clamped.to_f64().unwrap_or(0.0), window, cx);
                        }
                    }
                }
            });
        }

        if let Some(left_slot) = self.left_slot.take() {
            input = input.child(
                div()
                    .flex_none()
                    .text_color(resolve_hsla(&self.theme, &self.theme.semantic.text_muted))
                    .child(left_slot()),
            );
        }

        let mut value_container = div().flex_1().min_w_0().flex().items_center().gap_1();
        if current_text.is_empty() && !is_focused {
            value_container = value_container.child(
                div()
                    .truncate()
                    .text_color(resolve_hsla(&self.theme, &tokens.placeholder))
                    .child(self.placeholder.clone().unwrap_or_default()),
            );
        } else {
            value_container = value_container.child(div().truncate().child(current_text.clone()));
        }

        let show_caret = self.focus_handle.is_none() || is_focused;
        if !self.disabled && !self.read_only && show_caret {
            value_container = value_container.child(
                div()
                    .id(format!("{}-caret", self.id))
                    .flex_none()
                    .w(px(1.5))
                    .h(px(self.caret_height_px()))
                    .bg(resolve_hsla(&self.theme, &tokens.fg))
                    .rounded_sm()
                    .with_animation(
                        format!("{}-caret-blink", self.id),
                        Animation::new(Duration::from_millis(CARET_BLINK_CYCLE_MS))
                            .repeat()
                            .with_easing(gpui::linear),
                        |this, delta| {
                            let visible = ((delta * 2.0).fract()) < 0.5;
                            this.opacity(if visible { 1.0 } else { 0.0 })
                        },
                    ),
            );
        }
        input = input.child(value_container);

        if let Some(right_slot) = self.right_slot.take() {
            input = input.child(
                div()
                    .flex_none()
                    .text_color(resolve_hsla(&self.theme, &self.theme.semantic.text_muted))
                    .child(right_slot()),
            );
        }

        if self.controls {
            let controls_bg = resolve_hsla(&self.theme, &tokens.controls_bg);
            let controls_fg = resolve_hsla(&self.theme, &tokens.controls_fg);
            let controls_border = resolve_hsla(&self.theme, &tokens.controls_border);

            let mut up = div()
                .id(format!("{}-control-up", self.id))
                .w(px(18.0))
                .h(px(12.0))
                .flex()
                .items_center()
                .justify_center()
                .bg(controls_bg)
                .text_color(controls_fg)
                .border_1()
                .border_color(controls_border)
                .child(
                    Icon::named_outline("chevron-up")
                        .with_id(format!("{}-chevron-up", self.id))
                        .size(12.0)
                        .color(controls_fg),
                );

            let mut down = div()
                .id(format!("{}-control-down", self.id))
                .w(px(18.0))
                .h(px(12.0))
                .flex()
                .items_center()
                .justify_center()
                .bg(controls_bg)
                .text_color(controls_fg)
                .border_1()
                .border_color(controls_border)
                .child(
                    Icon::named_outline("chevron-down")
                        .with_id(format!("{}-chevron-down", self.id))
                        .size(12.0)
                        .color(controls_fg),
                );

            if !self.disabled && !self.read_only {
                let id = self.id.clone();
                let current = current_text.clone();
                let value_controlled = self.value_controlled;
                let on_change = self.on_change.clone();
                let step = self.step;
                let min = self.min;
                let max = self.max;
                let precision = self.precision;
                let default_value = self.default_value;
                up = up
                    .cursor_pointer()
                    .on_click(move |_: &ClickEvent, window, cx| {
                        let (next_text, next_value) = Self::stepped_value_text_for(
                            &current,
                            1.0,
                            step,
                            min,
                            max,
                            precision,
                            default_value,
                        );
                        if !value_controlled {
                            control::set_text_state(&id, "value-text", next_text);
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(next_value, window, cx);
                        }
                    });

                let id = self.id.clone();
                let current = current_text;
                let value_controlled = self.value_controlled;
                let on_change = self.on_change.clone();
                let step = self.step;
                let min = self.min;
                let max = self.max;
                let precision = self.precision;
                let default_value = self.default_value;
                down = down
                    .cursor_pointer()
                    .on_click(move |_: &ClickEvent, window, cx| {
                        let (next_text, next_value) = Self::stepped_value_text_for(
                            &current,
                            -1.0,
                            step,
                            min,
                            max,
                            precision,
                            default_value,
                        );
                        if !value_controlled {
                            control::set_text_state(&id, "value-text", next_text);
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(next_value, window, cx);
                        }
                    });
            } else {
                up = up.opacity(0.55);
                down = down.opacity(0.55);
            }

            let controls = v_stack().gap_0().child(up).child(down);
            input = input.child(controls);
        }

        input
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl WithId for NumberInput {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl FieldLike for NumberInput {
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

impl VariantSupport for NumberInput {
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

impl MotionAware for NumberInput {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for NumberInput {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        match self.layout {
            FieldLayout::Vertical => v_stack()
                .gap_2()
                .child(self.render_label_block())
                .child(self.render_input_box(window)),
            FieldLayout::Horizontal => h_stack()
                .items_start()
                .gap_3()
                .child(div().w(px(168.0)).child(self.render_label_block()))
                .child(div().flex_1().child(self.render_input_box(window))),
        }
    }
}

impl IntoElement for NumberInput {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for NumberInput {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
