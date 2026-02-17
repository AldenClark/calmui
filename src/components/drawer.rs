use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;

use super::control;
use super::icon::Icon;
use super::overlay::{Overlay, OverlayCoverage, OverlayMaterialMode};
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type CloseHandler = Rc<dyn Fn(&mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawerPlacement {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(IntoElement)]
pub struct Drawer {
    id: ComponentId,
    opened: Option<bool>,
    default_opened: bool,
    title: SharedString,
    body: Option<SharedString>,
    placement: DrawerPlacement,
    size_px: f32,
    close_button: bool,
    close_on_click_outside: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    content: Option<SlotRenderer>,
    on_close: Option<CloseHandler>,
}

impl Drawer {
    #[track_caller]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            id: ComponentId::default(),
            opened: None,
            default_opened: false,
            title: title.into(),
            body: None,
            placement: DrawerPlacement::Right,
            size_px: 360.0,
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

    pub fn placement(mut self, value: DrawerPlacement) -> Self {
        self.placement = value;
        self
    }

    pub fn size(mut self, value: f32) -> Self {
        self.size_px = value.max(160.0);
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

impl Drawer {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl MotionAware for Drawer {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Drawer {
    fn render(mut self, window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let opened = self.resolved_opened();
        if !opened {
            return div().id(self.id);
        }

        let is_controlled = self.opened.is_some();
        let tokens = &self.theme.components.drawer;
        let close_on_click_outside = self.close_on_click_outside;
        let outside_on_close = self.on_close.clone();
        let drawer_id_for_overlay = self.id.clone();

        let overlay = Overlay::new()
            .with_id(self.id.slot("overlay"))
            .coverage(OverlayCoverage::Window)
            .material_mode(OverlayMaterialMode::Auto)
            .color(tokens.overlay_bg.clone())
            .on_click(
                move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                    if close_on_click_outside {
                        if !is_controlled {
                            control::set_bool_state(&drawer_id_for_overlay, "opened", false);
                            window.refresh();
                        }
                        if let Some(handler) = outside_on_close.as_ref() {
                            (handler)(window, cx);
                        }
                    }
                },
            );

        let mut close_action = div().id(self.id.slot("close"));
        if self.close_button {
            let on_close = self.on_close.clone();
            let close_id = self.id.clone();
            let close_fg = resolve_hsla(&self.theme, &tokens.title);
            close_action = div()
                .id(self.id.slot("close"))
                .w(px(28.0))
                .h(px(28.0))
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
                .text_color(close_fg)
                .hover(|style| style.opacity(0.82))
                .child(
                    Icon::named("x")
                        .with_id(self.id.slot("close-icon"))
                        .size(14.0)
                        .color(close_fg),
                )
                .on_click(move |_, window, cx| {
                    if !is_controlled {
                        control::set_bool_state(&close_id, "opened", false);
                        window.refresh();
                    }
                    if let Some(handler) = on_close.as_ref() {
                        (handler)(window, cx);
                    }
                });
        }

        let mut body = div();
        if let Some(text) = self.body.clone() {
            body = div()
                .text_sm()
                .text_color(resolve_hsla(&self.theme, &tokens.body))
                .child(text);
        }

        let mut panel = div()
            .id(self.id.slot("panel"))
            .flex()
            .flex_col()
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

        panel = match self.placement {
            DrawerPlacement::Left | DrawerPlacement::Right => panel.w(px(self.size_px)).h_full(),
            DrawerPlacement::Top | DrawerPlacement::Bottom => panel.h(px(self.size_px)).w_full(),
        };

        let panel = panel.with_enter_transition(self.id.slot("panel-enter"), self.motion);

        let host = match self.placement {
            DrawerPlacement::Left => div().absolute().top_0().left_0().h_full().child(panel),
            DrawerPlacement::Right => div().absolute().top_0().right_0().h_full().child(panel),
            DrawerPlacement::Top => div().absolute().top_0().left_0().w_full().child(panel),
            DrawerPlacement::Bottom => div().absolute().bottom_0().left_0().w_full().child(panel),
        };

        div()
            .id(self.id)
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(overlay)
            .child(host)
    }
}

impl crate::contracts::ComponentThemeOverridable for Drawer {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_openable!(Drawer);
crate::impl_placeable!(Drawer, DrawerPlacement);

impl gpui::Styled for Drawer {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
