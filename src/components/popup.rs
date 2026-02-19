use gpui::{
    AnyElement, Corner, InteractiveElement, IntoElement, ParentElement, Styled, anchored, deferred,
    div, point, px,
};

use crate::id::ComponentId;

use super::control;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopupPlacement {
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PopupState {
    pub opened: bool,
    pub controlled: bool,
}

impl PopupState {
    pub fn resolve(id: &str, opened: Option<bool>, default_opened: bool) -> Self {
        Self {
            opened: control::bool_state(id, "opened", opened, default_opened),
            controlled: opened.is_some(),
        }
    }
}

pub fn anchored_host(
    id: &ComponentId,
    slot: &str,
    placement: PopupPlacement,
    offset_px: f32,
    snap_margin: gpui::Pixels,
    floating: AnyElement,
    priority: usize,
    snap_to_window: bool,
    full_width_host: bool,
) -> AnyElement {
    let anchor_corner = match placement {
        PopupPlacement::Top => Corner::BottomLeft,
        PopupPlacement::Bottom => Corner::TopLeft,
    };
    let offset = match placement {
        PopupPlacement::Top => point(px(0.0), px(-offset_px.max(0.0))),
        PopupPlacement::Bottom => point(px(0.0), px(offset_px.max(0.0))),
    };
    let host_slot = id.slot(slot.to_owned());
    let anchored_panel = if snap_to_window {
        anchored()
            .anchor(anchor_corner)
            .offset(offset)
            .snap_to_window_with_margin(snap_margin)
            .child(floating)
    } else {
        anchored()
            .anchor(anchor_corner)
            .offset(offset)
            .child(floating)
    };
    let mut host = match placement {
        PopupPlacement::Top => div().id(host_slot).absolute().top_0().left_0(),
        PopupPlacement::Bottom => div().id(host_slot).absolute().bottom_0().left_0(),
    };
    if full_width_host {
        host = host.w_full().h(px(0.0));
    } else {
        host = host.w(px(0.0)).h(px(0.0));
    }
    host.child(deferred(anchored_panel).priority(priority))
        .into_any_element()
}
