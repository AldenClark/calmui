use std::time::Duration;

use gpui::{
    AnyElement, Corner, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, anchored, canvas, deferred, div, point, px,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;

use super::Stack;
use super::control;
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type OpenChangeHandler = std::rc::Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

fn panel_width_px(id: &str, fallback: f32) -> f32 {
    control::text_state(id, "trigger-width-px", None, fallback.to_string())
        .parse::<f32>()
        .ok()
        .filter(|width| *width >= 1.0)
        .unwrap_or(fallback)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HoverCardPlacement {
    Top,
    Bottom,
}

#[derive(IntoElement)]
pub struct HoverCard {
    id: ComponentId,
    title: SharedString,
    body: Option<SharedString>,
    opened: Option<bool>,
    default_opened: bool,
    disabled: bool,
    placement: HoverCardPlacement,
    offset_px: f32,
    match_trigger_width: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    trigger: Option<SlotRenderer>,
    content: Option<SlotRenderer>,
    on_open_change: Option<OpenChangeHandler>,
}

impl HoverCard {
    #[track_caller]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            id: ComponentId::default(),
            title: title.into(),
            body: None,
            opened: None,
            default_opened: false,
            disabled: false,
            placement: HoverCardPlacement::Bottom,
            offset_px: 2.0,
            match_trigger_width: true,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            trigger: None,
            content: None,
            on_open_change: None,
        }
    }

    pub fn body(mut self, value: impl Into<SharedString>) -> Self {
        self.body = Some(value.into());
        self
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

    pub fn placement(mut self, value: HoverCardPlacement) -> Self {
        self.placement = value;
        self
    }

    pub fn offset(mut self, value: f32) -> Self {
        self.offset_px = value.max(0.0);
        self
    }

    pub fn match_trigger_width(mut self, value: bool) -> Self {
        self.match_trigger_width = value;
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
        self.on_open_change = Some(std::rc::Rc::new(handler));
        self
    }

    fn trigger_hovered(&self) -> bool {
        control::bool_state(&self.id, "trigger-hovered", None, false)
    }

    fn panel_hovered(&self) -> bool {
        control::bool_state(&self.id, "panel-hovered", None, false)
    }

    fn resolved_opened(&self) -> bool {
        let base = control::bool_state(&self.id, "opened", self.opened, self.default_opened);
        if self.opened.is_some() {
            base
        } else {
            base || self.trigger_hovered() || self.panel_hovered()
        }
    }

    fn render_card(&mut self, is_controlled: bool, window: &gpui::Window) -> AnyElement {
        let tokens = &self.theme.components.hover_card;
        let panel_width = if self.match_trigger_width {
            panel_width_px(&self.id, 260.0).max(120.0)
        } else {
            260.0
        };
        let mut card = Stack::vertical()
            .id(self.id.slot("card"))
            .gap_1p5()
            .w(px(panel_width))
            .max_w_full()
            .p_3()
            .rounded_md()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.border))
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .child(
                div()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(resolve_hsla(&self.theme, &tokens.title))
                    .child(self.title.clone()),
            );

        if let Some(body) = self.body.clone() {
            card = card.child(
                div()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.body))
                    .child(body),
            );
        }

        if let Some(content) = self.content.take() {
            card = card.child(content());
        }

        let id = self.id.clone();
        let handler = self.on_open_change.clone();
        card = card.on_hover(move |hovered, window, cx| {
            control::set_bool_state(&id, "panel-hovered", *hovered);
            let next = *hovered || control::bool_state(&id, "trigger-hovered", None, false);
            if !is_controlled {
                if *hovered {
                    control::set_bool_state(&id, "opened", true);
                    window.refresh();
                } else {
                    let id_for_delay = id.clone();
                    let window_handle = window.window_handle();
                    cx.spawn({
                        async move |cx| {
                            cx.background_executor()
                                .timer(Duration::from_millis(120))
                                .await;
                            let _ = window_handle.update(cx, |_, window, _| {
                                let still_open = control::bool_state(
                                    &id_for_delay,
                                    "trigger-hovered",
                                    None,
                                    false,
                                ) || control::bool_state(
                                    &id_for_delay,
                                    "panel-hovered",
                                    None,
                                    false,
                                );
                                control::set_bool_state(&id_for_delay, "opened", still_open);
                                window.refresh();
                            });
                        }
                    })
                    .detach();
                }
            }
            if let Some(on_open_change) = handler.as_ref() {
                (on_open_change)(next, window, cx);
            }
        });

        card.with_enter_transition(self.id.slot("card-enter"), self.motion)
            .into_any_element()
    }
}

impl HoverCard {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl MotionAware for HoverCard {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for HoverCard {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
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
            .id(self.id.slot("trigger"))
            .relative()
            .child(trigger_content);
        trigger = trigger.child({
            let id_for_width = self.id.clone();
            canvas(
                move |bounds, _, _cx| {
                    control::set_text_state(
                        &id_for_width,
                        "trigger-width-px",
                        format!("{:.2}", f32::from(bounds.size.width)),
                    );
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        });

        if self.disabled {
            trigger = trigger.cursor_default().opacity(0.55);
        } else {
            trigger = trigger.cursor_pointer();
            let id = self.id.clone();
            let handler = self.on_open_change.clone();
            trigger = trigger.on_hover(move |hovered, window, cx| {
                control::set_bool_state(&id, "trigger-hovered", *hovered);
                let next = *hovered || control::bool_state(&id, "panel-hovered", None, false);
                if !is_controlled {
                    if *hovered {
                        control::set_bool_state(&id, "opened", true);
                        window.refresh();
                    } else {
                        let id_for_delay = id.clone();
                        let window_handle = window.window_handle();
                        cx.spawn({
                            async move |cx| {
                                cx.background_executor()
                                    .timer(Duration::from_millis(120))
                                    .await;
                                let _ = window_handle.update(cx, |_, window, _| {
                                    let still_open = control::bool_state(
                                        &id_for_delay,
                                        "trigger-hovered",
                                        None,
                                        false,
                                    ) || control::bool_state(
                                        &id_for_delay,
                                        "panel-hovered",
                                        None,
                                        false,
                                    );
                                    control::set_bool_state(&id_for_delay, "opened", still_open);
                                    window.refresh();
                                });
                            }
                        })
                        .detach();
                    }
                }
                if let Some(on_open_change) = handler.as_ref() {
                    (on_open_change)(next, window, cx);
                }
            });
        }

        if opened {
            let card = self.render_card(is_controlled, window);
            let floating = card;
            let anchor_corner = match self.placement {
                HoverCardPlacement::Top => Corner::BottomLeft,
                HoverCardPlacement::Bottom => Corner::TopLeft,
            };
            let offset = match self.placement {
                HoverCardPlacement::Top => point(px(0.0), px(-self.offset_px)),
                HoverCardPlacement::Bottom => point(px(0.0), px(self.offset_px)),
            };

            let anchor_host = match self.placement {
                HoverCardPlacement::Top => div()
                    .id(self.id.slot("anchor-host"))
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
                        .priority(26),
                    )
                    .into_any_element(),
                HoverCardPlacement::Bottom => div()
                    .id(self.id.slot("anchor-host"))
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
                        .priority(26),
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

impl crate::contracts::ComponentThemeOverridable for HoverCard {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(HoverCard);
crate::impl_openable!(HoverCard);
crate::impl_placeable!(HoverCard, HoverCardPlacement);

impl gpui::Styled for HoverCard {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
