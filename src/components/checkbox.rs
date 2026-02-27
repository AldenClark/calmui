use std::{collections::BTreeSet, rc::Rc};

use gpui::InteractiveElement;
use gpui::{
    AnyElement, IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, px,
};

use crate::contracts::Disableable as _;
use crate::contracts::{FieldLike, MotionAware, Radiused, Sized, Varianted};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::selection_state;
use super::toggle::{ToggleConfig, wire_toggle_handlers};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type CheckboxChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type CheckboxGroupChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

fn group_meta_line(color: gpui::Hsla, text: SharedString) -> AnyElement {
    div().text_color(color).child(text).into_any_element()
}

#[derive(IntoElement)]
pub struct Checkbox {
    pub(crate) id: ComponentId,
    value: SharedString,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    variant: Variant,
    size: Size,
    radius: Radius,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<CheckboxChangeHandler>,
}

impl Checkbox {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            value: SharedString::from(""),
            label: None,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            checked: None,
            default_checked: false,
            disabled: false,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Xs,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    #[track_caller]
    pub fn labeled(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self::new().value(label.clone()).label(label)
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }

    pub fn clear_label(mut self) -> Self {
        self.label = None;
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
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

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }
    pub fn on_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_checked(&self) -> bool {
        control::bool_state(&self.id, "checked", self.checked, self.default_checked)
    }

    fn variant_accent_color(&self, base: gpui::Hsla) -> gpui::Hsla {
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.9),
            Variant::Subtle => base.alpha(0.75),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.64),
        }
    }

    fn variant_border_color(&self, base: gpui::Hsla) -> gpui::Hsla {
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.9),
            Variant::Subtle => base.alpha(0.78),
            Variant::Outline => base,
            Variant::Ghost => base.alpha(0.58),
        }
    }

    fn variant_surface_color(&self, base: gpui::Hsla) -> gpui::Hsla {
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.92),
            Variant::Subtle => base.alpha(0.8),
            Variant::Outline | Variant::Ghost => gpui::transparent_black(),
        }
    }
}

impl Checkbox {}

crate::impl_variant_size_radius_via_methods!(Checkbox, variant, size, radius);

impl MotionAware for Checkbox {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let checked = self.resolved_checked();
        let is_controlled = self.checked.is_some();
        let tokens = &self.theme.components.checkbox;
        let size_preset = tokens.sizes.for_size(self.size);
        let size = f32::from(size_preset.control_size);
        let is_focused = control::focused_state(&self.id, None, false);
        let base_border = resolve_hsla(&self.theme, tokens.border);
        let base_checked_border = resolve_hsla(&self.theme, tokens.border_checked);
        let base_focus_border = resolve_hsla(&self.theme, tokens.border_focus);
        let border = if is_focused {
            self.variant_accent_color(base_focus_border)
        } else if checked {
            self.variant_accent_color(base_checked_border)
        } else {
            self.variant_border_color(base_border)
        };
        let bg = self.variant_surface_color(if checked {
            resolve_hsla(&self.theme, tokens.control_bg_checked)
        } else {
            resolve_hsla(&self.theme, tokens.control_bg)
        });
        let fg = resolve_hsla(&self.theme, tokens.label);
        let muted = resolve_hsla(&self.theme, tokens.description);
        let error_fg = resolve_hsla(&self.theme, self.theme.semantic.status_error);

        let mut control = div()
            .w(px(size))
            .h(px(size))
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(border)
            .bg(bg)
            .flex()
            .items_center()
            .justify_center();
        control = apply_radius(&self.theme, control, self.radius);
        if !self.disabled {
            let hover_border = resolve_hsla(&self.theme, tokens.border_hover);
            control = control.hover(move |style| style.border_color(hover_border));
        }

        if checked {
            control = control
                .text_size(size_preset.indicator_size)
                .text_color(self.variant_accent_color(resolve_hsla(&self.theme, tokens.indicator)))
                .child("✓");
        }

        let label_text = self.label.clone().map(|label| {
            if self.required {
                SharedString::from(format!("{label} *"))
            } else {
                label
            }
        });

        let primary = Stack::horizontal()
            .items_center()
            .gap(size_preset.content_gap)
            .children(
                Some(control.into_any_element())
                    .into_iter()
                    .chain(label_text.map(|label| {
                        div()
                            .text_size(size_preset.label_size)
                            .text_color(fg)
                            .child(label)
                            .into_any_element()
                    })),
            );

        let support = if self.description.is_some() || self.error.is_some() {
            let mut rows = Vec::<AnyElement>::new();
            if let Some(description) = self.description.clone() {
                let row = div()
                    .text_size(size_preset.description_size)
                    .text_color(muted)
                    .child(description);
                if self.layout == FieldLayout::Vertical {
                    rows.push(
                        row.ml(px(size + f32::from(size_preset.description_indent_gap)))
                            .into_any_element(),
                    );
                } else {
                    rows.push(row.into_any_element());
                }
            }
            if let Some(error) = self.error.clone() {
                let row = div()
                    .text_size(size_preset.description_size)
                    .text_color(error_fg)
                    .child(error);
                if self.layout == FieldLayout::Vertical {
                    rows.push(
                        row.ml(px(size + f32::from(size_preset.description_indent_gap)))
                            .into_any_element(),
                    );
                } else {
                    rows.push(row.into_any_element());
                }
            }
            Some(
                Stack::vertical()
                    .gap(tokens.label_description_gap)
                    .children(rows)
                    .into_any_element(),
            )
        } else {
            None
        };

        let mut content_children = Some(vec![primary.into_any_element()]);
        if let Some(support) = support {
            content_children
                .as_mut()
                .expect("content children vector is initialized")
                .push(support);
        }
        let content = match self.layout {
            FieldLayout::Vertical => Stack::vertical()
                .gap(tokens.label_description_gap)
                .children(content_children.take().expect("content children available"))
                .into_any_element(),
            FieldLayout::Horizontal => Stack::horizontal()
                .items_start()
                .gap(tokens.label_description_gap)
                .children(content_children.take().expect("content children available"))
                .into_any_element(),
        };

        let mut row = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .focusable()
            .cursor_pointer()
            .child(content);

        if self.disabled {
            row = row.cursor_default().opacity(0.55);
        } else {
            row = wire_toggle_handlers(
                row,
                ToggleConfig {
                    id: self.id.clone(),
                    checked,
                    controlled: is_controlled,
                    allow_uncheck: true,
                    on_change: self.on_change.clone(),
                },
            );
        }

        row.with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckboxOption {
    pub value: SharedString,
    pub label: Option<SharedString>,
    pub description: Option<SharedString>,
    pub disabled: bool,
}

impl CheckboxOption {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: None,
            description: None,
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

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(IntoElement)]
pub struct CheckboxGroup {
    pub(crate) id: ComponentId,
    options: Vec<CheckboxOption>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    values: Vec<SharedString>,
    values_controlled: bool,
    default_values: Vec<SharedString>,
    orientation: GroupOrientation,
    variant: Variant,
    size: Size,
    radius: Radius,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<CheckboxGroupChangeHandler>,
}

impl CheckboxGroup {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            options: Vec::new(),
            label: None,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            values: Vec::new(),
            values_controlled: false,
            default_values: Vec::new(),
            orientation: GroupOrientation::Vertical,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Xs,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn option(mut self, option: CheckboxOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = CheckboxOption>) -> Self {
        self.options.extend(options);
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

    pub fn values(mut self, values: impl IntoIterator<Item = SharedString>) -> Self {
        self.values = values.into_iter().collect();
        self.values_controlled = true;
        self
    }

    pub fn default_values(mut self, values: impl IntoIterator<Item = SharedString>) -> Self {
        self.default_values = values.into_iter().collect();
        self
    }

    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn contains_value(values: &[SharedString], target: &SharedString) -> bool {
        values.iter().any(|value| value.as_ref() == target.as_ref())
    }

    fn toggled_values(values: &[SharedString], target: &SharedString) -> Vec<SharedString> {
        let mut set = values
            .iter()
            .map(|value| value.to_string())
            .collect::<BTreeSet<_>>();
        if !set.insert(target.to_string()) {
            set.remove(target.as_ref());
        }
        set.into_iter().map(SharedString::from).collect()
    }

    fn resolved_values(&self) -> Vec<SharedString> {
        selection_state::resolve_list(
            &self.id,
            "values",
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
}

impl CheckboxGroup {}

crate::impl_variant_size_radius_via_methods!(CheckboxGroup, variant, size, radius);

impl MotionAware for CheckboxGroup {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for CheckboxGroup {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.checkbox;
        let label_color = resolve_hsla(&self.theme, tokens.label);
        let description_color = resolve_hsla(&self.theme, tokens.description);
        let error_color = resolve_hsla(&self.theme, self.theme.semantic.status_error);
        let values = self.resolved_values();
        let is_controlled = self.values_controlled;
        let items = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let checked = Self::contains_value(&values, &option.value);
                let mut checkbox = self
                    .id
                    .ctx()
                    .child_index("option", index.to_string(), Checkbox::new())
                    .value(option.value.clone())
                    .checked(checked)
                    .disabled(option.disabled);
                if let Some(label) = option.label.clone() {
                    checkbox = checkbox.label(label);
                }
                checkbox = Sized::with_size(checkbox, self.size);
                checkbox = Radiused::with_radius(checkbox, self.radius);
                checkbox = Varianted::with_variant(checkbox, self.variant);
                checkbox = checkbox.motion(self.motion);

                if let Some(description) = option.description {
                    checkbox = checkbox.description(description);
                }

                let value = option.value;
                let current = values.clone();
                let on_change = self.on_change.clone();
                let id = self.id.clone();
                checkbox = checkbox.on_change(move |_, window, cx| {
                    let next = Self::toggled_values(&current, &value);
                    if selection_state::apply_list(
                        &id,
                        "values",
                        is_controlled,
                        next.iter().map(|value| value.to_string()).collect(),
                    ) {
                        window.refresh();
                    }
                    if let Some(handler) = on_change.as_ref() {
                        (handler)(next, window, cx);
                    }
                });
                checkbox.into_any_element()
            })
            .collect::<Vec<_>>();

        let group = match self.orientation {
            GroupOrientation::Horizontal => div()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .flex()
                .flex_row()
                .items_start()
                .gap(tokens.group_gap_horizontal)
                .flex_wrap()
                .children(items)
                .into_any_element(),
            GroupOrientation::Vertical => Stack::vertical()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap(tokens.group_gap_vertical)
                .children(items)
                .into_any_element(),
        };

        let label = self.label.clone().map(|value| {
            if self.required {
                SharedString::from(format!("{value} *"))
            } else {
                value
            }
        });
        let has_meta = label.is_some() || self.description.is_some() || self.error.is_some();
        if !has_meta {
            return group;
        }

        let mut meta_rows = Vec::<AnyElement>::new();
        if let Some(label) = label {
            meta_rows.push(group_meta_line(label_color, label));
        }
        if let Some(description) = self.description {
            meta_rows.push(group_meta_line(description_color, description));
        }
        if let Some(error) = self.error {
            meta_rows.push(group_meta_line(error_color, error));
        }
        let meta = Stack::vertical()
            .gap(tokens.label_description_gap)
            .children(meta_rows)
            .into_any_element();

        match self.layout {
            FieldLayout::Vertical => Stack::vertical()
                .id(self.id.clone())
                .gap(tokens.label_description_gap)
                .children(vec![meta, group])
                .into_any_element(),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap(tokens.label_description_gap)
                .children(vec![meta, group])
                .into_any_element(),
        }
    }
}

crate::impl_disableable!(Checkbox, |this, value| this.disabled = value);
crate::impl_disableable!(CheckboxOption, |this, value| this.disabled = value);

impl FieldLike for Checkbox {
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

impl FieldLike for CheckboxGroup {
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
