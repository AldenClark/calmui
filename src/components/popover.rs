use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;

use super::Stack;
use super::popup::{PopupPlacement, PopupState, anchored_host};
use super::transition::TransitionExt;
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
    id: ComponentId,
    opened: Option<bool>,
    default_opened: bool,
    disabled: bool,
    placement: PopoverPlacement,
    offset_px: f32,
    close_on_click_outside: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
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
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            trigger: None,
            content: None,
            on_open_change: None,
        }
    }

    pub fn opened(mut self, value: bool) -> Self {
        self.opened = Some(value);
        self
    }

    pub fn default_opened(mut self, value: bool) -> Self {
        self.default_opened = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
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
        let tokens = &self.theme.components.popover;
        let mut panel = Stack::vertical()
            .id(self.id.slot("panel"))
            .gap_2()
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.border))
            .text_color(resolve_hsla(&self.theme, &tokens.body))
            .rounded_md()
            .p_3();

        if self.close_on_click_outside {
            if let Some(handler) = self.on_open_change.clone() {
                let id = self.id.clone();
                panel = panel.on_mouse_down_out(move |_, window, cx| {
                    if !is_controlled {
                        super::control::set_bool_state(&id, "opened", false);
                        window.refresh();
                    }
                    (handler)(false, window, cx);
                });
            } else if !is_controlled {
                let id = self.id.clone();
                panel = panel.on_mouse_down_out(move |_, window, _cx| {
                    super::control::set_bool_state(&id, "opened", false);
                    window.refresh();
                });
            }
        }

        if let Some(content) = self.content.take() {
            panel = panel.child(content());
        }

        panel
            .with_enter_transition(self.id.slot("panel-enter"), self.motion)
            .into_any_element()
    }
}

impl Popover {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl MotionAware for Popover {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Popover {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let popup_state = PopupState::resolve(&self.id, self.opened, self.default_opened);
        let opened = !self.disabled && popup_state.opened;
        let is_controlled = popup_state.controlled;

        let mut trigger = div().id(self.id.slot("trigger")).relative().child(
            self.trigger
                .take()
                .map(|content| content())
                .unwrap_or_else(|| div().child("Open").into_any_element()),
        );

        if self.disabled {
            trigger = trigger.cursor_default().opacity(0.55);
        } else if let Some(handler) = self.on_open_change.clone() {
            let next = !opened;
            let id = self.id.clone();
            trigger = trigger.cursor_pointer();
            trigger = trigger.on_click(
                move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                    if !is_controlled {
                        super::control::set_bool_state(&id, "opened", next);
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
                    super::control::set_bool_state(&id, "opened", next);
                    window.refresh();
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

impl crate::contracts::ComponentThemeOverridable for Popover {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Popover);
crate::impl_openable!(Popover);
crate::impl_placeable!(Popover, PopoverPlacement);

impl gpui::Styled for Popover {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
