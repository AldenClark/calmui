use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, ThemeScoped, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};
use crate::theme::Theme;

use super::control;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type CheckboxChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type CheckboxGroupChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

pub struct Checkbox {
    id: String,
    value: SharedString,
    label: SharedString,
    description: Option<SharedString>,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    size: Size,
    radius: Radius,
    theme: Theme,
    motion: MotionConfig,
    on_change: Option<CheckboxChangeHandler>,
}

impl Checkbox {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            id: stable_auto_id("checkbox"),
            value: label.clone(),
            label,
            description: None,
            checked: None,
            default_checked: false,
            disabled: false,
            size: Size::Md,
            radius: Radius::Xs,
            theme: Theme::default(),
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

impl WithId for Checkbox {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for Checkbox {
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

impl MotionAware for Checkbox {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl ThemeScoped for Checkbox {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let checked = self.resolved_checked();
        let is_controlled = self.checked.is_some();
        let tokens = &self.theme.components.checkbox;
        let size = self.control_size_px();
        let border = if checked {
            resolve_hsla(&self.theme, &tokens.border_checked)
        } else {
            resolve_hsla(&self.theme, &tokens.border)
        };
        let bg = if checked {
            resolve_hsla(&self.theme, &tokens.control_bg_checked)
        } else {
            resolve_hsla(&self.theme, &tokens.control_bg)
        };
        let fg = resolve_hsla(&self.theme, &tokens.label);
        let muted = resolve_hsla(&self.theme, &tokens.description);

        let mut control = div()
            .w(px(size))
            .h(px(size))
            .border_1()
            .border_color(border)
            .bg(bg)
            .flex()
            .items_center()
            .justify_center();
        control = apply_radius(control, self.radius);

        if checked {
            control = control.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.indicator))
                    .child("✓"),
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
                        .ml(px(size + 8.0))
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
                let next = !checked;
                if !is_controlled {
                    control::set_bool_state(&id, "checked", next);
                    window.refresh();
                }
                (handler)(next, window, cx);
            });
        } else if !is_controlled {
            let id = self.id.clone();
            row = row.on_click(move |_, window, _cx| {
                control::set_bool_state(&id, "checked", !checked);
                window.refresh();
            });
        }

        row.with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Checkbox {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckboxOption {
    pub value: SharedString,
    pub label: SharedString,
    pub description: Option<SharedString>,
    pub disabled: bool,
}

impl CheckboxOption {
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

pub struct CheckboxGroup {
    id: String,
    options: Vec<CheckboxOption>,
    values: Vec<SharedString>,
    values_controlled: bool,
    default_values: Vec<SharedString>,
    orientation: GroupOrientation,
    size: Size,
    radius: Radius,
    theme: Theme,
    motion: MotionConfig,
    on_change: Option<CheckboxGroupChangeHandler>,
}

impl CheckboxGroup {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("checkbox-group"),
            options: Vec::new(),
            values: Vec::new(),
            values_controlled: false,
            default_values: Vec::new(),
            orientation: GroupOrientation::Vertical,
            size: Size::Md,
            radius: Radius::Xs,
            theme: Theme::default(),
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
}

impl WithId for CheckboxGroup {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for CheckboxGroup {
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

impl MotionAware for CheckboxGroup {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl ThemeScoped for CheckboxGroup {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for CheckboxGroup {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let values = self.resolved_values();
        let is_controlled = self.values_controlled;
        let items = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let checked = Self::contains_value(&values, &option.value);
                let mut checkbox = Checkbox::new(option.label)
                    .with_id(format!("{}-option-{index}", self.id))
                    .value(option.value.clone())
                    .checked(checked)
                    .disabled(option.disabled)
                    .size(self.size)
                    .radius(self.radius)
                    .with_theme(self.theme.clone())
                    .motion(self.motion);

                if let Some(description) = option.description {
                    checkbox = checkbox.description(description);
                }

                let value = option.value;
                let current = values.clone();
                let on_change = self.on_change.clone();
                let id = self.id.clone();
                checkbox = checkbox.on_change(move |_, window, cx| {
                    let next = Self::toggled_values(&current, &value);
                    if !is_controlled {
                        control::set_list_state(
                            &id,
                            "values",
                            next.iter().map(|value| value.to_string()).collect(),
                        );
                        window.refresh();
                    }
                    if let Some(handler) = on_change.as_ref() {
                        (handler)(next, window, cx);
                    }
                });
                checkbox.into_any_element()
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
                .children(items),
            GroupOrientation::Vertical => v_stack().id(self.id.clone()).gap_2().children(items),
        }
    }
}

impl IntoElement for CheckboxGroup {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}
