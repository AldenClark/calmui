use std::rc::Rc;

use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};

use super::control;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type RadioChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type RadioGroupChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

pub struct Radio {
    id: String,
    value: SharedString,
    label: SharedString,
    description: Option<SharedString>,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<RadioChangeHandler>,
}

impl Radio {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            id: stable_auto_id("radio"),
            value: label.clone(),
            label,
            description: None,
            checked: None,
            default_checked: false,
            disabled: false,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
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

    fn control_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 12.0,
            Size::Sm => 14.0,
            Size::Md => 16.0,
            Size::Lg => 18.0,
            Size::Xl => 20.0,
        }
    }

    fn resolved_checked(&self) -> bool {
        control::bool_state(&self.id, "checked", self.checked, self.default_checked)
    }
}

impl WithId for Radio {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for Radio {
    fn variant(self, _value: Variant) -> Self {
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

impl MotionAware for Radio {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Radio {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let checked = self.resolved_checked();
        let is_controlled = self.checked.is_some();
        let tokens = &self.theme.components.radio;
        let dot_size = self.control_size_px();
        let indicator_size = (dot_size * 0.45).max(4.0);
        let border = if checked {
            resolve_hsla(&self.theme, &tokens.border_checked)
        } else {
            resolve_hsla(&self.theme, &tokens.border)
        };
        let fg = resolve_hsla(&self.theme, &tokens.label);
        let muted = resolve_hsla(&self.theme, &tokens.description);

        let mut control = div()
            .w(px(dot_size))
            .h(px(dot_size))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .border_1()
            .border_color(border)
            .bg(resolve_hsla(&self.theme, &tokens.control_bg));
        if checked {
            control = control.child(
                div()
                    .w(px(indicator_size))
                    .h(px(indicator_size))
                    .rounded_full()
                    .bg(resolve_hsla(&self.theme, &tokens.indicator)),
            );
        }

        let mut row = h_stack().id(self.id.clone()).cursor_pointer().child(
            v_stack()
                .gap_0p5()
                .child(
                    h_stack()
                        .items_center()
                        .gap_2()
                        .child(control)
                        .child(div().text_color(fg).child(self.label)),
                )
                .children(self.description.map(|description| {
                    div()
                        .ml(px(dot_size + 8.0))
                        .text_sm()
                        .text_color(muted)
                        .child(description)
                })),
        );

        if self.disabled {
            row = row.cursor_default().opacity(0.55);
        } else if let Some(handler) = self.on_change.clone() {
            let id = self.id.clone();
            row = row.on_click(move |_, window, cx| {
                if !checked {
                    if !is_controlled {
                        control::set_bool_state(&id, "checked", true);
                        window.refresh();
                    }
                    (handler)(true, window, cx);
                }
            });
        } else if !is_controlled {
            let id = self.id.clone();
            row = row.on_click(move |_, window, _cx| {
                if !checked {
                    control::set_bool_state(&id, "checked", true);
                    window.refresh();
                }
            });
        }

        row.with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Radio {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioOption {
    pub value: SharedString,
    pub label: SharedString,
    pub description: Option<SharedString>,
    pub disabled: bool,
}

impl RadioOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            disabled: false,
        }
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

pub struct RadioGroup {
    id: String,
    options: Vec<RadioOption>,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    orientation: GroupOrientation,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<RadioGroupChangeHandler>,
}

impl RadioGroup {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("radio-group"),
            options: Vec::new(),
            value: None,
            value_controlled: false,
            default_value: None,
            orientation: GroupOrientation::Vertical,
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
        control::optional_text_state(
            &self.id,
            "value",
            self.value_controlled
                .then_some(self.value.as_ref().map(|value| value.to_string())),
            self.default_value.as_ref().map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }
}

impl WithId for RadioGroup {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for RadioGroup {
    fn variant(self, _value: Variant) -> Self {
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

impl MotionAware for RadioGroup {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
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
                let mut radio = Radio::new(option.label)
                    .with_id(format!("{}-option-{index}", self.id))
                    .value(option.value.clone())
                    .checked(checked)
                    .disabled(option.disabled)
                    .size(self.size)
                    .radius(self.radius)
                    .motion(self.motion);

                if let Some(description) = option.description {
                    radio = radio.description(description);
                }

                let value = option.value;
                let on_change = self.on_change.clone();
                let id = self.id.clone();
                radio = radio.on_change(move |next, window, cx| {
                    if next {
                        if !is_controlled {
                            control::set_optional_text_state(&id, "value", Some(value.to_string()));
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(value.clone(), window, cx);
                        }
                    }
                });
                radio.into_any_element()
            })
            .collect::<Vec<_>>();

        match self.orientation {
            GroupOrientation::Horizontal => div()
                .id(self.id.clone())
                .flex()
                .flex_row()
                .items_start()
                .gap_3()
                .flex_wrap()
                .children(radios),
            GroupOrientation::Vertical => v_stack().id(self.id.clone()).gap_2().children(radios),
        }
    }
}

impl IntoElement for RadioGroup {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Radio {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemePatchable for RadioGroup {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
