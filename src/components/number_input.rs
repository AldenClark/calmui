use std::{rc::Rc, str::FromStr};

use gpui::{
    AnyElement, ClickEvent, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::contracts::{FieldLike, MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};

use super::TextInput;
use super::control;
use super::icon::Icon;
use super::utils::{apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn(f64, &mut Window, &mut gpui::App)>;
type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

#[derive(IntoElement)]
pub struct NumberInput {
    id: ComponentId,
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
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    focus_handle: Option<FocusHandle>,
    on_change: Option<ChangeHandler>,
}

impl NumberInput {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
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
            style: gpui::StyleRefinement::default(),
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
        if let Some(max) = self.max
            && max < value
        {
            self.max = Some(value);
        }
        self
    }

    pub fn max(mut self, value: f64) -> Self {
        self.max = Some(value);
        if let Some(min) = self.min
            && min > value
        {
            self.min = Some(value);
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

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
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

    pub fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    pub fn radius(mut self, value: Radius) -> Self {
        self.radius = value;
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

    fn sanitize_numeric_text(raw: &str, max_length: Option<usize>) -> String {
        let mut next = String::new();
        let mut has_dot = false;

        for ch in raw.chars() {
            if ch.is_ascii_digit() {
                next.push(ch);
                continue;
            }

            if ch == '-' {
                if next.is_empty() {
                    next.push('-');
                }
                continue;
            }

            if ch == '.' && !has_dot {
                if next.is_empty() || next == "-" {
                    next.push('0');
                }
                next.push('.');
                has_dot = true;
            }
        }

        if let Some(limit) = max_length
            && next.chars().count() > limit
        {
            next = next.chars().take(limit).collect();
        }

        next
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

    fn current_text_for(id: &str, fallback: &str, value_controlled: bool) -> String {
        control::text_state(
            id,
            "value-text",
            value_controlled.then_some(fallback.to_string()),
            fallback.to_string(),
        )
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

    fn compose_right_slot(
        user_right_slot: Option<AnyElement>,
        controls_slot: Option<AnyElement>,
    ) -> Option<AnyElement> {
        match (user_right_slot, controls_slot) {
            (Some(user), Some(controls)) => Some(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(user)
                    .child(controls)
                    .into_any_element(),
            ),
            (Some(user), None) => Some(user),
            (None, Some(controls)) => Some(controls),
            (None, None) => None,
        }
    }

    fn render_controls_slot(&self, fallback_text: String) -> AnyElement {
        let tokens = &self.theme.components.number_input;
        let controls_bg = resolve_hsla(&self.theme, &tokens.controls_bg);
        let controls_fg = resolve_hsla(&self.theme, &tokens.controls_fg);
        let controls_border = resolve_hsla(&self.theme, &tokens.controls_border);

        let mut up = div()
            .id(self.id.slot("control-up"))
            .w(px(18.0))
            .h(px(12.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(controls_bg)
            .text_color(controls_fg)
            .border(px(1.0))
            .border_color(controls_border)
            .child(
                Icon::named("chevron-up")
                    .with_id(self.id.slot("chevron-up"))
                    .size(12.0)
                    .color(controls_fg),
            );

        let mut down = div()
            .id(self.id.slot("control-down"))
            .w(px(18.0))
            .h(px(12.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(controls_bg)
            .text_color(controls_fg)
            .border(px(1.0))
            .border_color(controls_border)
            .child(
                Icon::named("chevron-down")
                    .with_id(self.id.slot("chevron-down"))
                    .size(12.0)
                    .color(controls_fg),
            );

        if !self.disabled && !self.read_only {
            let id = self.id.clone();
            let fallback = fallback_text.clone();
            let value_controlled = self.value_controlled;
            let on_change = self.on_change.clone();
            let step = self.step;
            let min = self.min;
            let max = self.max;
            let precision = self.precision;
            let default_value = self.default_value;
            let focus_handle = self.focus_handle.clone();
            up = up
                .cursor_pointer()
                .on_click(move |_: &ClickEvent, window, cx| {
                    let current = Self::current_text_for(&id, &fallback, value_controlled);
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
                    }
                    if let Some(handler) = on_change.as_ref() {
                        (handler)(next_value, window, cx);
                    }
                    if let Some(handle) = focus_handle.as_ref() {
                        window.focus(handle, cx);
                    }
                    window.refresh();
                });

            let id = self.id.clone();
            let fallback = fallback_text;
            let value_controlled = self.value_controlled;
            let on_change = self.on_change.clone();
            let step = self.step;
            let min = self.min;
            let max = self.max;
            let precision = self.precision;
            let default_value = self.default_value;
            let focus_handle = self.focus_handle.clone();
            down = down
                .cursor_pointer()
                .on_click(move |_: &ClickEvent, window, cx| {
                    let current = Self::current_text_for(&id, &fallback, value_controlled);
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
                    }
                    if let Some(handler) = on_change.as_ref() {
                        (handler)(next_value, window, cx);
                    }
                    if let Some(handle) = focus_handle.as_ref() {
                        window.focus(handle, cx);
                    }
                    window.refresh();
                });
        } else {
            up = up.opacity(0.55);
            down = down.opacity(0.55);
        }

        let controls = super::Stack::vertical().gap_0().child(up).child(down);
        apply_radius(&self.theme, controls, self.radius).into_any_element()
    }
}

impl NumberInput {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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

impl VariantConfigurable for NumberInput {
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
    fn render(mut self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(cx);

        let current_text = self.resolved_text();
        let id = self.id.clone();
        let value_controlled = self.value_controlled;
        let on_change = self.on_change.clone();
        let min = self.min;
        let max = self.max;
        let max_length = self.max_length;

        let mut input = TextInput::new()
            .with_id(self.id.clone())
            .value(current_text.clone());

        if let Some(placeholder) = self.placeholder.clone() {
            input = input.placeholder(placeholder);
        }
        if let Some(label) = self.label.clone() {
            input = input.label(label);
        }
        if let Some(description) = self.description.clone() {
            input = input.description(description);
        }
        if let Some(error) = self.error.clone() {
            input = input.error(error);
        }

        input = input
            .required(self.required)
            .layout(self.layout)
            .disabled(self.disabled)
            .read_only(self.read_only);

        input = VariantConfigurable::variant(input, self.variant);
        input = VariantConfigurable::size(input, self.size);
        input = VariantConfigurable::radius(input, self.radius);
        input = MotionAware::motion(input, self.motion).on_change(
            move |next_text: SharedString, window, cx| {
                let sanitized = Self::sanitize_numeric_text(next_text.as_ref(), max_length);
                if !value_controlled {
                    control::set_text_state(&id, "value-text", sanitized.clone());
                }

                if let Some(parsed) = Self::parse_number(&sanitized) {
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

                window.refresh();
            },
        );

        if let Some(max_length) = self.max_length {
            input = input.max_length(max_length);
        }
        if let Some(focus_handle) = self.focus_handle.clone() {
            input = input.focus_handle(focus_handle);
        }

        if let Some(left_slot) = self.left_slot.take() {
            input = input.left_slot(left_slot());
        }

        let user_right_slot = self.right_slot.take().map(|slot| slot());
        let controls_slot = self
            .controls
            .then(|| self.render_controls_slot(current_text.clone()));
        if let Some(right_slot) = Self::compose_right_slot(user_right_slot, controls_slot) {
            input = input.right_slot(right_slot);
        }

        gpui::Refineable::refine(gpui::Styled::style(&mut input), &self.style);

        let field = input.render(window, cx).into_any_element();

        if self.disabled || self.read_only {
            return field;
        }

        let id_for_step = self.id.clone();
        let fallback_for_step = current_text;
        let value_controlled_for_step = self.value_controlled;
        let on_change_for_step = self.on_change.clone();
        let step = self.step;
        let min = self.min;
        let max = self.max;
        let precision = self.precision;
        let default_value = self.default_value;

        div()
            .id(self.id.slot("keyboard-proxy"))
            .on_key_down(move |event, window, cx| {
                if !control::focused_state(&id_for_step, None, false) {
                    return;
                }

                let key = event.keystroke.key.as_str();
                if key != "up" && key != "down" {
                    return;
                }

                if event.keystroke.modifiers.control
                    || event.keystroke.modifiers.platform
                    || event.keystroke.modifiers.function
                    || event.keystroke.modifiers.alt
                {
                    return;
                }

                let current = Self::current_text_for(
                    &id_for_step,
                    &fallback_for_step,
                    value_controlled_for_step,
                );
                let direction = if key == "up" { 1.0 } else { -1.0 };
                let (next_text, next_value) = Self::stepped_value_text_for(
                    &current,
                    direction,
                    step,
                    min,
                    max,
                    precision,
                    default_value,
                );

                if !value_controlled_for_step {
                    control::set_text_state(&id_for_step, "value-text", next_text);
                }
                if let Some(handler) = on_change_for_step.as_ref() {
                    (handler)(next_value, window, cx);
                }

                cx.stop_propagation();
                window.prevent_default();
                window.refresh();
            })
            .child(field)
            .into_any_element()
    }
}

impl crate::contracts::ComponentThemeOverridable for NumberInput {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(NumberInput);

impl gpui::Styled for NumberInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
