use std::rc::Rc;

use gpui::InteractiveElement;
use gpui::StatefulInteractiveElement;
use gpui::{AnyElement, ClickEvent, IntoElement, ParentElement, RenderOnce, Styled, Window, div};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;

use super::Stack;
use super::popup::{PopupPlacement, anchored_host};
use super::popup_state::{self, PopupStateInput, PopupStateValue};
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type OpenChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopoverPlacement {
    Top,
    Bottom,
}

#[derive(IntoElement)]
pub struct Popover {
    pub(crate) id: ComponentId,
    opened: Option<bool>,
    default_opened: bool,
    disabled: bool,
    placement: PopoverPlacement,
    offset_px: f32,
    close_on_click_outside: bool,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    trigger: Option<SlotRenderer>,
    content: Option<SlotRenderer>,
    on_open_change: Option<OpenChangeHandler>,
}

impl Popover {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            opened: None,
            default_opened: false,
            disabled: false,
            placement: PopoverPlacement::Bottom,
            offset_px: 3.0,
            close_on_click_outside: true,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            trigger: None,
            content: None,
            on_open_change: None,
        }
    }
    pub fn default_opened(mut self, value: bool) -> Self {
        self.default_opened = value;
        self
    }
    pub fn placement(mut self, value: PopoverPlacement) -> Self {
        self.placement = value;
        self
    }

    pub fn offset(mut self, value: f32) -> Self {
        self.offset_px = value.max(0.0);
        self
    }

    pub fn close_on_click_outside(mut self, value: bool) -> Self {
        self.close_on_click_outside = value;
        self
    }

    pub fn trigger(mut self, trigger: impl IntoElement + 'static) -> Self {
        self.trigger = Some(Box::new(|| trigger.into_any_element()));
        self
    }

    pub fn content(mut self, content: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }

    fn render_panel(&mut self, is_controlled: bool, window: &gpui::Window) -> AnyElement {
        let tokens = self.theme.components.popover.clone();
        let mut panel = Stack::vertical()
            .id(self.id.slot("panel"))
            .gap(tokens.gap)
            .bg(resolve_hsla(&self.theme, tokens.bg))
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, tokens.border))
            .rounded(tokens.radius)
            .p(tokens.padding);

        if self.close_on_click_outside {
            if let Some(handler) = self.on_open_change.clone() {
                let id = self.id.clone();
                panel = panel.on_mouse_down_out(move |_, window: &mut Window, cx| {
                    if popup_state::on_close_request(&id, is_controlled) {
                        window.refresh();
                    }
                    (handler)(false, window, cx);
                });
            } else if !is_controlled {
                let id = self.id.clone();
                panel = panel.on_mouse_down_out(move |_, window: &mut Window, _cx| {
                    if popup_state::on_close_request(&id, false) {
                        window.refresh();
                    }
                });
            }
        }

        if let Some(content) = self.content.take() {
            panel = panel.child(content());
        }

        div()
            .text_color(resolve_hsla(&self.theme, tokens.body))
            .child(panel.with_enter_transition(self.id.slot("panel-enter"), self.motion))
            .into_any_element()
    }
}

impl Popover {}

impl MotionAware for Popover {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Popover {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let popup_state = PopupStateValue::resolve(PopupStateInput {
            id: &self.id,
            opened: self.opened,
            default_opened: self.default_opened,
            disabled: self.disabled,
        });
        let opened = popup_state.opened;
        let is_controlled = popup_state.controlled;

        let mut trigger = div().id(self.id.slot("trigger")).relative();
        if let Some(content) = self.trigger.take() {
            trigger = trigger.child(content());
        } else {
            trigger = trigger.child("Open");
        }

        if self.disabled {
            trigger = trigger.cursor_default().opacity(0.55);
        } else if let Some(handler) = self.on_open_change.clone() {
            let next = !opened;
            let id = self.id.clone();
            trigger = trigger.cursor_pointer();
            trigger = trigger.on_click(
                move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                    if popup_state::apply_opened(&id, is_controlled, next) {
                        window.refresh();
                    }
                    (handler)(next, window, cx);
                },
            );
        } else if !is_controlled {
            let next = !opened;
            let id = self.id.clone();
            trigger = trigger.cursor_pointer();
            trigger = trigger.on_click(
                move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                    if popup_state::apply_opened(&id, false, next) {
                        window.refresh();
                    }
                },
            );
        } else {
            trigger = trigger.cursor_pointer();
        }

        if opened {
            let panel = self.render_panel(is_controlled, window);
            let placement = match self.placement {
                PopoverPlacement::Top => PopupPlacement::Top,
                PopoverPlacement::Bottom => PopupPlacement::Bottom,
            };
            let anchor_host = anchored_host(
                &self.id,
                "anchor-host",
                placement,
                self.offset_px,
                self.theme.components.layout.popup_snap_margin,
                panel,
                20,
                matches!(self.placement, PopoverPlacement::Bottom),
                false,
            );

            trigger = trigger.child(anchor_host);
        }

        div().id(self.id.clone()).relative().child(trigger)
    }
}

crate::impl_disableable!(Popover, |this, value| this.disabled = value);
crate::impl_openable!(Popover, |this, value| this.opened = Some(value));
crate::impl_placeable!(Popover, PopoverPlacement);
