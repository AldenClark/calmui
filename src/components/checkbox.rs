use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, Radiused, Sized, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type CheckboxChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;
type CheckboxGroupChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct Checkbox {
    id: ComponentId,
    value: SharedString,
    label: SharedString,
    description: Option<SharedString>,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<CheckboxChangeHandler>,
}

impl Checkbox {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            id: ComponentId::default(),
            value: label.clone(),
            label,
            description: None,
            checked: None,
            default_checked: false,
            disabled: false,
            size: Size::Md,
            radius: Radius::Xs,
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

impl Checkbox {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Checkbox {
    fn with_variant(self, _value: Variant) -> Self {
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
        let size = self.control_size_px();
        let is_focused = control::focused_state(&self.id, None, false);
        let border = if is_focused {
            resolve_hsla(&self.theme, &tokens.border_focus)
        } else if checked {
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
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.indicator))
                    .child("✓"),
            );
        }

        let mut row = Stack::horizontal()
            .id(self.id.clone())
            .focusable()
            .cursor_pointer()
            .child(
                Stack::vertical()
                    .gap_0p5()
                    .child(
                        Stack::horizontal()
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
            let handler_for_click = handler.clone();
            let handler_for_key = handler.clone();
            let id = self.id.clone();
            let id_for_key = self.id.clone();
            let id_for_blur = self.id.clone();
            row = row
                .on_click(move |_, window, cx| {
                    control::set_focused_state(&id, true);
                    window.refresh();
                    let next = !checked;
                    if !is_controlled {
                        control::set_bool_state(&id, "checked", next);
                        window.refresh();
                    }
                    (handler_for_click)(next, window, cx);
                })
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if control::is_activation_key(key) {
                        control::set_focused_state(&id_for_key, true);
                        window.refresh();
                        let next = !checked;
                        if !is_controlled {
                            control::set_bool_state(&id_for_key, "checked", next);
                            window.refresh();
                        }
                        (handler_for_key)(next, window, cx);
                    }
                })
                .on_mouse_down_out(move |_, window, _cx| {
                    control::set_focused_state(&id_for_blur, false);
                    window.refresh();
                });
        } else if !is_controlled {
            let id = self.id.clone();
            let id_for_key = self.id.clone();
            let id_for_blur = self.id.clone();
            row = row
                .on_click(move |_, window, _cx| {
                    control::set_focused_state(&id, true);
                    control::set_bool_state(&id, "checked", !checked);
                    window.refresh();
                })
                .on_key_down(move |event, window, _cx| {
                    let key = event.keystroke.key.as_str();
                    if control::is_activation_key(key) {
                        control::set_focused_state(&id_for_key, true);
                        control::set_bool_state(&id_for_key, "checked", !checked);
                        window.refresh();
                    }
                })
                .on_mouse_down_out(move |_, window, _cx| {
                    control::set_focused_state(&id_for_blur, false);
                    window.refresh();
                });
        } else {
            let id = self.id.clone();
            let id_for_key = self.id.clone();
            let id_for_blur = self.id.clone();
            row = row
                .on_click(move |_, window, _cx| {
                    control::set_focused_state(&id, true);
                    window.refresh();
                })
                .on_key_down(move |event, window, _cx| {
                    let key = event.keystroke.key.as_str();
                    if control::is_activation_key(key) {
                        control::set_focused_state(&id_for_key, true);
                        window.refresh();
                    }
                })
                .on_mouse_down_out(move |_, window, _cx| {
                    control::set_focused_state(&id_for_blur, false);
                    window.refresh();
                });
        }

        row.with_enter_transition(self.id.slot("enter"), self.motion)
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

#[derive(IntoElement)]
pub struct CheckboxGroup {
    id: ComponentId,
    options: Vec<CheckboxOption>,
    values: Vec<SharedString>,
    values_controlled: bool,
    default_values: Vec<SharedString>,
    orientation: GroupOrientation,
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

impl CheckboxGroup {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for CheckboxGroup {
    fn with_variant(self, _value: Variant) -> Self {
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
        let values = self.resolved_values();
        let is_controlled = self.values_controlled;
        let items = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let checked = Self::contains_value(&values, &option.value);
                let mut checkbox = Checkbox::new(option.label)
                    .with_id(self.id.slot_index("option", index.to_string()))
                    .value(option.value.clone())
                    .checked(checked)
                    .disabled(option.disabled);
                checkbox = Sized::with_size(checkbox, self.size);
                checkbox = Radiused::with_radius(checkbox, self.radius);
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
                .gap_3()
                .flex_wrap()
                .children(items),
            GroupOrientation::Vertical => Stack::vertical()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap_2()
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
