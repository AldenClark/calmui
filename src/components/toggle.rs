use std::rc::Rc;

use gpui::{InteractiveElement, StatefulInteractiveElement, Window};

use crate::id::ComponentId;

use super::control;

pub type ToggleChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone)]
pub struct ToggleConfig {
    pub id: ComponentId,
    pub checked: bool,
    pub controlled: bool,
    pub allow_uncheck: bool,
    pub on_change: Option<ToggleChangeHandler>,
}

impl ToggleConfig {
    fn next_checked(&self) -> bool {
        if self.allow_uncheck {
            !self.checked
        } else {
            true
        }
    }

    fn should_emit(&self, next: bool) -> bool {
        next != self.checked
    }
}

pub fn wire_toggle_handlers<T>(node: T, config: ToggleConfig) -> T
where
    T: InteractiveElement + StatefulInteractiveElement,
{
    let click_cfg = config.clone();
    let key_cfg = config.clone();
    let id = config.id.clone();
    let id_for_key = config.id.clone();
    let id_for_blur = config.id.clone();
    let click_next = click_cfg.next_checked();
    let key_next = key_cfg.next_checked();

    node.on_click(move |_, window, cx| {
        control::set_focused_state(&id, true);
        window.refresh();

        if !click_cfg.controlled && click_cfg.should_emit(click_next) {
            control::set_bool_state(&id, "checked", click_next);
            window.refresh();
        }

        if let Some(handler) = click_cfg.on_change.as_ref() {
            if click_cfg.should_emit(click_next) {
                (handler)(click_next, window, cx);
            }
        }
    })
    .on_key_down(move |event, window, cx| {
        if control::is_activation_keystroke(event) {
            control::set_focused_state(&id_for_key, true);
            window.refresh();

            if !key_cfg.controlled && key_cfg.should_emit(key_next) {
                control::set_bool_state(&id_for_key, "checked", key_next);
                window.refresh();
            }

            if let Some(handler) = key_cfg.on_change.as_ref() {
                if key_cfg.should_emit(key_next) {
                    (handler)(key_next, window, cx);
                }
            }
        }
    })
    .on_mouse_down_out(move |_, window, _cx| {
        control::set_focused_state(&id_for_blur, false);
        window.refresh();
    })
}
