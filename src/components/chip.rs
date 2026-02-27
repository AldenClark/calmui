use std::{collections::BTreeSet, rc::Rc};

use gpui::InteractiveElement;
use gpui::{
    Hsla, IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div,
};

use crate::contracts::Disableable as _;
use crate::contracts::{FieldLike, MotionAware, Radiused, Sized, Varianted};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::icon::Icon;
use super::selection_state;
use super::toggle::{ToggleConfig, wire_toggle_handlers};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla, variant_text_weight};

type ChipChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type ChipGroupChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChipSelectionMode {
    Single,
    Multiple,
}

#[derive(IntoElement)]
pub struct Chip {
    pub(crate) id: ComponentId,
    value: SharedString,
    label: Option<SharedString>,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    size: Size,
    radius: Radius,
    variant: Variant,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<ChipChangeHandler>,
}

impl Chip {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            value: SharedString::from(""),
            label: None,
            checked: None,
            default_checked: false,
            disabled: false,
            size: Size::Sm,
            radius: Radius::Pill,
            variant: Variant::Light,
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

    fn color_tokens(&self) -> (Hsla, Hsla, Hsla) {
        let tokens = &self.theme.components.chip;
        if self.resolved_checked() {
            match self.variant {
                Variant::Filled => (tokens.filled_bg, tokens.filled_fg, tokens.filled_bg),
                Variant::Light => (tokens.light_bg, tokens.light_fg, tokens.light_bg),
                Variant::Subtle => (tokens.subtle_bg, tokens.subtle_fg, tokens.subtle_bg),
                Variant::Outline => (
                    gpui::transparent_black(),
                    tokens.outline_fg,
                    tokens.outline_border,
                ),
                Variant::Ghost => (
                    gpui::transparent_black(),
                    tokens.ghost_fg,
                    gpui::transparent_black(),
                ),
                Variant::Default => (tokens.default_bg, tokens.default_fg, tokens.default_border),
            }
        } else {
            (
                tokens.unchecked_bg,
                tokens.unchecked_fg,
                tokens.unchecked_border,
            )
        }
    }

    fn resolved_checked(&self) -> bool {
        control::bool_state(&self.id, "checked", self.checked, self.default_checked)
    }
}

impl Chip {}

crate::impl_variant_size_radius_via_methods!(Chip, variant, size, radius);

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
        let size_preset = tokens.sizes.for_size(self.size);
        let (bg_token, fg_token, border_token) = self.color_tokens();
        let bg = resolve_hsla(&self.theme, bg_token);
        let fg = resolve_hsla(&self.theme, fg_token);
        let border = if is_focused {
            resolve_hsla(&self.theme, tokens.border_focus)
        } else {
            resolve_hsla(&self.theme, border_token)
        };

        let mut content = Stack::horizontal().items_center().gap(tokens.content_gap);
        if checked {
            content = content.child(
                self.id
                    .ctx()
                    .child("indicator", Icon::named("check"))
                    .size(f32::from(tokens.indicator_size))
                    .color(fg),
            );
        }
        if let Some(label) = self.label.clone() {
            content = content.child(label);
        }

        let mut chip = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(tokens.content_gap)
            .focusable()
            .cursor_pointer()
            .text_color(fg)
            .bg(bg)
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(border)
            .font_weight(variant_text_weight(self.variant))
            .text_size(size_preset.font_size)
            .line_height(size_preset.line_height)
            .py(size_preset.padding_y)
            .px(size_preset.padding_x)
            .child(content);
        chip = apply_radius(&self.theme, chip, self.radius);

        if self.disabled {
            chip = chip.cursor_default().opacity(0.55);
        } else {
            let hover_border = resolve_hsla(&self.theme, tokens.border_hover);
            chip = chip.hover(move |style: gpui::StyleRefinement| style.border_color(hover_border));
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
    pub label: Option<SharedString>,
    pub disabled: bool,
}

impl ChipOption {
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
}
#[derive(IntoElement)]
pub struct ChipGroup {
    pub(crate) id: ComponentId,
    options: Vec<ChipOption>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
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
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<ChipGroupChangeHandler>,
}

impl ChipGroup {
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
                let value = selection_state::resolve_optional_text(
                    &self.id,
                    "value",
                    self.value_controlled,
                    self.value.as_ref().map(|value| value.to_string()),
                    self.default_value.as_ref().map(|value| value.to_string()),
                );
                value.map(SharedString::from).into_iter().collect()
            }
            ChipSelectionMode::Multiple => selection_state::resolve_list(
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
            .collect(),
        }
    }
}

impl ChipGroup {}

crate::impl_variant_size_radius_via_methods!(ChipGroup, variant, size, radius);

impl MotionAware for ChipGroup {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for ChipGroup {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let group_gap_horizontal = self.theme.components.chip.group_gap_horizontal;
        let group_gap_vertical = self.theme.components.chip.group_gap_vertical;
        let label_color = resolve_hsla(&self.theme, self.theme.components.chip.unchecked_fg);
        let description_color = label_color.alpha(0.78);
        let error_color = resolve_hsla(&self.theme, self.theme.semantic.status_error);
        let selected_values = self.resolved_selected_values();
        let single_controlled = self.value_controlled;
        let multiple_controlled = self.values_controlled;

        let chips = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let checked = Self::contains(&selected_values, &option.value);
                let mut chip = self
                    .id
                    .ctx()
                    .child_index("option", index.to_string(), Chip::new())
                    .value(option.value.clone())
                    .checked(checked)
                    .disabled(option.disabled)
                    .with_variant(self.variant);
                if let Some(label) = option.label.clone() {
                    chip = chip.label(label);
                }
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
                                let refresh = selection_state::apply_optional_text(
                                    &id,
                                    "value",
                                    false,
                                    updated.first().map(|value| value.to_string()),
                                );
                                if refresh {
                                    window.refresh();
                                }
                            }
                            ChipSelectionMode::Multiple if !multiple_controlled => {
                                let refresh = selection_state::apply_list(
                                    &id,
                                    "values",
                                    false,
                                    updated.iter().map(|value| value.to_string()).collect(),
                                );
                                if refresh {
                                    window.refresh();
                                }
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
                                let refresh = selection_state::apply_optional_text(
                                    &id,
                                    "value",
                                    false,
                                    updated.first().map(|value| value.to_string()),
                                );
                                if refresh {
                                    window.refresh();
                                }
                            }
                            ChipSelectionMode::Multiple if !multiple_controlled => {
                                let refresh = selection_state::apply_list(
                                    &id,
                                    "values",
                                    false,
                                    updated.iter().map(|value| value.to_string()).collect(),
                                );
                                if refresh {
                                    window.refresh();
                                }
                            }
                            _ => {}
                        }
                    });
                }

                div().group(self.id.clone()).child(chip)
            })
            .collect::<Vec<_>>();

        let group = match self.orientation {
            GroupOrientation::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap(group_gap_horizontal)
                .flex_wrap()
                .children(chips),
            GroupOrientation::Vertical => Stack::vertical()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap(group_gap_vertical)
                .children(chips),
        }
        .into_any_element();

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

        let mut meta = Stack::vertical().gap(group_gap_vertical);
        if let Some(label) = label {
            meta = meta.child(div().text_color(label_color).child(label));
        }
        if let Some(description) = self.description {
            meta = meta.child(div().text_color(description_color).child(description));
        }
        if let Some(error) = self.error {
            meta = meta.child(div().text_color(error_color).child(error));
        }

        match self.layout {
            FieldLayout::Vertical => Stack::vertical()
                .id(self.id.clone())
                .gap(group_gap_vertical)
                .child(meta)
                .child(group)
                .into_any_element(),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap(group_gap_horizontal)
                .child(meta)
                .child(group)
                .into_any_element(),
        }
    }
}

crate::impl_disableable!(Chip, |this, value| this.disabled = value);
crate::impl_disableable!(ChipOption, |this, value| this.disabled = value);

impl FieldLike for ChipGroup {
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
