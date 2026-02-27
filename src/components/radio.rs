use std::rc::Rc;

use gpui::InteractiveElement;
use gpui::{
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, px,
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
use super::utils::resolve_hsla;

type RadioChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type RadioGroupChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct Radio {
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
    on_change: Option<RadioChangeHandler>,
}

impl Radio {
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
            radius: Radius::Pill,
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

impl Radio {}

crate::impl_variant_size_radius_via_methods!(Radio, variant, size, radius);

impl MotionAware for Radio {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Radio {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let checked = self.resolved_checked();
        let is_controlled = self.checked.is_some();
        let tokens = &self.theme.components.radio;
        let size_preset = tokens.sizes.for_size(self.size);
        let dot_size = f32::from(size_preset.control_size);
        let indicator_size = f32::from(size_preset.indicator_size);
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
        let fg = resolve_hsla(&self.theme, tokens.label);
        let muted = resolve_hsla(&self.theme, tokens.description);
        let error_fg = resolve_hsla(&self.theme, self.theme.semantic.status_error);

        let mut control = div()
            .w(px(dot_size))
            .h(px(dot_size))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(border)
            .bg(self.variant_surface_color(resolve_hsla(&self.theme, tokens.control_bg)));
        if !self.disabled {
            let hover_border = resolve_hsla(&self.theme, tokens.border_hover);
            control = control.hover(move |style| style.border_color(hover_border));
        }
        if checked {
            control = control.child(
                div()
                    .w(px(indicator_size))
                    .h(px(indicator_size))
                    .rounded_full()
                    .bg(self.variant_accent_color(resolve_hsla(&self.theme, tokens.indicator))),
            );
        }

        let label_text = self.label.clone().map(|label| {
            if self.required {
                SharedString::from(format!("{label} *"))
            } else {
                label
            }
        });

        let primary = {
            let mut content = Stack::horizontal()
                .items_center()
                .gap(size_preset.content_gap)
                .child(control);
            if let Some(label) = label_text {
                content = content.child(
                    div()
                        .text_size(size_preset.label_size)
                        .text_color(fg)
                        .child(label),
                );
            }
            content
        };

        let support = if self.description.is_some() || self.error.is_some() {
            let mut block = Stack::vertical().gap(tokens.label_description_gap);
            if let Some(description) = self.description.clone() {
                let row = div()
                    .text_size(size_preset.description_size)
                    .text_color(muted)
                    .child(description);
                if self.layout == FieldLayout::Vertical {
                    block = block.child(
                        row.ml(px(dot_size + f32::from(size_preset.description_indent_gap))),
                    );
                } else {
                    block = block.child(row);
                }
            }
            if let Some(error) = self.error.clone() {
                let row = div()
                    .text_size(size_preset.description_size)
                    .text_color(error_fg)
                    .child(error);
                if self.layout == FieldLayout::Vertical {
                    block = block.child(
                        row.ml(px(dot_size + f32::from(size_preset.description_indent_gap))),
                    );
                } else {
                    block = block.child(row);
                }
            }
            Some(block.into_any_element())
        } else {
            None
        };

        let content = match self.layout {
            FieldLayout::Vertical => {
                let mut block = Stack::vertical()
                    .gap(tokens.label_description_gap)
                    .child(primary);
                if let Some(support) = support {
                    block = block.child(support);
                }
                block.into_any_element()
            }
            FieldLayout::Horizontal => {
                let mut block = Stack::horizontal()
                    .items_start()
                    .gap(tokens.label_description_gap)
                    .child(primary);
                if let Some(support) = support {
                    block = block.child(support);
                }
                block.into_any_element()
            }
        };

        let mut row = Stack::horizontal()
            .id(self.id.clone())
            .focusable()
            .child(content);

        if self.disabled {
            row = row.opacity(0.55);
        } else {
            row = wire_toggle_handlers(
                row,
                ToggleConfig {
                    id: self.id.clone(),
                    checked,
                    controlled: is_controlled,
                    allow_uncheck: false,
                    on_change: self.on_change.clone(),
                },
            );
        }

        row.with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioOption {
    pub value: SharedString,
    pub label: Option<SharedString>,
    pub description: Option<SharedString>,
    pub disabled: bool,
}

impl RadioOption {
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
pub struct RadioGroup {
    pub(crate) id: ComponentId,
    options: Vec<RadioOption>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    orientation: GroupOrientation,
    variant: Variant,
    size: Size,
    radius: Radius,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<RadioGroupChangeHandler>,
}

impl RadioGroup {
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
            value: None,
            value_controlled: false,
            default_value: None,
            orientation: GroupOrientation::Vertical,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn option(mut self, option: RadioOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = RadioOption>) -> Self {
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

    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_value(&self) -> Option<SharedString> {
        selection_state::resolve_optional_text(
            &self.id,
            "value",
            self.value_controlled,
            self.value.as_ref().map(|value| value.to_string()),
            self.default_value.as_ref().map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }
}

impl RadioGroup {}

crate::impl_variant_size_radius_via_methods!(RadioGroup, variant, size, radius);

impl MotionAware for RadioGroup {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.radio;
        let label_color = resolve_hsla(&self.theme, tokens.label);
        let description_color = resolve_hsla(&self.theme, tokens.description);
        let error_color = resolve_hsla(&self.theme, self.theme.semantic.status_error);
        let selected_value = self.resolved_value();
        let is_controlled = self.value_controlled;
        let radios = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let checked = selected_value
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == option.value.as_ref());
                let mut radio = self
                    .id
                    .ctx()
                    .child_index("option", index.to_string(), Radio::new())
                    .value(option.value.clone())
                    .checked(checked)
                    .disabled(option.disabled);
                if let Some(label) = option.label.clone() {
                    radio = radio.label(label);
                }
                radio = Sized::with_size(radio, self.size);
                radio = Radiused::with_radius(radio, self.radius);
                radio = Varianted::with_variant(radio, self.variant);
                radio = radio.motion(self.motion);

                if let Some(description) = option.description {
                    radio = radio.description(description);
                }

                let value = option.value;
                let on_change = self.on_change.clone();
                let id = self.id.clone();
                radio = radio.on_change(move |next, window, cx| {
                    if next {
                        if selection_state::apply_optional_text(
                            &id,
                            "value",
                            is_controlled,
                            Some(value.to_string()),
                        ) {
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(value.clone(), window, cx);
                        }
                    }
                });
                div().group(self.id.clone()).child(radio)
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
                .children(radios)
                .into_any_element(),
            GroupOrientation::Vertical => Stack::vertical()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap(tokens.group_gap_vertical)
                .children(radios)
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

        let mut meta = Stack::vertical().gap(tokens.label_description_gap);
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
                .gap(tokens.label_description_gap)
                .child(meta)
                .child(group)
                .into_any_element(),
            FieldLayout::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .items_start()
                .gap(tokens.label_description_gap)
                .child(meta)
                .child(group)
                .into_any_element(),
        }
    }
}

crate::impl_disableable!(Radio, |this, value| this.disabled = value);
crate::impl_disableable!(RadioOption, |this, value| this.disabled = value);

impl FieldLike for Radio {
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

impl FieldLike for RadioGroup {
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
