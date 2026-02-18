use std::{
    collections::HashMap,
    rc::Rc,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use gpui::{
    Animation, AnimationExt, AnyElement, ClickEvent, ClipboardItem, FocusHandle,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, canvas, div, px,
};

use crate::contracts::{FieldLike, MotionAware, Sizeable, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::transition::TransitionExt;
use super::utils::{apply_input_size, apply_radius, quantized_stroke_px, resolve_hsla};

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

#[derive(IntoElement)]
pub struct TextInput {
    id: ComponentId,
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
            id: ComponentId::default(),
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

    fn char_width_px(&self) -> f32 {
        match self.size {
            Size::Xs => 6.8,
            Size::Sm => 7.2,
            Size::Md => 7.8,
            Size::Lg => 8.6,
            Size::Xl => 9.4,
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

    fn byte_index_at_char(value: &str, char_index: usize) -> usize {
        value
            .char_indices()
            .nth(char_index)
            .map(|(index, _)| index)
            .unwrap_or(value.len())
    }

    fn selection_bounds_for(id: &str, len: usize) -> Option<(usize, usize)> {
        let start = control::text_state(id, "selection-start", None, "0".to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(0)
            .min(len);
        let end = control::text_state(id, "selection-end", None, "0".to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(0)
            .min(len);
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        (start < end).then_some((start, end))
    }

    fn set_selection_for(id: &str, start: usize, end: usize) {
        control::set_text_state(id, "selection-start", start.to_string());
        control::set_text_state(id, "selection-end", end.to_string());
    }

    fn clear_selection_for(id: &str, caret: usize) {
        Self::set_selection_for(id, caret, caret);
    }

    fn replace_char_range(value: &str, start: usize, end: usize, insert: &str) -> (String, usize) {
        let start = start.min(value.chars().count());
        let end = end.min(value.chars().count()).max(start);
        let byte_start = Self::byte_index_at_char(value, start);
        let byte_end = Self::byte_index_at_char(value, end);
        let mut next = value.to_string();
        next.replace_range(byte_start..byte_end, insert);
        (next, start + insert.chars().count())
    }

    fn content_geometry(id: &str) -> (f32, f32, f32, f32) {
        let x = control::text_state(id, "content-origin-x", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let y = control::text_state(id, "content-origin-y", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let width = control::text_state(id, "content-width", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let height = control::text_state(id, "content-height", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        (x, y, width, height)
    }

    fn caret_from_click(
        id: &str,
        position: gpui::Point<gpui::Pixels>,
        value: &str,
        char_width: f32,
    ) -> usize {
        let (origin_x, _origin_y, _width, _height) = Self::content_geometry(id);
        let local_x = (f32::from(position.x) - origin_x).max(0.0);
        let target_col = ((local_x / char_width.max(1.0)) + 0.5).floor() as usize;
        target_col.min(value.chars().count())
    }

    fn with_value_update(
        current: &str,
        event: &KeyDownEvent,
        max_length: Option<usize>,
        caret_index: usize,
        selection: Option<(usize, usize)>,
    ) -> Option<(String, usize)> {
        let key = event.keystroke.key.as_str();
        let char_len = current.chars().count();
        let caret_index = caret_index.min(char_len);
        let selection = selection.map(|(start, end)| {
            let start = start.min(char_len);
            let end = end.min(char_len);
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        });
        let has_selection = selection.is_some_and(|(start, end)| start < end);

        if key == "backspace" {
            if let Some((start, end)) = selection
                && start < end
            {
                return Some(Self::replace_char_range(current, start, end, ""));
            }
            if caret_index == 0 {
                return Some((current.to_string(), 0));
            }
            let start = Self::byte_index_at_char(current, caret_index - 1);
            let end = Self::byte_index_at_char(current, caret_index);
            let mut next = current.to_string();
            next.replace_range(start..end, "");
            return Some((next, caret_index - 1));
        }

        if key == "delete" {
            if let Some((start, end)) = selection
                && start < end
            {
                return Some(Self::replace_char_range(current, start, end, ""));
            }
            if caret_index >= char_len {
                return Some((current.to_string(), caret_index));
            }
            let start = Self::byte_index_at_char(current, caret_index);
            let end = Self::byte_index_at_char(current, caret_index + 1);
            let mut next = current.to_string();
            next.replace_range(start..end, "");
            return Some((next, caret_index));
        }

        if key == "left" {
            if has_selection {
                let (start, _) = selection.unwrap_or((caret_index, caret_index));
                return Some((current.to_string(), start));
            }
            return Some((current.to_string(), caret_index.saturating_sub(1)));
        }
        if key == "right" {
            if has_selection {
                let (_, end) = selection.unwrap_or((caret_index, caret_index));
                return Some((current.to_string(), end));
            }
            return Some((current.to_string(), (caret_index + 1).min(char_len)));
        }
        if key == "home" {
            return Some((current.to_string(), 0));
        }
        if key == "end" {
            return Some((current.to_string(), char_len));
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

        if inserted == "\n" || key == "enter" {
            return None;
        }

        let start = Self::byte_index_at_char(current, caret_index);
        let (mut next, mut next_caret) = if let Some((selection_start, selection_end)) = selection {
            if selection_start < selection_end {
                Self::replace_char_range(current, selection_start, selection_end, &inserted)
            } else {
                let mut next = current.to_string();
                next.insert_str(start, &inserted);
                (next, caret_index + inserted.chars().count())
            }
        } else {
            let mut next = current.to_string();
            next.insert_str(start, &inserted);
            (next, caret_index + inserted.chars().count())
        };

        if let Some(max_length) = max_length {
            if next.chars().count() > max_length {
                next = next.chars().take(max_length).collect();
                next_caret = next_caret.min(next.chars().count());
            }
        }

        Some((next, next_caret))
    }

    fn render_input_box(&mut self, window: &Window) -> AnyElement {
        let tokens = &self.theme.components.input;
        let resolved_value = self.resolved_value();
        let current_value = resolved_value.to_string();
        let tracked_focus = control::focused_state(&self.id, None, false);
        let handle_focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|focus_handle| focus_handle.is_focused(window));
        let is_focused = handle_focused || tracked_focus;
        let current_len = current_value.chars().count();
        let current_caret =
            control::text_state(&self.id, "caret-index", None, current_len.to_string())
                .parse::<usize>()
                .ok()
                .map(|value| value.min(current_len))
                .unwrap_or(current_len);
        let selection = Self::selection_bounds_for(&self.id, current_len);
        let char_width = self.char_width_px();

        let mut input = div()
            .id(self.id.slot("box"))
            .focusable()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .w_full()
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .text_color(resolve_hsla(&self.theme, &tokens.fg))
            .border(quantized_stroke_px(window, 1.0));

        input = apply_input_size(input, self.size);
        input = apply_radius(&self.theme, input, self.radius);

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
            input = input.track_focus(focus_handle);
        }

        let id_for_blur = self.id.clone();
        input = input.on_mouse_down_out(move |_, window, _cx| {
            control::set_focused_state(&id_for_blur, false);
            control::set_bool_state(&id_for_blur, "mouse-selecting", false);
            window.refresh();
        });

        if !self.disabled && !self.read_only {
            let on_change = self.on_change.clone();
            let on_submit = self.on_submit.clone();
            let rendered_value = current_value.clone();
            let max_length = self.max_length;
            let input_id = self.id.clone();
            let focus_state_id = self.id.clone();
            let masked = self.masked;
            let mask_reveal_ms = self.mask_reveal_ms;
            let value_controlled = self.value_controlled;
            let value_for_mouse_down = current_value.clone();
            let value_for_mouse_move = current_value.clone();
            let value_for_right_click = current_value.clone();
            let char_width_for_click = char_width;
            let char_width_for_move = char_width;
            let char_width_for_right_click = char_width;
            let focus_handle_for_mouse = self.focus_handle.clone();
            let focus_handle_for_right_click = self.focus_handle.clone();
            let id_for_mouse_down = self.id.clone();
            let id_for_mouse_move = self.id.clone();
            let id_for_mouse_up = self.id.clone();
            let id_for_mouse_up_out = self.id.clone();
            let id_for_right_click = self.id.clone();

            input = input
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    control::set_focused_state(&id_for_mouse_down, true);
                    if let Some(handle) = focus_handle_for_mouse.as_ref() {
                        window.focus(handle, cx);
                    }

                    let click_caret = Self::caret_from_click(
                        &id_for_mouse_down,
                        event.position,
                        &value_for_mouse_down,
                        char_width_for_click,
                    );
                    let len = value_for_mouse_down.chars().count();
                    let current_caret = control::text_state(
                        &id_for_mouse_down,
                        "caret-index",
                        None,
                        len.to_string(),
                    )
                    .parse::<usize>()
                    .ok()
                    .map(|value| value.min(len))
                    .unwrap_or(len);

                    if event.modifiers.shift {
                        let existing_selection =
                            Self::selection_bounds_for(&id_for_mouse_down, len);
                        let anchor = if let Some((start, end)) = existing_selection {
                            if current_caret == start { end } else { start }
                        } else {
                            current_caret
                        };
                        Self::set_selection_for(&id_for_mouse_down, anchor, click_caret);
                        control::set_text_state(
                            &id_for_mouse_down,
                            "selection-anchor",
                            anchor.to_string(),
                        );
                    } else {
                        Self::clear_selection_for(&id_for_mouse_down, click_caret);
                        control::set_text_state(
                            &id_for_mouse_down,
                            "selection-anchor",
                            click_caret.to_string(),
                        );
                    }
                    control::set_text_state(
                        &id_for_mouse_down,
                        "caret-index",
                        click_caret.to_string(),
                    );
                    control::set_bool_state(&id_for_mouse_down, "mouse-selecting", true);
                    window.refresh();
                })
                .on_mouse_move(move |event, window, _cx| {
                    if !control::bool_state(&id_for_mouse_move, "mouse-selecting", None, false) {
                        return;
                    }

                    let caret = Self::caret_from_click(
                        &id_for_mouse_move,
                        event.position,
                        &value_for_mouse_move,
                        char_width_for_move,
                    );
                    let anchor = control::text_state(
                        &id_for_mouse_move,
                        "selection-anchor",
                        None,
                        caret.to_string(),
                    )
                    .parse::<usize>()
                    .ok()
                    .unwrap_or(caret);
                    control::set_text_state(&id_for_mouse_move, "caret-index", caret.to_string());
                    Self::set_selection_for(&id_for_mouse_move, anchor, caret);
                    window.refresh();
                })
                .on_mouse_up(MouseButton::Left, move |_, _, _| {
                    control::set_bool_state(&id_for_mouse_up, "mouse-selecting", false);
                })
                .on_mouse_up_out(MouseButton::Left, move |_, _, _| {
                    control::set_bool_state(&id_for_mouse_up_out, "mouse-selecting", false);
                })
                .on_click(move |event: &ClickEvent, window, cx| {
                    if !event.is_right_click() {
                        return;
                    }
                    control::set_focused_state(&id_for_right_click, true);
                    if let Some(handle) = focus_handle_for_right_click.as_ref() {
                        window.focus(handle, cx);
                    }
                    if let Some(position) = event.mouse_position() {
                        let caret = Self::caret_from_click(
                            &id_for_right_click,
                            position,
                            &value_for_right_click,
                            char_width_for_right_click,
                        );
                        let len = value_for_right_click.chars().count();
                        let selection = Self::selection_bounds_for(&id_for_right_click, len);
                        let keep_selection =
                            selection.is_some_and(|(start, end)| caret >= start && caret <= end);
                        if !keep_selection {
                            control::set_text_state(
                                &id_for_right_click,
                                "caret-index",
                                caret.to_string(),
                            );
                            Self::clear_selection_for(&id_for_right_click, caret);
                            control::set_text_state(
                                &id_for_right_click,
                                "selection-anchor",
                                caret.to_string(),
                            );
                        }
                    }
                    window.refresh();
                });

            input = input.on_key_down(move |event, window, cx| {
                control::set_focused_state(&focus_state_id, true);
                let current_value = control::text_state(
                    &input_id,
                    "value",
                    value_controlled.then_some(rendered_value.clone()),
                    rendered_value.clone(),
                );
                let len = current_value.chars().count();
                let current_caret =
                    control::text_state(&input_id, "caret-index", None, len.to_string())
                        .parse::<usize>()
                        .ok()
                        .map(|value| value.min(len))
                        .unwrap_or(len);
                let selection = Self::selection_bounds_for(&input_id, len);
                let modifiers =
                    event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

                if event.keystroke.key == "enter" {
                    if let Some(handler) = &on_submit {
                        (handler)(current_value.clone().into(), window, cx);
                    }
                    return;
                }

                if modifiers && event.keystroke.key == "a" {
                    control::set_text_state(&input_id, "caret-index", len.to_string());
                    Self::set_selection_for(&input_id, 0, len);
                    control::set_text_state(&input_id, "selection-anchor", "0".to_string());
                    window.refresh();
                    return;
                }

                if modifiers && event.keystroke.key == "c" {
                    if let Some((start, end)) = selection {
                        let selected = current_value
                            .chars()
                            .skip(start)
                            .take(end.saturating_sub(start))
                            .collect::<String>();
                        if !selected.is_empty() {
                            cx.write_to_clipboard(ClipboardItem::new_string(selected));
                        }
                    }
                    return;
                }

                if modifiers && event.keystroke.key == "x" {
                    if let Some((start, end)) = selection {
                        let selected = current_value
                            .chars()
                            .skip(start)
                            .take(end.saturating_sub(start))
                            .collect::<String>();
                        if !selected.is_empty() {
                            cx.write_to_clipboard(ClipboardItem::new_string(selected));
                            let (next, next_caret) =
                                Self::replace_char_range(&current_value, start, end, "");
                            control::set_text_state(
                                &input_id,
                                "caret-index",
                                next_caret.to_string(),
                            );
                            Self::clear_selection_for(&input_id, next_caret);
                            control::set_text_state(
                                &input_id,
                                "selection-anchor",
                                next_caret.to_string(),
                            );
                            if !value_controlled {
                                control::set_text_state(&input_id, "value", next.clone());
                            }
                            if let Some(handler) = on_change.as_ref() {
                                (handler)(next.into(), window, cx);
                            }
                            window.refresh();
                        }
                    }
                    return;
                }

                if modifiers
                    && event.keystroke.key == "v"
                    && let Some(item) = cx.read_from_clipboard()
                    && let Some(pasted) = item.text()
                {
                    let sanitized = pasted.replace('\r', " ").replace('\n', " ");
                    if sanitized.is_empty() {
                        return;
                    }
                    let (mut next, mut next_caret) = if let Some((start, end)) = selection {
                        Self::replace_char_range(&current_value, start, end, &sanitized)
                    } else {
                        let byte_start = Self::byte_index_at_char(&current_value, current_caret);
                        let mut next = current_value.clone();
                        next.insert_str(byte_start, &sanitized);
                        (next, current_caret + sanitized.chars().count())
                    };
                    if let Some(limit) = max_length
                        && next.chars().count() > limit
                    {
                        next = next.chars().take(limit).collect();
                        next_caret = next_caret.min(next.chars().count());
                    }
                    if masked {
                        if next.chars().count() > current_value.chars().count() {
                            Self::set_password_reveal(&input_id, &next, mask_reveal_ms);
                        } else {
                            Self::clear_password_reveal(&input_id);
                        }
                    }
                    control::set_text_state(&input_id, "caret-index", next_caret.to_string());
                    Self::clear_selection_for(&input_id, next_caret);
                    control::set_text_state(&input_id, "selection-anchor", next_caret.to_string());
                    if !value_controlled {
                        control::set_text_state(&input_id, "value", next.clone());
                    }
                    if let Some(handler) = on_change.as_ref() {
                        (handler)(next.into(), window, cx);
                    }
                    window.refresh();
                    return;
                }

                if event.keystroke.modifiers.shift
                    && matches!(
                        event.keystroke.key.as_str(),
                        "left" | "right" | "home" | "end"
                    )
                {
                    let anchor = if let Some((start, end)) = selection {
                        if current_caret == start { end } else { start }
                    } else {
                        current_caret
                    };
                    if let Some((_next, next_caret)) = Self::with_value_update(
                        &current_value,
                        event,
                        max_length,
                        current_caret,
                        None,
                    ) {
                        control::set_text_state(&input_id, "caret-index", next_caret.to_string());
                        Self::set_selection_for(&input_id, anchor, next_caret);
                        control::set_text_state(&input_id, "selection-anchor", anchor.to_string());
                        window.refresh();
                    }
                    return;
                }

                if let Some((next, next_caret)) = Self::with_value_update(
                    &current_value,
                    event,
                    max_length,
                    current_caret,
                    selection,
                ) {
                    let value_changed = next != current_value;
                    if value_changed && masked {
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

                    if value_changed && !value_controlled {
                        control::set_text_state(&input_id, "value", next.clone());
                    }
                    control::set_text_state(&input_id, "caret-index", next_caret.to_string());
                    Self::clear_selection_for(&input_id, next_caret);
                    control::set_text_state(&input_id, "selection-anchor", next_caret.to_string());
                    window.refresh();

                    if value_changed && let Some(handler) = &on_change {
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
        let mut value_container = div()
            .id(self.id.slot("content"))
            .relative()
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .gap_0()
            .overflow_hidden()
            .whitespace_nowrap();
        value_container = value_container.child({
            let id_for_metrics = self.id.clone();
            canvas(
                move |bounds, _, _cx| {
                    control::set_text_state(
                        &id_for_metrics,
                        "content-origin-x",
                        f32::from(bounds.origin.x).to_string(),
                    );
                    control::set_text_state(
                        &id_for_metrics,
                        "content-origin-y",
                        f32::from(bounds.origin.y).to_string(),
                    );
                    control::set_text_state(
                        &id_for_metrics,
                        "content-width",
                        f32::from(bounds.size.width).to_string(),
                    );
                    control::set_text_state(
                        &id_for_metrics,
                        "content-height",
                        f32::from(bounds.size.height).to_string(),
                    );
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        });

        if value.is_empty() && !is_focused {
            value_container = value_container.child(
                div()
                    .truncate()
                    .text_color(resolve_hsla(&self.theme, &tokens.placeholder))
                    .child(self.placeholder.clone().unwrap_or_default()),
            );
        } else {
            let show_caret = is_focused;
            let selection_bg =
                resolve_hsla(&self.theme, &self.theme.semantic.focus_ring).alpha(0.28);
            if let Some((selection_start, selection_end)) = selection {
                let left = value.chars().take(selection_start).collect::<String>();
                let selected = value
                    .chars()
                    .skip(selection_start)
                    .take(selection_end - selection_start)
                    .collect::<String>();
                let right = value.chars().skip(selection_end).collect::<String>();
                value_container = value_container.child(
                    div()
                        .flex()
                        .items_center()
                        .whitespace_nowrap()
                        .child(if left.is_empty() {
                            "".to_string()
                        } else {
                            left
                        })
                        .child(div().bg(selection_bg).child(if selected.is_empty() {
                            " ".to_string()
                        } else {
                            selected
                        }))
                        .child(if right.is_empty() {
                            "".to_string()
                        } else {
                            right
                        }),
                );
            } else if !self.disabled && !self.read_only && show_caret {
                let left = value.chars().take(current_caret).collect::<String>();
                let right = value.chars().skip(current_caret).collect::<String>();
                let caret = div()
                    .id(self.id.slot("caret"))
                    .flex_none()
                    .w(quantized_stroke_px(window, 1.5))
                    .h(px(self.caret_height_px()))
                    .bg(resolve_hsla(&self.theme, &tokens.fg))
                    .rounded_sm()
                    .with_animation(
                        self.id.slot("caret-blink"),
                        Animation::new(Duration::from_millis(CARET_BLINK_CYCLE_MS))
                            .repeat()
                            .with_easing(gpui::linear),
                        |this, delta| {
                            let visible = ((delta * 2.0).fract()) < 0.5;
                            this.opacity(if visible { 1.0 } else { 0.0 })
                        },
                    );
                value_container = value_container.child(
                    div()
                        .flex()
                        .items_center()
                        .whitespace_nowrap()
                        .child(if left.is_empty() {
                            "".to_string()
                        } else {
                            left
                        })
                        .child(caret)
                        .child(if right.is_empty() {
                            "".to_string()
                        } else {
                            right
                        }),
                );
            } else if value.is_empty() {
                value_container = value_container.child("".to_string());
            } else {
                value_container = value_container.child(div().child(value));
            }
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
            .with_enter_transition(self.id.slot("enter"), self.motion)
            .into_any_element()
    }

    fn render_label_block(&self) -> AnyElement {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return div().into_any_element();
        }

        let mut block = Stack::vertical().gap_1();

        if let Some(label) = &self.label {
            let mut label_row = Stack::horizontal().gap_1().child(
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

impl TextInput {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
            FieldLayout::Vertical => Stack::vertical()
                .id(self.id.clone())
                .gap_2()
                .child(self.render_label_block())
                .child(self.render_input_box(window)),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap_3()
                .child(div().w(gpui::px(168.0)).child(self.render_label_block()))
                .child(div().flex_1().child(self.render_input_box(window))),
        }
    }
}

#[derive(IntoElement)]
pub struct PasswordInput {
    id: ComponentId,
    inner: TextInput,
    style: gpui::StyleRefinement,
}

impl PasswordInput {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
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

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.label(value);
        self
    }

    pub fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.description(value);
        self
    }

    pub fn error(mut self, value: impl Into<SharedString>) -> Self {
        self.inner = self.inner.error(value);
        self
    }

    pub fn required(mut self, value: bool) -> Self {
        self.inner = self.inner.required(value);
        self
    }

    pub fn layout(mut self, value: FieldLayout) -> Self {
        self.inner = self.inner.layout(value);
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

impl PasswordInput {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
        let mut inner = self.inner.with_id(self.id);
        gpui::Refineable::refine(gpui::Styled::style(&mut inner), &self.style);
        inner.render(window, cx)
    }
}

#[derive(IntoElement)]
pub struct PinInput {
    id: ComponentId,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: SharedString,
    error: Option<SharedString>,
    disabled: bool,
    read_only: bool,
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
            id: ComponentId::default(),
            value: None,
            value_controlled: false,
            default_value: SharedString::default(),
            error: None,
            disabled: false,
            read_only: false,
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

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }

    pub fn error(mut self, value: impl Into<SharedString>) -> Self {
        self.error = Some(value.into());
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

impl PinInput {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
        let tracked_focus = control::focused_state(&self.id, None, false);
        let is_focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|focus_handle| focus_handle.is_focused(window))
            || tracked_focus;
        let caret_height = match self.size {
            Size::Xs => 13.0,
            Size::Sm => 15.0,
            Size::Md => 17.0,
            Size::Lg => 19.0,
            Size::Xl => 21.0,
        };
        let caret_color = resolve_hsla(&self.theme, &self.theme.components.input.fg);
        let has_error = self.error.is_some();
        let interactive = !self.disabled && !self.read_only;

        let mut root = Stack::horizontal().id(self.id.clone()).focusable().gap_2();

        if self.disabled {
            root = root.cursor_default();
        } else if self.read_only {
            root = root.cursor_default();
        } else {
            root = root.cursor_text();
        }

        if interactive {
            let focus_state_id = self.id.clone();
            root = root.on_key_down(move |event, window, cx| {
                control::set_focused_state(&focus_state_id, true);
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
        }

        if let Some(focus_handle) = &self.focus_handle
            && !self.disabled
        {
            let handle_for_click = focus_handle.clone();
            let focus_state_id = self.id.clone();
            root = root
                .track_focus(focus_handle)
                .on_click(move |_, window, cx| {
                    control::set_focused_state(&focus_state_id, true);
                    window.focus(&handle_for_click, cx);
                    window.refresh();
                });
        } else if !self.disabled {
            let focus_state_id = self.id.clone();
            root = root.on_click(move |_, window, _cx| {
                control::set_focused_state(&focus_state_id, true);
                window.refresh();
            });
        }

        if self.disabled {
            root = root.opacity(0.55);
        }

        let blur_state_id = self.id.clone();
        root = root.on_mouse_down_out(move |_, window, _cx| {
            control::set_focused_state(&blur_state_id, false);
            window.refresh();
        });

        for index in 0..self.length {
            let content = value_chars.get(index).map(|ch| ch.to_string());
            let border = if self.disabled {
                resolve_hsla(&self.theme, &self.theme.semantic.border_subtle)
            } else if has_error {
                resolve_hsla(&self.theme, &self.theme.components.input.border_error)
            } else if is_focused {
                resolve_hsla(&self.theme, &self.theme.components.input.border_focus)
            } else {
                resolve_hsla(&self.theme, &self.theme.components.input.border)
            };
            let mut cell = div()
                .w(gpui::px(34.0))
                .h(gpui::px(40.0))
                .border(quantized_stroke_px(window, 1.0))
                .border_color(border)
                .bg(resolve_hsla(&self.theme, &self.theme.components.input.bg))
                .flex()
                .items_center()
                .justify_center();

            if let Some(content) = content {
                cell = cell.child(content);
            } else if index == active_index
                && value_chars.len() < self.length
                && interactive
                && (self.focus_handle.is_none() || is_focused)
            {
                cell = cell.child(
                    div()
                        .id(self.id.slot_index("caret", index.to_string()))
                        .w(quantized_stroke_px(window, 1.5))
                        .h(px(caret_height))
                        .bg(caret_color)
                        .rounded_sm()
                        .with_animation(
                            self.id.slot_index("caret-blink", index.to_string()),
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

        let field = root.with_enter_transition(self.id.slot("enter"), self.motion);

        if let Some(error) = self.error {
            Stack::vertical()
                .id(self.id.slot("field"))
                .gap_1()
                .child(field)
                .child(
                    div()
                        .text_sm()
                        .text_color(resolve_hsla(
                            &self.theme,
                            &self.theme.components.input.error,
                        ))
                        .child(error),
                )
                .into_any_element()
        } else {
            field.into_any_element()
        }
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
crate::impl_disableable!(PinInput);

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
