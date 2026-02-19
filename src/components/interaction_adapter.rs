use std::rc::Rc;

use gpui::{ClickEvent, FocusHandle, InteractiveElement, StatefulInteractiveElement, Window};

use crate::id::ComponentId;

use super::control;

pub type ActivateHandler = Rc<dyn Fn(&mut Window, &mut gpui::App)>;
pub type ClickActivateHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>;

#[derive(Clone, Default)]
pub struct PressAdapter {
    pub id: ComponentId,
    pub focus_handle: Option<FocusHandle>,
    pub on_activate: Option<ActivateHandler>,
    pub on_click: Option<ClickActivateHandler>,
}

impl PressAdapter {
    pub fn new(id: impl Into<ComponentId>) -> Self {
        Self {
            id: id.into(),
            focus_handle: None,
            on_activate: None,
            on_click: None,
        }
    }

    pub fn on_activate(mut self, value: Option<ActivateHandler>) -> Self {
        self.on_activate = value;
        self
    }

    pub fn on_click(mut self, value: Option<ClickActivateHandler>) -> Self {
        self.on_click = value;
        self
    }

    pub fn focus_handle(mut self, value: Option<FocusHandle>) -> Self {
        self.focus_handle = value;
        self
    }
}

pub fn bind_press_adapter<T>(mut node: T, adapter: PressAdapter) -> T
where
    T: InteractiveElement + StatefulInteractiveElement,
{
    if adapter.on_activate.is_none() && adapter.on_click.is_none() {
        return node;
    }

    node = node.focusable();
    if let Some(focus_handle) = adapter.focus_handle.as_ref() {
        node = node.track_focus(focus_handle);
    }

    let click_handler = adapter.on_click.clone();
    let activate_handler = adapter.on_activate.clone();
    let id_for_click = adapter.id.clone();
    let focus_for_click = adapter.focus_handle.clone();
    node = node.on_click(move |event, window, cx| {
        control::set_focused_state(&id_for_click, true);
        if let Some(focus_handle) = focus_for_click.as_ref() {
            window.focus(focus_handle, cx);
        }
        if let Some(handler) = click_handler.as_ref() {
            (handler)(event, window, cx);
        } else if let Some(handler) = activate_handler.as_ref() {
            (handler)(window, cx);
        }
        window.refresh();
    });

    let click_handler = adapter.on_click.clone();
    let activate_handler = adapter.on_activate.clone();
    let id_for_key = adapter.id.clone();
    let focus_for_key = adapter.focus_handle.clone();
    node = node.on_key_down(move |event, window, cx| {
        if !control::is_activation_keystroke(event) {
            return;
        }
        control::set_focused_state(&id_for_key, true);
        if let Some(focus_handle) = focus_for_key.as_ref() {
            window.focus(focus_handle, cx);
        }
        if let Some(handler) = activate_handler.as_ref() {
            (handler)(window, cx);
        } else if let Some(handler) = click_handler.as_ref() {
            (handler)(&ClickEvent::default(), window, cx);
        }
        window.refresh();
        cx.stop_propagation();
    });

    let id_for_blur = adapter.id.clone();
    node = node.on_mouse_down_out(move |_, window, _cx| {
        control::set_focused_state(&id_for_blur, false);
        window.refresh();
    });

    node
}
