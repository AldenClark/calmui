use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;

use super::control;
use super::icon::Icon;
use super::overlay::{Overlay, OverlayCoverage, OverlayMaterialMode};
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type CloseHandler = Rc<dyn Fn(&mut Window, &mut gpui::App)>;

pub struct Modal {
    id: String,
    opened: Option<bool>,
    default_opened: bool,
    title: SharedString,
    body: Option<SharedString>,
    width_px: f32,
    close_button: bool,
    close_on_click_outside: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    content: Option<SlotRenderer>,
    on_close: Option<CloseHandler>,
}

impl Modal {
    #[track_caller]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("modal"),
            opened: None,
            default_opened: false,
            title: title.into(),
            body: None,
            width_px: 560.0,
            close_button: true,
            close_on_click_outside: true,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            content: None,
            on_close: None,
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

    pub fn body(mut self, value: impl Into<SharedString>) -> Self {
        self.body = Some(value.into());
        self
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width_px = value.max(240.0);
        self
    }

    pub fn close_button(mut self, value: bool) -> Self {
        self.close_button = value;
        self
    }

    pub fn close_on_click_outside(mut self, value: bool) -> Self {
        self.close_on_click_outside = value;
        self
    }

    pub fn content(mut self, content: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut gpui::App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    fn resolved_opened(&self) -> bool {
        control::bool_state(&self.id, "opened", self.opened, self.default_opened)
    }
}

impl WithId for Modal {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl MotionAware for Modal {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Modal {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let opened = self.resolved_opened();
        if !opened {
            return div().into_any_element();
        }
        let is_controlled = self.opened.is_some();

        let tokens = &self.theme.components.modal;
        let close_on_click_outside = self.close_on_click_outside;
        let outside_on_close = self.on_close.clone();
        let id_for_overlay = self.id.clone();

        let overlay = Overlay::new()
            .with_id(format!("{}-overlay", self.id))
            .coverage(OverlayCoverage::Window)
            .material_mode(OverlayMaterialMode::Auto)
            .color(tokens.overlay_bg.clone())
            .on_click(
                move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                    if close_on_click_outside {
                        if !is_controlled {
                            control::set_bool_state(&id_for_overlay, "opened", false);
                            window.refresh();
                        }
                        if let Some(handler) = outside_on_close.as_ref() {
                            (handler)(window, cx);
                        }
                    }
                },
            );

        let mut close_action = div().into_any_element();
        if self.close_button {
            let close_on_close = self.on_close.clone();
            let id_for_close = self.id.clone();
            close_action = div()
                .id(format!("{}-close", self.id))
                .w(px(26.0))
                .h(px(26.0))
                .rounded_full()
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(
                    &self.theme,
                    &self.theme.semantic.border_subtle,
                ))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_color(resolve_hsla(&self.theme, &tokens.title))
                .hover(|style| style.opacity(0.8))
                .child(
                    Icon::named("x")
                        .with_id(format!("{}-close-icon", self.id))
                        .size(14.0)
                        .color(resolve_hsla(&self.theme, &tokens.title)),
                )
                .on_click(
                    move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                        if !is_controlled {
                            control::set_bool_state(&id_for_close, "opened", false);
                            window.refresh();
                        }
                        if let Some(handler) = close_on_close.as_ref() {
                            (handler)(window, cx);
                        }
                    },
                )
                .into_any_element();
        }

        let mut body = div().into_any_element();
        if let Some(text) = self.body.clone() {
            body = div()
                .text_sm()
                .text_color(resolve_hsla(&self.theme, &tokens.body))
                .child(text)
                .into_any_element();
        }

        let mut panel = div()
            .id(format!("{}-panel", self.id))
            .w(px(self.width_px))
            .max_w_full()
            .rounded_lg()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.panel_border))
            .bg(resolve_hsla(&self.theme, &tokens.panel_bg))
            .p_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_2()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(resolve_hsla(&self.theme, &tokens.title))
                            .child(self.title),
                    )
                    .child(close_action),
            )
            .child(body);

        if let Some(content) = self.content.take() {
            panel = panel.child(content());
        }

        let panel = panel.with_enter_transition(format!("{}-panel-enter", self.id), self.motion);

        div()
            .id(self.id.clone())
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(overlay)
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(panel),
            )
            .into_any_element()
    }
}

impl IntoElement for Modal {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Modal {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_openable!(Modal);

impl gpui::Styled for Modal {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
