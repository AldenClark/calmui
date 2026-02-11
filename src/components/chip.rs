use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::{MotionAware, ThemeScoped, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};
use crate::theme::{ColorValue, Theme};

use super::control;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::{apply_button_size, apply_radius, resolve_hsla, variant_text_weight};

type ChipChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type ChipGroupChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChipSelectionMode {
    Single,
    Multiple,
}

pub struct Chip {
    id: String,
    value: SharedString,
    label: SharedString,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    size: Size,
    radius: Radius,
    variant: Variant,
    theme: Theme,
    motion: MotionConfig,
    on_change: Option<ChipChangeHandler>,
}

impl Chip {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            id: stable_auto_id("chip"),
            value: label.clone(),
            label,
            checked: None,
            default_checked: false,
            disabled: false,
            size: Size::Sm,
            radius: Radius::Pill,
            variant: Variant::Light,
            theme: Theme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
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

    fn color_tokens(&self) -> (ColorValue, ColorValue, ColorValue) {
        let tokens = &self.theme.components.chip;
        if self.resolved_checked() {
            match self.variant {
                Variant::Filled => (
                    tokens.filled_bg.clone(),
                    tokens.filled_fg.clone(),
                    tokens.filled_bg.clone(),
                ),
                Variant::Light => (
                    tokens.light_bg.clone(),
                    tokens.light_fg.clone(),
                    tokens.light_bg.clone(),
                ),
                Variant::Subtle => (
                    tokens.subtle_bg.clone(),
                    tokens.subtle_fg.clone(),
                    tokens.subtle_bg.clone(),
                ),
                Variant::Outline => (
                    ColorValue::Custom("#00000000".to_string()),
                    tokens.outline_fg.clone(),
                    tokens.outline_border.clone(),
                ),
                Variant::Ghost => (
                    ColorValue::Custom("#00000000".to_string()),
                    tokens.ghost_fg.clone(),
                    ColorValue::Custom("#00000000".to_string()),
                ),
                Variant::Default => (
                    tokens.default_bg.clone(),
                    tokens.default_fg.clone(),
                    tokens.default_border.clone(),
                ),
            }
        } else {
            (
                tokens.unchecked_bg.clone(),
                tokens.unchecked_fg.clone(),
                tokens.unchecked_border.clone(),
            )
        }
    }

    fn resolved_checked(&self) -> bool {
        control::bool_state(&self.id, "checked", self.checked, self.default_checked)
    }
}

impl WithId for Chip {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for Chip {
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

impl MotionAware for Chip {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl ThemeScoped for Chip {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let checked = self.resolved_checked();
        let is_controlled = self.checked.is_some();
        let (bg_token, fg_token, border_token) = self.color_tokens();
        let bg = resolve_hsla(&self.theme, &bg_token);
        let fg = resolve_hsla(&self.theme, &fg_token);
        let border = resolve_hsla(&self.theme, &border_token);

        let mut chip = h_stack()
            .id(self.id.clone())
            .items_center()
            .gap_1()
            .cursor_pointer()
            .text_color(fg)
            .bg(bg)
            .border_1()
            .border_color(border)
            .child({
                let mut content = h_stack().items_center().gap_1();
                if checked {
                    content = content.child(div().text_sm().child("✓"));
                }
                content.child(
                    div()
                        .font_weight(variant_text_weight(self.variant))
                        .child(self.label),
                )
            });

        chip = apply_button_size(chip, self.size);
        chip = apply_radius(chip, self.radius);

        if self.disabled {
            chip = chip.cursor_default().opacity(0.55);
        } else if let Some(handler) = self.on_change.clone() {
            let id = self.id.clone();
            chip = chip.on_click(move |_, window, cx| {
                let next = !checked;
                if !is_controlled {
                    control::set_bool_state(&id, "checked", next);
                    window.refresh();
                }
                (handler)(next, window, cx);
            });
        } else if !is_controlled {
            let id = self.id.clone();
            chip = chip.on_click(move |_, window, _cx| {
                control::set_bool_state(&id, "checked", !checked);
                window.refresh();
            });
        }

        chip.with_enter_transition(format!("{}-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl IntoElement for Chip {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChipOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl ChipOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct ChipGroup {
    id: String,
    options: Vec<ChipOption>,
    mode: ChipSelectionMode,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    values: Vec<SharedString>,
    values_controlled: bool,
    default_values: Vec<SharedString>,
    orientation: GroupOrientation,
    size: Size,
    radius: Radius,
    variant: Variant,
    theme: Theme,
    motion: MotionConfig,
    on_change: Option<ChipGroupChangeHandler>,
}

impl ChipGroup {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("chip-group"),
            options: Vec::new(),
            mode: ChipSelectionMode::Multiple,
            value: None,
            value_controlled: false,
            default_value: None,
            values: Vec::new(),
            values_controlled: false,
            default_values: Vec::new(),
            orientation: GroupOrientation::Horizontal,
            size: Size::Sm,
            radius: Radius::Pill,
            variant: Variant::Light,
            theme: Theme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn option(mut self, option: ChipOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = ChipOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn mode(mut self, mode: ChipSelectionMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
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

    fn contains(values: &[SharedString], value: &SharedString) -> bool {
        values
            .iter()
            .any(|candidate| candidate.as_ref() == value.as_ref())
    }

    fn toggled_values(values: &[SharedString], value: &SharedString) -> Vec<SharedString> {
        let mut set = values
            .iter()
            .map(|value| value.to_string())
            .collect::<BTreeSet<_>>();
        if !set.insert(value.to_string()) {
            set.remove(value.as_ref());
        }
        set.into_iter().map(SharedString::from).collect()
    }

    fn resolved_selected_values(&self) -> Vec<SharedString> {
        match self.mode {
            ChipSelectionMode::Single => {
                let value = control::optional_text_state(
                    &self.id,
                    "value",
                    self.value_controlled
                        .then_some(self.value.as_ref().map(|value| value.to_string())),
                    self.default_value.as_ref().map(|value| value.to_string()),
                );
                value.map(SharedString::from).into_iter().collect()
            }
            ChipSelectionMode::Multiple => control::list_state(
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
            .collect(),
        }
    }
}

impl WithId for ChipGroup {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for ChipGroup {
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

impl MotionAware for ChipGroup {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl ThemeScoped for ChipGroup {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for ChipGroup {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let selected_values = self.resolved_selected_values();
        let single_controlled = self.value_controlled;
        let multiple_controlled = self.values_controlled;

        let chips = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let checked = Self::contains(&selected_values, &option.value);
                let mut chip = Chip::new(option.label)
                    .with_id(format!("{}-option-{index}", self.id))
                    .value(option.value.clone())
                    .checked(checked)
                    .disabled(option.disabled)
                    .variant(self.variant)
                    .size(self.size)
                    .radius(self.radius)
                    .with_theme(self.theme.clone())
                    .motion(self.motion);

                if let Some(handler) = self.on_change.clone() {
                    let value = option.value;
                    let current = selected_values.clone();
                    let mode = self.mode;
                    let id = self.id.clone();
                    chip = chip.on_change(move |next, window, cx| {
                        let updated = match mode {
                            ChipSelectionMode::Single => {
                                if next {
                                    vec![value.clone()]
                                } else {
                                    current.clone()
                                }
                            }
                            ChipSelectionMode::Multiple => Self::toggled_values(&current, &value),
                        };
                        match mode {
                            ChipSelectionMode::Single if !single_controlled => {
                                control::set_optional_text_state(
                                    &id,
                                    "value",
                                    updated.first().map(|value| value.to_string()),
                                );
                                window.refresh();
                            }
                            ChipSelectionMode::Multiple if !multiple_controlled => {
                                control::set_list_state(
                                    &id,
                                    "values",
                                    updated.iter().map(|value| value.to_string()).collect(),
                                );
                                window.refresh();
                            }
                            _ => {}
                        }
                        (handler)(updated, window, cx);
                    });
                } else {
                    let value = option.value;
                    let current = selected_values.clone();
                    let mode = self.mode;
                    let id = self.id.clone();
                    chip = chip.on_change(move |next, window, _cx| {
                        let updated = match mode {
                            ChipSelectionMode::Single => {
                                if next {
                                    vec![value.clone()]
                                } else {
                                    current.clone()
                                }
                            }
                            ChipSelectionMode::Multiple => Self::toggled_values(&current, &value),
                        };
                        match mode {
                            ChipSelectionMode::Single if !single_controlled => {
                                control::set_optional_text_state(
                                    &id,
                                    "value",
                                    updated.first().map(|value| value.to_string()),
                                );
                                window.refresh();
                            }
                            ChipSelectionMode::Multiple if !multiple_controlled => {
                                control::set_list_state(
                                    &id,
                                    "values",
                                    updated.iter().map(|value| value.to_string()).collect(),
                                );
                                window.refresh();
                            }
                            _ => {}
                        }
                    });
                }

                chip.into_any_element()
            })
            .collect::<Vec<_>>();

        match self.orientation {
            GroupOrientation::Horizontal => h_stack()
                .id(self.id.clone())
                .gap_2()
                .flex_wrap()
                .children(chips)
                .into_any_element(),
            GroupOrientation::Vertical => v_stack()
                .id(self.id.clone())
                .gap_2()
                .children(chips)
                .into_any_element(),
        }
    }
}

impl IntoElement for ChipGroup {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}
