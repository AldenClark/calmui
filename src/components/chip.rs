use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::{MotionAware, Radiused, Sized, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::toggle::{ToggleConfig, wire_toggle_handlers};
use super::transition::TransitionExt;
use super::utils::{apply_button_size, apply_radius, resolve_hsla, variant_text_weight};

type ChipChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type ChipGroupChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChipSelectionMode {
    Single,
    Multiple,
}

#[derive(IntoElement)]
pub struct Chip {
    id: ComponentId,
    value: SharedString,
    label: SharedString,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    size: Size,
    radius: Radius,
    variant: Variant,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChipChangeHandler>,
}

impl Chip {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            id: ComponentId::default(),
            value: label.clone(),
            label,
            checked: None,
            default_checked: false,
            disabled: false,
            size: Size::Sm,
            radius: Radius::Pill,
            variant: Variant::Light,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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

    fn color_tokens(&self) -> (Hsla, Hsla, Hsla) {
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
                    gpui::transparent_black(),
                    tokens.outline_fg.clone(),
                    tokens.outline_border.clone(),
                ),
                Variant::Ghost => (
                    gpui::transparent_black(),
                    tokens.ghost_fg.clone(),
                    gpui::transparent_black(),
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

impl Chip {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Chip {
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

impl MotionAware for Chip {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Chip {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let checked = self.resolved_checked();
        let is_controlled = self.checked.is_some();
        let is_focused = control::focused_state(&self.id, None, false);
        let tokens = &self.theme.components.chip;
        let (bg_token, fg_token, border_token) = self.color_tokens();
        let bg = resolve_hsla(&self.theme, &bg_token);
        let fg = resolve_hsla(&self.theme, &fg_token);
        let border = if is_focused {
            resolve_hsla(&self.theme, &tokens.border_focus)
        } else {
            resolve_hsla(&self.theme, &border_token)
        };

        let mut chip = Stack::horizontal()
            .id(self.id.clone())
            .focusable()
            .items_center()
            .gap_1()
            .cursor_pointer()
            .text_color(fg)
            .bg(bg)
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(border)
            .child({
                let mut content = Stack::horizontal().items_center().gap_1();
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
        chip = apply_radius(&self.theme, chip, self.radius);

        if self.disabled {
            chip = chip.cursor_default().opacity(0.55);
        } else {
            let hover_border = resolve_hsla(&self.theme, &tokens.border_hover);
            chip = chip.hover(move |style| style.border_color(hover_border));
            chip = wire_toggle_handlers(
                chip,
                ToggleConfig {
                    id: self.id.clone(),
                    checked,
                    controlled: is_controlled,
                    allow_uncheck: true,
                    on_change: self.on_change.clone(),
                },
            );
        }

        chip.with_enter_transition(self.id.slot("enter"), self.motion)
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
#[derive(IntoElement)]
pub struct ChipGroup {
    id: ComponentId,
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
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChipGroupChangeHandler>,
}

impl ChipGroup {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
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
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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

impl ChipGroup {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for ChipGroup {
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

impl MotionAware for ChipGroup {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for ChipGroup {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
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
                    .with_id(self.id.slot_index("option", index.to_string()))
                    .value(option.value.clone())
                    .checked(checked)
                    .disabled(option.disabled)
                    .with_variant(self.variant);
                chip = Sized::with_size(chip, self.size);
                chip = Radiused::with_radius(chip, self.radius);
                chip = chip.motion(self.motion);

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

                div().group(self.id.clone()).child(chip)
            })
            .collect::<Vec<_>>();

        match self.orientation {
            GroupOrientation::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap_2()
                .flex_wrap()
                .children(chips),
            GroupOrientation::Vertical => Stack::vertical()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap_2()
                .children(chips),
        }
    }
}

impl crate::contracts::ComponentThemeOverridable for Chip {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for ChipGroup {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Chip);
crate::impl_disableable!(ChipOption);

impl gpui::Styled for Chip {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for ChipGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
