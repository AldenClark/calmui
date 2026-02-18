use std::{
    collections::HashMap,
    ops::Range,
    rc::Rc,
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};

use gpui::{
    Animation, AnimationExt, AnyElement, Bounds, ClipboardItem, FocusHandle, InputHandler,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, UTF16Selection, Window, canvas, div, point,
    px,
};

use crate::contracts::{FieldLike, MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::transition::TransitionExt;
use super::utils::{apply_input_size, apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

const CARET_BLINK_TOGGLE_MS: u64 = 680;
const CARET_BLINK_CYCLE_MS: u64 = CARET_BLINK_TOGGLE_MS * 2;

#[derive(Clone)]
struct WrappedLine {
    text: String,
    start_char: usize,
    end_char: usize,
}

static TEXTAREA_FOCUS_HANDLES: LazyLock<Mutex<HashMap<String, FocusHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
struct TextareaImeHandler {
    id: String,
    value_controlled: bool,
    rendered_value: String,
    max_length: Option<usize>,
    disabled: bool,
    read_only: bool,
    on_change: Option<ChangeHandler>,
    line_height: f32,
    vertical_padding: f32,
    horizontal_padding: f32,
    font_size: f32,
}

impl TextareaImeHandler {
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
        if let Some((start, end)) = Textarea::selection_bounds_for(&self.id, len) {
            return (start, end);
        }
        let caret = control::text_state(&self.id, "caret-index", None, len.to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(len)
            .min(len);
        (caret, caret)
    }

    fn normalized_text(text: &str) -> String {
        text.replace("\r\n", "\n").replace('\r', "\n")
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
        if changed && !self.value_controlled {
            control::set_text_state(&self.id, "value", next.clone());
        }
        control::set_text_state(&self.id, "caret-index", caret.to_string());
        if let Some((start, end)) = selection {
            Textarea::set_selection_for(&self.id, start, end);
        } else {
            Textarea::clear_selection_for(&self.id, caret);
        }
        control::set_text_state(&self.id, "selection-anchor", caret.to_string());
        self.set_marked_range_chars(marked);

        if changed && let Some(handler) = self.on_change.as_ref() {
            (handler)(next.into(), window, cx);
        }

        window.refresh();
    }
}

impl InputHandler for TextareaImeHandler {
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
        let range = if let Some((start, end)) = Textarea::selection_bounds_for(&self.id, len) {
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
        let sanitized = Self::normalized_text(text);
        let (next, caret) = Textarea::replace_char_range(&value, start, end, &sanitized);
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
        let sanitized = Self::normalized_text(new_text);
        let inserted_chars = sanitized.chars().count();
        let (next, fallback_caret) = Textarea::replace_char_range(&value, start, end, &sanitized);
        let next_len = next.chars().count();
        let marked = if inserted_chars > 0 {
            let mark_end = (start + inserted_chars).min(next_len);
            (start < mark_end).then_some((start, mark_end))
        } else {
            None
        };

        let selection = new_selected_range_utf16.and_then(|selection_utf16| {
            let relative = Self::char_range_from_utf16(&sanitized, selection_utf16);
            let selection_start = (start + relative.start).min(next_len);
            let selection_end = (start + relative.end).min(next_len);
            let (selection_start, selection_end) = if selection_start <= selection_end {
                (selection_start, selection_end)
            } else {
                (selection_end, selection_start)
            };
            (selection_start < selection_end).then_some((selection_start, selection_end))
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
        let content_width = Textarea::content_width_for_box(&self.id, self.horizontal_padding);
        let wrapped_lines =
            Textarea::wrapped_lines_for_width(&value, content_width, window, self.font_size);
        let (start_line, start_col) = Textarea::caret_visual_position(&wrapped_lines, range.start);
        let (end_line, end_col) = Textarea::caret_visual_position(&wrapped_lines, range.end);
        let (origin_x, origin_y, _width, _height) = Textarea::box_geometry(&self.id);
        let start_line_text = wrapped_lines
            .get(start_line)
            .map(|line| line.text.as_str())
            .unwrap_or_default();
        let end_line_text = wrapped_lines
            .get(end_line)
            .map(|line| line.text.as_str())
            .unwrap_or_default();
        let left = origin_x
            + self.horizontal_padding
            + Textarea::x_for_char(window, self.font_size, start_line_text, start_col);
        let top = origin_y + self.vertical_padding + start_line as f32 * self.line_height.max(1.0);
        let right = if start_line == end_line {
            origin_x
                + self.horizontal_padding
                + Textarea::x_for_char(window, self.font_size, end_line_text, end_col)
        } else {
            left + 1.0
        };
        let right = right.max(left + 1.0);
        let bottom = (top + self.line_height.max(1.0)).max(top + 1.0);
        Some(Bounds::from_corners(
            point(px(left), px(top)),
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
        let char_index = Textarea::caret_from_click(
            &self.id,
            point,
            &value,
            window,
            self.font_size,
            self.line_height,
            self.vertical_padding,
            self.horizontal_padding,
        );
        Some(Self::utf16_from_char(&value, char_index))
    }

    fn accepts_text_input(&mut self, _window: &mut Window, _cx: &mut gpui::App) -> bool {
        !self.disabled && !self.read_only
    }
}

#[derive(IntoElement)]
pub struct Textarea {
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
    min_rows: usize,
    max_rows: Option<usize>,
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

impl Textarea {
    fn is_enter_key(key: &str) -> bool {
        key == "enter" || key == "return" || key.ends_with("enter")
    }

    fn resolved_focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        if let Some(focus_handle) = self.focus_handle.as_ref() {
            return focus_handle.clone();
        }
        if let Ok(mut handles) = TEXTAREA_FOCUS_HANDLES.lock() {
            return handles
                .entry(self.id.to_string())
                .or_insert_with(|| cx.focus_handle())
                .clone();
        }
        cx.focus_handle()
    }

    fn is_text_entry_keystroke(event: &KeyDownEvent) -> bool {
        if event.keystroke.modifiers.control
            || event.keystroke.modifiers.platform
            || event.keystroke.modifiers.function
        {
            return false;
        }

        if Self::is_enter_key(event.keystroke.key.as_str()) {
            return false;
        }

        if let Some(key_char) = event.keystroke.key_char.as_ref() {
            return !key_char.is_empty()
                && !key_char
                    .chars()
                    .any(|ch| ch.is_control() && ch != '\n' && ch != '\t');
        }

        event.keystroke.key.chars().count() == 1
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
            min_rows: 3,
            max_rows: Some(8),
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

    pub fn min_rows(mut self, rows: usize) -> Self {
        self.min_rows = rows.max(1);
        self
    }

    pub fn max_rows(mut self, rows: usize) -> Self {
        self.max_rows = Some(rows.max(self.min_rows));
        self
    }

    pub fn unlimited_rows(mut self) -> Self {
        self.max_rows = None;
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
            let (line, _column) = Self::line_col_from_char(current, caret_index);
            return Some((
                current.to_string(),
                Self::char_from_line_col(current, line, 0),
            ));
        }
        if key == "end" {
            let (line, _column) = Self::line_col_from_char(current, caret_index);
            let line_len = current
                .split('\n')
                .nth(line)
                .map(|segment| segment.chars().count())
                .unwrap_or(0);
            return Some((
                current.to_string(),
                Self::char_from_line_col(current, line, line_len),
            ));
        }
        if key == "up" || key == "down" {
            let (line, column) = Self::line_col_from_char(current, caret_index);
            let line_count = current.chars().filter(|ch| *ch == '\n').count() + 1;
            let target_line = if key == "up" {
                line.saturating_sub(1)
            } else {
                (line + 1).min(line_count.saturating_sub(1))
            };
            return Some((
                current.to_string(),
                Self::char_from_line_col(current, target_line, column),
            ));
        }

        let has_modifier = event.keystroke.modifiers.control
            || event.keystroke.modifiers.platform
            || event.keystroke.modifiers.function;
        if has_modifier {
            return None;
        }

        let inserted = if Self::is_enter_key(key) {
            Some("\n".to_string())
        } else {
            event
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
                })
        }?;
        let inserted = inserted.replace('\r', "\n");
        if inserted.is_empty() {
            return None;
        }

        if inserted.chars().count() > 1 && inserted.contains('\u{7f}') {
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

    fn byte_index_at_char(value: &str, char_index: usize) -> usize {
        value
            .char_indices()
            .nth(char_index)
            .map(|(index, _)| index)
            .unwrap_or(value.len())
    }

    fn line_col_from_char(value: &str, char_index: usize) -> (usize, usize) {
        let mut line = 0usize;
        let mut col = 0usize;
        for (index, ch) in value.chars().enumerate() {
            if index >= char_index {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn char_from_line_col(value: &str, line: usize, col: usize) -> usize {
        let mut current_line = 0usize;
        let mut current_col = 0usize;
        let mut index = 0usize;
        for ch in value.chars() {
            if current_line == line && current_col == col {
                return index;
            }
            if ch == '\n' {
                if current_line == line {
                    return index;
                }
                current_line += 1;
                current_col = 0;
            } else if current_line == line {
                current_col += 1;
            }
            index += 1;
        }
        index
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

    fn replace_char_range(value: &str, start: usize, end: usize, insert: &str) -> (String, usize) {
        let start = start.min(value.chars().count());
        let end = end.min(value.chars().count()).max(start);
        let byte_start = Self::byte_index_at_char(value, start);
        let byte_end = Self::byte_index_at_char(value, end);
        let mut next = value.to_string();
        next.replace_range(byte_start..byte_end, insert);
        (next, start + insert.chars().count())
    }

    fn selected_text(value: &str, start: usize, end: usize) -> String {
        value
            .chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect()
    }

    fn box_geometry(id: &str) -> (f32, f32, f32, f32) {
        let x = control::text_state(id, "box-origin-x", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let y = control::text_state(id, "box-origin-y", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let width = control::text_state(id, "box-width", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let height = control::text_state(id, "box-height", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        (x, y, width, height)
    }

    fn context_menu_position(id: &str) -> (f32, f32) {
        let x = control::text_state(id, "context-menu-x", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        let y = control::text_state(id, "context-menu-y", None, "0".to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(0.0);
        (x, y)
    }

    fn caret_from_click(
        id: &str,
        position: gpui::Point<gpui::Pixels>,
        value: &str,
        window: &Window,
        font_size: f32,
        line_height: f32,
        vertical_padding: f32,
        horizontal_padding: f32,
    ) -> usize {
        let (origin_x, origin_y, _width, _height) = Self::box_geometry(id);
        let local_x = (f32::from(position.x) - origin_x - horizontal_padding).max(0.0);
        let local_y = (f32::from(position.y) - origin_y - vertical_padding).max(0.0);
        let content_width = Self::content_width_for_box(id, horizontal_padding);
        let wrapped_lines = Self::wrapped_lines_for_width(value, content_width, window, font_size);

        let target_line = (local_y / line_height.max(1.0)).floor() as usize;
        if wrapped_lines.is_empty() {
            return 0;
        }
        let line_index = target_line.min(wrapped_lines.len().saturating_sub(1));
        let line = &wrapped_lines[line_index];
        let local_char = Self::char_from_x(window, font_size, &line.text, local_x);
        line.start_char + local_char.min(line.end_char.saturating_sub(line.start_char))
    }

    fn line_height_px(&self) -> f32 {
        match self.size {
            Size::Xs => 14.0,
            Size::Sm => 16.0,
            Size::Md => 18.0,
            Size::Lg => 20.0,
            Size::Xl => 22.0,
        }
    }

    fn vertical_padding_px(&self) -> f32 {
        match self.size {
            Size::Xs => 5.0,
            Size::Sm => 6.0,
            Size::Md => 8.0,
            Size::Lg => 10.0,
            Size::Xl => 12.0,
        }
    }

    fn horizontal_padding_px(&self) -> f32 {
        match self.size {
            Size::Xs => 8.0,
            Size::Sm => 10.0,
            Size::Md => 12.0,
            Size::Lg => 14.0,
            Size::Xl => 16.0,
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

    fn content_width_for_box(id: &str, horizontal_padding: f32) -> f32 {
        let (_, _, width, _) = Self::box_geometry(id);
        if width <= 0.0 {
            return 240.0;
        }
        (width - horizontal_padding * 2.0).max(1.0)
    }

    fn wrap_chars_for_width(window: &Window, font_size: f32, text: &str, width: f32) -> usize {
        let total_chars = text.chars().count();
        if total_chars <= 1 {
            return total_chars;
        }
        let layout = Self::line_layout(window, font_size, text);
        if f32::from(layout.width) <= width.max(1.0) {
            return total_chars;
        }
        let byte_index = layout.index_for_x(px(width.max(0.0))).unwrap_or(text.len());
        let wrapped = Self::char_index_at_byte(text, byte_index);
        wrapped.clamp(1, total_chars)
    }

    fn wrapped_lines_for_width(
        value: &str,
        content_width: f32,
        window: &Window,
        font_size: f32,
    ) -> Vec<WrappedLine> {
        if value.is_empty() {
            return vec![WrappedLine {
                text: String::new(),
                start_char: 0,
                end_char: 0,
            }];
        }

        let mut wrapped = Vec::new();
        let mut global_char_index = 0usize;
        let physical_lines: Vec<&str> = value.split('\n').collect();

        for (line_index, line) in physical_lines.iter().enumerate() {
            let line_len = line.chars().count();

            if line_len == 0 {
                wrapped.push(WrappedLine {
                    text: String::new(),
                    start_char: global_char_index,
                    end_char: global_char_index,
                });
            } else {
                let mut local_start_char = 0usize;
                let mut local_start_byte = 0usize;
                while local_start_char < line_len {
                    let remaining = &line[local_start_byte..];
                    let take_chars =
                        Self::wrap_chars_for_width(window, font_size, remaining, content_width);
                    let local_end = (local_start_char + take_chars).min(line_len);
                    let local_end_byte =
                        local_start_byte + Self::byte_index_at_char(remaining, take_chars);
                    wrapped.push(WrappedLine {
                        text: line[local_start_byte..local_end_byte].to_string(),
                        start_char: global_char_index + local_start_char,
                        end_char: global_char_index + local_end,
                    });
                    local_start_char = local_end;
                    local_start_byte = local_end_byte;
                }
            }

            global_char_index += line_len;
            if line_index + 1 < physical_lines.len() {
                global_char_index += 1;
            }
        }

        wrapped
    }

    fn caret_visual_position(wrapped_lines: &[WrappedLine], caret_index: usize) -> (usize, usize) {
        if wrapped_lines.is_empty() {
            return (0, 0);
        }

        for (line_index, line) in wrapped_lines.iter().enumerate() {
            if caret_index >= line.start_char && caret_index <= line.end_char {
                return (line_index, caret_index - line.start_char);
            }
        }

        let last_index = wrapped_lines.len().saturating_sub(1);
        let last = &wrapped_lines[last_index];
        (last_index, last.end_char.saturating_sub(last.start_char))
    }

    fn resolved_rows(&self, visual_lines: usize) -> (usize, bool) {
        let visual_lines = visual_lines.max(1);
        let max_rows = self.max_rows.unwrap_or(visual_lines.max(self.min_rows));
        let rows = visual_lines.clamp(self.min_rows, max_rows);
        (rows, visual_lines > rows)
    }

    fn render_label_block(&self) -> AnyElement {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return div().into_any_element();
        }

        let tokens = &self.theme.components.textarea;
        let mut block = Stack::vertical().gap_1();

        if let Some(label) = &self.label {
            let mut label_row = Stack::horizontal().gap_1().child(
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

    fn render_input_box(&mut self, window: &mut Window, cx: &mut gpui::App) -> AnyElement {
        let tokens = &self.theme.components.textarea;
        let resolved_value = self.resolved_value();
        let current_value = resolved_value.to_string();
        let focus_handle = self.resolved_focus_handle(cx);
        let tracked_focus = control::focused_state(&self.id, None, false);
        let handle_focused = focus_handle.is_focused(window);
        let is_focused = handle_focused || tracked_focus;
        let current_caret = control::text_state(
            &self.id,
            "caret-index",
            None,
            current_value.chars().count().to_string(),
        )
        .parse::<usize>()
        .ok()
        .map(|value| value.min(current_value.chars().count()))
        .unwrap_or_else(|| current_value.chars().count());
        let selection = Self::selection_bounds_for(&self.id, current_value.chars().count());

        let line_height = self.line_height_px();
        let vertical_padding = self.vertical_padding_px();
        let horizontal_padding = self.horizontal_padding_px();
        let font_size = self.font_size_px();
        let content_width = Self::content_width_for_box(&self.id, horizontal_padding);
        let wrapped_lines =
            Self::wrapped_lines_for_width(&current_value, content_width, window, font_size);
        let (rows, should_scroll) = self.resolved_rows(wrapped_lines.len());
        let box_height = (rows as f32 * line_height) + (vertical_padding * 2.0) + 2.0;

        let mut input = div()
            .id(self.id.slot("box"))
            .relative()
            .focusable()
            .flex()
            .flex_col()
            .items_start()
            .justify_start()
            .gap_0()
            .w_full()
            .h(px(box_height))
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .text_color(resolve_hsla(&self.theme, &tokens.fg))
            .border(super::utils::quantized_stroke_px(window, 1.0));

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

        if should_scroll {
            input = input.overflow_y_scroll();
        }

        if self.disabled {
            input = input.cursor_default().opacity(0.55);
        } else {
            input = input.cursor_text();
        }
        input = input.line_height(px(line_height));

        input = input.child({
            let id_for_metrics = self.id.clone();
            canvas(
                move |bounds, _, _cx| {
                    control::set_text_state(
                        &id_for_metrics,
                        "box-origin-x",
                        f32::from(bounds.origin.x).to_string(),
                    );
                    control::set_text_state(
                        &id_for_metrics,
                        "box-origin-y",
                        f32::from(bounds.origin.y).to_string(),
                    );
                    control::set_text_state(
                        &id_for_metrics,
                        "box-width",
                        f32::from(bounds.size.width).to_string(),
                    );
                    control::set_text_state(
                        &id_for_metrics,
                        "box-height",
                        f32::from(bounds.size.height).to_string(),
                    );
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        });

        let handle_for_click = focus_handle.clone();
        let handle_for_right_click = focus_handle.clone();
        let handle_for_right_up = focus_handle.clone();
        let id_for_focus = self.id.clone();
        let id_for_right_click = self.id.clone();
        let id_for_right_up = self.id.clone();
        let id_for_mouse_move = self.id.clone();
        let id_for_mouse_up = self.id.clone();
        let id_for_mouse_up_out = self.id.clone();
        let value_for_click = current_value.clone();
        let value_for_right_click = current_value.clone();
        let value_for_right_up = current_value.clone();
        let value_for_mouse_move = current_value.clone();
        let line_height_for_click = line_height;
        let line_height_for_right_click = line_height;
        let line_height_for_right_up = line_height;
        let line_height_for_mouse_move = line_height;
        let vertical_padding_for_click = vertical_padding;
        let vertical_padding_for_right_click = vertical_padding;
        let vertical_padding_for_right_up = vertical_padding;
        let vertical_padding_for_mouse_move = vertical_padding;
        let horizontal_padding_for_click = horizontal_padding;
        let horizontal_padding_for_right_click = horizontal_padding;
        let horizontal_padding_for_right_up = horizontal_padding;
        let horizontal_padding_for_mouse_move = horizontal_padding;
        let font_size_for_click = font_size;
        let font_size_for_right_click = font_size;
        let font_size_for_right_up = font_size;
        let font_size_for_mouse_move = font_size;
        let value_controlled_for_mouse = self.value_controlled;
        input = input.track_focus(&focus_handle).on_mouse_down(
            MouseButton::Left,
            move |event, window, cx| {
                control::set_focused_state(&id_for_focus, true);
                control::set_bool_state(&id_for_focus, "context-open", false);
                let current_value_for_click = control::text_state(
                    &id_for_focus,
                    "value",
                    value_controlled_for_mouse.then_some(value_for_click.clone()),
                    value_for_click.clone(),
                );
                let click_caret = Self::caret_from_click(
                    &id_for_focus,
                    event.position,
                    &current_value_for_click,
                    window,
                    font_size_for_click,
                    line_height_for_click,
                    vertical_padding_for_click,
                    horizontal_padding_for_click,
                );
                let len = current_value_for_click.chars().count();
                let current_caret =
                    control::text_state(&id_for_focus, "caret-index", None, len.to_string())
                        .parse::<usize>()
                        .ok()
                        .map(|value| value.min(len))
                        .unwrap_or(len);
                let existing_selection = Self::selection_bounds_for(&id_for_focus, len);
                control::set_text_state(&id_for_focus, "caret-index", click_caret.to_string());
                if event.modifiers.shift {
                    let anchor = if let Some((start, end)) = existing_selection {
                        if current_caret == start { end } else { start }
                    } else {
                        current_caret
                    };
                    Self::set_selection_for(&id_for_focus, anchor, click_caret);
                    control::set_text_state(&id_for_focus, "selection-anchor", anchor.to_string());
                } else {
                    Self::clear_selection_for(&id_for_focus, click_caret);
                    control::set_text_state(
                        &id_for_focus,
                        "selection-anchor",
                        click_caret.to_string(),
                    );
                }
                control::set_bool_state(&id_for_focus, "mouse-selecting", true);
                window.focus(&handle_for_click, cx);
                window.refresh();
            },
        );
        input = input.on_mouse_move(move |event, window, _cx| {
            if !control::bool_state(&id_for_mouse_move, "mouse-selecting", None, false) {
                return;
            }
            let current_value_for_drag = control::text_state(
                &id_for_mouse_move,
                "value",
                value_controlled_for_mouse.then_some(value_for_mouse_move.clone()),
                value_for_mouse_move.clone(),
            );
            let caret = Self::caret_from_click(
                &id_for_mouse_move,
                event.position,
                &current_value_for_drag,
                window,
                font_size_for_mouse_move,
                line_height_for_mouse_move,
                vertical_padding_for_mouse_move,
                horizontal_padding_for_mouse_move,
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
        });
        input = input
            .on_mouse_up(MouseButton::Left, move |_, _, _| {
                control::set_bool_state(&id_for_mouse_up, "mouse-selecting", false);
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, _| {
                control::set_bool_state(&id_for_mouse_up_out, "mouse-selecting", false);
            });
        input = input.on_mouse_down(MouseButton::Right, move |event, window, cx| {
            control::set_focused_state(&id_for_right_click, true);
            window.focus(&handle_for_right_click, cx);

            let current_value_for_click = control::text_state(
                &id_for_right_click,
                "value",
                value_controlled_for_mouse.then_some(value_for_right_click.clone()),
                value_for_right_click.clone(),
            );
            let click_caret = Self::caret_from_click(
                &id_for_right_click,
                event.position,
                &current_value_for_click,
                window,
                font_size_for_right_click,
                line_height_for_right_click,
                vertical_padding_for_right_click,
                horizontal_padding_for_right_click,
            );
            let len = current_value_for_click.chars().count();
            let existing_selection = Self::selection_bounds_for(&id_for_right_click, len);
            let keep_selection = existing_selection
                .is_some_and(|(start, end)| click_caret >= start && click_caret <= end);
            control::set_text_state(&id_for_right_click, "caret-index", click_caret.to_string());
            if !keep_selection {
                Self::clear_selection_for(&id_for_right_click, click_caret);
                control::set_text_state(
                    &id_for_right_click,
                    "selection-anchor",
                    click_caret.to_string(),
                );
            }

            let (origin_x, origin_y, _width, _height) = Self::box_geometry(&id_for_right_click);
            let local_x = (f32::from(event.position.x) - origin_x).max(0.0);
            let local_y = (f32::from(event.position.y) - origin_y).max(0.0);
            control::set_text_state(&id_for_right_click, "context-menu-x", local_x.to_string());
            control::set_text_state(&id_for_right_click, "context-menu-y", local_y.to_string());
            control::set_bool_state(&id_for_right_click, "context-open", true);
            control::set_bool_state(&id_for_right_click, "mouse-selecting", false);
            window.refresh();
        });
        input = input.on_mouse_up(MouseButton::Right, move |event, window, cx| {
            control::set_focused_state(&id_for_right_up, true);
            window.focus(&handle_for_right_up, cx);

            let current_value_for_click = control::text_state(
                &id_for_right_up,
                "value",
                value_controlled_for_mouse.then_some(value_for_right_up.clone()),
                value_for_right_up.clone(),
            );
            let click_caret = Self::caret_from_click(
                &id_for_right_up,
                event.position,
                &current_value_for_click,
                window,
                font_size_for_right_up,
                line_height_for_right_up,
                vertical_padding_for_right_up,
                horizontal_padding_for_right_up,
            );
            let len = current_value_for_click.chars().count();
            let existing_selection = Self::selection_bounds_for(&id_for_right_up, len);
            let keep_selection = existing_selection
                .is_some_and(|(start, end)| click_caret >= start && click_caret <= end);
            control::set_text_state(&id_for_right_up, "caret-index", click_caret.to_string());
            if !keep_selection {
                Self::clear_selection_for(&id_for_right_up, click_caret);
                control::set_text_state(
                    &id_for_right_up,
                    "selection-anchor",
                    click_caret.to_string(),
                );
            }

            let (origin_x, origin_y, _width, _height) = Self::box_geometry(&id_for_right_up);
            let local_x = (f32::from(event.position.x) - origin_x).max(0.0);
            let local_y = (f32::from(event.position.y) - origin_y).max(0.0);
            control::set_text_state(&id_for_right_up, "context-menu-x", local_x.to_string());
            control::set_text_state(&id_for_right_up, "context-menu-y", local_y.to_string());
            control::set_bool_state(&id_for_right_up, "context-open", true);
            control::set_bool_state(&id_for_right_up, "mouse-selecting", false);
            window.refresh();
        });

        let id_for_blur = self.id.clone();
        input = input.on_mouse_down_out(move |_, window, _cx| {
            control::set_focused_state(&id_for_blur, false);
            control::set_bool_state(&id_for_blur, "context-open", false);
            window.refresh();
        });

        if !self.disabled && !self.read_only {
            let on_change = self.on_change.clone();
            let value_controlled = self.value_controlled;
            let input_id = self.id.clone();
            let max_length = self.max_length;
            let rendered_value_for_input = current_value.clone();
            let font_size_for_input = font_size;
            let line_height_for_input = line_height;
            let vertical_padding_for_input = vertical_padding;
            let horizontal_padding_for_input = horizontal_padding;
            input = input.on_key_down(move |event, window, cx| {
                control::set_focused_state(&input_id, true);
                let current_value_for_input = control::text_state(
                    &input_id,
                    "value",
                    value_controlled.then_some(rendered_value_for_input.clone()),
                    rendered_value_for_input.clone(),
                );
                let len = current_value_for_input.chars().count();
                let current_caret_for_input =
                    control::text_state(&input_id, "caret-index", None, len.to_string())
                        .parse::<usize>()
                        .ok()
                        .map(|value| value.min(len))
                        .unwrap_or(len);
                let selection = Self::selection_bounds_for(&input_id, len);
                let modifiers =
                    event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

                let open_context_menu = event.keystroke.key == "menu"
                    || (event.keystroke.modifiers.shift && event.keystroke.key == "f10");
                if open_context_menu {
                    let content_width =
                        Self::content_width_for_box(&input_id, horizontal_padding_for_input);
                    let wrapped_lines = Self::wrapped_lines_for_width(
                        &current_value_for_input,
                        content_width,
                        window,
                        font_size_for_input,
                    );
                    let (caret_line, caret_col) =
                        Self::caret_visual_position(&wrapped_lines, current_caret_for_input);
                    let caret_x = wrapped_lines
                        .get(caret_line)
                        .map(|line| {
                            Self::x_for_char(window, font_size_for_input, &line.text, caret_col)
                        })
                        .unwrap_or(0.0);
                    let local_x = (horizontal_padding_for_input + caret_x).max(0.0);
                    let local_y = (vertical_padding_for_input
                        + (caret_line as f32 + 1.0) * line_height_for_input)
                        .max(0.0);
                    control::set_text_state(&input_id, "context-menu-x", local_x.to_string());
                    control::set_text_state(&input_id, "context-menu-y", local_y.to_string());
                    control::set_bool_state(&input_id, "context-open", true);
                    window.refresh();
                    return;
                }

                if event.keystroke.modifiers.shift
                    && matches!(
                        event.keystroke.key.as_str(),
                        "left" | "right" | "home" | "end" | "up" | "down"
                    )
                {
                    let anchor = if let Some((start, end)) = selection {
                        if current_caret_for_input == start {
                            end
                        } else {
                            start
                        }
                    } else {
                        current_caret_for_input
                    };
                    if let Some((_next, next_caret)) = Self::with_value_update(
                        &current_value_for_input,
                        event,
                        max_length,
                        current_caret_for_input,
                        None,
                    ) {
                        control::set_text_state(&input_id, "caret-index", next_caret.to_string());
                        Self::set_selection_for(&input_id, anchor, next_caret);
                        window.refresh();
                    }
                    return;
                }

                if modifiers && event.keystroke.key == "a" {
                    control::set_text_state(&input_id, "caret-index", len.to_string());
                    Self::set_selection_for(&input_id, 0, len);
                    window.refresh();
                    return;
                }

                if modifiers && event.keystroke.key == "c" {
                    if let Some((start, end)) = selection {
                        let selected = Self::selected_text(&current_value_for_input, start, end);
                        if !selected.is_empty() {
                            cx.write_to_clipboard(ClipboardItem::new_string(selected));
                        }
                    }
                    return;
                }

                if modifiers && event.keystroke.key == "x" {
                    if let Some((start, end)) = selection {
                        let selected = Self::selected_text(&current_value_for_input, start, end);
                        if !selected.is_empty() {
                            cx.write_to_clipboard(ClipboardItem::new_string(selected));
                            let (next, next_caret) =
                                Self::replace_char_range(&current_value_for_input, start, end, "");
                            control::set_text_state(
                                &input_id,
                                "caret-index",
                                next_caret.to_string(),
                            );
                            Self::clear_selection_for(&input_id, next_caret);
                            if !value_controlled {
                                control::set_text_state(&input_id, "value", next.clone());
                                window.refresh();
                            } else {
                                window.refresh();
                            }
                            if let Some(handler) = on_change.as_ref() {
                                (handler)(next.into(), window, cx);
                            }
                        }
                    }
                    return;
                }

                if (event.keystroke.modifiers.control || event.keystroke.modifiers.platform)
                    && event.keystroke.key == "v"
                    && let Some(item) = cx.read_from_clipboard()
                    && let Some(pasted) = item.text()
                {
                    let (mut next, mut next_caret) = if let Some((start, end)) = selection {
                        Self::replace_char_range(&current_value_for_input, start, end, &pasted)
                    } else {
                        let start = Self::byte_index_at_char(
                            &current_value_for_input,
                            current_caret_for_input,
                        );
                        let mut next = current_value_for_input.clone();
                        next.insert_str(start, &pasted);
                        (
                            next,
                            (current_caret_for_input + pasted.chars().count()).min(
                                current_value_for_input.chars().count() + pasted.chars().count(),
                            ),
                        )
                    };
                    if let Some(limit) = max_length
                        && next.chars().count() > limit
                    {
                        next = next.chars().take(limit).collect();
                        next_caret = next_caret.min(next.chars().count());
                    }
                    if !value_controlled {
                        control::set_text_state(&input_id, "value", next.clone());
                        control::set_text_state(&input_id, "caret-index", next_caret.to_string());
                        Self::clear_selection_for(&input_id, next_caret);
                        window.refresh();
                    } else {
                        control::set_text_state(&input_id, "caret-index", next_caret.to_string());
                        Self::clear_selection_for(&input_id, next_caret);
                        window.refresh();
                    }
                    if let Some(handler) = on_change.as_ref() {
                        (handler)(next.into(), window, cx);
                    }
                    return;
                }

                if Self::is_text_entry_keystroke(event) {
                    return;
                }

                if let Some((next, next_caret)) = Self::with_value_update(
                    &current_value_for_input,
                    event,
                    max_length,
                    current_caret_for_input,
                    selection,
                ) {
                    control::set_text_state(&input_id, "caret-index", next_caret.to_string());
                    Self::clear_selection_for(&input_id, next_caret);
                    if !value_controlled && next != current_value_for_input {
                        control::set_text_state(&input_id, "value", next.clone());
                        window.refresh();
                    } else if value_controlled {
                        window.refresh();
                    }
                    if next != current_value_for_input
                        && let Some(handler) = on_change.as_ref()
                    {
                        (handler)(next.into(), window, cx);
                    }
                }
            });
        }

        window.handle_input(
            &focus_handle,
            TextareaImeHandler {
                id: self.id.to_string(),
                value_controlled: self.value_controlled,
                rendered_value: current_value.clone(),
                max_length: self.max_length,
                disabled: self.disabled,
                read_only: self.read_only,
                on_change: self.on_change.clone(),
                line_height,
                vertical_padding,
                horizontal_padding,
                font_size,
            },
            cx,
        );

        if current_value.is_empty() && !is_focused {
            input = input.child(
                div()
                    .w_full()
                    .text_color(resolve_hsla(&self.theme, &tokens.placeholder))
                    .child(self.placeholder.clone().unwrap_or_default()),
            );
        } else {
            let mut content = Stack::vertical().w_full().gap_0();
            let (caret_line, caret_col) =
                Self::caret_visual_position(&wrapped_lines, current_caret);
            let selection_bg =
                resolve_hsla(&self.theme, &self.theme.semantic.focus_ring).alpha(0.28);
            for line in &wrapped_lines {
                if let Some((selection_start, selection_end)) = selection {
                    let seg_start = selection_start.clamp(line.start_char, line.end_char);
                    let seg_end = selection_end.clamp(line.start_char, line.end_char);
                    if seg_start < seg_end {
                        let local_start = seg_start - line.start_char;
                        let local_end = seg_end - line.start_char;
                        let left = line.text.chars().take(local_start).collect::<String>();
                        let selected = line
                            .text
                            .chars()
                            .skip(local_start)
                            .take(local_end - local_start)
                            .collect::<String>();
                        let right = line.text.chars().skip(local_end).collect::<String>();
                        content = content.child(
                            div()
                                .w_full()
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
                        continue;
                    }
                }

                if line.text.is_empty() {
                    content = content.child(div().w_full().child(" "));
                } else {
                    content =
                        content.child(div().w_full().whitespace_nowrap().child(line.text.clone()));
                }
            }

            let mut content_host = div().relative().w_full().child(content);
            if !self.disabled && !self.read_only && is_focused && selection.is_none() {
                let caret_left = wrapped_lines
                    .get(caret_line)
                    .map(|line| Self::x_for_char(window, font_size, &line.text, caret_col))
                    .unwrap_or(0.0);
                let caret_top = caret_line as f32 * line_height;
                let caret = div()
                    .id(self.id.slot("caret"))
                    .flex_none()
                    .w(super::utils::quantized_stroke_px(window, 1.5))
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
                content_host = content_host.child(
                    div()
                        .absolute()
                        .left(px(caret_left.max(0.0)))
                        .top(px(caret_top.max(0.0)))
                        .child(caret),
                );
            }

            input = input.child(content_host);
        }

        if !self.disabled && control::bool_state(&self.id, "context-open", None, false) {
            let (menu_x, menu_y) = Self::context_menu_position(&self.id);
            let textarea_id = self.id.clone();
            let rendered_value = current_value.clone();
            let value_controlled = self.value_controlled;
            let max_length = self.max_length;
            let on_change = self.on_change.clone();
            let read_only = self.read_only;

            let item_style = || {
                div()
                    .w_full()
                    .px(px(10.0))
                    .py(px(7.0))
                    .text_sm()
                    .cursor_pointer()
                    .hover(|style| style.bg(gpui::black().opacity(0.08)))
            };

            let copy_id = textarea_id.clone();
            let copy_value = rendered_value.clone();
            let copy_action = item_style().child("Copy").on_mouse_down(
                MouseButton::Left,
                move |_, window: &mut Window, cx: &mut gpui::App| {
                    let current = control::text_state(
                        &copy_id,
                        "value",
                        value_controlled.then_some(copy_value.clone()),
                        copy_value.clone(),
                    );
                    let len = current.chars().count();
                    if let Some((start, end)) = Self::selection_bounds_for(&copy_id, len) {
                        let selected = Self::selected_text(&current, start, end);
                        if !selected.is_empty() {
                            cx.write_to_clipboard(ClipboardItem::new_string(selected));
                        }
                    }
                    control::set_bool_state(&copy_id, "context-open", false);
                    window.refresh();
                },
            );

            let cut_id = textarea_id.clone();
            let cut_value = rendered_value.clone();
            let cut_on_change = on_change.clone();
            let cut_action = if !read_only {
                Some(item_style().child("Cut").on_mouse_down(
                    MouseButton::Left,
                    move |_, window: &mut Window, cx: &mut gpui::App| {
                        let current = control::text_state(
                            &cut_id,
                            "value",
                            value_controlled.then_some(cut_value.clone()),
                            cut_value.clone(),
                        );
                        let len = current.chars().count();
                        if let Some((start, end)) = Self::selection_bounds_for(&cut_id, len) {
                            let selected = Self::selected_text(&current, start, end);
                            if !selected.is_empty() {
                                cx.write_to_clipboard(ClipboardItem::new_string(selected));
                                let (next, next_caret) =
                                    Self::replace_char_range(&current, start, end, "");
                                if !value_controlled {
                                    control::set_text_state(&cut_id, "value", next.clone());
                                }
                                control::set_text_state(
                                    &cut_id,
                                    "caret-index",
                                    next_caret.to_string(),
                                );
                                Self::clear_selection_for(&cut_id, next_caret);
                                if let Some(handler) = cut_on_change.as_ref() {
                                    (handler)(next.into(), window, cx);
                                }
                            }
                        }
                        control::set_bool_state(&cut_id, "context-open", false);
                        window.refresh();
                    },
                ))
            } else {
                None
            };

            let paste_id = textarea_id.clone();
            let paste_value = rendered_value.clone();
            let paste_on_change = on_change.clone();
            let paste_action = if !read_only {
                Some(item_style().child("Paste").on_mouse_down(
                    MouseButton::Left,
                    move |_, window: &mut Window, cx: &mut gpui::App| {
                        let Some(item) = cx.read_from_clipboard() else {
                            control::set_bool_state(&paste_id, "context-open", false);
                            window.refresh();
                            return;
                        };
                        let Some(pasted) = item.text() else {
                            control::set_bool_state(&paste_id, "context-open", false);
                            window.refresh();
                            return;
                        };

                        let normalized = pasted.replace("\r\n", "\n").replace('\r', "\n");
                        let current = control::text_state(
                            &paste_id,
                            "value",
                            value_controlled.then_some(paste_value.clone()),
                            paste_value.clone(),
                        );
                        let len = current.chars().count();
                        let caret =
                            control::text_state(&paste_id, "caret-index", None, len.to_string())
                                .parse::<usize>()
                                .ok()
                                .unwrap_or(len)
                                .min(len);
                        let selection = Self::selection_bounds_for(&paste_id, len);
                        let (mut next, mut next_caret) = if let Some((start, end)) = selection {
                            Self::replace_char_range(&current, start, end, &normalized)
                        } else {
                            let start = Self::byte_index_at_char(&current, caret);
                            let mut next = current.clone();
                            next.insert_str(start, &normalized);
                            (next, caret + normalized.chars().count())
                        };

                        if let Some(limit) = max_length
                            && next.chars().count() > limit
                        {
                            next = next.chars().take(limit).collect();
                            next_caret = next_caret.min(next.chars().count());
                        }
                        if !value_controlled {
                            control::set_text_state(&paste_id, "value", next.clone());
                        }
                        control::set_text_state(&paste_id, "caret-index", next_caret.to_string());
                        Self::clear_selection_for(&paste_id, next_caret);
                        if let Some(handler) = paste_on_change.as_ref() {
                            (handler)(next.into(), window, cx);
                        }
                        control::set_bool_state(&paste_id, "context-open", false);
                        window.refresh();
                    },
                ))
            } else {
                None
            };

            let select_all_id = textarea_id.clone();
            let select_all_value = rendered_value.clone();
            let select_all = item_style().child("Select All").on_mouse_down(
                MouseButton::Left,
                move |_, window: &mut Window, _cx: &mut gpui::App| {
                    let current = control::text_state(
                        &select_all_id,
                        "value",
                        value_controlled.then_some(select_all_value.clone()),
                        select_all_value.clone(),
                    );
                    let len = current.chars().count();
                    control::set_text_state(&select_all_id, "caret-index", len.to_string());
                    Self::set_selection_for(&select_all_id, 0, len);
                    control::set_bool_state(&select_all_id, "context-open", false);
                    window.refresh();
                },
            );

            let mut menu = div()
                .id(self.id.slot("context-menu"))
                .absolute()
                .left(px(menu_x))
                .top(px(menu_y))
                .min_w(px(132.0))
                .rounded_md()
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(&self.theme, &tokens.border))
                .bg(resolve_hsla(&self.theme, &tokens.bg))
                .shadow_sm()
                .flex()
                .flex_col()
                .py(px(4.0));
            menu = menu.child(copy_action);
            if let Some(cut_action) = cut_action {
                menu = menu.child(cut_action);
            }
            if let Some(paste_action) = paste_action {
                menu = menu.child(paste_action);
            }
            menu = menu.child(select_all);
            input = input.child(menu);
        }

        input
            .with_enter_transition(self.id.slot("enter"), self.motion)
            .into_any_element()
    }
}

impl Textarea {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl FieldLike for Textarea {
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

impl VariantConfigurable for Textarea {
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

impl MotionAware for Textarea {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Textarea {
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
                .child(div().w(px(168.0)).child(self.render_label_block()))
                .child(div().flex_1().child(self.render_input_box(window, _cx))),
        }
    }
}

impl crate::contracts::ComponentThemeOverridable for Textarea {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Textarea);

impl gpui::Styled for Textarea {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
