use std::sync::Arc;

use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, Varianted};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::overlay::{ModalCloseReason, ModalKind, ModalStateChange};
use crate::style::Variant;

use super::button::Button;
use super::control;
use super::icon::Icon;
use super::overlay::{Overlay, OverlayCoverage, OverlayMaterialMode};
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type SlotRenderer = Arc<dyn Fn() -> AnyElement>;
type OpenHandler = Arc<dyn Fn()>;
type CloseHandler = Arc<dyn Fn(ModalCloseReason)>;
type ActionHandler = Arc<dyn Fn()>;
type StateChangeHandler = Arc<dyn Fn(ModalStateChange)>;

#[derive(IntoElement)]
pub struct Modal {
    id: ComponentId,
    opened: Option<bool>,
    default_opened: bool,
    title: SharedString,
    body: Option<SharedString>,
    width_px: f32,
    kind: ModalKind,
    close_button: bool,
    close_on_click_outside: bool,
    close_on_escape: bool,
    confirm_label: SharedString,
    cancel_label: SharedString,
    complete_label: SharedString,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    content: Option<SlotRenderer>,
    on_open: Option<OpenHandler>,
    on_close: Option<CloseHandler>,
    on_confirm: Option<ActionHandler>,
    on_cancel: Option<ActionHandler>,
    on_complete: Option<ActionHandler>,
    on_state_change: Option<StateChangeHandler>,
}

impl Modal {
    #[track_caller]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            id: ComponentId::default(),
            opened: None,
            default_opened: false,
            title: title.into(),
            body: None,
            width_px: 560.0,
            kind: ModalKind::Custom,
            close_button: true,
            close_on_click_outside: true,
            close_on_escape: true,
            confirm_label: "Confirm".into(),
            cancel_label: "Cancel".into(),
            complete_label: "Done".into(),
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            content: None,
            on_open: None,
            on_close: None,
            on_confirm: None,
            on_cancel: None,
            on_complete: None,
            on_state_change: None,
        }
    }

    pub fn confirm(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self::new(title).with_kind(ModalKind::Confirm).body(body)
    }

    pub fn info(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self::new(title).with_kind(ModalKind::Info).body(body)
    }

    pub fn success(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self::new(title).with_kind(ModalKind::Success).body(body)
    }

    pub fn warning(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self::new(title).with_kind(ModalKind::Warning).body(body)
    }

    pub fn error(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self::new(title).with_kind(ModalKind::Error).body(body)
    }

    pub fn opened(mut self, value: bool) -> Self {
        self.opened = Some(value);
        self
    }

    pub fn default_opened(mut self, value: bool) -> Self {
        self.default_opened = value;
        self
    }

    fn with_kind(mut self, value: ModalKind) -> Self {
        self.kind = value;
        self
    }

    pub fn body(mut self, value: impl Into<SharedString>) -> Self {
        self.body = Some(value.into());
        self
    }

    pub fn message(self, value: impl Into<SharedString>) -> Self {
        self.body(value)
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

    pub fn close_on_escape(mut self, value: bool) -> Self {
        self.close_on_escape = value;
        self
    }

    pub fn confirm_label(mut self, value: impl Into<SharedString>) -> Self {
        self.confirm_label = value.into();
        self
    }

    pub fn cancel_label(mut self, value: impl Into<SharedString>) -> Self {
        self.cancel_label = value.into();
        self
    }

    pub fn complete_label(mut self, value: impl Into<SharedString>) -> Self {
        self.complete_label = value.into();
        self
    }

    pub fn custom<F, E>(mut self, content: F) -> Self
    where
        F: Fn() -> E + 'static,
        E: IntoElement + 'static,
    {
        self.kind = ModalKind::Custom;
        self.content = Some(Arc::new(move || content().into_any_element()));
        self
    }

    pub fn on_open(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_open = Some(Arc::new(handler));
        self
    }

    pub fn on_close(mut self, handler: impl Fn(ModalCloseReason) + 'static) -> Self {
        self.on_close = Some(Arc::new(handler));
        self
    }

    pub fn on_confirm(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_confirm = Some(Arc::new(handler));
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Arc::new(handler));
        self
    }

    pub fn on_complete(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_complete = Some(Arc::new(handler));
        self
    }

    pub fn on_state_change(mut self, handler: impl Fn(ModalStateChange) + 'static) -> Self {
        self.on_state_change = Some(Arc::new(handler));
        self
    }

    fn resolved_opened(&self) -> bool {
        control::bool_state(&self.id, "opened", self.opened, self.default_opened)
    }

    pub(crate) fn kind_ref(&self) -> ModalKind {
        self.kind
    }

    pub(crate) fn title_ref(&self) -> &SharedString {
        &self.title
    }

    pub(crate) fn body_ref(&self) -> Option<&SharedString> {
        self.body.as_ref()
    }

    pub(crate) fn width_px(&self) -> f32 {
        self.width_px
    }

    pub(crate) fn close_button_enabled(&self) -> bool {
        self.close_button
    }

    pub(crate) fn close_on_click_outside_enabled(&self) -> bool {
        self.close_on_click_outside
    }

    pub(crate) fn close_on_escape_enabled(&self) -> bool {
        self.close_on_escape
    }

    pub(crate) fn confirm_label_ref(&self) -> &SharedString {
        &self.confirm_label
    }

    pub(crate) fn cancel_label_ref(&self) -> &SharedString {
        &self.cancel_label
    }

    pub(crate) fn complete_label_ref(&self) -> &SharedString {
        &self.complete_label
    }

    pub(crate) fn motion_ref(&self) -> MotionConfig {
        self.motion
    }

    pub(crate) fn render_content(&self) -> Option<AnyElement> {
        self.content.as_ref().map(|content| content())
    }

    pub(crate) fn is_confirm_kind(&self) -> bool {
        self.kind == ModalKind::Confirm
    }

    pub(crate) fn has_complete_action(&self) -> bool {
        matches!(
            self.kind,
            ModalKind::Info | ModalKind::Success | ModalKind::Warning | ModalKind::Error
        )
    }

    pub(crate) fn emit_opened(&self) {
        if let Some(handler) = self.on_open.as_ref() {
            (handler)();
        }
        self.emit_state_change(ModalStateChange::Opened);
    }

    pub(crate) fn emit_closed(&self, reason: ModalCloseReason) {
        if let Some(handler) = self.on_close.as_ref() {
            (handler)(reason);
        }
        self.emit_state_change(ModalStateChange::Closed(reason));
    }

    pub(crate) fn emit_confirmed(&self) {
        if let Some(handler) = self.on_confirm.as_ref() {
            (handler)();
        }
        self.emit_state_change(ModalStateChange::Confirmed);
    }

    pub(crate) fn emit_canceled(&self) {
        if let Some(handler) = self.on_cancel.as_ref() {
            (handler)();
        }
        self.emit_state_change(ModalStateChange::Canceled);
    }

    pub(crate) fn emit_completed(&self) {
        if let Some(handler) = self.on_complete.as_ref() {
            (handler)();
        }
        self.emit_state_change(ModalStateChange::Completed);
    }

    fn emit_state_change(&self, state: ModalStateChange) {
        if let Some(handler) = self.on_state_change.as_ref() {
            (handler)(state);
        }
    }

    fn close_from_callbacks(
        close: &Option<CloseHandler>,
        state_change: &Option<StateChangeHandler>,
        reason: ModalCloseReason,
    ) {
        if let Some(handler) = close.as_ref() {
            (handler)(reason);
        }
        if let Some(handler) = state_change.as_ref() {
            (handler)(ModalStateChange::Closed(reason));
        }
    }

    fn action_from_callbacks(
        action: &Option<ActionHandler>,
        state_change: &Option<StateChangeHandler>,
        state: ModalStateChange,
    ) {
        if let Some(handler) = action.as_ref() {
            (handler)();
        }
        if let Some(handler) = state_change.as_ref() {
            (handler)(state);
        }
    }
}

impl Modal {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl MotionAware for Modal {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Modal {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        self.render_standalone(window, cx)
    }
}

impl crate::contracts::ComponentThemeOverridable for Modal {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_openable!(Modal);

impl Styled for Modal {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl Modal {
    pub(crate) fn render_standalone(
        mut self,
        window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) -> AnyElement {
        self.theme.sync_from_provider(_cx);
        let opened = self.resolved_opened();
        if !opened {
            return div().into_any_element();
        }

        let is_controlled = self.opened.is_some();
        let tokens = &self.theme.components.modal;
        let close_on_click_outside = self.close_on_click_outside;
        let id_for_overlay = self.id.clone();
        let close_callbacks_for_overlay = self.on_close.clone();
        let state_change_for_overlay = self.on_state_change.clone();

        let overlay = Overlay::new()
            .with_id(self.id.slot("overlay"))
            .coverage(OverlayCoverage::Window)
            .material_mode(OverlayMaterialMode::Auto)
            .color(tokens.overlay_bg.clone())
            .on_click(
                move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                    if close_on_click_outside {
                        if !is_controlled {
                            control::set_bool_state(&id_for_overlay, "opened", false);
                            window.refresh();
                        }
                        Self::close_from_callbacks(
                            &close_callbacks_for_overlay,
                            &state_change_for_overlay,
                            ModalCloseReason::OverlayClick,
                        );
                    }
                },
            );

        let mut close_action: AnyElement = div().into_any_element();
        if self.close_button {
            let id_for_close = self.id.clone();
            let close_callbacks_for_close = self.on_close.clone();
            let state_change_for_close = self.on_state_change.clone();
            close_action = div()
                .id(self.id.slot("close"))
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
                        .with_id(self.id.slot("close-icon"))
                        .size(14.0)
                        .color(resolve_hsla(&self.theme, &tokens.title)),
                )
                .on_click(
                    move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                        if !is_controlled {
                            control::set_bool_state(&id_for_close, "opened", false);
                            window.refresh();
                        }
                        Self::close_from_callbacks(
                            &close_callbacks_for_close,
                            &state_change_for_close,
                            ModalCloseReason::CloseButton,
                        );
                    },
                )
                .into_any_element();
        }

        let mut panel = div()
            .id(self.id.slot("panel"))
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
                            .child(self.title.clone()),
                    )
                    .child(close_action),
            );

        if let Some(body) = self.body.clone() {
            panel = panel.child(
                div()
                    .mb_2()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.body))
                    .child(body),
            );
        }

        if let Some(content) = self.content.as_ref() {
            panel = panel.child(content());
        }

        if self.is_confirm_kind() {
            let id_for_cancel = self.id.clone();
            let id_for_confirm = self.id.clone();
            let state_change_for_cancel = self.on_state_change.clone();
            let state_change_for_confirm = self.on_state_change.clone();
            let cancel_cb = self.on_cancel.clone();
            let confirm_cb = self.on_confirm.clone();
            let close_cb_cancel = self.on_close.clone();
            let close_cb_confirm = self.on_close.clone();
            panel = panel.child(
                div()
                    .mt_3()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new(self.cancel_label.clone())
                            .with_variant(Variant::Default)
                            .on_click(move |_, window, _| {
                                if !is_controlled {
                                    control::set_bool_state(&id_for_cancel, "opened", false);
                                }
                                Self::action_from_callbacks(
                                    &cancel_cb,
                                    &state_change_for_cancel,
                                    ModalStateChange::Canceled,
                                );
                                Self::close_from_callbacks(
                                    &close_cb_cancel,
                                    &state_change_for_cancel,
                                    ModalCloseReason::CancelAction,
                                );
                                window.refresh();
                            }),
                    )
                    .child(
                        Button::new(self.confirm_label.clone())
                            .with_variant(Variant::Filled)
                            .on_click(move |_, window, _| {
                                if !is_controlled {
                                    control::set_bool_state(&id_for_confirm, "opened", false);
                                }
                                Self::action_from_callbacks(
                                    &confirm_cb,
                                    &state_change_for_confirm,
                                    ModalStateChange::Confirmed,
                                );
                                Self::close_from_callbacks(
                                    &close_cb_confirm,
                                    &state_change_for_confirm,
                                    ModalCloseReason::ConfirmAction,
                                );
                                window.refresh();
                            }),
                    ),
            );
        } else if self.has_complete_action() {
            let id_for_complete = self.id.clone();
            let state_change_for_complete = self.on_state_change.clone();
            let complete_cb = self.on_complete.clone();
            let close_cb_complete = self.on_close.clone();
            panel = panel.child(
                div().mt_3().flex().justify_end().child(
                    Button::new(self.complete_label.clone())
                        .with_variant(Variant::Filled)
                        .on_click(move |_, window, _| {
                            if !is_controlled {
                                control::set_bool_state(&id_for_complete, "opened", false);
                            }
                            Self::action_from_callbacks(
                                &complete_cb,
                                &state_change_for_complete,
                                ModalStateChange::Completed,
                            );
                            Self::close_from_callbacks(
                                &close_cb_complete,
                                &state_change_for_complete,
                                ModalCloseReason::CompleteAction,
                            );
                            window.refresh();
                        }),
                ),
            );
        }

        let panel = panel.with_enter_transition(self.id.slot("panel-enter"), self.motion);

        let close_on_escape = self.close_on_escape;
        let id_for_escape = self.id.clone();
        let close_callbacks_for_escape = self.on_close.clone();
        let state_change_for_escape = self.on_state_change.clone();

        div()
            .id(self.id.clone())
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .on_key_down(move |event, window, _cx| {
                if close_on_escape && event.keystroke.key == "escape" {
                    if !is_controlled {
                        control::set_bool_state(&id_for_escape, "opened", false);
                    }
                    Self::close_from_callbacks(
                        &close_callbacks_for_escape,
                        &state_change_for_escape,
                        ModalCloseReason::EscapeKey,
                    );
                    window.refresh();
                }
            })
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
