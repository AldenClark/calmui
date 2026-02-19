use std::rc::Rc;

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, canvas, div, px,
};

use crate::contracts::{FieldLike, MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};
use crate::theme::{SelectTokens, Theme};

use super::Stack;
use super::field_variant::FieldVariantRuntime;
use super::icon::Icon;
use super::interaction_adapter::{ActivateHandler, PressAdapter, bind_press_adapter};
use super::popup::{PopupPlacement, anchored_host};
use super::select_state::{self, SelectState, SelectStateInput};
use super::transition::TransitionExt;
use super::utils::{
    InteractionStyles, apply_field_size, apply_interaction_styles, apply_radius, interaction_style,
    resolve_hsla,
};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type SelectChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;
type MultiSelectChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;
type OpenChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

struct SelectRuntime;

impl SelectRuntime {
    fn control_bg_for_variant(
        theme: &Theme,
        tokens: &SelectTokens,
        variant: Variant,
    ) -> gpui::Hsla {
        let base = resolve_hsla(theme, &tokens.bg);
        FieldVariantRuntime::control_bg(base, variant)
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

        FieldVariantRuntime::control_border(base, variant, opened, has_error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectOption {
    pub value: SharedString,
    pub label: Option<SharedString>,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: None,
            disabled: false,
        }
    }

    pub fn labeled(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(value).label(label)
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

#[derive(IntoElement)]
pub struct Select {
    id: ComponentId,
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
            id: ComponentId::default(),
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
        select_state::resolve_single_value(
            &self.id,
            self.value_controlled,
            self.value.as_ref().map(|value| value.to_string()),
            self.default_value.as_ref().map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }

    fn resolved_opened(&self) -> bool {
        SelectState::resolve(SelectStateInput {
            id: &self.id,
            opened_controlled: self.opened_controlled,
            opened: self.opened,
            default_opened: self.default_opened,
        })
        .opened
    }

    fn selected_label(&self) -> Option<SharedString> {
        let current = self.resolved_value()?;
        self.options
            .iter()
            .find(|option| option.value.as_ref() == current.as_ref())
            .and_then(|option| option.label.clone())
    }

    fn render_label_block(&self) -> AnyElement {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return div().into_any_element();
        }

        let tokens = &self.theme.components.select;
        let mut block = Stack::vertical().gap(tokens.label_block_gap);
        if let Some(label) = self.label.clone() {
            let mut label_row = Stack::horizontal().gap(tokens.label_row_gap).child(
                div()
                    .text_size(tokens.label_size)
                    .font_weight(tokens.label_weight)
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
                    .text_size(tokens.description_size)
                    .text_color(resolve_hsla(&self.theme, &tokens.description))
                    .child(description),
            );
        }

        if let Some(error) = self.error.clone() {
            block = block.child(
                div()
                    .text_size(tokens.error_size)
                    .text_color(resolve_hsla(&self.theme, &tokens.error))
                    .child(error),
            );
        }

        block.into_any_element()
    }

    fn render_control(&mut self, window: &gpui::Window) -> AnyElement {
        let tokens = &self.theme.components.select;
        let dropdown_preferred_height = f32::from(tokens.dropdown_open_preferred_height);
        let opened = self.resolved_opened();
        let value = self.resolved_value();
        let control_bg = SelectRuntime::control_bg_for_variant(&self.theme, tokens, self.variant);
        let mut control = div()
            .id(self.id.slot("control"))
            .relative()
            .w_full()
            .flex()
            .items_center()
            .gap(tokens.slot_gap)
            .bg(control_bg)
            .text_color(resolve_hsla(&self.theme, &tokens.fg))
            .border(super::utils::quantized_stroke_px(window, 1.0));

        control = apply_field_size(control, tokens.sizes.for_size(self.size));
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
        } else {
            let activate_handler = if let Some(handler) = self.on_open_change.clone() {
                let next = !opened;
                let id = self.id.clone();
                let opened_controlled = self.opened_controlled;
                Some(Rc::new(move |window: &mut Window, cx: &mut gpui::App| {
                    if select_state::on_trigger_toggle_without_click(
                        &id,
                        opened_controlled,
                        next,
                        window,
                        dropdown_preferred_height,
                    ) {
                        window.refresh();
                    }
                    (handler)(next, window, cx);
                }) as ActivateHandler)
            } else if !self.opened_controlled {
                let id = self.id.clone();
                let next = !opened;
                Some(Rc::new(move |window: &mut Window, _cx: &mut gpui::App| {
                    if select_state::on_trigger_toggle_without_click(
                        &id,
                        false,
                        next,
                        window,
                        dropdown_preferred_height,
                    ) {
                        window.refresh();
                    }
                }) as ActivateHandler)
            } else {
                None
            };

            if let Some(activate_handler) = activate_handler {
                let hover_bg = control_bg.blend(gpui::white().opacity(0.04));
                let press_bg = control_bg.blend(gpui::black().opacity(0.08));
                let focus_border = if self.error.is_some() {
                    resolve_hsla(&self.theme, &tokens.border_error)
                } else {
                    resolve_hsla(&self.theme, &tokens.border_focus)
                };
                control = apply_interaction_styles(
                    control.cursor_pointer(),
                    InteractionStyles::new()
                        .hover(interaction_style(move |style| style.bg(hover_bg)))
                        .active(interaction_style(move |style| style.bg(press_bg)))
                        .focus(interaction_style(move |style| {
                            style.border_color(focus_border)
                        })),
                );
                control = bind_press_adapter(
                    control,
                    PressAdapter::new(self.id.slot("control")).on_activate(Some(activate_handler)),
                );
            } else {
                control = control.cursor_default();
            }
        }

        if let Some(left_slot) = self.left_slot.take() {
            control = control.child(
                div()
                    .flex_none()
                    .min_w(tokens.slot_min_width)
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
                    .min_w(tokens.slot_min_width)
                    .text_color(resolve_hsla(&self.theme, &tokens.icon))
                    .child(right_slot()),
            );
        }

        let id_for_width = self.id.clone();
        control
            .child(
                Icon::named(if opened { "chevron-up" } else { "chevron-down" })
                    .with_id(self.id.slot("chevron"))
                    .size(f32::from(tokens.icon_size))
                    .color(resolve_hsla(&self.theme, &tokens.icon)),
            )
            .child(
                canvas(
                    move |bounds, _, _cx| {
                        select_state::set_dropdown_width(
                            &id_for_width,
                            f32::from(bounds.size.width),
                        );
                        select_state::set_trigger_metrics(
                            &id_for_width,
                            f32::from(bounds.origin.y),
                            f32::from(bounds.size.height),
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
                let row_id = self.id.slot_index("option", option.value.to_string());
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
                    .id(row_id.clone())
                    .px(tokens.option_padding_x)
                    .py(tokens.option_padding_y)
                    .rounded_sm()
                    .text_size(tokens.option_size)
                    .text_color(resolve_hsla(&self.theme, &tokens.option_fg))
                    .bg(row_bg)
                    .child({
                        let mut label_node = div().flex_1().min_w_0().truncate();
                        if let Some(label) = option.label.clone() {
                            label_node = label_node.child(label);
                        }
                        Stack::horizontal()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .gap(tokens.option_content_gap)
                            .child(label_node)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .flex_none()
                                    .w(tokens.option_check_size)
                                    .h(tokens.option_check_size)
                                    .children(
                                        selected.then_some(
                                            Icon::named("check")
                                                .with_id(self.id.slot_index(
                                                    "selected",
                                                    option.value.to_string(),
                                                ))
                                                .size(f32::from(tokens.option_check_size))
                                                .color(resolve_hsla(&self.theme, &tokens.icon)),
                                        ),
                                    ),
                            )
                    });

                if option.disabled {
                    row = row.opacity(0.45).cursor_default();
                } else {
                    let value = option.value.clone();
                    let on_change = self.on_change.clone();
                    let on_open_change = self.on_open_change.clone();
                    let id = self.id.clone();
                    let value_controlled = self.value_controlled;
                    let opened_controlled = self.opened_controlled;
                    let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                    let activate_handler: ActivateHandler =
                        Rc::new(move |window: &mut Window, cx: &mut gpui::App| {
                            if select_state::apply_single_option_commit(
                                &id,
                                value_controlled,
                                opened_controlled,
                                value.as_ref(),
                            ) {
                                window.refresh();
                            }
                            if let Some(handler) = on_change.as_ref() {
                                (handler)(value.clone(), window, cx);
                            }
                            if let Some(handler) = on_open_change.as_ref() {
                                (handler)(false, window, cx);
                            }
                        });
                    row = apply_interaction_styles(
                        row.cursor_pointer(),
                        InteractionStyles::new()
                            .hover(interaction_style(move |style| style.bg(hover_bg)))
                            .active(interaction_style(move |style| style.bg(press_bg)))
                            .focus(interaction_style(move |style| style.bg(hover_bg))),
                    );
                    row = bind_press_adapter(
                        row,
                        PressAdapter::new(row_id.clone()).on_activate(Some(activate_handler)),
                    );
                }

                row
            })
            .collect::<Vec<_>>();

        let mut dropdown = div()
            .id(self.id.slot("dropdown"))
            .w(px(select_state::dropdown_width_px(
                &self.id,
                f32::from(tokens.dropdown_width_fallback),
            )))
            .rounded_md()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.dropdown_border))
            .bg(resolve_hsla(&self.theme, &tokens.dropdown_bg))
            .shadow_sm()
            .max_h(tokens.dropdown_max_height)
            .overflow_y_scroll()
            .p(tokens.dropdown_padding)
            .child(Stack::vertical().gap(tokens.dropdown_gap).children(items));

        if self.close_on_click_outside {
            if let Some(on_open_change) = self.on_open_change.clone() {
                let id = self.id.clone();
                let opened_controlled = self.opened_controlled;
                dropdown = dropdown.on_mouse_down_out(
                    move |_, window: &mut Window, cx: &mut gpui::App| {
                        if select_state::apply_opened(&id, opened_controlled, false) {
                            window.refresh();
                        }
                        (on_open_change)(false, window, cx);
                    },
                );
            } else if !self.opened_controlled {
                let id = self.id.clone();
                dropdown = dropdown.on_mouse_down_out(
                    move |_, window: &mut Window, _cx: &mut gpui::App| {
                        if select_state::apply_opened(&id, false, false) {
                            window.refresh();
                        }
                    },
                );
            }
        }

        dropdown
            .with_enter_transition(self.id.slot("dropdown-enter"), self.motion)
            .into_any_element()
    }
}

impl Select {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
    fn with_variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn with_radius(mut self, value: Radius) -> Self {
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
        let layout_gap_vertical = self.theme.components.select.layout_gap_vertical;
        let label_block_gap = self.theme.components.select.label_block_gap;
        let dropdown_anchor_offset = self.theme.components.select.dropdown_anchor_offset;
        let layout_gap_horizontal = self.theme.components.select.layout_gap_horizontal;
        let horizontal_label_width = self.theme.components.select.horizontal_label_width;
        let popup_snap_margin = self.theme.components.layout.popup_snap_margin;
        let state = SelectState::resolve(SelectStateInput {
            id: &self.id,
            opened_controlled: self.opened_controlled,
            opened: self.opened,
            default_opened: self.default_opened,
        });
        let opened = state.opened;
        let dropdown_upward = state.dropdown_upward;
        let mut container = Stack::vertical()
            .id(self.id.clone())
            .gap(layout_gap_vertical)
            .relative()
            .w_full();

        if self.layout == FieldLayout::Vertical {
            container = container.child(self.render_label_block());
        }

        let mut field = Stack::vertical().gap(label_block_gap);
        let mut trigger = div()
            .id(self.id.slot("trigger"))
            .relative()
            .w_full()
            .child(self.render_control(window));

        if opened {
            let floating = self.render_dropdown(window);
            let anchor_host = if dropdown_upward {
                anchored_host(
                    &self.id,
                    "anchor-host",
                    PopupPlacement::Top,
                    f32::from(dropdown_anchor_offset),
                    popup_snap_margin,
                    floating,
                    24,
                    true,
                    true,
                )
            } else {
                anchored_host(
                    &self.id,
                    "anchor-host",
                    PopupPlacement::Bottom,
                    f32::from(dropdown_anchor_offset),
                    popup_snap_margin,
                    floating,
                    24,
                    true,
                    true,
                )
            };
            trigger = trigger.child(anchor_host);
        }
        field = field.child(trigger);

        match self.layout {
            FieldLayout::Vertical => container.child(field),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap(layout_gap_horizontal)
                .child(
                    div()
                        .w(horizontal_label_width)
                        .child(self.render_label_block()),
                )
                .child(div().flex_1().child(field)),
        }
    }
}

#[derive(IntoElement)]
pub struct MultiSelect {
    id: ComponentId,
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
            id: ComponentId::default(),
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

    fn selected_labels(&self) -> Vec<SharedString> {
        let values = self
            .resolved_values()
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        self.options
            .iter()
            .filter(|option| select_state::contains(&values, option.value.as_ref()))
            .filter_map(|option| option.label.clone())
            .collect()
    }

    fn resolved_values(&self) -> Vec<SharedString> {
        select_state::resolve_multi_values(
            &self.id,
            self.values_controlled,
            self.values
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
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
        SelectState::resolve(SelectStateInput {
            id: &self.id,
            opened_controlled: self.opened_controlled,
            opened: self.opened,
            default_opened: self.default_opened,
        })
        .opened
    }

    fn render_label_block(&self) -> AnyElement {
        if self.label.is_none() && self.description.is_none() && self.error.is_none() {
            return div().into_any_element();
        }

        let tokens = &self.theme.components.select;
        let mut block = Stack::vertical().gap(tokens.label_block_gap);

        if let Some(label) = self.label.clone() {
            let mut label_row = Stack::horizontal().gap(tokens.label_row_gap).child(
                div()
                    .text_size(tokens.label_size)
                    .font_weight(tokens.label_weight)
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
                    .text_size(tokens.description_size)
                    .text_color(resolve_hsla(&self.theme, &tokens.description))
                    .child(description),
            );
        }

        if let Some(error) = self.error.clone() {
            block = block.child(
                div()
                    .text_size(tokens.error_size)
                    .text_color(resolve_hsla(&self.theme, &tokens.error))
                    .child(error),
            );
        }

        block.into_any_element()
    }

    fn render_control(&mut self, window: &gpui::Window) -> AnyElement {
        let tokens = &self.theme.components.select;
        let dropdown_preferred_height = f32::from(tokens.dropdown_open_preferred_height);
        let opened = self.resolved_opened();
        let control_bg = SelectRuntime::control_bg_for_variant(&self.theme, tokens, self.variant);

        let mut control = div()
            .id(self.id.slot("control"))
            .relative()
            .w_full()
            .flex()
            .items_center()
            .gap(tokens.slot_gap)
            .bg(control_bg)
            .border(super::utils::quantized_stroke_px(window, 1.0));

        control = apply_field_size(control, tokens.sizes.for_size(self.size));
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
        } else {
            let activate_handler = if let Some(handler) = self.on_open_change.clone() {
                let next = !opened;
                let id = self.id.clone();
                let opened_controlled = self.opened_controlled;
                Some(Rc::new(move |window: &mut Window, cx: &mut gpui::App| {
                    if select_state::on_trigger_toggle_without_click(
                        &id,
                        opened_controlled,
                        next,
                        window,
                        dropdown_preferred_height,
                    ) {
                        window.refresh();
                    }
                    (handler)(next, window, cx);
                }) as ActivateHandler)
            } else if !self.opened_controlled {
                let next = !opened;
                let id = self.id.clone();
                Some(Rc::new(move |window: &mut Window, _cx: &mut gpui::App| {
                    if select_state::on_trigger_toggle_without_click(
                        &id,
                        false,
                        next,
                        window,
                        dropdown_preferred_height,
                    ) {
                        window.refresh();
                    }
                }) as ActivateHandler)
            } else {
                None
            };

            if let Some(activate_handler) = activate_handler {
                let hover_bg = control_bg.blend(gpui::white().opacity(0.04));
                let press_bg = control_bg.blend(gpui::black().opacity(0.08));
                let focus_border = if self.error.is_some() {
                    resolve_hsla(&self.theme, &tokens.border_error)
                } else {
                    resolve_hsla(&self.theme, &tokens.border_focus)
                };
                control = apply_interaction_styles(
                    control.cursor_pointer(),
                    InteractionStyles::new()
                        .hover(interaction_style(move |style| style.bg(hover_bg)))
                        .active(interaction_style(move |style| style.bg(press_bg)))
                        .focus(interaction_style(move |style| {
                            style.border_color(focus_border)
                        })),
                );
                control = bind_press_adapter(
                    control,
                    PressAdapter::new(self.id.slot("control")).on_activate(Some(activate_handler)),
                );
            } else {
                control = control.cursor_default();
            }
        }

        if let Some(left_slot) = self.left_slot.take() {
            control = control.child(
                div()
                    .flex_none()
                    .min_w(tokens.slot_min_width)
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
                    .px(tokens.tag_padding_x)
                    .py(tokens.tag_padding_y)
                    .text_size(tokens.tag_size)
                    .rounded_full()
                    .border(super::utils::quantized_stroke_px(window, 1.0))
                    .border_color(resolve_hsla(&self.theme, &tokens.tag_border))
                    .bg(resolve_hsla(&self.theme, &tokens.tag_bg))
                    .text_color(resolve_hsla(&self.theme, &tokens.tag_fg))
                    .child(div().max_w(tokens.tag_max_width).truncate().child(label))
            });

            control = control.child(
                Stack::horizontal()
                    .flex_1()
                    .min_w_0()
                    .gap(tokens.tag_gap)
                    .overflow_hidden()
                    .children(tags),
            );
        }

        if let Some(right_slot) = self.right_slot.take() {
            control = control.child(
                div()
                    .ml_auto()
                    .flex_none()
                    .min_w(tokens.slot_min_width)
                    .text_color(resolve_hsla(&self.theme, &tokens.icon))
                    .child(right_slot()),
            );
        }

        let id_for_width = self.id.clone();
        control
            .child(
                Icon::named(if opened { "chevron-up" } else { "chevron-down" })
                    .with_id(self.id.slot("chevron"))
                    .size(f32::from(tokens.icon_size))
                    .color(resolve_hsla(&self.theme, &tokens.icon)),
            )
            .child(
                canvas(
                    move |bounds, _, _cx| {
                        select_state::set_dropdown_width(
                            &id_for_width,
                            f32::from(bounds.size.width),
                        );
                        select_state::set_trigger_metrics(
                            &id_for_width,
                            f32::from(bounds.origin.y),
                            f32::from(bounds.size.height),
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
                let row_id = self.id.slot_index("option", option.value.to_string());
                let checked = current_values
                    .iter()
                    .any(|selected| selected.as_ref() == option.value.as_ref());
                let row_bg = if checked {
                    resolve_hsla(&self.theme, &tokens.option_selected_bg)
                } else {
                    resolve_hsla(&self.theme, &gpui::transparent_black())
                };
                let hover_bg = resolve_hsla(&self.theme, &tokens.option_hover_bg);

                let mut row = div()
                    .id(row_id.clone())
                    .px(tokens.option_padding_x)
                    .py(tokens.option_padding_y)
                    .rounded_sm()
                    .text_size(tokens.option_size)
                    .text_color(resolve_hsla(&self.theme, &tokens.option_fg))
                    .bg(row_bg)
                    .child({
                        let mut label_node = div().flex_1().min_w_0().truncate();
                        if let Some(label) = option.label.clone() {
                            label_node = label_node.child(label);
                        }
                        Stack::horizontal()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .gap(tokens.option_content_gap)
                            .child(label_node)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .flex_none()
                                    .w(tokens.option_check_size)
                                    .h(tokens.option_check_size)
                                    .children(
                                        checked.then_some(
                                            Icon::named("check")
                                                .with_id(self.id.slot_index(
                                                    "selected",
                                                    option.value.to_string(),
                                                ))
                                                .size(f32::from(tokens.option_check_size))
                                                .color(resolve_hsla(&self.theme, &tokens.icon)),
                                        ),
                                    ),
                            )
                    });

                if option.disabled {
                    row = row.opacity(0.45).cursor_default();
                } else {
                    let value = option.value.clone();
                    let on_change = self.on_change.clone();
                    let selected_values = current_values.clone();
                    let id = self.id.clone();
                    let values_controlled = self.values_controlled;
                    let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                    let activate_handler: ActivateHandler =
                        Rc::new(move |window: &mut Window, cx: &mut gpui::App| {
                            let selected = selected_values
                                .iter()
                                .map(|value| value.to_string())
                                .collect::<Vec<_>>();
                            let updated = select_state::toggled_values(&selected, value.as_ref());
                            if select_state::apply_multi_values(
                                &id,
                                values_controlled,
                                updated.clone(),
                            ) {
                                window.refresh();
                            }
                            if let Some(handler) = on_change.as_ref() {
                                (handler)(
                                    updated.into_iter().map(SharedString::from).collect(),
                                    window,
                                    cx,
                                );
                            }
                        });
                    row = apply_interaction_styles(
                        row.cursor_pointer(),
                        InteractionStyles::new()
                            .hover(interaction_style(move |style| style.bg(hover_bg)))
                            .active(interaction_style(move |style| style.bg(press_bg)))
                            .focus(interaction_style(move |style| style.bg(hover_bg))),
                    );
                    row = bind_press_adapter(
                        row,
                        PressAdapter::new(row_id.clone()).on_activate(Some(activate_handler)),
                    );
                }

                row
            })
            .collect::<Vec<_>>();

        let mut dropdown = div()
            .id(self.id.slot("dropdown"))
            .w(px(select_state::dropdown_width_px(
                &self.id,
                f32::from(tokens.dropdown_width_fallback),
            )))
            .rounded_md()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.dropdown_border))
            .bg(resolve_hsla(&self.theme, &tokens.dropdown_bg))
            .shadow_sm()
            .max_h(tokens.dropdown_max_height)
            .overflow_y_scroll()
            .p(tokens.dropdown_padding)
            .child(Stack::vertical().gap(tokens.dropdown_gap).children(rows));

        if self.close_on_click_outside {
            if let Some(on_open_change) = self.on_open_change.clone() {
                let id = self.id.clone();
                let opened_controlled = self.opened_controlled;
                dropdown = dropdown.on_mouse_down_out(
                    move |_, window: &mut Window, cx: &mut gpui::App| {
                        if select_state::apply_opened(&id, opened_controlled, false) {
                            window.refresh();
                        }
                        (on_open_change)(false, window, cx);
                    },
                );
            } else if !self.opened_controlled {
                let id = self.id.clone();
                dropdown = dropdown.on_mouse_down_out(
                    move |_, window: &mut Window, _cx: &mut gpui::App| {
                        if select_state::apply_opened(&id, false, false) {
                            window.refresh();
                        }
                    },
                );
            }
        }

        dropdown
            .with_enter_transition(self.id.slot("dropdown-enter"), self.motion)
            .into_any_element()
    }
}

impl MultiSelect {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
    fn with_variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn with_radius(mut self, value: Radius) -> Self {
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
        let layout_gap_vertical = self.theme.components.select.layout_gap_vertical;
        let label_block_gap = self.theme.components.select.label_block_gap;
        let dropdown_anchor_offset = self.theme.components.select.dropdown_anchor_offset;
        let layout_gap_horizontal = self.theme.components.select.layout_gap_horizontal;
        let horizontal_label_width = self.theme.components.select.horizontal_label_width;
        let popup_snap_margin = self.theme.components.layout.popup_snap_margin;
        let state = SelectState::resolve(SelectStateInput {
            id: &self.id,
            opened_controlled: self.opened_controlled,
            opened: self.opened,
            default_opened: self.default_opened,
        });
        let opened = state.opened;
        let dropdown_upward = state.dropdown_upward;
        let mut container = Stack::vertical()
            .id(self.id.clone())
            .gap(layout_gap_vertical)
            .relative()
            .w_full();
        if self.layout == FieldLayout::Vertical {
            container = container.child(self.render_label_block());
        }

        let mut field = Stack::vertical().gap(label_block_gap);
        let mut trigger = div()
            .id(self.id.slot("trigger"))
            .relative()
            .w_full()
            .child(self.render_control(window));

        if opened {
            let floating = self.render_dropdown(window);
            let anchor_host = if dropdown_upward {
                anchored_host(
                    &self.id,
                    "anchor-host",
                    PopupPlacement::Top,
                    f32::from(dropdown_anchor_offset),
                    popup_snap_margin,
                    floating,
                    24,
                    true,
                    true,
                )
            } else {
                anchored_host(
                    &self.id,
                    "anchor-host",
                    PopupPlacement::Bottom,
                    f32::from(dropdown_anchor_offset),
                    popup_snap_margin,
                    floating,
                    24,
                    true,
                    true,
                )
            };
            trigger = trigger.child(anchor_host);
        }
        field = field.child(trigger);

        match self.layout {
            FieldLayout::Vertical => container.child(field),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap(layout_gap_horizontal)
                .child(
                    div()
                        .w(horizontal_label_width)
                        .child(self.render_label_block()),
                )
                .child(div().flex_1().child(field)),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn select_id_once() -> String {
        Select::new().id.to_string()
    }

    #[test]
    fn select_default_id_differs_across_callsites() {
        let first = select_id_once();
        let second = { Select::new().id.to_string() };
        assert_ne!(first, second);
    }

    #[test]
    fn select_default_id_reuses_same_callsite() {
        let ids = (0..3).map(|_| select_id_once()).collect::<Vec<_>>();
        assert!(ids.windows(2).all(|pair| pair[0] == pair[1]));
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
