use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use crate::icon::{IconRegistry, IconSource};
use crate::id::ComponentId;

use super::Stack;
use super::control;
use super::icon::Icon;
use super::utils::{deepened_surface_border, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type CloseHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertKind {
    Info,
    Success,
    Warning,
    Error,
    Loading,
}

#[derive(IntoElement)]
pub struct Alert {
    id: ComponentId,
    title: SharedString,
    message: SharedString,
    kind: AlertKind,
    icon: Option<IconSource>,
    closable: bool,
    visible: Option<bool>,
    default_visible: bool,
    right_slot: Option<SlotRenderer>,
    on_close: Option<CloseHandler>,
    icons: IconRegistry,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
}

impl Alert {
    #[track_caller]
    pub fn new(title: impl Into<SharedString>, message: impl Into<SharedString>) -> Self {
        Self {
            id: ComponentId::default(),
            title: title.into(),
            message: message.into(),
            kind: AlertKind::Info,
            icon: None,
            closable: true,
            visible: None,
            default_visible: true,
            right_slot: None,
            on_close: None,
            icons: IconRegistry::new(),
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn kind(mut self, value: AlertKind) -> Self {
        self.kind = value;
        self
    }

    pub fn icon(mut self, value: impl Into<SharedString>) -> Self {
        self.icon = Some(IconSource::named(value.into().to_string()));
        self
    }

    pub fn icon_source(mut self, source: IconSource) -> Self {
        self.icon = Some(source);
        self
    }

    pub fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }

    pub fn visible(mut self, value: bool) -> Self {
        self.visible = Some(value);
        self
    }

    pub fn default_visible(mut self, value: bool) -> Self {
        self.default_visible = value;
        self
    }

    pub fn right_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.right_slot = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn on_close(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    pub fn icons(mut self, icons: IconRegistry) -> Self {
        self.icons = icons;
        self
    }

    fn colors(&self) -> (gpui::Hsla, gpui::Hsla) {
        let tokens = &self.theme.components.toast;
        match self.kind {
            AlertKind::Info => (
                resolve_hsla(&self.theme, &tokens.info_bg),
                resolve_hsla(&self.theme, &tokens.info_fg),
            ),
            AlertKind::Success => (
                resolve_hsla(&self.theme, &tokens.success_bg),
                resolve_hsla(&self.theme, &tokens.success_fg),
            ),
            AlertKind::Warning => (
                resolve_hsla(&self.theme, &tokens.warning_bg),
                resolve_hsla(&self.theme, &tokens.warning_fg),
            ),
            AlertKind::Error => (
                resolve_hsla(&self.theme, &tokens.error_bg),
                resolve_hsla(&self.theme, &tokens.error_fg),
            ),
            AlertKind::Loading => (
                resolve_hsla(&self.theme, &tokens.info_bg),
                resolve_hsla(&self.theme, &tokens.info_fg),
            ),
        }
    }

    fn default_icon(kind: AlertKind) -> IconSource {
        match kind {
            AlertKind::Info => IconSource::named("info-circle"),
            AlertKind::Success => IconSource::named("circle-check"),
            AlertKind::Warning => IconSource::named("alert-triangle"),
            AlertKind::Error => IconSource::named("alert-circle"),
            AlertKind::Loading => IconSource::named("loader-2"),
        }
    }
}

impl Alert {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Alert {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let visible = control::bool_state(&self.id, "visible", self.visible, self.default_visible);
        if !visible {
            return div().id(self.id.clone());
        }

        let (bg, fg) = self.colors();
        let icon = self.icon.unwrap_or_else(|| Self::default_icon(self.kind));
        let icon_registry = self.icons.clone();
        let show_right_actions = self.closable || self.right_slot.is_some();
        let close_handler = self.on_close.clone();
        let alert_id = self.id.clone();

        let icon_badge = div()
            .id(self.id.slot("icon"))
            .flex_none()
            .w(gpui::px(24.0))
            .h(gpui::px(24.0))
            .mt_0p5()
            .flex()
            .items_center()
            .justify_center()
            .child(
                Icon::new(icon)
                    .with_id(self.id.slot("kind-icon"))
                    .size(17.0)
                    .color(fg)
                    .registry(icon_registry.clone()),
            );

        let close_button = div()
            .id(self.id.slot("close"))
            .flex_none()
            .w(gpui::px(24.0))
            .h(gpui::px(24.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(fg.opacity(0.32))
            .bg(fg.opacity(0.08))
            .cursor_pointer()
            .text_color(fg)
            .hover(|style| style.bg(fg.opacity(0.16)))
            .active(|style| style.bg(fg.opacity(0.24)))
            .child(
                Icon::named("x")
                    .with_id(self.id.slot("close-icon"))
                    .size(13.0)
                    .color(fg)
                    .registry(icon_registry),
            )
            .on_click(
                move |event: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                    control::set_bool_state(&alert_id, "visible", false);
                    if let Some(handler) = close_handler.as_ref() {
                        (handler)(event, window, cx);
                    }
                    window.refresh();
                },
            );

        let mut right = div().flex_none().flex().items_center().gap_2();
        if let Some(slot) = self.right_slot.take() {
            right = right.child(slot());
        }
        if self.closable {
            right = right.child(close_button);
        }

        div()
            .id(self.id.clone())
            .w_full()
            .max_w_full()
            .p_3()
            .rounded_md()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(deepened_surface_border(bg))
            .bg(bg)
            .text_color(fg)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap_2()
                    .child(icon_badge)
                    .child(
                        div().flex_1().overflow_hidden().child(
                            Stack::vertical()
                                .gap_1()
                                .child(
                                    div()
                                        .w_full()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .truncate()
                                        .child(self.title),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .text_sm()
                                        .whitespace_normal()
                                        .line_clamp(4)
                                        .child(self.message),
                                ),
                        ),
                    )
                    .children(show_right_actions.then_some(right)),
            )
    }
}

impl crate::contracts::ComponentThemeOverridable for Alert {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Alert {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
