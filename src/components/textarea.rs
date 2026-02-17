use std::{rc::Rc, time::Duration};

use gpui::{
    Animation, AnimationExt, AnyElement, AppContext, ClickEvent, ClipboardItem, EmptyView,
    FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, canvas, div, px,
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
struct TextareaSelectionDragState {
    textarea_id: String,
    anchor: usize,
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

        let inserted = if key == "enter" {
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

    fn char_width_px(&self) -> f32 {
        match self.size {
            Size::Xs => 6.8,
            Size::Sm => 7.2,
            Size::Md => 7.8,
            Size::Lg => 8.6,
            Size::Xl => 9.4,
        }
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

    fn caret_from_click(
        id: &str,
        position: gpui::Point<gpui::Pixels>,
        value: &str,
        line_height: f32,
        vertical_padding: f32,
        char_width: f32,
    ) -> usize {
        let (origin_x, origin_y, _width, _height) = Self::box_geometry(id);
        let local_x = (f32::from(position.x) - origin_x).max(0.0);
        let local_y = (f32::from(position.y) - origin_y - vertical_padding).max(0.0);

        let target_line = (local_y / line_height.max(1.0)).floor() as usize;
        let target_col = (local_x / char_width.max(1.0)).floor() as usize;
        let lines = value.split('\n').collect::<Vec<_>>();
        if lines.is_empty() {
            return 0;
        }
        let line_index = target_line.min(lines.len().saturating_sub(1));
        let col = target_col.min(lines[line_index].chars().count());
        Self::char_from_line_col(value, line_index, col)
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

    fn caret_height_px(&self) -> f32 {
        match self.size {
            Size::Xs => 13.0,
            Size::Sm => 15.0,
            Size::Md => 17.0,
            Size::Lg => 19.0,
            Size::Xl => 21.0,
        }
    }

    fn resolved_rows(&self, value: &str) -> (usize, bool) {
        let lines = value.chars().filter(|ch| *ch == '\n').count() + 1;
        let max_rows = self.max_rows.unwrap_or(lines.max(self.min_rows));
        let rows = lines.clamp(self.min_rows, max_rows);
        (rows, lines > rows)
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

    fn render_input_box(&self, window: &Window) -> AnyElement {
        let tokens = &self.theme.components.textarea;
        let resolved_value = self.resolved_value();
        let current_value = resolved_value.to_string();
        let tracked_focus = control::focused_state(&self.id, None, false);
        let handle_focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|focus_handle| focus_handle.is_focused(window));
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

        let (rows, should_scroll) = self.resolved_rows(&current_value);
        let line_height = self.line_height_px();
        let vertical_padding = self.vertical_padding_px();
        let char_width = self.char_width_px();
        let box_height = (rows as f32 * line_height) + (vertical_padding * 2.0) + 2.0;

        let mut input = div()
            .id(self.id.slot("box"))
            .focusable()
            .flex()
            .flex_col()
            .items_start()
            .justify_start()
            .gap_1()
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

        if let Some(focus_handle) = &self.focus_handle {
            let handle_for_click = focus_handle.clone();
            let id_for_focus = self.id.clone();
            let value_for_click = current_value.clone();
            let line_height_for_click = line_height;
            let vertical_padding_for_click = vertical_padding;
            let char_width_for_click = char_width;
            input =
                input
                    .track_focus(focus_handle)
                    .on_click(move |event: &ClickEvent, window, cx| {
                        control::set_focused_state(&id_for_focus, true);
                        let click_caret = Self::caret_from_click(
                            &id_for_focus,
                            event.position(),
                            &value_for_click,
                            line_height_for_click,
                            vertical_padding_for_click,
                            char_width_for_click,
                        );
                        control::set_text_state(
                            &id_for_focus,
                            "caret-index",
                            click_caret.to_string(),
                        );
                        Self::clear_selection_for(&id_for_focus, click_caret);
                        window.focus(&handle_for_click, cx);
                        window.refresh();
                    });
        } else {
            let id_for_focus = self.id.clone();
            let value_for_click = current_value.clone();
            let line_height_for_click = line_height;
            let vertical_padding_for_click = vertical_padding;
            let char_width_for_click = char_width;
            input = input.on_click(move |event: &ClickEvent, window, _cx| {
                control::set_focused_state(&id_for_focus, true);
                let click_caret = Self::caret_from_click(
                    &id_for_focus,
                    event.position(),
                    &value_for_click,
                    line_height_for_click,
                    vertical_padding_for_click,
                    char_width_for_click,
                );
                control::set_text_state(&id_for_focus, "caret-index", click_caret.to_string());
                Self::clear_selection_for(&id_for_focus, click_caret);
                window.refresh();
            });
        }

        let id_for_blur = self.id.clone();
        input = input.on_mouse_down_out(move |_, window, _cx| {
            control::set_focused_state(&id_for_blur, false);
            window.refresh();
        });

        if !self.disabled && !self.read_only {
            let drag_state = TextareaSelectionDragState {
                textarea_id: self.id.to_string(),
                anchor: current_caret,
            };
            let id_for_drag = self.id.to_string();
            let value_for_drag = current_value.clone();
            let line_height_for_drag = line_height;
            let vertical_padding_for_drag = vertical_padding;
            let char_width_for_drag = char_width;
            input = input
                .on_drag(drag_state, |_drag, _, _, cx| cx.new(|_| EmptyView))
                .on_drag_move::<TextareaSelectionDragState>(move |event, window, cx| {
                    let drag = event.drag(cx);
                    if drag.textarea_id != id_for_drag {
                        return;
                    }
                    let caret = Self::caret_from_click(
                        &id_for_drag,
                        event.event.position,
                        &value_for_drag,
                        line_height_for_drag,
                        vertical_padding_for_drag,
                        char_width_for_drag,
                    );
                    control::set_text_state(&id_for_drag, "caret-index", caret.to_string());
                    Self::set_selection_for(&id_for_drag, drag.anchor, caret);
                    window.refresh();
                });

            let on_change = self.on_change.clone();
            let value_controlled = self.value_controlled;
            let input_id = self.id.clone();
            let max_length = self.max_length;
            let current_value_for_input = current_value.clone();
            let current_caret_for_input = current_caret;
            input = input.on_key_down(move |event, window, cx| {
                control::set_focused_state(&input_id, true);
                let len = current_value_for_input.chars().count();
                let selection = Self::selection_bounds_for(&input_id, len);
                let modifiers =
                    event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

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

        if current_value.is_empty() && !is_focused {
            input = input.child(
                div()
                    .w_full()
                    .text_color(resolve_hsla(&self.theme, &tokens.placeholder))
                    .child(self.placeholder.clone().unwrap_or_default()),
            );
        } else {
            let lines = if current_value.is_empty() {
                vec![String::new()]
            } else {
                current_value
                    .split('\n')
                    .map(|line| line.to_string())
                    .collect()
            };

            let mut content = Stack::vertical().w_full().gap_0();
            let (caret_line, caret_col) = Self::line_col_from_char(&current_value, current_caret);
            let selection_bg =
                resolve_hsla(&self.theme, &self.theme.semantic.focus_ring).alpha(0.28);
            let mut line_start_char = 0usize;
            for (line_index, line) in lines.into_iter().enumerate() {
                let line_len = line.chars().count();
                let line_end_char = line_start_char + line_len;
                if let Some((selection_start, selection_end)) = selection {
                    let seg_start = selection_start.clamp(line_start_char, line_end_char);
                    let seg_end = selection_end.clamp(line_start_char, line_end_char);
                    if seg_start < seg_end {
                        let local_start = seg_start - line_start_char;
                        let local_end = seg_end - line_start_char;
                        let left = line.chars().take(local_start).collect::<String>();
                        let selected = line
                            .chars()
                            .skip(local_start)
                            .take(local_end - local_start)
                            .collect::<String>();
                        let right = line.chars().skip(local_end).collect::<String>();
                        content = content.child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .child(if left.is_empty() {
                                    "".to_string().into_any_element()
                                } else {
                                    left.into_any_element()
                                })
                                .child(div().bg(selection_bg).child(if selected.is_empty() {
                                    " ".to_string()
                                } else {
                                    selected
                                }))
                                .child(if right.is_empty() {
                                    "".to_string().into_any_element()
                                } else {
                                    right.into_any_element()
                                }),
                        );
                        line_start_char = line_end_char.saturating_add(1);
                        continue;
                    }
                }

                if line_index == caret_line
                    && !self.disabled
                    && !self.read_only
                    && is_focused
                    && selection.is_none()
                {
                    let left = line.chars().take(caret_col).collect::<String>();
                    let right = line.chars().skip(caret_col).collect::<String>();
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
                    content = content.child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .child(if left.is_empty() {
                                " ".to_string().into_any_element()
                            } else {
                                left.into_any_element()
                            })
                            .child(caret)
                            .child(if right.is_empty() {
                                "".to_string().into_any_element()
                            } else {
                                right.into_any_element()
                            }),
                    );
                } else if line.is_empty() {
                    content = content.child(div().w_full().child(" "));
                } else {
                    content = content.child(div().w_full().child(line));
                }
                line_start_char = line_end_char.saturating_add(1);
            }

            let show_caret = is_focused;
            if !self.disabled
                && !self.read_only
                && show_caret
                && current_value.is_empty()
                && selection.is_none()
            {
                content = content.child(
                    div()
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
                        ),
                );
            }

            input = input.child(content);
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
                .gap_2()
                .child(self.render_label_block())
                .child(self.render_input_box(window)),
            FieldLayout::Horizontal => Stack::horizontal()
                .items_start()
                .gap_3()
                .child(div().w(px(168.0)).child(self.render_label_block()))
                .child(div().flex_1().child(self.render_input_box(window))),
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
