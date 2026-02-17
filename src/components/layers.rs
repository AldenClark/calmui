use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use std::time::Duration;

use crate::contracts::Variantable;
use crate::feedback::{ToastEntry, ToastKind, ToastManager, ToastPosition};
use crate::icon::{IconRegistry, IconSource};
use crate::motion::{MotionConfig, MotionTransition, TransitionPreset};
use crate::overlay::{ManagedModal, ModalCloseReason, ModalKind, ModalManager};
use crate::{contracts::WithId, id::stable_auto_id};

use super::Stack;
use super::button::Button;
use super::icon::Icon;
use super::overlay::{Overlay, OverlayCoverage, OverlayMaterialMode};
use super::transition::TransitionExt;
use super::utils::{deepened_surface_border, resolve_hsla};

#[derive(IntoElement)]
pub struct ToastLayer {
    id: String,
    manager: ToastManager,
    icons: IconRegistry,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
}

impl ToastLayer {
    #[track_caller]
    pub fn new(manager: ToastManager) -> Self {
        Self {
            id: stable_auto_id("toast-layer"),
            manager,
            icons: IconRegistry::new(),
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::new().enter(
                MotionTransition::new()
                    .preset(TransitionPreset::FadeRight)
                    .duration_ms(180)
                    .offset_px(18),
            ),
        }
    }

    pub fn motion(mut self, motion: MotionConfig) -> Self {
        self.motion = motion;
        self
    }

    pub fn icons(mut self, icons: IconRegistry) -> Self {
        self.icons = icons;
        self
    }

    fn toast_colors(&self, entry: &ToastEntry) -> (gpui::Hsla, gpui::Hsla) {
        let tokens = &self.theme.components.toast;
        match entry.kind {
            ToastKind::Info => (
                resolve_hsla(&self.theme, &tokens.info_bg),
                resolve_hsla(&self.theme, &tokens.info_fg),
            ),
            ToastKind::Success => (
                resolve_hsla(&self.theme, &tokens.success_bg),
                resolve_hsla(&self.theme, &tokens.success_fg),
            ),
            ToastKind::Warning => (
                resolve_hsla(&self.theme, &tokens.warning_bg),
                resolve_hsla(&self.theme, &tokens.warning_fg),
            ),
            ToastKind::Error => (
                resolve_hsla(&self.theme, &tokens.error_bg),
                resolve_hsla(&self.theme, &tokens.error_fg),
            ),
            ToastKind::Loading => (
                resolve_hsla(&self.theme, &tokens.info_bg),
                resolve_hsla(&self.theme, &tokens.info_fg),
            ),
        }
    }

    fn default_icon(kind: ToastKind) -> IconSource {
        match kind {
            ToastKind::Info => IconSource::named("info-circle"),
            ToastKind::Success => IconSource::named("circle-check"),
            ToastKind::Warning => IconSource::named("alert-triangle"),
            ToastKind::Error => IconSource::named("alert-circle"),
            ToastKind::Loading => IconSource::named("loader-2"),
        }
    }

    fn render_toast_card(&self, entry: ToastEntry, window: &gpui::Window) -> AnyElement {
        let (bg, fg) = self.toast_colors(&entry);
        let manager = self.manager.clone();
        let toast_id = entry.id;
        let toast_key = entry.id.map(|value| value.0).unwrap_or_default();
        let title = entry.title;
        let message = entry.message;
        let icon = entry.icon.unwrap_or_else(|| Self::default_icon(entry.kind));
        let closable = entry.closable;
        let icons = self.icons.clone();

        let close_button = div()
            .id(format!("{}-toast-close-{}", self.id, toast_key))
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
                    .with_id(format!("{}-toast-close-icon-{}", self.id, toast_key))
                    .size(13.0)
                    .color(fg)
                    .registry(icons.clone()),
            )
            .on_click(
                move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                    if let Some(id) = toast_id {
                        manager.dismiss(id);
                        window.refresh();
                    }
                },
            );

        let icon_badge = div()
            .id(format!("{}-toast-icon-{}", self.id, toast_key))
            .flex_none()
            .w(gpui::px(24.0))
            .h(gpui::px(24.0))
            .mt_0p5()
            .flex()
            .items_center()
            .justify_center()
            .child(
                Icon::new(icon)
                    .with_id(format!("{}-toast-kind-icon-{}", self.id, toast_key))
                    .size(17.0)
                    .color(fg)
                    .registry(icons),
            );

        div()
            .id(format!("{}-toast-{}", self.id, toast_key))
            .w(gpui::px(360.0))
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
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .text_sm()
                                        .whitespace_normal()
                                        .line_clamp(3)
                                        .child(message),
                                ),
                        ),
                    )
                    .children(closable.then_some(close_button)),
            )
            .with_enter_transition(
                format!("{}-toast-enter-{}", self.id, toast_key),
                self.motion,
            )
            .into_any_element()
    }

    fn anchor_for(position: ToastPosition) -> gpui::Div {
        let top_offset = if cfg!(target_os = "macos") {
            38.0
        } else if cfg!(target_os = "windows") {
            42.0
        } else {
            16.0
        };
        match position {
            ToastPosition::TopLeft => div()
                .absolute()
                .top(px(top_offset))
                .left_4()
                .flex()
                .flex_col()
                .gap_2(),
            ToastPosition::TopCenter => div()
                .absolute()
                .top(px(top_offset))
                .left_0()
                .right_0()
                .flex()
                .items_center()
                .flex_col()
                .gap_2(),
            ToastPosition::TopRight => div()
                .absolute()
                .top(px(top_offset))
                .right_4()
                .flex()
                .flex_col()
                .gap_2(),
            ToastPosition::BottomLeft => div()
                .absolute()
                .bottom_4()
                .left_4()
                .flex()
                .flex_col()
                .gap_2(),
            ToastPosition::BottomCenter => div()
                .absolute()
                .bottom_4()
                .left_0()
                .right_0()
                .flex()
                .items_center()
                .flex_col()
                .gap_2(),
            ToastPosition::BottomRight => div()
                .absolute()
                .bottom_4()
                .right_4()
                .flex()
                .flex_col()
                .gap_2(),
        }
    }

    fn schedule_auto_dismiss(&self, entry: &ToastEntry, window: &Window, cx: &mut gpui::App) {
        let Some(id) = entry.id else {
            return;
        };
        let Some(delay_ms) = entry.auto_close_ms else {
            return;
        };
        let Some(version) = self.manager.version_of(id) else {
            return;
        };
        if !self.manager.mark_auto_close_scheduled(id, version) {
            return;
        }

        let manager = self.manager.clone();
        let window_handle = window.window_handle();
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(u64::from(delay_ms)))
                .await;
            let _ = window_handle.update(cx, |_, window, _| {
                if manager.dismiss_if_version(id, version) {
                    window.refresh();
                }
            });
        })
        .detach();
    }
}

impl WithId for ToastLayer {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for ToastLayer {
    fn render(mut self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(cx);
        let positions = [
            ToastPosition::TopLeft,
            ToastPosition::TopCenter,
            ToastPosition::TopRight,
            ToastPosition::BottomLeft,
            ToastPosition::BottomCenter,
            ToastPosition::BottomRight,
        ];

        let mut root = div()
            .id(self.id.clone())
            .size_full()
            .absolute()
            .top_0()
            .left_0();

        for position in positions {
            let toasts = self.manager.list(position);
            if toasts.is_empty() {
                continue;
            }

            let mut cards = Vec::with_capacity(toasts.len());
            for entry in toasts {
                self.schedule_auto_dismiss(&entry, window, cx);
                cards.push(self.render_toast_card(entry, window));
            }

            root = root.child(Self::anchor_for(position).children(cards));
        }

        root.into_any_element()
    }
}

pub struct ModalLayer {
    id: String,
    manager: ModalManager,
    icons: IconRegistry,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
}

impl ModalLayer {
    #[track_caller]
    pub fn new(manager: ModalManager) -> Self {
        Self {
            id: stable_auto_id("modal-layer"),
            manager,
            icons: IconRegistry::new(),
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::new().enter(
                MotionTransition::new()
                    .preset(TransitionPreset::Pop)
                    .duration_ms(210)
                    .offset_px(10),
            ),
        }
    }

    pub fn motion(mut self, motion: MotionConfig) -> Self {
        self.motion = motion;
        self
    }

    pub fn icons(mut self, icons: IconRegistry) -> Self {
        self.icons = icons;
        self
    }

    fn modal_kind_icon(&self, kind: ModalKind) -> Option<IconSource> {
        match kind {
            ModalKind::Custom => None,
            ModalKind::Info => Some(IconSource::named("info-circle")),
            ModalKind::Success => Some(IconSource::named("circle-check")),
            ModalKind::Warning => Some(IconSource::named("alert-triangle")),
            ModalKind::Error => Some(IconSource::named("alert-circle")),
            ModalKind::Confirm => Some(IconSource::named("info-circle")),
        }
    }

    fn modal_kind_color(&self, kind: ModalKind) -> gpui::Hsla {
        let tokens = &self.theme.components.toast;
        match kind {
            ModalKind::Success => resolve_hsla(&self.theme, &tokens.success_fg),
            ModalKind::Warning => resolve_hsla(&self.theme, &tokens.warning_fg),
            ModalKind::Error => resolve_hsla(&self.theme, &tokens.error_fg),
            ModalKind::Info | ModalKind::Confirm | ModalKind::Custom => {
                resolve_hsla(&self.theme, &tokens.info_fg)
            }
        }
    }

    fn render_modal(&self, managed: ManagedModal, window: &gpui::Window) -> AnyElement {
        let id = managed.id();
        let modal = managed.modal_arc();
        let entry = modal.as_ref();
        let panel_bg = resolve_hsla(&self.theme, &self.theme.components.modal.panel_bg);
        let panel_border = resolve_hsla(&self.theme, &self.theme.components.modal.panel_border);
        let title_color = resolve_hsla(&self.theme, &self.theme.components.modal.title);
        let body_color = resolve_hsla(&self.theme, &self.theme.components.modal.body);

        let manager_for_overlay = self.manager.clone();
        let manager_for_close = self.manager.clone();
        let manager_for_cancel = self.manager.clone();
        let manager_for_confirm = self.manager.clone();
        let manager_for_complete = self.manager.clone();
        let manager_for_escape = self.manager.clone();
        let icons = self.icons.clone();

        let close_on_click_outside = entry.close_on_click_outside_enabled();
        let overlay = Overlay::new()
            .with_id(format!("{}-overlay", self.id))
            .coverage(OverlayCoverage::Window)
            .material_mode(OverlayMaterialMode::Auto)
            .color(self.theme.components.modal.overlay_bg.clone())
            .on_click(
                move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                    if close_on_click_outside {
                        manager_for_overlay.close_with_reason(id, ModalCloseReason::OverlayClick);
                        window.refresh();
                    }
                },
            );

        let mut header_title = div().flex().items_center();

        if let Some(icon) = self.modal_kind_icon(entry.kind_ref()) {
            header_title = header_title.child(
                div().mr_2().flex().child(
                    Icon::new(icon)
                        .size(16.0)
                        .color(self.modal_kind_color(entry.kind_ref()))
                        .registry(self.icons.clone()),
                ),
            );
        }
        header_title = header_title.child(
            div()
                .text_color(title_color)
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(entry.title_ref().clone()),
        );

        let mut close_action = div().into_any_element();
        if entry.close_button_enabled() {
            close_action = div()
                .id(format!("{}-modal-close-{}", self.id, id.0))
                .cursor_pointer()
                .child(
                    Icon::named("x")
                        .with_id(format!("{}-modal-close-icon-{}", self.id, id.0))
                        .size(14.0)
                        .color(title_color)
                        .registry(icons.clone()),
                )
                .on_click(
                    move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                        manager_for_close.close_with_reason(id, ModalCloseReason::CloseButton);
                        window.refresh();
                    },
                )
                .into_any_element();
        }

        let mut panel = div()
            .id(format!("{}-modal-panel", self.id))
            .w(px(entry.width_px()))
            .max_w_full()
            .bg(panel_bg)
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(panel_border)
            .rounded_lg()
            .p_4()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .mb_2()
                    .child(header_title)
                    .child(close_action),
            )
            .children(entry.body_ref().map(|body| {
                div()
                    .mb_2()
                    .text_color(body_color)
                    .text_sm()
                    .child(body.clone())
                    .into_any_element()
            }))
            .children(entry.render_content());

        if entry.is_confirm_kind() {
            panel = panel.child(
                div()
                    .mt_3()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new(entry.cancel_label_ref().clone())
                            .variant(crate::style::Variant::Default)
                            .on_click(move |_, window, _| {
                                manager_for_cancel.cancel(id);
                                window.refresh();
                            }),
                    )
                    .child(
                        Button::new(entry.confirm_label_ref().clone())
                            .variant(crate::style::Variant::Filled)
                            .on_click(move |_, window, _| {
                                manager_for_confirm.confirm(id);
                                window.refresh();
                            }),
                    ),
            );
        } else if entry.has_complete_action() {
            panel = panel.child(
                div().mt_3().flex().justify_end().child(
                    Button::new(entry.complete_label_ref().clone())
                        .variant(crate::style::Variant::Filled)
                        .on_click(move |_, window, _| {
                            manager_for_complete.complete(id);
                            window.refresh();
                        }),
                ),
            );
        }

        let panel = panel.with_enter_transition(
            format!("{}-modal-enter-{}", self.id, id.0),
            entry.motion_ref(),
        );

        let close_on_escape = entry.close_on_escape_enabled();

        div()
            .id(format!("{}-modal-root-{}", self.id, id.0))
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .on_key_down(move |event, window, _cx| {
                if close_on_escape && event.keystroke.key == "escape" {
                    manager_for_escape.close_with_reason(id, ModalCloseReason::EscapeKey);
                    window.refresh();
                }
            })
            .child(overlay)
            .child(
                div()
                    .size_full()
                    .absolute()
                    .top_0()
                    .left_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(panel),
            )
            .into_any_element()
    }
}

impl WithId for ModalLayer {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for ModalLayer {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let Some(entry) = self.manager.top() else {
            return div().into_any_element();
        };
        self.render_modal(entry, window)
    }
}

impl IntoElement for ModalLayer {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for ToastLayer {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for ModalLayer {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for ModalLayer {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for ToastLayer {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
