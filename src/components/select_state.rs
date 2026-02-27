use std::collections::BTreeSet;

use gpui::Window;

use super::control;
use super::popup_state::{self, PopupStateInput, PopupStateValue};
use super::selection_state;

pub struct SelectStateInput<'a> {
    pub id: &'a str,
    pub opened_controlled: bool,
    pub opened: Option<bool>,
    pub default_opened: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SelectState {
    pub opened: bool,
    pub dropdown_upward: bool,
}

impl SelectState {
    pub fn resolve(input: SelectStateInput<'_>) -> Self {
        let popup_state = PopupStateValue::resolve(PopupStateInput {
            id: input.id,
            opened: input
                .opened_controlled
                .then_some(input.opened.unwrap_or(false)),
            default_opened: input.default_opened,
            disabled: false,
        });

        Self {
            opened: popup_state.opened,
            dropdown_upward: control::bool_state(input.id, "dropdown-upward", None, false),
        }
    }
}

pub fn resolve_single_value(
    id: &str,
    value_controlled: bool,
    controlled_value: Option<String>,
    default_value: Option<String>,
) -> Option<String> {
    selection_state::resolve_optional_text(
        id,
        "value",
        value_controlled,
        controlled_value,
        default_value,
    )
}

pub fn resolve_multi_values(
    id: &str,
    values_controlled: bool,
    controlled_values: Vec<String>,
    default_values: Vec<String>,
) -> Vec<String> {
    selection_state::resolve_list(
        id,
        "values",
        values_controlled,
        controlled_values,
        default_values,
    )
}

pub fn contains(values: &[String], value: &str) -> bool {
    values.iter().any(|candidate| candidate == value)
}

pub fn toggled_values(values: &[String], value: &str) -> Vec<String> {
    let mut set = values.iter().cloned().collect::<BTreeSet<_>>();
    if !set.insert(value.to_string()) {
        set.remove(value);
    }
    set.into_iter().collect::<Vec<_>>()
}

pub fn apply_single_value(id: &str, value_controlled: bool, next: Option<String>) -> bool {
    selection_state::apply_optional_text(id, "value", value_controlled, next)
}

pub fn apply_multi_values(id: &str, values_controlled: bool, next: Vec<String>) -> bool {
    selection_state::apply_list(id, "values", values_controlled, next)
}

pub fn apply_opened(id: &str, opened_controlled: bool, next: bool) -> bool {
    popup_state::apply_opened(id, opened_controlled, next)
}

pub fn apply_single_option_commit(
    id: &str,
    value_controlled: bool,
    opened_controlled: bool,
    next_value: &str,
) -> bool {
    let mut refresh = false;
    refresh |= apply_single_value(id, value_controlled, Some(next_value.to_string()));
    refresh |= apply_opened(id, opened_controlled, false);
    refresh
}

pub fn set_dropdown_width(id: &str, width_px: f32) {
    control::set_text_state(id, "dropdown-width-px", format!("{width_px:.2}"));
}

pub fn set_trigger_metrics(id: &str, origin_y: f32, height: f32) {
    control::set_f32_state(id, "trigger-origin-y", origin_y);
    control::set_f32_state(id, "trigger-height", height);
}

pub fn dropdown_width_px(id: &str, fallback_width: f32) -> f32 {
    let width = control::f32_state(id, "dropdown-width-px", None, 0.0);
    if width >= 1.0 { width } else { fallback_width }
}

pub fn capture_dropdown_metrics_without_click(id: &str, window: &Window, preferred_height: f32) {
    control::set_bool_state(
        id,
        "dropdown-upward",
        should_open_dropdown_upward_from_trigger(id, window, preferred_height),
    );
}

pub fn on_trigger_toggle_without_click(
    id: &str,
    opened_controlled: bool,
    next_opened: bool,
    window: &Window,
    preferred_height: f32,
) -> bool {
    if next_opened {
        capture_dropdown_metrics_without_click(id, window, preferred_height);
    }
    apply_opened(id, opened_controlled, next_opened)
}

fn should_open_dropdown_upward_from_trigger(
    id: &str,
    window: &Window,
    preferred_height: f32,
) -> bool {
    let trigger_origin_y = control::f32_state(id, "trigger-origin-y", None, 0.0);
    let trigger_height = control::f32_state(id, "trigger-height", None, 0.0);
    let anchor_y = trigger_origin_y + (trigger_height * 0.5);
    let viewport_height = f32::from(window.viewport_size().height);
    let space_above = anchor_y.max(0.0);
    let space_below = (viewport_height - anchor_y).max(0.0);

    space_below < preferred_height && space_above > space_below
}
