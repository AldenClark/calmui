use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, Radiused, Sized, VariantConfigurable, Varianted};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::selection_state;
use super::toggle::{ToggleConfig, wire_toggle_handlers};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type CheckboxChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type CheckboxGroupChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct Checkbox {
    id: ComponentId,
    value: SharedString,
    label: Option<SharedString>,
    description: Option<SharedString>,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
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
            checked: None,
            default_checked: false,
            disabled: false,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Xs,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl Checkbox {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Checkbox {
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
        let base_border = resolve_hsla(&self.theme, &tokens.border);
        let base_checked_border = resolve_hsla(&self.theme, &tokens.border_checked);
        let base_focus_border = resolve_hsla(&self.theme, &tokens.border_focus);
        let border = if is_focused {
            self.variant_accent_color(base_focus_border)
        } else if checked {
            self.variant_accent_color(base_checked_border)
        } else {
            self.variant_border_color(base_border)
        };
        let bg = self.variant_surface_color(if checked {
            resolve_hsla(&self.theme, &tokens.control_bg_checked)
        } else {
            resolve_hsla(&self.theme, &tokens.control_bg)
        });
        let fg = resolve_hsla(&self.theme, &tokens.label);
        let muted = resolve_hsla(&self.theme, &tokens.description);

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
            let hover_border = resolve_hsla(&self.theme, &tokens.border_hover);
            control = control.hover(move |style| style.border_color(hover_border));
        }

        if checked {
            control = control.child(
                div()
                    .text_size(size_preset.indicator_size)
                    .text_color(
                        self.variant_accent_color(resolve_hsla(&self.theme, &tokens.indicator)),
                    )
                    .child("✓"),
            );
        }

        let mut row = Stack::horizontal()
            .id(self.id.clone())
            .focusable()
            .cursor_pointer()
            .child(
                Stack::vertical()
                    .gap(tokens.label_description_gap)
                    .child({
                        let mut content = Stack::horizontal()
                            .items_center()
                            .gap(size_preset.content_gap)
                            .child(control);
                        if let Some(label) = self.label {
                            content = content.child(
                                div()
                                    .text_size(size_preset.label_size)
                                    .text_color(fg)
                                    .child(label),
                            );
                        }
                        content
                    })
                    .children(self.description.map(|description| {
                        div()
                            .ml(px(size + f32::from(size_preset.description_indent_gap)))
                            .text_size(size_preset.description_size)
                            .text_color(muted)
                            .child(description)
                    })),
            );

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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(IntoElement)]
pub struct CheckboxGroup {
    id: ComponentId,
    options: Vec<CheckboxOption>,
    values: Vec<SharedString>,
    values_controlled: bool,
    default_values: Vec<SharedString>,
    orientation: GroupOrientation,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<CheckboxGroupChangeHandler>,
}

impl CheckboxGroup {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            options: Vec::new(),
            values: Vec::new(),
            values_controlled: false,
            default_values: Vec::new(),
            orientation: GroupOrientation::Vertical,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Xs,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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

impl CheckboxGroup {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for CheckboxGroup {
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
        let values = self.resolved_values();
        let is_controlled = self.values_controlled;
        let items = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let checked = Self::contains_value(&values, &option.value);
                let mut checkbox = Checkbox::new()
                    .with_id(self.id.slot_index("option", index.to_string()))
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
                div().group(self.id.clone()).child(checkbox)
            })
            .collect::<Vec<_>>();

        match self.orientation {
            GroupOrientation::Horizontal => div()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .flex()
                .flex_row()
                .items_start()
                .gap(tokens.group_gap_horizontal)
                .flex_wrap()
                .children(items),
            GroupOrientation::Vertical => Stack::vertical()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap(tokens.group_gap_vertical)
                .children(items),
        }
    }
}

impl crate::contracts::ComponentThemeOverridable for Checkbox {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for CheckboxGroup {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Checkbox);
crate::impl_disableable!(CheckboxOption);

impl gpui::Styled for Checkbox {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for CheckboxGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
