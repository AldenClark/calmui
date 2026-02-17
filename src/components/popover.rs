use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, Corner, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, anchored, deferred, div, point, px,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;

use super::Stack;
use super::control;
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type OpenChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopoverPlacement {
    Top,
    Bottom,
}

pub struct Popover {
    id: String,
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
            id: stable_auto_id("popover"),
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

    fn resolved_opened(&self) -> bool {
        control::bool_state(&self.id, "opened", self.opened, self.default_opened)
    }

    fn render_panel(&mut self, is_controlled: bool) -> AnyElement {
        let tokens = &self.theme.components.popover;
        let mut panel = Stack::vertical()
            .id(format!("{}-panel", self.id))
            .gap_2()
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.border))
            .text_color(resolve_hsla(&self.theme, &tokens.body))
            .rounded_md()
            .p_3();

        if self.close_on_click_outside {
            if let Some(handler) = self.on_open_change.clone() {
                let id = self.id.clone();
                panel = panel.on_mouse_down_out(move |_, window, cx| {
                    if !is_controlled {
                        control::set_bool_state(&id, "opened", false);
                        window.refresh();
                    }
                    (handler)(false, window, cx);
                });
            } else if !is_controlled {
                let id = self.id.clone();
                panel = panel.on_mouse_down_out(move |_, window, _cx| {
                    control::set_bool_state(&id, "opened", false);
                    window.refresh();
                });
            }
        }

        if let Some(content) = self.content.take() {
            panel = panel.child(content());
        }

        panel
            .with_enter_transition(format!("{}-panel-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl WithId for Popover {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl MotionAware for Popover {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Popover {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let opened = if self.disabled {
            false
        } else {
            self.resolved_opened()
        };
        let is_controlled = self.opened.is_some();

        let mut trigger = div().id(format!("{}-trigger", self.id)).relative().child(
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
                        control::set_bool_state(&id, "opened", next);
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
                    control::set_bool_state(&id, "opened", next);
                    window.refresh();
                },
            );
        } else {
            trigger = trigger.cursor_pointer();
        }

        if opened {
            let panel = self.render_panel(is_controlled);
            let floating = panel;

            let anchor_corner = match self.placement {
                PopoverPlacement::Top => Corner::BottomLeft,
                PopoverPlacement::Bottom => Corner::TopLeft,
            };
            let offset = match self.placement {
                PopoverPlacement::Top => point(px(0.0), px(-self.offset_px)),
                PopoverPlacement::Bottom => point(px(0.0), px(self.offset_px)),
            };

            let anchor_host = match self.placement {
                PopoverPlacement::Top => div()
                    .id(format!("{}-anchor-host", self.id))
                    .absolute()
                    .top_0()
                    .left_0()
                    .w(px(0.0))
                    .h(px(0.0))
                    .child(
                        deferred(
                            anchored()
                                .anchor(anchor_corner)
                                .offset(offset)
                                .child(floating),
                        )
                        .priority(20),
                    )
                    .into_any_element(),
                PopoverPlacement::Bottom => div()
                    .id(format!("{}-anchor-host", self.id))
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .w(px(0.0))
                    .h(px(0.0))
                    .child(
                        deferred(
                            anchored()
                                .anchor(anchor_corner)
                                .offset(offset)
                                .snap_to_window_with_margin(px(8.0))
                                .child(floating),
                        )
                        .priority(20),
                    )
                    .into_any_element(),
            };

            trigger = trigger.child(anchor_host);
        }

        div()
            .id(self.id.clone())
            .relative()
            .child(trigger)
            .into_any_element()
    }
}

impl IntoElement for Popover {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
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
