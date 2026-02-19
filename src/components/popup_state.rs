use super::control;
use super::popup::PopupState;

pub struct PopupStateInput<'a> {
    pub id: &'a str,
    pub opened: Option<bool>,
    pub default_opened: bool,
    pub disabled: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct PopupStateValue {
    pub opened: bool,
    pub controlled: bool,
}

impl PopupStateValue {
    pub fn resolve(input: PopupStateInput<'_>) -> Self {
        let popup_state = PopupState::resolve(input.id, input.opened, input.default_opened);
        Self {
            opened: if input.disabled {
                false
            } else {
                popup_state.opened
            },
            controlled: popup_state.controlled,
        }
    }
}

pub fn apply_opened(id: &str, controlled: bool, next: bool) -> bool {
    if controlled {
        return false;
    }
    control::set_bool_state(id, "opened", next);
    true
}

pub fn on_close_request(id: &str, controlled: bool) -> bool {
    apply_opened(id, controlled, false)
}

pub fn on_open_request(id: &str, controlled: bool) -> bool {
    apply_opened(id, controlled, true)
}
