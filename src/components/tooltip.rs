use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;

use super::control;
use super::popup::{PopupPlacement, PopupState, anchored_host};
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type OpenChangeHandler = std::rc::Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TooltipPlacement {
    Top,
    Bottom,
}

#[derive(IntoElement)]
pub struct Tooltip {
    id: ComponentId,
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
            id: ComponentId::default(),
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

    fn render_bubble(&self, window: &gpui::Window) -> AnyElement {
        let tokens = &self.theme.components.tooltip;
        div()
            .id(self.id.slot("bubble"))
            .text_xs()
            .px(px(8.0))
            .py(px(5.0))
            .rounded_md()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.border))
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .text_color(resolve_hsla(&self.theme, &tokens.fg))
            .child(self.label.clone())
            .with_enter_transition(self.id.slot("bubble-enter"), self.motion)
            .into_any_element()
    }
}

impl Tooltip {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl MotionAware for Tooltip {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Tooltip {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let popup_state = PopupState::resolve(&self.id, self.opened, self.default_opened);
        let opened = !self.disabled && popup_state.opened;
        let is_controlled = popup_state.controlled;
        let trigger_content = self
            .trigger
            .take()
            .map(|render| render())
            .unwrap_or_else(|| div().child("target").into_any_element());

        let mut trigger = div()
            .id(self.id.slot("trigger"))
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
            let bubble = self.render_bubble(window);
            let placement = match self.placement {
                TooltipPlacement::Top => PopupPlacement::Top,
                TooltipPlacement::Bottom => PopupPlacement::Bottom,
            };
            let anchor_host = anchored_host(
                &self.id,
                "anchor-host",
                placement,
                self.offset_px,
                bubble,
                24,
                matches!(self.placement, TooltipPlacement::Bottom),
                false,
            );

            trigger = trigger.child(anchor_host);
        }

        div().id(self.id.clone()).relative().child(trigger)
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
