use gpui::{
    AnyElement, Component, Corner, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, anchored, deferred, div, point, px,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;

use super::control;
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type OpenChangeHandler = std::rc::Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TooltipPlacement {
    Top,
    Bottom,
}

pub struct Tooltip {
    id: String,
    label: SharedString,
    opened: Option<bool>,
    default_opened: bool,
    disabled: bool,
    trigger_on_click: bool,
    placement: TooltipPlacement,
    offset_px: f32,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    trigger: Option<SlotRenderer>,
    on_open_change: Option<OpenChangeHandler>,
}

impl Tooltip {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("tooltip"),
            label: label.into(),
            opened: None,
            default_opened: false,
            disabled: false,
            trigger_on_click: false,
            placement: TooltipPlacement::Top,
            offset_px: 3.0,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            trigger: None,
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

    pub fn trigger_on_click(mut self, value: bool) -> Self {
        self.trigger_on_click = value;
        self
    }

    pub fn placement(mut self, value: TooltipPlacement) -> Self {
        self.placement = value;
        self
    }

    pub fn offset(mut self, value: f32) -> Self {
        self.offset_px = value.max(0.0);
        self
    }

    pub fn trigger(mut self, trigger: impl IntoElement + 'static) -> Self {
        self.trigger = Some(Box::new(|| trigger.into_any_element()));
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_open_change = Some(std::rc::Rc::new(handler));
        self
    }

    fn resolved_opened(&self) -> bool {
        control::bool_state(&self.id, "opened", self.opened, self.default_opened)
    }

    fn render_bubble(&self) -> AnyElement {
        let tokens = &self.theme.components.tooltip;
        div()
            .id(format!("{}-bubble", self.id))
            .text_xs()
            .px(px(8.0))
            .py(px(5.0))
            .rounded_md()
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.border))
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .text_color(resolve_hsla(&self.theme, &tokens.fg))
            .child(self.label.clone())
            .with_enter_transition(format!("{}-bubble-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl WithId for Tooltip {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl MotionAware for Tooltip {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Tooltip {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let opened = if self.disabled {
            false
        } else {
            self.resolved_opened()
        };
        let is_controlled = self.opened.is_some();
        let trigger_content = self
            .trigger
            .take()
            .map(|render| render())
            .unwrap_or_else(|| div().child("target").into_any_element());

        let mut trigger = div()
            .id(format!("{}-trigger", self.id))
            .relative()
            .child(trigger_content);

        if self.disabled {
            trigger = trigger.cursor_default().opacity(0.55);
        } else if let Some(handler) = self.on_open_change.clone() {
            trigger = trigger.cursor_pointer();
            let id = self.id.clone();
            trigger = trigger.on_hover(move |hovered, window, cx| {
                if !is_controlled {
                    control::set_bool_state(&id, "opened", *hovered);
                    window.refresh();
                }
                (handler)(*hovered, window, cx);
            });
        } else if !is_controlled {
            trigger = trigger.cursor_pointer();
            let id = self.id.clone();
            trigger = trigger.on_hover(move |hovered, window, _cx| {
                control::set_bool_state(&id, "opened", *hovered);
                window.refresh();
            });
        } else {
            trigger = trigger.cursor_pointer();
        }

        if self.trigger_on_click && !self.disabled {
            if let Some(handler) = self.on_open_change.clone() {
                let next = !opened;
                let id = self.id.clone();
                trigger = trigger.on_click(move |_, window, cx| {
                    if !is_controlled {
                        control::set_bool_state(&id, "opened", next);
                        window.refresh();
                    }
                    (handler)(next, window, cx);
                });
            } else if !is_controlled {
                let next = !opened;
                let id = self.id.clone();
                trigger = trigger.on_click(move |_, window, _cx| {
                    control::set_bool_state(&id, "opened", next);
                    window.refresh();
                });
            }
        }

        if opened {
            let bubble = self.render_bubble();
            let floating = bubble;
            let anchor_corner = match self.placement {
                TooltipPlacement::Top => Corner::BottomLeft,
                TooltipPlacement::Bottom => Corner::TopLeft,
            };
            let offset = match self.placement {
                TooltipPlacement::Top => point(px(0.0), px(-self.offset_px)),
                TooltipPlacement::Bottom => point(px(0.0), px(self.offset_px)),
            };

            let anchor_host = match self.placement {
                TooltipPlacement::Top => div()
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
                        .priority(24),
                    )
                    .into_any_element(),
                TooltipPlacement::Bottom => div()
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
                        .priority(24),
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

impl IntoElement for Tooltip {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Tooltip {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Tooltip);
crate::impl_openable!(Tooltip);
crate::impl_placeable!(Tooltip, TooltipPlacement);

impl gpui::Styled for Tooltip {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
