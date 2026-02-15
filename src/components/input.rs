use std::{
    collections::HashMap,
    rc::Rc,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use gpui::{
    Animation, AnimationExt, AnyElement, Component, FocusHandle, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, px,
};

use crate::contracts::WithId;
use crate::contracts::{FieldLike, MotionAware, Sizeable, VariantConfigurable};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};

use super::control;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::{apply_input_size, apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;
type SubmitHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;
type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
const CARET_BLINK_TOGGLE_MS: u64 = 680;
const CARET_BLINK_CYCLE_MS: u64 = CARET_BLINK_TOGGLE_MS * 2;

#[derive(Clone, Copy)]
struct PasswordRevealState {
    until: Instant,
    value_len: usize,
    last_char: Option<char>,
}

static PASSWORD_REVEAL_STATE: LazyLock<Mutex<HashMap<String, PasswordRevealState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct TextInput {
    id: String,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: SharedString,
    placeholder: Option<SharedString>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    left_slot: Option<SlotRenderer>,
    right_slot: Option<SlotRenderer>,
    disabled: bool,
    read_only: bool,
    masked: bool,
    mask_reveal_ms: u64,
    max_length: Option<usize>,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    focus_handle: Option<FocusHandle>,
    on_change: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler>,
}

impl TextInput {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("text-input"),
            value: None,
            value_controlled: false,
            default_value: SharedString::default(),
            placeholder: None,
            label: None,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            left_slot: None,
            right_slot: None,
            disabled: false,
            read_only: false,
            masked: false,
            mask_reveal_ms: 0,
            max_length: None,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Sm,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            focus_handle: None,
            on_change: None,
            on_submit: None,
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn mask_reveal_ms(mut self, duration_ms: u64) -> Self {
        self.mask_reveal_ms = duration_ms;
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length.max(1));
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_submit(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }

    fn resolved_value(&self) -> SharedString {
        let controlled = self
            .value_controlled
            .then_some(self.value.clone().unwrap_or_default().to_string());
        control::text_state(
            &self.id,
            "value",
            controlled,
            self.default_value.to_string(),
        )
        .into()
    }

    fn display_value(&self, value: &SharedString) -> String {
        if self.masked {
            let mut chars = vec!['*'; value.as_ref().chars().count()];
            if let Some(last_char) = Self::password_reveal_char(&self.id, chars.len()) {
                if let Some(last) = chars.last_mut() {
                    *last = last_char;
                }
            }
            chars.into_iter().collect()
        } else {
            value.to_string()
        }
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

    fn set_password_reveal(id: &str, value: &str, duration_ms: u64) {
        if duration_ms == 0 {
            Self::clear_password_reveal(id);
            return;
        }

        let state = PasswordRevealState {
            until: Instant::now() + Duration::from_millis(duration_ms),
            value_len: value.chars().count(),
            last_char: value.chars().last(),
        };

        if let Ok(mut states) = PASSWORD_REVEAL_STATE.lock() {
            states.insert(id.to_string(), state);
        }
    }

    fn clear_password_reveal(id: &str) {
        if let Ok(mut states) = PASSWORD_REVEAL_STATE.lock() {
            states.remove(id);
        }
    }

    fn password_reveal_char(id: &str, current_len: usize) -> Option<char> {
        let now = Instant::now();
        let mut states = PASSWORD_REVEAL_STATE.lock().ok()?;
        let state = states.get(id).copied();

        match state {
            Some(state) if now <= state.until && state.value_len == current_len => state.last_char,
            Some(_) => {
                states.remove(id);
                None
            }
            None => None,
        }
    }

    fn with_value_update(
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
            .filter(|value| !value.is_empty())?;

        if inserted == "\n" || key == "enter" {
            return None;
        }

        let mut next = current.to_string();
        next.push_str(&inserted);

        if let Some(max_length) = max_length {
            if next.chars().count() > max_length {
                next = next.chars().take(max_length).collect();
            }
        }

        Some(next)
    }

    fn render_input_box(&mut self, window: &Window) -> AnyElement {
        let tokens = &self.theme.components.input;
        let resolved_value = self.resolved_value();
        let tracked_focus = control::bool_state(&self.id, "focused", None, false);
        let handle_focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|focus_handle| focus_handle.is_focused(window));
        let is_focused = handle_focused || tracked_focus;

        let mut input = div()
            .id(format!("{}-box", self.id))
            .focusable()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .w_full()
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .text_color(resolve_hsla(&self.theme, &tokens.fg))
            .border_1();

        input = apply_input_size(input, self.size);
        input = apply_radius(&self.theme, input, self.radius);

        if self.error.is_some() {
            input = input.border_color(resolve_hsla(&self.theme, &tokens.border_error));
        } else {
            input = input.border_color(resolve_hsla(&self.theme, &tokens.border));
        }

        if self.disabled {
            input = input.cursor_default().opacity(0.55);
        } else {
            input = input.cursor_text();
        }

        if let Some(focus_handle) = &self.focus_handle {
            let handle_for_click = focus_handle.clone();
            let id_for_focus = self.id.clone();
            input = input
                .track_focus(focus_handle)
                .on_click(move |_, window, cx| {
                    control::set_bool_state(&id_for_focus, "focused", true);
                    window.focus(&handle_for_click, cx);
                    window.refresh();
                });
        } else {
            let id_for_focus = self.id.clone();
            input = input.on_click(move |_, window, _cx| {
                control::set_bool_state(&id_for_focus, "focused", true);
                window.refresh();
            });
        }

        let id_for_blur = self.id.clone();
        input = input.on_mouse_down_out(move |_, window, _cx| {
            control::set_bool_state(&id_for_blur, "focused", false);
            window.refresh();
        });

        if !self.disabled && !self.read_only {
            let on_change = self.on_change.clone();
            let on_submit = self.on_submit.clone();
            let current_value = resolved_value.to_string();
            let max_length = self.max_length;
            let input_id = self.id.clone();
            let focus_state_id = self.id.clone();
            let masked = self.masked;
            let mask_reveal_ms = self.mask_reveal_ms;
            let value_controlled = self.value_controlled;

            input = input.on_key_down(move |event, window, cx| {
                control::set_bool_state(&focus_state_id, "focused", true);
                if event.keystroke.key == "enter" {
                    if let Some(handler) = &on_submit {
                        (handler)(current_value.clone().into(), window, cx);
                    }
                    return;
                }

                if let Some(next) = Self::with_value_update(&current_value, event, max_length) {
                    if masked {
                        let previous_len = current_value.chars().count();
                        let next_len = next.chars().count();
                        if next_len > previous_len {
                            Self::set_password_reveal(&input_id, &next, mask_reveal_ms);
                            if mask_reveal_ms > 0 {
                                let window_handle = window.window_handle();
                                cx.spawn({
                                    async move |cx| {
                                        cx.background_executor()
                                            .timer(Duration::from_millis(mask_reveal_ms))
                                            .await;
                                        let _ = window_handle.update(cx, |_, window, _| {
                                            window.refresh();
                                        });
                                    }
                                })
                                .detach();
                            }
                        } else {
                            Self::clear_password_reveal(&input_id);
                        }
                    }

                    if !value_controlled {
                        control::set_text_state(&input_id, "value", next.clone());
                        window.refresh();
                    }

                    if let Some(handler) = &on_change {
                        (handler)(next.into(), window, cx);
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

        let value = self.display_value(&resolved_value);
        let mut value_container = div().flex_1().min_w_0().flex().items_center().gap_1();
        if value.is_empty() && !is_focused {
            value_container = value_container.child(
                div()
                    .truncate()
                    .text_color(resolve_hsla(&self.theme, &tokens.placeholder))
                    .child(self.placeholder.clone().unwrap_or_default()),
            );
        } else {
            value_container = value_container.child(div().truncate().child(value));
        }

        let show_caret = is_focused;
        if !self.disabled && !self.read_only && show_caret {
            let caret_color = resolve_hsla(&self.theme, &tokens.fg);
            value_container = value_container.child(
                div()
                    .id(format!("{}-caret", self.id))
                    .flex_none()
                    .w(px(1.5))
                    .h(px(self.caret_height_px()))
                    .bg(caret_color)
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
                    .ml_auto()
                    .flex_none()
                    .text_color(resolve_hsla(&self.theme, &self.theme.semantic.text_muted))
                    .child(right_slot()),
            );
        }

        input
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
            .into_any_element()
    }

    fn render_label_block(&self) -> AnyElement {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return div().into_any_element();
        }

        let mut block = v_stack().gap_1();

        if let Some(label) = &self.label {
            let mut label_row = h_stack().gap_1().child(
                div()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(resolve_hsla(
                        &self.theme,
                        &self.theme.components.input.label,
                    ))
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
                    .text_color(resolve_hsla(
                        &self.theme,
                        &self.theme.components.input.description,
                    ))
                    .child(description.clone()),
            );
        }

        if let Some(error) = &self.error {
            block = block.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(
                        &self.theme,
                        &self.theme.components.input.error,
                    ))
                    .child(error.clone()),
            );
        }

        block.into_any_element()
    }
}

impl WithId for TextInput {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl FieldLike for TextInput {
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

impl VariantConfigurable for TextInput {
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

impl MotionAware for TextInput {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for TextInput {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        match self.layout {
            FieldLayout::Vertical => v_stack()
                .gap_2()
                .child(self.render_label_block())
                .child(self.render_input_box(window))
                .into_any_element(),
            FieldLayout::Horizontal => h_stack()
                .items_start()
                .gap_3()
                .child(div().w(gpui::px(168.0)).child(self.render_label_block()))
                .child(div().flex_1().child(self.render_input_box(window)))
                .into_any_element(),
        }
    }
}

impl IntoElement for TextInput {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

pub struct PasswordInput {
    inner: TextInput,
    style: gpui::StyleRefinement,
}

impl PasswordInput {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            inner: TextInput::new().masked(true).mask_reveal_ms(700),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.value(value);
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.default_value(value);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.inner = self.inner.placeholder(placeholder);
        self
    }

    pub fn left_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.inner = self.inner.left_slot(content);
        self
    }

    pub fn right_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.inner = self.inner.right_slot(content);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    pub fn reveal(mut self, reveal: bool) -> Self {
        self.inner = self.inner.masked(!reveal);
        self
    }

    pub fn reveal_duration_ms(mut self, duration_ms: u64) -> Self {
        self.inner = self.inner.mask_reveal_ms(duration_ms);
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.inner = self.inner.focus_handle(focus_handle);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }

    pub fn on_submit(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.inner = self.inner.on_submit(handler);
        self
    }
}

impl WithId for PasswordInput {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn id_mut(&mut self) -> &mut String {
        self.inner.id_mut()
    }
}

impl FieldLike for PasswordInput {
    fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.label(value);
        self
    }

    fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.description(value);
        self
    }

    fn error(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.error(value);
        self
    }

    fn required(mut self, value: bool) -> Self {
        self.inner = self.inner.required(value);
        self
    }

    fn layout(mut self, value: FieldLayout) -> Self {
        self.inner = self.inner.layout(value);
        self
    }
}

impl VariantConfigurable for PasswordInput {
    fn variant(mut self, value: Variant) -> Self {
        self.inner = self.inner.variant(value);
        self
    }

    fn size(mut self, value: Size) -> Self {
        self.inner = Sizeable::size(self.inner, value);
        self
    }

    fn radius(mut self, value: Radius) -> Self {
        self.inner = self.inner.radius(value);
        self
    }
}

impl MotionAware for PasswordInput {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.inner = self.inner.motion(value);
        self
    }
}

impl RenderOnce for PasswordInput {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let mut inner = self.inner;
        gpui::Refineable::refine(gpui::Styled::style(&mut inner), &self.style);
        inner.render(window, cx)
    }
}

impl IntoElement for PasswordInput {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

pub struct PinInput {
    id: String,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: SharedString,
    length: usize,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    focus_handle: Option<FocusHandle>,
    on_change: Option<ChangeHandler>,
}

impl PinInput {
    #[track_caller]
    pub fn new(length: usize) -> Self {
        Self {
            id: stable_auto_id("pin-input"),
            value: None,
            value_controlled: false,
            default_value: SharedString::default(),
            length: length.max(1),
            size: Size::Md,
            radius: Radius::Sm,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            focus_handle: None,
            on_change: None,
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    fn normalize_value(value: impl Into<SharedString>, length: usize) -> SharedString {
        let mut value = value.into().to_string();
        value.retain(|ch| ch.is_ascii_digit());
        value.chars().take(length).collect::<String>().into()
    }

    fn resolved_value(&self) -> SharedString {
        let controlled = self
            .value_controlled
            .then_some(Self::normalize_value(
                self.value.clone().unwrap_or_default(),
                self.length,
            ))
            .map(|value| value.to_string());
        let default = Self::normalize_value(self.default_value.clone(), self.length).to_string();
        control::text_state(&self.id, "value", controlled, default).into()
    }
}

impl WithId for PinInput {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for PinInput {
    fn variant(self, _value: Variant) -> Self {
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

impl MotionAware for PinInput {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for PinInput {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let on_change = self.on_change.clone();
        let value = self.resolved_value().to_string();
        let id = self.id.clone();
        let value_controlled = self.value_controlled;
        let length = self.length;
        let value_chars = value.chars().collect::<Vec<_>>();
        let active_index = value_chars.len().min(self.length.saturating_sub(1));
        let is_focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|focus_handle| focus_handle.is_focused(window));
        let caret_height = match self.size {
            Size::Xs => 13.0,
            Size::Sm => 15.0,
            Size::Md => 17.0,
            Size::Lg => 19.0,
            Size::Xl => 21.0,
        };
        let caret_color = resolve_hsla(&self.theme, &self.theme.components.input.fg);

        let mut root = h_stack()
            .id(self.id.clone())
            .focusable()
            .gap_2()
            .cursor_text()
            .on_key_down(move |event, window, cx| {
                let mut next = value.clone();
                if event.keystroke.key == "backspace" {
                    next.pop();
                } else {
                    let digit = event
                        .keystroke
                        .key_char
                        .as_ref()
                        .and_then(|c| c.chars().next())
                        .filter(|ch| ch.is_ascii_digit())
                        .or_else(|| {
                            let key = event.keystroke.key.as_str();
                            if key.len() == 1 {
                                key.chars().next().filter(|ch| ch.is_ascii_digit())
                            } else {
                                None
                            }
                        });

                    if let Some(ch) = digit {
                        if next.chars().count() < length {
                            next.push(ch);
                        }
                    }
                }

                if !value_controlled {
                    control::set_text_state(&id, "value", next.clone());
                    window.refresh();
                }

                if let Some(handler) = &on_change {
                    (handler)(next.into(), window, cx);
                }
            });

        if let Some(focus_handle) = &self.focus_handle {
            let handle_for_click = focus_handle.clone();
            root = root
                .track_focus(focus_handle)
                .on_click(move |_, window, cx| {
                    window.focus(&handle_for_click, cx);
                });
        }

        for index in 0..self.length {
            let content = value_chars.get(index).map(|ch| ch.to_string());
            let mut cell = div()
                .w(gpui::px(34.0))
                .h(gpui::px(40.0))
                .border_1()
                .border_color(resolve_hsla(
                    &self.theme,
                    &self.theme.components.input.border,
                ))
                .bg(resolve_hsla(&self.theme, &self.theme.components.input.bg))
                .flex()
                .items_center()
                .justify_center();

            if let Some(content) = content {
                cell = cell.child(content);
            } else if index == active_index
                && value_chars.len() < self.length
                && (self.focus_handle.is_none() || is_focused)
            {
                cell = cell.child(
                    div()
                        .id(format!("{}-caret-{index}", self.id))
                        .w(px(1.5))
                        .h(px(caret_height))
                        .bg(caret_color)
                        .rounded_sm()
                        .with_animation(
                            format!("{}-caret-blink-{index}", self.id),
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
            cell = apply_radius(&self.theme, cell, self.radius);
            root = root.child(cell);
        }

        root.with_enter_transition(format!("{}-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl IntoElement for PinInput {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for TextInput {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for PasswordInput {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.inner.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for PinInput {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(TextInput);
crate::impl_disableable!(PasswordInput);

impl gpui::Styled for PinInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for TextInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for PasswordInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
