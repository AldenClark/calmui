use super::control;
use super::popup_state::{self, PopupStateInput, PopupStateValue};

pub struct MenuStateInput<'a> {
    pub id: &'a str,
    pub opened: Option<bool>,
    pub default_opened: bool,
    pub disabled: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct MenuState {
    pub opened: bool,
    pub controlled: bool,
    pub dropdown_width_px: f32,
}

impl MenuState {
    pub fn resolve(input: MenuStateInput<'_>) -> Self {
        let popup_state = PopupStateValue::resolve(PopupStateInput {
            id: input.id,
            opened: input.opened,
            default_opened: input.default_opened,
            disabled: input.disabled,
        });
        let width = control::f32_state(input.id, "dropdown-width-px", None, 0.0);

        Self {
            opened: popup_state.opened,
            controlled: popup_state.controlled,
            dropdown_width_px: if width >= 1.0 {
                width.max(180.0)
            } else {
                220.0
            },
        }
    }
}

pub fn set_dropdown_width(id: &str, width_px: f32) {
    control::set_text_state(id, "dropdown-width-px", format!("{width_px:.2}"));
}

pub fn apply_opened(id: &str, controlled: bool, next: bool) -> bool {
    popup_state::apply_opened(id, controlled, next)
}

pub fn on_trigger_toggle(id: &str, controlled: bool, next: bool) -> bool {
    apply_opened(id, controlled, next)
}

pub fn on_item_click(id: &str, controlled: bool, close_on_item_click: bool) -> bool {
    if !close_on_item_click {
        return false;
    }
    apply_opened(id, controlled, false)
}

pub fn on_close_request(id: &str, controlled: bool) -> bool {
    apply_opened(id, controlled, false)
}
