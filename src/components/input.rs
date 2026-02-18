use std::{
    collections::HashMap,
    ops::Range,
    rc::Rc,
    sync::{Arc, LazyLock, Mutex},
    time::{Duration, Instant},
};

use gpui::{
    Animation, AnimationExt, AnyElement, Bounds, ClipboardItem, FocusHandle, InputHandler,
    InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, UTF16Selection, Window, canvas, div, point, px,
};

use crate::contracts::{FieldLike, MotionAware, Sized, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::text_input_actions::{
    CopySelection, CutSelection, DeleteBackward, DeleteForward, INPUT_KEY_CONTEXT, MoveEnd,
    MoveHome, MoveLeft, MoveRight, PasteClipboard, SelectAll, SelectEnd, SelectHome, SelectLeft,
    SelectRight, Submit, ensure_text_keybindings,
};
use super::text_input_state::InputState;
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
static INPUT_FOCUS_HANDLES: LazyLock<Mutex<HashMap<String, FocusHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
struct TextInputImeHandler {
    id: String,
    value_controlled: bool,
    rendered_value: String,
    max_length: Option<usize>,
    disabled: bool,
    read_only: bool,
    masked: bool,
    mask_reveal_ms: u64,
    font_size: f32,
    on_change: Option<ChangeHandler>,
}

impl TextInputImeHandler {
    fn current_value(&self) -> String {
        control::text_state(
            &self.id,
            "value",
            self.value_controlled.then_some(self.rendered_value.clone()),
            self.rendered_value.clone(),
        )
    }

    fn char_index_from_utf16(value: &str, utf16_index: usize) -> usize {
        let mut utf16_count = 0usize;
        let mut char_index = 0usize;
        for ch in value.chars() {
            if utf16_count >= utf16_index {
                break;
            }
            utf16_count += ch.len_utf16();
            char_index += 1;
        }
        char_index
    }

    fn utf16_from_char(value: &str, char_index: usize) -> usize {
        value
            .chars()
            .take(char_index)
            .map(|ch| ch.len_utf16())
            .sum::<usize>()
    }

    fn char_range_from_utf16(value: &str, range_utf16: Range<usize>) -> Range<usize> {
        let start = Self::char_index_from_utf16(value, range_utf16.start);
        let end = Self::char_index_from_utf16(value, range_utf16.end);
        if start <= end { start..end } else { end..start }
    }

    fn utf16_range_from_char(value: &str, range: Range<usize>) -> Range<usize> {
        let start = Self::utf16_from_char(value, range.start);
        let end = Self::utf16_from_char(value, range.end);
        start..end
    }

    fn marked_range_chars(&self, len: usize) -> Option<(usize, usize)> {
        let start = control::optional_text_state(&self.id, "marked-start", None, None)
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX);
        let end = control::optional_text_state(&self.id, "marked-end", None, None)
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX);
        if start == usize::MAX || end == usize::MAX {
            return None;
        }
        let start = start.min(len);
        let end = end.min(len);
        (start < end).then_some((start, end))
    }

    fn set_marked_range_chars(&self, marked: Option<(usize, usize)>) {
        if let Some((start, end)) = marked {
            control::set_optional_text_state(&self.id, "marked-start", Some(start.to_string()));
            control::set_optional_text_state(&self.id, "marked-end", Some(end.to_string()));
        } else {
            control::set_optional_text_state(&self.id, "marked-start", None);
            control::set_optional_text_state(&self.id, "marked-end", None);
        }
    }

    fn resolve_replacement_range(
        &self,
        value: &str,
        replacement_range: Option<Range<usize>>,
    ) -> (usize, usize) {
        let len = value.chars().count();
        if let Some(range_utf16) = replacement_range {
            let range = Self::char_range_from_utf16(value, range_utf16);
            return (range.start.min(len), range.end.min(len));
        }
        if let Some((start, end)) = self.marked_range_chars(len) {
            return (start, end);
        }
        if let Some((start, end)) = TextInput::selection_bounds_for(&self.id, len) {
            return (start, end);
        }
        let caret = control::text_state(&self.id, "caret-index", None, len.to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(len)
            .min(len);
        (caret, caret)
    }

    fn apply_max_length(
        &self,
        mut next: String,
        mut caret: usize,
        mut marked: Option<(usize, usize)>,
        mut selection: Option<(usize, usize)>,
    ) -> (
        String,
        usize,
        Option<(usize, usize)>,
        Option<(usize, usize)>,
    ) {
        if let Some(limit) = self.max_length
            && next.chars().count() > limit
        {
            next = next.chars().take(limit).collect();
            let next_len = next.chars().count();
            caret = caret.min(next_len);
            marked = marked.and_then(|(start, end)| {
                let start = start.min(next_len);
                let end = end.min(next_len);
                (start < end).then_some((start, end))
            });
            selection = selection.and_then(|(start, end)| {
                let start = start.min(next_len);
                let end = end.min(next_len);
                (start < end).then_some((start, end))
            });
        }
        (next, caret, marked, selection)
    }

    fn apply_edit_result(
        &self,
        previous: &str,
        next: String,
        caret: usize,
        selection: Option<(usize, usize)>,
        marked: Option<(usize, usize)>,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        let changed = next != previous;
        if changed && self.masked {
            let previous_len = previous.chars().count();
            let next_len = next.chars().count();
            if next_len > previous_len {
                TextInput::set_password_reveal(&self.id, &next, self.mask_reveal_ms);
            } else {
                TextInput::clear_password_reveal(&self.id);
            }
        }

        if changed && !self.value_controlled {
            control::set_text_state(&self.id, "value", next.clone());
        }
        control::set_text_state(&self.id, "caret-index", caret.to_string());
        if let Some((start, end)) = selection {
            TextInput::set_selection_for(&self.id, start, end);
        } else {
            TextInput::clear_selection_for(&self.id, caret);
        }
        control::set_text_state(&self.id, "selection-anchor", caret.to_string());
        self.set_marked_range_chars(marked);

        if changed && let Some(handler) = self.on_change.as_ref() {
            (handler)(next.into(), window, cx);
        }

        window.refresh();
    }
}

impl InputHandler for TextInputImeHandler {
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut gpui::App,
    ) -> Option<UTF16Selection> {
        let value = self.current_value();
        let len = value.chars().count();
        let caret = control::text_state(&self.id, "caret-index", None, len.to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(len)
            .min(len);
        let range = if let Some((start, end)) = TextInput::selection_bounds_for(&self.id, len) {
            start..end
        } else {
            caret..caret
        };
        let reversed = !range.is_empty() && caret == range.start;
        Some(UTF16Selection {
            range: Self::utf16_range_from_char(&value, range),
            reversed,
        })
    }

    fn marked_text_range(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui::App,
    ) -> Option<Range<usize>> {
        let value = self.current_value();
        let len = value.chars().count();
        let (start, end) = self.marked_range_chars(len)?;
        Some(Self::utf16_range_from_char(&value, start..end))
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut gpui::App,
    ) -> Option<String> {
        let value = self.current_value();
        let len = value.chars().count();
        let range = Self::char_range_from_utf16(&value, range_utf16);
        let start = range.start.min(len);
        let end = range.end.min(len).max(start);
        adjusted_range.replace(Self::utf16_range_from_char(&value, start..end));
        Some(value.chars().skip(start).take(end - start).collect())
    }

    fn replace_text_in_range(
        &mut self,
        replacement_range: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        if self.disabled || self.read_only {
            return;
        }
        let value = self.current_value();
        let (start, end) = self.resolve_replacement_range(&value, replacement_range);
        let sanitized = text.replace(['\r', '\n'], "");
        let (next, caret) = TextInput::replace_char_range(&value, start, end, &sanitized);
        let (next, caret, _marked, selection) = self.apply_max_length(next, caret, None, None);
        self.apply_edit_result(&value, next, caret, selection, None, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        if self.disabled || self.read_only {
            return;
        }
        let value = self.current_value();
        let (start, end) = self.resolve_replacement_range(&value, range_utf16);
        let sanitized = new_text.replace(['\r', '\n'], "");
        let inserted_chars = sanitized.chars().count();
        let (next, fallback_caret) = TextInput::replace_char_range(&value, start, end, &sanitized);
        let next_len = next.chars().count();
        let marked = if inserted_chars > 0 {
            let mark_end = (start + inserted_chars).min(next_len);
            (start < mark_end).then_some((start, mark_end))
        } else {
            None
        };

        let selection = new_selected_range_utf16.map(|selection_utf16| {
            let relative = Self::char_range_from_utf16(&sanitized, selection_utf16);
            let selection_start = (start + relative.start).min(next_len);
            let selection_end = (start + relative.end).min(next_len);
            if selection_start <= selection_end {
                (selection_start, selection_end)
            } else {
                (selection_end, selection_start)
            }
        });

        let caret = selection.map(|(_, end)| end).unwrap_or(fallback_caret);
        let (next, caret, marked, selection) =
            self.apply_max_length(next, caret, marked, selection);
        self.apply_edit_result(&value, next, caret, selection, marked, window, cx);
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut gpui::App) {
        self.set_marked_range_chars(None);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        window: &mut Window,
        _cx: &mut gpui::App,
    ) -> Option<Bounds<gpui::Pixels>> {
        let value = self.current_value();
        let range = Self::char_range_from_utf16(&value, range_utf16);
        let (origin_x, origin_y, _width, height) = TextInput::content_geometry(&self.id);
        let scroll_x = control::text_state(&self.id, "scroll-x", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let metric_text = if self.masked {
            "*".repeat(value.chars().count())
        } else {
            value.clone()
        };
        let start_x = origin_x
            + TextInput::x_for_char(window, self.font_size, &metric_text, range.start)
            - scroll_x;
        let end_x = origin_x
            + TextInput::x_for_char(window, self.font_size, &metric_text, range.end)
            - scroll_x;
        let top = origin_y;
        let bottom = origin_y + height.max(1.0);
        let right = if end_x > start_x {
            end_x
        } else {
            start_x + 1.0
        };
        Some(Bounds::from_corners(
            point(px(start_x), px(top)),
            point(px(right), px(bottom)),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<gpui::Pixels>,
        window: &mut Window,
        _cx: &mut gpui::App,
    ) -> Option<usize> {
        let value = self.current_value();
        let (origin_x, _origin_y, _width, _height) = TextInput::content_geometry(&self.id);
        let scroll_x = control::text_state(&self.id, "scroll-x", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let local_x = (f32::from(point.x) - origin_x + scroll_x).max(0.0);
        let metric_text = if self.masked {
            "*".repeat(value.chars().count())
        } else {
            value.clone()
        };
        let char_index = TextInput::char_from_x(window, self.font_size, &metric_text, local_x)
            .min(value.chars().count());
        Some(Self::utf16_from_char(&value, char_index))
    }

    fn accepts_text_input(&mut self, _window: &mut Window, _cx: &mut gpui::App) -> bool {
        !self.disabled && !self.read_only
    }
}

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
    fn resolved_focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        if let Some(focus_handle) = self.focus_handle.as_ref() {
            return focus_handle.clone();
        }
        if let Ok(mut handles) = INPUT_FOCUS_HANDLES.lock() {
            return handles
                .entry(self.id.to_string())
                .or_insert_with(|| cx.focus_handle())
                .clone();
        }
        cx.focus_handle()
    }

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

    fn font_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 12.0,
            Size::Sm => 14.0,
            Size::Md => 16.0,
            Size::Lg => 18.0,
            Size::Xl => 20.0,
        }
    }

    fn char_width_px(&self, window: &Window) -> f32 {
        let font_size = px(self.font_size_px());
        let mut text_style = window.text_style();
        text_style.font_size = font_size.into();
        let sample = "0000000000";
        let run = text_style.to_run(sample.len());
        let layout = window
            .text_system()
            .layout_line(sample, font_size, &[run], None);
        let measured = f32::from(layout.width) / sample.len() as f32;
        if measured.is_finite() && measured > 0.1 {
            measured
        } else {
            (self.font_size_px() * 0.6).max(1.0)
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

    fn char_index_at_byte(value: &str, byte_index: usize) -> usize {
        let mut byte_index = byte_index.min(value.len());
        while byte_index > 0 && !value.is_char_boundary(byte_index) {
            byte_index -= 1;
        }
        value[..byte_index].chars().count()
    }

    fn line_layout(window: &Window, font_size: f32, text: &str) -> Arc<gpui::LineLayout> {
        let font_size = px(font_size);
        let mut text_style = window.text_style();
        text_style.font_size = font_size.into();
        let run = text_style.to_run(text.len());
        window
            .text_system()
            .layout_line(text, font_size, &[run], None)
    }

    fn x_for_char(window: &Window, font_size: f32, text: &str, char_index: usize) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let char_index = char_index.min(text.chars().count());
        let byte_index = Self::byte_index_at_char(text, char_index);
        let layout = Self::line_layout(window, font_size, text);
        f32::from(layout.x_for_index(byte_index))
    }

    fn char_from_x(window: &Window, font_size: f32, text: &str, x: f32) -> usize {
        if text.is_empty() {
            return 0;
        }
        let layout = Self::line_layout(window, font_size, text);
        let byte_index = layout.closest_index_for_x(px(x.max(0.0))).min(text.len());
        Self::char_index_at_byte(text, byte_index).min(text.chars().count())
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

    fn editor_state_for(id: &str, current_value: &str) -> InputState {
        let len = current_value.chars().count();
        let caret = control::text_state(id, "caret-index", None, len.to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(len)
            .min(len);
        let anchor = control::text_state(id, "selection-anchor", None, caret.to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(caret)
            .min(len);
        let selection = Self::selection_bounds_for(id, len);
        InputState::new(current_value.to_string(), caret, anchor, selection)
    }

    fn persist_editor_state(id: &str, state: &InputState) {
        control::set_text_state(id, "caret-index", state.caret.to_string());
        if let Some((start, end)) = state.selection {
            Self::set_selection_for(id, start, end);
        } else {
            Self::clear_selection_for(id, state.caret);
        }
        control::set_text_state(id, "selection-anchor", state.anchor.to_string());
    }

    fn apply_editor_state(
        id: &str,
        previous_value: &str,
        state: &InputState,
        value_controlled: bool,
        masked: bool,
        mask_reveal_ms: u64,
        on_change: Option<&ChangeHandler>,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        let next_value = state.value.clone();
        let value_changed = next_value != previous_value;

        if value_changed && masked {
            let previous_len = previous_value.chars().count();
            let next_len = next_value.chars().count();
            if next_len > previous_len {
                Self::set_password_reveal(id, &next_value, mask_reveal_ms);
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
                Self::clear_password_reveal(id);
            }
        }

        if value_changed && !value_controlled {
            control::set_text_state(id, "value", next_value.clone());
        }
        Self::persist_editor_state(id, state);
        window.refresh();

        if value_changed && let Some(handler) = on_change {
            (handler)(next_value.into(), window, cx);
        }
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
        window: &Window,
        font_size: f32,
    ) -> usize {
        let (origin_x, _origin_y, _width, _height) = Self::content_geometry(id);
        let scroll_x = control::text_state(id, "scroll-x", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let local_x = (f32::from(position.x) - origin_x + scroll_x).max(0.0);
        Self::char_from_x(window, font_size, value, local_x).min(value.chars().count())
    }

    fn render_input_box(&mut self, window: &mut Window, cx: &mut gpui::App) -> AnyElement {
        ensure_text_keybindings(cx);
        let tokens = &self.theme.components.input;
        let resolved_value = self.resolved_value();
        let current_value = resolved_value.to_string();
        let focus_handle = self.resolved_focus_handle(cx);
        let tracked_focus = control::focused_state(&self.id, None, false);
        let handle_focused = focus_handle.is_focused(window);
        let is_focused = handle_focused || tracked_focus;
        let current_len = current_value.chars().count();
        let current_caret =
            control::text_state(&self.id, "caret-index", None, current_len.to_string())
                .parse::<usize>()
                .ok()
                .map(|value| value.min(current_len))
                .unwrap_or(current_len);
        let selection = Self::selection_bounds_for(&self.id, current_len);
        let font_size = self.font_size_px();
        let char_width = self.char_width_px(window);

        let mut input = div()
            .id(self.id.slot("box"))
            .relative()
            .focusable()
            .key_context(INPUT_KEY_CONTEXT)
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
        input = input.track_focus(&focus_handle);
        let id_for_blur = self.id.clone();
        input = input.on_mouse_down_out(move |_, window, _cx| {
            control::set_focused_state(&id_for_blur, false);
            control::set_bool_state(&id_for_blur, "mouse-selecting", false);
            window.refresh();
        });

        if !self.disabled && !self.read_only {
            let value_controlled = self.value_controlled;
            let value_for_mouse = current_value.clone();
            let value_for_mouse_down = value_for_mouse.clone();
            let value_for_mouse_move = value_for_mouse.clone();
            let font_size_for_mouse = font_size;
            let focus_handle_for_mouse = focus_handle.clone();
            let id_for_mouse_down = self.id.clone();
            let id_for_mouse_move = self.id.clone();
            let id_for_mouse_up = self.id.clone();
            let id_for_mouse_up_out = self.id.clone();

            input = input
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    control::set_focused_state(&id_for_mouse_down, true);
                    window.focus(&focus_handle_for_mouse, cx);

                    let current_value = control::text_state(
                        &id_for_mouse_down,
                        "value",
                        value_controlled.then_some(value_for_mouse_down.clone()),
                        value_for_mouse_down.clone(),
                    );
                    let click_caret = Self::caret_from_click(
                        &id_for_mouse_down,
                        event.position,
                        &current_value,
                        window,
                        font_size_for_mouse,
                    );
                    let len = current_value.chars().count();
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

                    let current_value = control::text_state(
                        &id_for_mouse_move,
                        "value",
                        value_controlled.then_some(value_for_mouse_move.clone()),
                        value_for_mouse_move.clone(),
                    );
                    let caret = Self::caret_from_click(
                        &id_for_mouse_move,
                        event.position,
                        &current_value,
                        window,
                        font_size_for_mouse,
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
                });
        }

        let max_length = self.max_length;
        if !self.disabled {
            let input_id = self.id.clone();
            let rendered_value = current_value.clone();
            let value_controlled = self.value_controlled;
            let on_change = self.on_change.clone();
            let masked = self.masked;
            let mask_reveal_ms = self.mask_reveal_ms;
            input = input
                .on_action(move |_: &MoveLeft, window, cx| {
                    let current_value = control::text_state(
                        &input_id,
                        "value",
                        value_controlled.then_some(rendered_value.clone()),
                        rendered_value.clone(),
                    );
                    let mut state = Self::editor_state_for(&input_id, &current_value);
                    state.move_left(false);
                    Self::apply_editor_state(
                        &input_id,
                        &current_value,
                        &state,
                        value_controlled,
                        masked,
                        mask_reveal_ms,
                        on_change.as_ref(),
                        window,
                        cx,
                    );
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &MoveRight, window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let mut state = Self::editor_state_for(&input_id, &current_value);
                        state.move_right(false);
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            masked,
                            mask_reveal_ms,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &MoveHome, window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let mut state = Self::editor_state_for(&input_id, &current_value);
                        state.move_to(0, false);
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            masked,
                            mask_reveal_ms,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &MoveEnd, window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let mut state = Self::editor_state_for(&input_id, &current_value);
                        state.move_to(state.len(), false);
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            masked,
                            mask_reveal_ms,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &SelectLeft, window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let mut state = Self::editor_state_for(&input_id, &current_value);
                        state.move_left(true);
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            masked,
                            mask_reveal_ms,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &SelectRight, window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let mut state = Self::editor_state_for(&input_id, &current_value);
                        state.move_right(true);
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            masked,
                            mask_reveal_ms,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &SelectHome, window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let mut state = Self::editor_state_for(&input_id, &current_value);
                        state.move_to(0, true);
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            masked,
                            mask_reveal_ms,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &SelectEnd, window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let mut state = Self::editor_state_for(&input_id, &current_value);
                        let len = state.len();
                        state.move_to(len, true);
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            masked,
                            mask_reveal_ms,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &SelectAll, window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let mut state = Self::editor_state_for(&input_id, &current_value);
                        let len = state.len();
                        state.set_selection_from_anchor(0, len);
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            masked,
                            mask_reveal_ms,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value = current_value.clone();
                    move |_: &CopySelection, _window, cx| {
                        let current_value = control::text_state(
                            &input_id,
                            "value",
                            value_controlled.then_some(rendered_value.clone()),
                            rendered_value.clone(),
                        );
                        let state = Self::editor_state_for(&input_id, &current_value);
                        let selected = state.selected_text();
                        if !selected.is_empty() {
                            cx.write_to_clipboard(ClipboardItem::new_string(selected));
                        }
                    }
                });

            if !self.read_only {
                input = input
                    .on_action({
                        let input_id = self.id.clone();
                        let rendered_value = current_value.clone();
                        let on_change = self.on_change.clone();
                        move |_: &DeleteBackward, window, cx| {
                            let current_value = control::text_state(
                                &input_id,
                                "value",
                                value_controlled.then_some(rendered_value.clone()),
                                rendered_value.clone(),
                            );
                            let mut state = Self::editor_state_for(&input_id, &current_value);
                            if state.delete_backward() {
                                state.clamp_to_max_length(max_length);
                            }
                            Self::apply_editor_state(
                                &input_id,
                                &current_value,
                                &state,
                                value_controlled,
                                masked,
                                mask_reveal_ms,
                                on_change.as_ref(),
                                window,
                                cx,
                            );
                        }
                    })
                    .on_action({
                        let input_id = self.id.clone();
                        let rendered_value = current_value.clone();
                        let on_change = self.on_change.clone();
                        move |_: &DeleteForward, window, cx| {
                            let current_value = control::text_state(
                                &input_id,
                                "value",
                                value_controlled.then_some(rendered_value.clone()),
                                rendered_value.clone(),
                            );
                            let mut state = Self::editor_state_for(&input_id, &current_value);
                            if state.delete_forward() {
                                state.clamp_to_max_length(max_length);
                            }
                            Self::apply_editor_state(
                                &input_id,
                                &current_value,
                                &state,
                                value_controlled,
                                masked,
                                mask_reveal_ms,
                                on_change.as_ref(),
                                window,
                                cx,
                            );
                        }
                    })
                    .on_action({
                        let input_id = self.id.clone();
                        let rendered_value = current_value.clone();
                        let on_change = self.on_change.clone();
                        move |_: &CutSelection, window, cx| {
                            let current_value = control::text_state(
                                &input_id,
                                "value",
                                value_controlled.then_some(rendered_value.clone()),
                                rendered_value.clone(),
                            );
                            let mut state = Self::editor_state_for(&input_id, &current_value);
                            let selected = state.selected_text();
                            if selected.is_empty() {
                                return;
                            }
                            cx.write_to_clipboard(ClipboardItem::new_string(selected));
                            if let Some((start, end)) = state.selection {
                                state.replace_char_range(start, end, "");
                            }
                            Self::apply_editor_state(
                                &input_id,
                                &current_value,
                                &state,
                                value_controlled,
                                masked,
                                mask_reveal_ms,
                                on_change.as_ref(),
                                window,
                                cx,
                            );
                        }
                    })
                    .on_action({
                        let input_id = self.id.clone();
                        let rendered_value = current_value.clone();
                        let on_change = self.on_change.clone();
                        move |_: &PasteClipboard, window, cx| {
                            let Some(item) = cx.read_from_clipboard() else {
                                return;
                            };
                            let Some(text) = item.text() else {
                                return;
                            };
                            let sanitized = text.replace('\r', " ").replace('\n', " ");
                            if sanitized.is_empty() {
                                return;
                            }
                            let current_value = control::text_state(
                                &input_id,
                                "value",
                                value_controlled.then_some(rendered_value.clone()),
                                rendered_value.clone(),
                            );
                            let mut state = Self::editor_state_for(&input_id, &current_value);
                            if state.insert_text(&sanitized) {
                                state.clamp_to_max_length(max_length);
                            }
                            Self::apply_editor_state(
                                &input_id,
                                &current_value,
                                &state,
                                value_controlled,
                                masked,
                                mask_reveal_ms,
                                on_change.as_ref(),
                                window,
                                cx,
                            );
                        }
                    })
                    .on_action({
                        let input_id = self.id.clone();
                        let rendered_value = current_value.clone();
                        let on_submit = self.on_submit.clone();
                        move |_: &Submit, window, cx| {
                            let current_value = control::text_state(
                                &input_id,
                                "value",
                                value_controlled.then_some(rendered_value.clone()),
                                rendered_value.clone(),
                            );
                            if let Some(handler) = on_submit.as_ref() {
                                (handler)(current_value.into(), window, cx);
                            }
                        }
                    });
            }
        }

        window.handle_input(
            &focus_handle,
            TextInputImeHandler {
                id: self.id.to_string(),
                value_controlled: self.value_controlled,
                rendered_value: current_value.clone(),
                max_length: self.max_length,
                disabled: self.disabled,
                read_only: self.read_only,
                masked: self.masked,
                mask_reveal_ms: self.mask_reveal_ms,
                font_size,
                on_change: self.on_change.clone(),
            },
            cx,
        );

        if let Some(left_slot) = self.left_slot.take() {
            input = input.child(
                div()
                    .flex_none()
                    .text_color(resolve_hsla(&self.theme, &self.theme.semantic.text_muted))
                    .child(left_slot()),
            );
        }

        let value = self.display_value(&resolved_value);
        let (_, _, content_width, _) = Self::content_geometry(&self.id);
        let value_width = Self::x_for_char(window, font_size, &value, value.chars().count());
        let max_scroll = (value_width - content_width.max(0.0)).max(0.0);
        let mut scroll_x = control::text_state(&self.id, "scroll-x", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0)
            .clamp(0.0, max_scroll);
        if content_width <= 0.0 {
            scroll_x = 0.0;
        } else if !self.disabled && !self.read_only && is_focused {
            let caret_x = Self::x_for_char(window, font_size, &value, current_caret);
            let viewport_width = content_width.max(1.0);
            let right_guard = (viewport_width - char_width.max(2.0)).max(1.0);
            if caret_x < scroll_x {
                scroll_x = caret_x;
            } else if caret_x > scroll_x + right_guard {
                scroll_x = caret_x - right_guard;
            }
            scroll_x = scroll_x.clamp(0.0, max_scroll);
        } else if !is_focused {
            scroll_x = 0.0;
        }
        control::set_text_state(&self.id, "scroll-x", format!("{scroll_x:.3}"));
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
            let mut content_row = div()
                .relative()
                .left(px(-scroll_x))
                .flex()
                .items_center()
                .whitespace_nowrap();
            if let Some((selection_start, selection_end)) = selection {
                let left = value.chars().take(selection_start).collect::<String>();
                let selected = value
                    .chars()
                    .skip(selection_start)
                    .take(selection_end - selection_start)
                    .collect::<String>();
                let right = value.chars().skip(selection_end).collect::<String>();
                content_row = content_row
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
                    });
            } else if value.is_empty() {
                content_row = content_row.child("".to_string());
            } else {
                content_row = content_row.child(value.clone());
            }
            value_container = value_container.child(content_row);

            if !self.disabled && !self.read_only && show_caret && selection.is_none() {
                let caret_left = (Self::x_for_char(window, font_size, &value, current_caret)
                    - scroll_x)
                    .clamp(0.0, content_width.max(0.0));
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
                        .absolute()
                        .left(px(caret_left))
                        .top_0()
                        .bottom_0()
                        .flex()
                        .items_center()
                        .child(caret),
                );
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
                .child(self.render_input_box(window, _cx)),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap_3()
                .child(div().w(gpui::px(168.0)).child(self.render_label_block()))
                .child(div().flex_1().child(self.render_input_box(window, _cx))),
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

    pub fn size(mut self, value: Size) -> Self {
        self.inner = self.inner.size(value);
        self
    }

    pub fn radius(mut self, value: Radius) -> Self {
        self.inner = self.inner.radius(value);
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
        self.inner = Sized::with_size(self.inner, value);
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

    pub fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    pub fn radius(mut self, value: Radius) -> Self {
        self.radius = value;
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

    fn normalized_digits(value: &str, length: usize) -> String {
        value
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .take(length)
            .collect::<String>()
    }

    fn current_value_for(id: &str, rendered_value: &str, value_controlled: bool) -> String {
        let raw = control::text_state(
            id,
            "value",
            value_controlled.then_some(rendered_value.to_string()),
            rendered_value.to_string(),
        );
        Self::normalized_digits(&raw, usize::MAX)
    }

    fn editor_state_for(
        id: &str,
        rendered_value: &str,
        value_controlled: bool,
        length: usize,
    ) -> InputState {
        let current_value = Self::current_value_for(id, rendered_value, value_controlled);
        let len = current_value.chars().count().min(length);
        let caret = control::text_state(id, "caret-index", None, len.to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(len)
            .min(len);
        InputState::new(current_value, caret, caret, None)
    }

    fn apply_editor_state(
        id: &str,
        previous_value: &str,
        state: &InputState,
        value_controlled: bool,
        length: usize,
        on_change: Option<&ChangeHandler>,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        let mut next_state = state.clone();
        next_state.value = Self::normalized_digits(&next_state.value, length);
        next_state.clamp_to_max_length(Some(length));
        next_state.clear_selection();
        let next_len = next_state.value.chars().count();
        next_state.set_caret(next_len, false);

        let changed = next_state.value != previous_value;
        if changed && !value_controlled {
            control::set_text_state(id, "value", next_state.value.clone());
        }
        control::set_text_state(id, "caret-index", next_state.caret.to_string());
        window.refresh();

        if changed && let Some(handler) = on_change {
            (handler)(next_state.value.clone().into(), window, cx);
        }
    }

    fn digit_from_key(event: &gpui::KeyDownEvent) -> Option<char> {
        event
            .keystroke
            .key_char
            .as_ref()
            .and_then(|value| value.chars().next())
            .filter(|ch| ch.is_ascii_digit())
            .or_else(|| {
                let key = event.keystroke.key.as_str();
                if key.len() == 1 {
                    key.chars().next().filter(|ch| ch.is_ascii_digit())
                } else {
                    None
                }
            })
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
    fn render(mut self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(cx);
        ensure_text_keybindings(cx);

        let rendered_value = self.resolved_value().to_string();
        let normalized_value = Self::normalized_digits(&rendered_value, self.length);
        let value_chars = normalized_value.chars().collect::<Vec<_>>();
        let current_len = value_chars.len();
        let current_caret =
            control::text_state(&self.id, "caret-index", None, current_len.to_string())
                .parse::<usize>()
                .ok()
                .unwrap_or(current_len)
                .min(current_len);
        let active_index = current_caret.min(self.length.saturating_sub(1));
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

        let mut root = Stack::horizontal()
            .id(self.id.clone())
            .focusable()
            .key_context(INPUT_KEY_CONTEXT)
            .gap_2();

        if self.disabled {
            root = root.cursor_default();
        } else if self.read_only {
            root = root.cursor_default();
        } else {
            root = root.cursor_text();
        }

        if interactive {
            let input_id = self.id.clone();
            let rendered_value_for_edit = normalized_value.clone();
            let value_controlled = self.value_controlled;
            let on_change = self.on_change.clone();
            let length = self.length;
            root = root
                .on_action(move |_: &DeleteBackward, window, cx| {
                    let current_value = Self::current_value_for(
                        &input_id,
                        &rendered_value_for_edit,
                        value_controlled,
                    );
                    let mut state = Self::editor_state_for(
                        &input_id,
                        &rendered_value_for_edit,
                        value_controlled,
                        length,
                    );
                    if !state.delete_backward() {
                        return;
                    }
                    Self::apply_editor_state(
                        &input_id,
                        &current_value,
                        &state,
                        value_controlled,
                        length,
                        on_change.as_ref(),
                        window,
                        cx,
                    );
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value_for_edit = normalized_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &DeleteForward, window, cx| {
                        let current_value = Self::current_value_for(
                            &input_id,
                            &rendered_value_for_edit,
                            value_controlled,
                        );
                        let mut state = Self::editor_state_for(
                            &input_id,
                            &rendered_value_for_edit,
                            value_controlled,
                            length,
                        );
                        if !state.delete_forward() {
                            return;
                        }
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            length,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let input_id = self.id.clone();
                    let rendered_value_for_edit = normalized_value.clone();
                    let on_change = self.on_change.clone();
                    move |_: &PasteClipboard, window, cx| {
                        let Some(item) = cx.read_from_clipboard() else {
                            return;
                        };
                        let Some(pasted) = item.text() else {
                            return;
                        };
                        let digits = pasted
                            .chars()
                            .filter(|ch| ch.is_ascii_digit())
                            .collect::<String>();
                        if digits.is_empty() {
                            return;
                        }

                        let current_value = Self::current_value_for(
                            &input_id,
                            &rendered_value_for_edit,
                            value_controlled,
                        );
                        let mut state = Self::editor_state_for(
                            &input_id,
                            &rendered_value_for_edit,
                            value_controlled,
                            length,
                        );
                        state.insert_text(&digits);
                        state.clamp_to_max_length(Some(length));
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            length,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                    }
                })
                .on_key_down({
                    let input_id = self.id.clone();
                    let rendered_value_for_edit = normalized_value.clone();
                    let on_change = self.on_change.clone();
                    move |event, window, cx| {
                        if event.keystroke.modifiers.control
                            || event.keystroke.modifiers.platform
                            || event.keystroke.modifiers.function
                            || event.keystroke.modifiers.alt
                        {
                            return;
                        }

                        let Some(digit) = Self::digit_from_key(event) else {
                            return;
                        };
                        control::set_focused_state(&input_id, true);
                        let current_value = Self::current_value_for(
                            &input_id,
                            &rendered_value_for_edit,
                            value_controlled,
                        );
                        let mut state = Self::editor_state_for(
                            &input_id,
                            &rendered_value_for_edit,
                            value_controlled,
                            length,
                        );
                        state.insert_text(&digit.to_string());
                        state.clamp_to_max_length(Some(length));
                        Self::apply_editor_state(
                            &input_id,
                            &current_value,
                            &state,
                            value_controlled,
                            length,
                            on_change.as_ref(),
                            window,
                            cx,
                        );
                        cx.stop_propagation();
                    }
                });
        }

        if let Some(focus_handle) = &self.focus_handle
            && !self.disabled
        {
            let handle_for_click = focus_handle.clone();
            let focus_state_id = self.id.clone();
            let input_id = self.id.clone();
            let rendered_value_for_focus = normalized_value.clone();
            let value_controlled = self.value_controlled;
            let length = self.length;
            root = root
                .track_focus(focus_handle)
                .on_click(move |_, window, cx| {
                    control::set_focused_state(&focus_state_id, true);
                    let current_value = Self::current_value_for(
                        &input_id,
                        &rendered_value_for_focus,
                        value_controlled,
                    );
                    let caret = current_value.chars().count().min(length);
                    control::set_text_state(&input_id, "caret-index", caret.to_string());
                    window.focus(&handle_for_click, cx);
                    window.refresh();
                });
        } else if !self.disabled {
            let focus_state_id = self.id.clone();
            let input_id = self.id.clone();
            let rendered_value_for_focus = normalized_value.clone();
            let value_controlled = self.value_controlled;
            let length = self.length;
            root = root.on_click(move |_, window, _cx| {
                control::set_focused_state(&focus_state_id, true);
                let current_value =
                    Self::current_value_for(&input_id, &rendered_value_for_focus, value_controlled);
                let caret = current_value.chars().count().min(length);
                control::set_text_state(&input_id, "caret-index", caret.to_string());
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

            if interactive {
                let input_id = self.id.clone();
                let rendered_for_click = normalized_value.clone();
                let value_controlled = self.value_controlled;
                let length = self.length;
                cell =
                    cell.cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, window, _cx| {
                            control::set_focused_state(&input_id, true);
                            let current_value = Self::current_value_for(
                                &input_id,
                                &rendered_for_click,
                                value_controlled,
                            );
                            let caret = current_value.chars().count().min(length);
                            control::set_text_state(&input_id, "caret-index", caret.to_string());
                            window.refresh();
                        });
            }

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
