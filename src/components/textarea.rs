use std::{rc::Rc, time::Duration};

use gpui::{
    Animation, AnimationExt, AnyElement, Component, FocusHandle, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, px,
};

use crate::contracts::{FieldLike, MotionAware, ThemeScoped, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};
use crate::theme::Theme;

use super::control;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::{apply_input_size, apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

const CARET_BLINK_TOGGLE_MS: u64 = 680;
const CARET_BLINK_CYCLE_MS: u64 = CARET_BLINK_TOGGLE_MS * 2;

pub struct Textarea {
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
    min_rows: usize,
    max_rows: Option<usize>,
    disabled: bool,
    read_only: bool,
    max_length: Option<usize>,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: Theme,
    motion: MotionConfig,
    focus_handle: Option<FocusHandle>,
    on_change: Option<ChangeHandler>,
}

impl Textarea {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("textarea"),
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
            theme: Theme::default(),
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

        let mut next = current.to_string();
        next.push_str(&inserted);

        if let Some(max_length) = max_length {
            if next.chars().count() > max_length {
                next = next.chars().take(max_length).collect();
            }
        }

        Some(next)
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

    fn render_input_box(&self, window: &Window) -> AnyElement {
        let tokens = &self.theme.components.textarea;
        let resolved_value = self.resolved_value();
        let current_value = resolved_value.to_string();
        let is_focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|focus_handle| focus_handle.is_focused(window));

        let (rows, should_scroll) = self.resolved_rows(&current_value);
        let box_height =
            (rows as f32 * self.line_height_px()) + (self.vertical_padding_px() * 2.0) + 2.0;

        let mut input = div()
            .id(format!("{}-box", self.id))
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

        if should_scroll {
            input = input.overflow_y_scroll();
        }

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
            let current_value_for_input = current_value.clone();
            input = input.on_key_down(move |event, window, cx| {
                if let Some(next) =
                    Self::with_value_update(&current_value_for_input, event, max_length)
                {
                    if !value_controlled {
                        control::set_text_state(&input_id, "value", next.clone());
                        window.refresh();
                    }
                    if let Some(handler) = on_change.as_ref() {
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

            let mut content = v_stack().w_full().gap_0();
            for line in lines {
                if line.is_empty() {
                    content = content.child(div().w_full().child(" "));
                } else {
                    content = content.child(div().w_full().child(line));
                }
            }

            let show_caret = self.focus_handle.is_none() || is_focused;
            if !self.disabled && !self.read_only && show_caret {
                content = content.child(
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

            input = input.child(content);
        }

        input
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl WithId for Textarea {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
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

impl VariantSupport for Textarea {
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

impl ThemeScoped for Textarea {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for Textarea {
    fn render(self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
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

impl IntoElement for Textarea {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}
