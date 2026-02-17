use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    AnyElement, ClickEvent, Component, Corner, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, anchored, canvas,
    deferred, div, point, px,
};

use crate::contracts::{FieldLike, MotionAware, VariantConfigurable, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};
use crate::theme::{SelectTokens, Theme};

use super::Stack;
use super::control;
use super::icon::Icon;
use super::transition::TransitionExt;
use super::utils::{apply_input_size, apply_radius, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type SelectChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;
type MultiSelectChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;
type OpenChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

struct SelectRuntime;

impl SelectRuntime {
    fn should_open_dropdown_upward(event: &ClickEvent, window: &Window) -> bool {
        let click_y = event.position().y;
        let viewport_height = window.viewport_size().height;
        let space_above = click_y;
        let space_below = viewport_height - click_y;
        let preferred_height = px(260.0);

        space_below < preferred_height && space_above > space_below
    }

    fn capture_dropdown_metrics(id: &str, event: &ClickEvent, window: &Window) {
        control::set_bool_state(
            id,
            "dropdown-upward",
            Self::should_open_dropdown_upward(event, window),
        );
        if let ClickEvent::Keyboard(keyboard) = event {
            control::set_text_state(
                id,
                "dropdown-width-px",
                format!("{:.2}", f32::from(keyboard.bounds.size.width)),
            );
        }
    }

    fn dropdown_width_px(id: &str) -> f32 {
        control::text_state(id, "dropdown-width-px", None, String::new())
            .parse::<f32>()
            .ok()
            .filter(|width| *width >= 1.0)
            .unwrap_or(220.0)
    }

    fn control_bg_for_variant(
        theme: &Theme,
        tokens: &SelectTokens,
        variant: Variant,
    ) -> gpui::Hsla {
        let base = resolve_hsla(theme, &tokens.bg);
        match variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.9),
            Variant::Subtle => base.alpha(0.74),
            Variant::Outline => base.alpha(0.22),
            Variant::Ghost => base.alpha(0.0),
        }
    }

    fn control_border_for_variant(
        theme: &Theme,
        tokens: &SelectTokens,
        variant: Variant,
        opened: bool,
        has_error: bool,
    ) -> gpui::Hsla {
        let base = if has_error {
            resolve_hsla(theme, &tokens.border_error)
        } else if opened {
            resolve_hsla(theme, &tokens.border_focus)
        } else {
            resolve_hsla(theme, &tokens.border)
        };

        match variant {
            Variant::Ghost if !opened && !has_error => base.alpha(0.0),
            Variant::Ghost => base.alpha(0.88),
            Variant::Subtle => base.alpha(if opened { 0.78 } else { 0.52 }),
            Variant::Light => base.alpha(if opened { 0.92 } else { 0.7 }),
            _ => base,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

pub struct Select {
    id: String,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    options: Vec<SelectOption>,
    placeholder: Option<SharedString>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    opened: Option<bool>,
    opened_controlled: bool,
    default_opened: bool,
    close_on_click_outside: bool,
    disabled: bool,
    left_slot: Option<SlotRenderer>,
    right_slot: Option<SlotRenderer>,
    size: Size,
    radius: Radius,
    variant: Variant,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<SelectChangeHandler>,
    on_open_change: Option<OpenChangeHandler>,
}

impl Select {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("select"),
            value: None,
            value_controlled: false,
            default_value: None,
            options: Vec::new(),
            placeholder: None,
            label: None,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            opened: None,
            opened_controlled: false,
            default_opened: false,
            close_on_click_outside: true,
            disabled: false,
            left_slot: None,
            right_slot: None,
            size: Size::Md,
            radius: Radius::Sm,
            variant: Variant::Default,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
            on_open_change: None,
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self.value_controlled = true;
        self
    }

    pub fn clear_value(mut self) -> Self {
        self.value = None;
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn option(mut self, option: SelectOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn opened(mut self, value: bool) -> Self {
        self.opened = Some(value);
        self.opened_controlled = true;
        self
    }

    pub fn default_opened(mut self, value: bool) -> Self {
        self.default_opened = value;
        self
    }

    pub fn close_on_click_outside(mut self, value: bool) -> Self {
        self.close_on_click_outside = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
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

    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }

    fn resolved_value(&self) -> Option<SharedString> {
        control::optional_text_state(
            &self.id,
            "value",
            self.value_controlled
                .then_some(self.value.as_ref().map(|value| value.to_string())),
            self.default_value.as_ref().map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }

    fn resolved_opened(&self) -> bool {
        control::bool_state(
            &self.id,
            "opened",
            self.opened_controlled
                .then_some(self.opened.unwrap_or(false)),
            self.default_opened,
        )
    }

    fn selected_label(&self) -> Option<SharedString> {
        let current = self.resolved_value()?;
        self.options
            .iter()
            .find(|option| option.value.as_ref() == current.as_ref())
            .map(|option| option.label.clone())
    }

    fn render_label_block(&self) -> AnyElement {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return div().into_any_element();
        }

        let tokens = &self.theme.components.select;
        let mut block = Stack::vertical().gap_1();
        if let Some(label) = self.label.clone() {
            let mut label_row = Stack::horizontal().gap_1().child(
                div()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(resolve_hsla(&self.theme, &tokens.label))
                    .child(label),
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

        if let Some(description) = self.description.clone() {
            block = block.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.description))
                    .child(description),
            );
        }

        if let Some(error) = self.error.clone() {
            block = block.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.error))
                    .child(error),
            );
        }

        block.into_any_element()
    }

    fn render_control(&mut self, window: &gpui::Window) -> AnyElement {
        let tokens = &self.theme.components.select;
        let opened = self.resolved_opened();
        let value = self.resolved_value();
        let mut control = div()
            .id(format!("{}-control", self.id))
            .relative()
            .w_full()
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .bg(SelectRuntime::control_bg_for_variant(
                &self.theme,
                tokens,
                self.variant,
            ))
            .text_color(resolve_hsla(&self.theme, &tokens.fg))
            .border(super::utils::quantized_stroke_px(window, 1.0));

        control = apply_input_size(control, self.size);
        control = apply_radius(&self.theme, control, self.radius);

        let border = SelectRuntime::control_border_for_variant(
            &self.theme,
            tokens,
            self.variant,
            opened,
            self.error.is_some(),
        );
        control = control.border_color(border);
        if opened {
            control = control.shadow_sm();
        }

        if self.disabled {
            control = control.cursor_default().opacity(0.55);
        }

        if !self.disabled {
            if let Some(handler) = self.on_open_change.clone() {
                let next = !opened;
                let id = self.id.clone();
                let opened_controlled = self.opened_controlled;
                control = control.on_click(
                    move |event: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                        if next {
                            SelectRuntime::capture_dropdown_metrics(&id, event, window);
                        }
                        if !opened_controlled {
                            control::set_bool_state(&id, "opened", next);
                            window.refresh();
                        }
                        (handler)(next, window, cx);
                    },
                );
            } else if !self.opened_controlled {
                let id = self.id.clone();
                let next = !opened;
                control = control.on_click(move |event: &ClickEvent, window: &mut Window, _cx| {
                    if next {
                        SelectRuntime::capture_dropdown_metrics(&id, event, window);
                    }
                    control::set_bool_state(&id, "opened", next);
                    window.refresh();
                });
            }
        }

        if let Some(left_slot) = self.left_slot.take() {
            control = control.child(
                div()
                    .flex_none()
                    .text_color(resolve_hsla(&self.theme, &tokens.icon))
                    .child(left_slot()),
            );
        }

        let value_text = self
            .selected_label()
            .or_else(|| self.placeholder.clone())
            .unwrap_or_else(|| SharedString::from("Select"));
        let value_color = if value.is_some() {
            resolve_hsla(&self.theme, &tokens.fg)
        } else {
            resolve_hsla(&self.theme, &tokens.placeholder)
        };

        control = control.child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_color(value_color)
                .child(value_text),
        );

        if let Some(right_slot) = self.right_slot.take() {
            control = control.child(
                div()
                    .ml_auto()
                    .flex_none()
                    .text_color(resolve_hsla(&self.theme, &tokens.icon))
                    .child(right_slot()),
            );
        }

        let id_for_width = self.id.clone();
        control
            .child(
                Icon::named_outline(if opened { "chevron-up" } else { "chevron-down" })
                    .with_id(format!("{}-chevron", self.id))
                    .size(14.0)
                    .color(resolve_hsla(&self.theme, &tokens.icon)),
            )
            .child(
                canvas(
                    move |bounds, _, _cx| {
                        control::set_text_state(
                            &id_for_width,
                            "dropdown-width-px",
                            format!("{:.2}", f32::from(bounds.size.width)),
                        );
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .into_any_element()
    }

    fn render_dropdown(&mut self, window: &gpui::Window) -> AnyElement {
        let tokens = &self.theme.components.select;
        let current_value = self.resolved_value();

        let items = self
            .options
            .iter()
            .cloned()
            .map(|option| {
                let selected = current_value
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == option.value.as_ref());

                let row_bg = if selected {
                    resolve_hsla(&self.theme, &tokens.option_selected_bg)
                } else {
                    resolve_hsla(&self.theme, &gpui::transparent_black())
                };
                let hover_bg = resolve_hsla(&self.theme, &tokens.option_hover_bg);

                let mut row = div()
                    .id(format!("{}-option-{}", self.id, option.value))
                    .px(gpui::px(10.0))
                    .py(gpui::px(8.0))
                    .rounded_sm()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.option_fg))
                    .bg(row_bg)
                    .hover(move |style| style.bg(hover_bg))
                    .child(
                        Stack::horizontal()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .child(option.label.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .flex_none()
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .children(
                                        selected.then_some(
                                            Icon::named_outline("check")
                                                .with_id(format!(
                                                    "{}-selected-{}",
                                                    self.id, option.value
                                                ))
                                                .size(12.0)
                                                .color(resolve_hsla(&self.theme, &tokens.icon)),
                                        ),
                                    ),
                            ),
                    );

                if option.disabled {
                    row = row.opacity(0.45).cursor_default();
                } else {
                    let value = option.value.clone();
                    let on_change = self.on_change.clone();
                    let on_open_change = self.on_open_change.clone();
                    let id = self.id.clone();
                    let value_controlled = self.value_controlled;
                    let opened_controlled = self.opened_controlled;
                    row = row.cursor_pointer().on_click(
                        move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                            if !value_controlled {
                                control::set_optional_text_state(
                                    &id,
                                    "value",
                                    Some(value.to_string()),
                                );
                            }
                            if !opened_controlled {
                                control::set_bool_state(&id, "opened", false);
                            }
                            if !value_controlled || !opened_controlled {
                                window.refresh();
                            }
                            if let Some(handler) = on_change.as_ref() {
                                (handler)(value.clone(), window, cx);
                            }
                            if let Some(handler) = on_open_change.as_ref() {
                                (handler)(false, window, cx);
                            }
                        },
                    );
                }

                row.into_any_element()
            })
            .collect::<Vec<_>>();

        let mut dropdown = div()
            .id(format!("{}-dropdown", self.id))
            .w(px(SelectRuntime::dropdown_width_px(&self.id)))
            .rounded_md()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.dropdown_border))
            .bg(resolve_hsla(&self.theme, &tokens.dropdown_bg))
            .shadow_sm()
            .max_h(px(280.0))
            .overflow_y_scroll()
            .p_1p5()
            .child(Stack::vertical().gap_1().children(items));

        if self.close_on_click_outside {
            if let Some(on_open_change) = self.on_open_change.clone() {
                let id = self.id.clone();
                let opened_controlled = self.opened_controlled;
                dropdown = dropdown.on_mouse_down_out(move |_, window, cx| {
                    if !opened_controlled {
                        control::set_bool_state(&id, "opened", false);
                        window.refresh();
                    }
                    (on_open_change)(false, window, cx);
                });
            } else if !self.opened_controlled {
                let id = self.id.clone();
                dropdown = dropdown.on_mouse_down_out(move |_, window, _cx| {
                    control::set_bool_state(&id, "opened", false);
                    window.refresh();
                });
            }
        }

        dropdown
            .with_enter_transition(format!("{}-dropdown-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl WithId for Select {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl FieldLike for Select {
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

impl VariantConfigurable for Select {
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

impl MotionAware for Select {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Select {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let opened = self.resolved_opened();
        let dropdown_upward = control::bool_state(&self.id, "dropdown-upward", None, false);
        let mut container = Stack::vertical()
            .id(self.id.clone())
            .gap_2()
            .relative()
            .w_full();

        if self.layout == FieldLayout::Vertical {
            container = container.child(self.render_label_block());
        }

        let mut field = Stack::vertical().gap_1();
        let mut trigger = div()
            .id(format!("{}-trigger", self.id))
            .relative()
            .w_full()
            .child(self.render_control(window));

        if opened {
            let floating = self.render_dropdown(window);
            let anchor_host = if dropdown_upward {
                div()
                    .id(format!("{}-anchor-host", self.id))
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h(px(0.0))
                    .child(
                        deferred(
                            anchored()
                                .anchor(Corner::BottomLeft)
                                .offset(point(px(0.0), px(-2.0)))
                                .snap_to_window_with_margin(px(8.0))
                                .child(floating),
                        )
                        .priority(24),
                    )
            } else {
                div()
                    .id(format!("{}-anchor-host", self.id))
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .w_full()
                    .h(px(0.0))
                    .child(
                        deferred(
                            anchored()
                                .anchor(Corner::TopLeft)
                                .offset(point(px(0.0), px(2.0)))
                                .snap_to_window_with_margin(px(8.0))
                                .child(floating),
                        )
                        .priority(24),
                    )
            };
            trigger = trigger.child(anchor_host);
        }
        field = field.child(trigger);

        match self.layout {
            FieldLayout::Vertical => container.child(field).into_any_element(),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap_3()
                .child(div().w(gpui::px(168.0)).child(self.render_label_block()))
                .child(div().flex_1().child(field))
                .into_any_element(),
        }
    }
}

impl IntoElement for Select {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

pub struct MultiSelect {
    id: String,
    values: Vec<SharedString>,
    values_controlled: bool,
    default_values: Vec<SharedString>,
    options: Vec<SelectOption>,
    placeholder: Option<SharedString>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    opened: Option<bool>,
    opened_controlled: bool,
    default_opened: bool,
    close_on_click_outside: bool,
    disabled: bool,
    left_slot: Option<SlotRenderer>,
    right_slot: Option<SlotRenderer>,
    size: Size,
    radius: Radius,
    variant: Variant,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<MultiSelectChangeHandler>,
    on_open_change: Option<OpenChangeHandler>,
}

impl MultiSelect {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("multi-select"),
            values: Vec::new(),
            values_controlled: false,
            default_values: Vec::new(),
            options: Vec::new(),
            placeholder: None,
            label: None,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            opened: None,
            opened_controlled: false,
            default_opened: false,
            close_on_click_outside: true,
            disabled: false,
            left_slot: None,
            right_slot: None,
            size: Size::Md,
            radius: Radius::Sm,
            variant: Variant::Default,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
            on_open_change: None,
        }
    }

    pub fn values(mut self, values: impl IntoIterator<Item = SharedString>) -> Self {
        self.values = values.into_iter().collect();
        self.values_controlled = true;
        self
    }

    pub fn default_values(mut self, values: impl IntoIterator<Item = SharedString>) -> Self {
        self.default_values = values.into_iter().collect();
        self
    }

    pub fn option(mut self, option: SelectOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn opened(mut self, value: bool) -> Self {
        self.opened = Some(value);
        self.opened_controlled = true;
        self
    }

    pub fn default_opened(mut self, value: bool) -> Self {
        self.default_opened = value;
        self
    }

    pub fn close_on_click_outside(mut self, value: bool) -> Self {
        self.close_on_click_outside = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
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

    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }

    fn contains(values: &[SharedString], value: &SharedString) -> bool {
        values
            .iter()
            .any(|candidate| candidate.as_ref() == value.as_ref())
    }

    fn toggled_values(values: &[SharedString], value: &SharedString) -> Vec<SharedString> {
        let mut set = values
            .iter()
            .map(|candidate| candidate.to_string())
            .collect::<BTreeSet<_>>();
        if !set.insert(value.to_string()) {
            set.remove(value.as_ref());
        }
        set.into_iter().map(SharedString::from).collect()
    }

    fn selected_labels(&self) -> Vec<SharedString> {
        let values = self.resolved_values();
        self.options
            .iter()
            .filter(|option| Self::contains(&values, &option.value))
            .map(|option| option.label.clone())
            .collect()
    }

    fn resolved_values(&self) -> Vec<SharedString> {
        control::list_state(
            &self.id,
            "values",
            self.values_controlled.then_some(
                self.values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>(),
            ),
            self.default_values
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
        )
        .into_iter()
        .map(SharedString::from)
        .collect()
    }

    fn resolved_opened(&self) -> bool {
        control::bool_state(
            &self.id,
            "opened",
            self.opened_controlled
                .then_some(self.opened.unwrap_or(false)),
            self.default_opened,
        )
    }

    fn render_label_block(&self) -> AnyElement {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return div().into_any_element();
        }

        let tokens = &self.theme.components.select;
        let mut block = Stack::vertical().gap_1();

        if let Some(label) = self.label.clone() {
            let mut label_row = Stack::horizontal().gap_1().child(
                div()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(resolve_hsla(&self.theme, &tokens.label))
                    .child(label),
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

        if let Some(description) = self.description.clone() {
            block = block.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.description))
                    .child(description),
            );
        }

        if let Some(error) = self.error.clone() {
            block = block.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.error))
                    .child(error),
            );
        }

        block.into_any_element()
    }

    fn render_control(&mut self, window: &gpui::Window) -> AnyElement {
        let tokens = &self.theme.components.select;
        let opened = self.resolved_opened();

        let mut control = div()
            .id(format!("{}-control", self.id))
            .relative()
            .w_full()
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .bg(SelectRuntime::control_bg_for_variant(
                &self.theme,
                tokens,
                self.variant,
            ))
            .border(super::utils::quantized_stroke_px(window, 1.0));

        control = apply_input_size(control, self.size);
        control = apply_radius(&self.theme, control, self.radius);

        let border = SelectRuntime::control_border_for_variant(
            &self.theme,
            tokens,
            self.variant,
            opened,
            self.error.is_some(),
        );
        control = control.border_color(border);
        if opened {
            control = control.shadow_sm();
        }

        if self.disabled {
            control = control.cursor_default().opacity(0.55);
        } else if let Some(handler) = self.on_open_change.clone() {
            let next = !opened;
            let id = self.id.clone();
            let opened_controlled = self.opened_controlled;
            control = control.on_click(
                move |event: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                    if next {
                        SelectRuntime::capture_dropdown_metrics(&id, event, window);
                    }
                    if !opened_controlled {
                        control::set_bool_state(&id, "opened", next);
                        window.refresh();
                    }
                    (handler)(next, window, cx);
                },
            );
        } else if !self.opened_controlled {
            let next = !opened;
            let id = self.id.clone();
            control = control.on_click(move |event: &ClickEvent, window: &mut Window, _cx| {
                if next {
                    SelectRuntime::capture_dropdown_metrics(&id, event, window);
                }
                control::set_bool_state(&id, "opened", next);
                window.refresh();
            });
        }

        if let Some(left_slot) = self.left_slot.take() {
            control = control.child(
                div()
                    .flex_none()
                    .text_color(resolve_hsla(&self.theme, &tokens.icon))
                    .child(left_slot()),
            );
        }

        let selected = self.selected_labels();
        if selected.is_empty() {
            control = control.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(resolve_hsla(&self.theme, &tokens.placeholder))
                    .child(
                        self.placeholder
                            .clone()
                            .unwrap_or_else(|| SharedString::from("Select")),
                    ),
            );
        } else {
            let tags = selected.into_iter().map(|label| {
                div()
                    .px(gpui::px(8.0))
                    .py(gpui::px(3.0))
                    .text_xs()
                    .rounded_full()
                    .border(super::utils::quantized_stroke_px(window, 1.0))
                    .border_color(resolve_hsla(&self.theme, &tokens.tag_border))
                    .bg(resolve_hsla(&self.theme, &tokens.tag_bg))
                    .text_color(resolve_hsla(&self.theme, &tokens.tag_fg))
                    .child(div().max_w(px(120.0)).truncate().child(label))
                    .into_any_element()
            });

            control = control.child(
                Stack::horizontal()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .overflow_hidden()
                    .children(tags),
            );
        }

        if let Some(right_slot) = self.right_slot.take() {
            control = control.child(
                div()
                    .ml_auto()
                    .flex_none()
                    .text_color(resolve_hsla(&self.theme, &tokens.icon))
                    .child(right_slot()),
            );
        }

        let id_for_width = self.id.clone();
        control
            .child(
                Icon::named_outline(if opened { "chevron-up" } else { "chevron-down" })
                    .with_id(format!("{}-chevron", self.id))
                    .size(14.0)
                    .color(resolve_hsla(&self.theme, &tokens.icon)),
            )
            .child(
                canvas(
                    move |bounds, _, _cx| {
                        control::set_text_state(
                            &id_for_width,
                            "dropdown-width-px",
                            format!("{:.2}", f32::from(bounds.size.width)),
                        );
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .into_any_element()
    }

    fn render_dropdown(&mut self, window: &gpui::Window) -> AnyElement {
        let tokens = &self.theme.components.select;
        let current_values = self.resolved_values();

        let rows = self
            .options
            .iter()
            .cloned()
            .map(|option| {
                let checked = Self::contains(&current_values, &option.value);
                let row_bg = if checked {
                    resolve_hsla(&self.theme, &tokens.option_selected_bg)
                } else {
                    resolve_hsla(&self.theme, &gpui::transparent_black())
                };
                let hover_bg = resolve_hsla(&self.theme, &tokens.option_hover_bg);

                let mut row = div()
                    .id(format!("{}-option-{}", self.id, option.value))
                    .px(gpui::px(10.0))
                    .py(gpui::px(8.0))
                    .rounded_sm()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.option_fg))
                    .bg(row_bg)
                    .hover(move |style| style.bg(hover_bg))
                    .child(
                        Stack::horizontal()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .child(option.label.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .flex_none()
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .children(
                                        checked.then_some(
                                            Icon::named_outline("check")
                                                .with_id(format!(
                                                    "{}-selected-{}",
                                                    self.id, option.value
                                                ))
                                                .size(12.0)
                                                .color(resolve_hsla(&self.theme, &tokens.icon)),
                                        ),
                                    ),
                            ),
                    );

                if option.disabled {
                    row = row.opacity(0.45).cursor_default();
                } else {
                    let value = option.value.clone();
                    let on_change = self.on_change.clone();
                    let selected_values = current_values.clone();
                    let id = self.id.clone();
                    let values_controlled = self.values_controlled;
                    row = row.cursor_pointer().on_click(
                        move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                            let updated = Self::toggled_values(&selected_values, &value);
                            if !values_controlled {
                                control::set_list_state(
                                    &id,
                                    "values",
                                    updated.iter().map(|value| value.to_string()).collect(),
                                );
                                window.refresh();
                            }
                            if let Some(handler) = on_change.as_ref() {
                                (handler)(updated, window, cx);
                            }
                        },
                    );
                }

                row.into_any_element()
            })
            .collect::<Vec<_>>();

        let mut dropdown = div()
            .id(format!("{}-dropdown", self.id))
            .w(px(SelectRuntime::dropdown_width_px(&self.id)))
            .rounded_md()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.dropdown_border))
            .bg(resolve_hsla(&self.theme, &tokens.dropdown_bg))
            .shadow_sm()
            .max_h(px(280.0))
            .overflow_y_scroll()
            .p_1p5()
            .child(Stack::vertical().gap_1().children(rows));

        if self.close_on_click_outside {
            if let Some(on_open_change) = self.on_open_change.clone() {
                let id = self.id.clone();
                let opened_controlled = self.opened_controlled;
                dropdown = dropdown.on_mouse_down_out(move |_, window, cx| {
                    if !opened_controlled {
                        control::set_bool_state(&id, "opened", false);
                        window.refresh();
                    }
                    (on_open_change)(false, window, cx);
                });
            } else if !self.opened_controlled {
                let id = self.id.clone();
                dropdown = dropdown.on_mouse_down_out(move |_, window, _cx| {
                    control::set_bool_state(&id, "opened", false);
                    window.refresh();
                });
            }
        }

        dropdown
            .with_enter_transition(format!("{}-dropdown-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl WithId for MultiSelect {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl FieldLike for MultiSelect {
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

impl VariantConfigurable for MultiSelect {
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

impl MotionAware for MultiSelect {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for MultiSelect {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let opened = self.resolved_opened();
        let dropdown_upward = control::bool_state(&self.id, "dropdown-upward", None, false);
        let mut container = Stack::vertical()
            .id(self.id.clone())
            .gap_2()
            .relative()
            .w_full();
        if self.layout == FieldLayout::Vertical {
            container = container.child(self.render_label_block());
        }

        let mut field = Stack::vertical().gap_1();
        let mut trigger = div()
            .id(format!("{}-trigger", self.id))
            .relative()
            .w_full()
            .child(self.render_control(window));

        if opened {
            let floating = self.render_dropdown(window);
            let anchor_host = if dropdown_upward {
                div()
                    .id(format!("{}-anchor-host", self.id))
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h(px(0.0))
                    .child(
                        deferred(
                            anchored()
                                .anchor(Corner::BottomLeft)
                                .offset(point(px(0.0), px(-2.0)))
                                .snap_to_window_with_margin(px(8.0))
                                .child(floating),
                        )
                        .priority(24),
                    )
            } else {
                div()
                    .id(format!("{}-anchor-host", self.id))
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .w_full()
                    .h(px(0.0))
                    .child(
                        deferred(
                            anchored()
                                .anchor(Corner::TopLeft)
                                .offset(point(px(0.0), px(2.0)))
                                .snap_to_window_with_margin(px(8.0))
                                .child(floating),
                        )
                        .priority(24),
                    )
            };
            trigger = trigger.child(anchor_host);
        }
        field = field.child(trigger);

        match self.layout {
            FieldLayout::Vertical => container.child(field).into_any_element(),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap_3()
                .child(div().w(gpui::px(168.0)).child(self.render_label_block()))
                .child(div().flex_1().child(field))
                .into_any_element(),
        }
    }
}

impl IntoElement for MultiSelect {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Select {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for MultiSelect {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(SelectOption);
crate::impl_disableable!(Select);
crate::impl_disableable!(MultiSelect);
crate::impl_openable!(Select);
crate::impl_openable!(MultiSelect);

impl gpui::Styled for MultiSelect {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for Select {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
