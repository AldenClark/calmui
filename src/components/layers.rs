use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use std::time::Duration;

use crate::contracts::Varianted;
use crate::feedback::{ToastEntry, ToastKind, ToastManager, ToastPosition};
use crate::icon::{IconRegistry, IconSource};
use crate::id::ComponentId;
use crate::motion::{MotionConfig, MotionTransition, TransitionPreset};
use crate::overlay::{ManagedModal, ModalCloseReason, ModalKind, ModalManager};

use super::Stack;
use super::button::Button;
use super::control;
use super::icon::Icon;
use super::overlay::{Overlay, OverlayCoverage, OverlayMaterialMode};
use super::transition::TransitionExt;
use super::utils::{deepened_surface_border, resolve_hsla};

#[derive(IntoElement)]
pub struct ToastLayer {
    id: ComponentId,
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
            id: ComponentId::default(),
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
        let tokens = &self.theme.components.toast;
        let manager = self.manager.clone();
        let toast_id = entry.id;
        let toast_key = entry.id.map(|value| value.0).unwrap_or_default();
        let title = entry.title;
        let message = entry.message;
        let icon = entry.icon.unwrap_or_else(|| Self::default_icon(entry.kind));
        let closable = entry.closable;
        let icons = self.icons.clone();

        let close_button = div()
            .id(self.id.slot_index("toast-close", (toast_key).to_string()))
            .flex_none()
            .w(tokens.close_button_size)
            .h(tokens.close_button_size)
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
                    .with_id(
                        self.id
                            .slot_index("toast-close-icon", (toast_key).to_string()),
                    )
                    .size(f32::from(tokens.close_icon_size))
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
            .id(self.id.slot_index("toast-icon", (toast_key).to_string()))
            .flex_none()
            .w(tokens.icon_box_size)
            .h(tokens.icon_box_size)
            .mt_0p5()
            .flex()
            .items_center()
            .justify_center()
            .child(
                Icon::new(icon)
                    .with_id(
                        self.id
                            .slot_index("toast-kind-icon", (toast_key).to_string()),
                    )
                    .size(f32::from(tokens.icon_size))
                    .color(fg)
                    .registry(icons),
            );

        div()
            .id(self.id.slot_index("toast", (toast_key).to_string()))
            .w(tokens.card_width)
            .max_w_full()
            .p(tokens.card_padding)
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
                    .gap(tokens.row_gap)
                    .child(icon_badge)
                    .child(
                        Stack::vertical()
                            .flex_1()
                            .overflow_hidden()
                            .gap(tokens.content_gap)
                            .child(
                                div()
                                    .w_full()
                                    .text_size(tokens.title_size)
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .truncate()
                                    .child(title),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .text_size(tokens.body_size)
                                    .whitespace_normal()
                                    .line_clamp(3)
                                    .child(message),
                            ),
                    )
                    .children(closable.then_some(close_button)),
            )
            .with_enter_transition(
                self.id.slot_index("toast-enter", toast_key.to_string()),
                self.motion,
            )
            .into_any_element()
    }

    fn anchor_for(
        position: ToastPosition,
        top_offset: f32,
        edge_offset: gpui::Pixels,
        stack_gap: gpui::Pixels,
    ) -> gpui::Div {
        match position {
            ToastPosition::TopLeft => div()
                .absolute()
                .top(px(top_offset))
                .left(edge_offset)
                .flex()
                .flex_col()
                .gap(stack_gap),
            ToastPosition::TopCenter => div()
                .absolute()
                .top(px(top_offset))
                .left_0()
                .right_0()
                .flex()
                .items_center()
                .flex_col()
                .gap(stack_gap),
            ToastPosition::TopRight => div()
                .absolute()
                .top(px(top_offset))
                .right(edge_offset)
                .flex()
                .flex_col()
                .gap(stack_gap),
            ToastPosition::BottomLeft => div()
                .absolute()
                .bottom(edge_offset)
                .left(edge_offset)
                .flex()
                .flex_col()
                .gap(stack_gap),
            ToastPosition::BottomCenter => div()
                .absolute()
                .bottom(edge_offset)
                .left_0()
                .right_0()
                .flex()
                .items_center()
                .flex_col()
                .gap(stack_gap),
            ToastPosition::BottomRight => div()
                .absolute()
                .bottom(edge_offset)
                .right(edge_offset)
                .flex()
                .flex_col()
                .gap(stack_gap),
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

impl ToastLayer {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for ToastLayer {
    fn render(mut self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(cx);
        let toast_tokens = &self.theme.components.toast;
        let top_offset = f32::from(self.theme.components.title_bar.height)
            + f32::from(toast_tokens.top_offset_extra);
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

            root = root.child(
                Self::anchor_for(
                    position,
                    top_offset,
                    toast_tokens.edge_offset,
                    toast_tokens.stack_gap,
                )
                .children(cards),
            );
        }

        root
    }
}

#[derive(IntoElement)]
pub struct ModalLayer {
    id: ComponentId,
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
            id: ComponentId::default(),
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
        let modal_tokens = &self.theme.components.modal;
        let panel_bg = resolve_hsla(&self.theme, &modal_tokens.panel_bg);
        let panel_border = resolve_hsla(&self.theme, &modal_tokens.panel_border);
        let title_color = resolve_hsla(&self.theme, &modal_tokens.title);
        let body_color = resolve_hsla(&self.theme, &modal_tokens.body);

        let manager_for_overlay = self.manager.clone();
        let manager_for_close = self.manager.clone();
        let manager_for_cancel = self.manager.clone();
        let manager_for_confirm = self.manager.clone();
        let manager_for_complete = self.manager.clone();
        let manager_for_escape = self.manager.clone();
        let icons = self.icons.clone();

        let close_on_click_outside = entry.close_on_click_outside_enabled();
        let overlay = Overlay::new()
            .with_id(self.id.slot("overlay"))
            .coverage(OverlayCoverage::Window)
            .material_mode(OverlayMaterialMode::TintOnly)
            .frosted(false)
            .color(self.theme.components.modal.overlay_bg.clone())
            .opacity(1.0)
            .readability_boost(0.84)
            .on_click(
                move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                    if close_on_click_outside {
                        manager_for_overlay.close_with_reason(id, ModalCloseReason::OverlayClick);
                        window.refresh();
                    }
                },
            );

        let mut header_title = div().flex().items_center().flex_1().min_w_0();

        if let Some(icon) = self.modal_kind_icon(entry.kind_ref()) {
            header_title = header_title.child(
                Icon::new(icon)
                    .size(f32::from(modal_tokens.kind_icon_size))
                    .color(self.modal_kind_color(entry.kind_ref()))
                    .registry(self.icons.clone())
                    .mr(modal_tokens.kind_icon_gap),
            );
        }
        if let Some(title) = entry.title_ref() {
            header_title = header_title.child(
                div()
                    .text_size(modal_tokens.title_size)
                    .text_color(title_color)
                    .font_weight(modal_tokens.title_weight)
                    .child(title.clone()),
            );
        }

        let close_action = if entry.close_button_enabled() {
            Some(
                div()
                    .id(self.id.slot_index("modal-close", (id.0).to_string()))
                    .w(modal_tokens.close_size)
                    .h(modal_tokens.close_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .child(
                        Icon::named("x")
                            .with_id(self.id.slot_index("modal-close-icon", (id.0).to_string()))
                            .size(f32::from(modal_tokens.close_icon_size))
                            .color(title_color)
                            .registry(icons.clone()),
                    )
                    .on_click(
                        move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                            manager_for_close.close_with_reason(id, ModalCloseReason::CloseButton);
                            window.refresh();
                        },
                    )
                    .into_any_element(),
            )
        } else {
            None
        };

        let mut panel = div()
            .id(self.id.slot("modal-panel"))
            .w(px(entry.resolved_width_px(modal_tokens)))
            .max_w_full()
            .bg(panel_bg)
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(panel_border)
            .rounded(modal_tokens.panel_radius)
            .p(modal_tokens.panel_padding);

        if entry.title_ref().is_some() || entry.close_button_enabled() {
            panel = panel.child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .mb(modal_tokens.header_margin_bottom)
                    .child(header_title)
                    .children(close_action),
            );
        }

        panel = panel
            .children(entry.body_ref().map(|body| {
                div()
                    .mb(modal_tokens.body_margin_bottom)
                    .text_color(body_color)
                    .text_size(modal_tokens.body_size)
                    .child(body.clone())
            }))
            .children(entry.render_content());

        if entry.is_confirm_kind() {
            panel = panel.child(
                div()
                    .mt(modal_tokens.actions_margin_top)
                    .flex()
                    .justify_end()
                    .gap(modal_tokens.actions_gap)
                    .child(
                        Button::new()
                            .label(entry.cancel_label_ref().clone())
                            .with_variant(crate::style::Variant::Default)
                            .on_click(move |_, window, _| {
                                manager_for_cancel.cancel(id);
                                window.refresh();
                            }),
                    )
                    .child(
                        Button::new()
                            .label(entry.confirm_label_ref().clone())
                            .with_variant(crate::style::Variant::Filled)
                            .on_click(move |_, window, _| {
                                manager_for_confirm.confirm(id);
                                window.refresh();
                            }),
                    ),
            );
        } else if entry.has_complete_action() {
            panel = panel.child(
                div()
                    .mt(modal_tokens.actions_margin_top)
                    .flex()
                    .justify_end()
                    .child(
                        Button::new()
                            .label(entry.complete_label_ref().clone())
                            .with_variant(crate::style::Variant::Filled)
                            .on_click(move |_, window, _| {
                                manager_for_complete.complete(id);
                                window.refresh();
                            }),
                    ),
            );
        }

        let panel = panel.with_enter_transition(
            self.id.slot_index("modal-enter", id.0.to_string()),
            entry.motion_ref(),
        );

        let close_on_escape = entry.close_on_escape_enabled();

        div()
            .id(self.id.slot_index("modal-root", (id.0).to_string()))
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .on_key_down(move |event, window, _cx| {
                if close_on_escape && control::is_escape_keystroke(event) {
                    manager_for_escape.close_with_reason(id, ModalCloseReason::EscapeKey);
                    window.refresh();
                }
            })
            .child(overlay)
            .child(panel)
            .into_any_element()
    }
}

impl ModalLayer {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
