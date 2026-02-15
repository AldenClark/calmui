use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::feedback::{ToastEntry, ToastKind, ToastManager, ToastPosition};
use crate::icon::{IconRegistry, IconSource};
use crate::motion::{MotionConfig, MotionTransition, TransitionPreset};
use crate::overlay::{Layer, ModalEntry, ModalManager};
use crate::{contracts::WithId, id::stable_auto_id};

use super::icon::Icon;
use super::overlay::{Overlay, OverlayCoverage, OverlayMaterialMode};
use super::primitives::v_stack;
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

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
            ToastKind::Info => IconSource::named_outline("info-circle"),
            ToastKind::Success => IconSource::named_outline("circle-check"),
            ToastKind::Warning => IconSource::named_outline("alert-triangle"),
            ToastKind::Error => IconSource::named_outline("alert-circle"),
            ToastKind::Loading => IconSource::named_outline("loader-2"),
        }
    }

    fn render_toast_card(&self, entry: ToastEntry) -> AnyElement {
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
            .border_1()
            .border_color(fg.opacity(0.32))
            .bg(fg.opacity(0.08))
            .cursor_pointer()
            .text_color(fg)
            .hover(|style| style.bg(fg.opacity(0.16)))
            .active(|style| style.bg(fg.opacity(0.24)))
            .child(
                Icon::named_outline("x")
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
            .border_1()
            .border_color(resolve_hsla(
                &self.theme,
                &self.theme.semantic.border_subtle,
            ))
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
                            v_stack()
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
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
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

            let cards = toasts
                .into_iter()
                .map(|entry| self.render_toast_card(entry))
                .collect::<Vec<_>>();

            root = root.child(Self::anchor_for(position).children(cards));
        }

        root.into_any_element()
    }
}

impl IntoElement for ToastLayer {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
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

    fn render_modal(&self, entry: ModalEntry) -> AnyElement {
        let panel_bg = resolve_hsla(&self.theme, &self.theme.components.modal.panel_bg);
        let panel_border = resolve_hsla(&self.theme, &self.theme.components.modal.panel_border);
        let title_color = resolve_hsla(&self.theme, &self.theme.components.modal.title);
        let body_color = resolve_hsla(&self.theme, &self.theme.components.modal.body);

        let manager_for_overlay = self.manager.clone();
        let id_for_overlay = entry.id;
        let manager_for_close = self.manager.clone();
        let id_for_close = entry.id;
        let icons = self.icons.clone();

        let close_on_click_outside = entry.close_on_click_outside;
        let overlay = Overlay::new()
            .with_id(format!("{}-overlay", self.id))
            .coverage(OverlayCoverage::Window)
            .material_mode(OverlayMaterialMode::Auto)
            .color(self.theme.components.modal.overlay_bg.clone())
            .on_click(
                move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                    if close_on_click_outside && let Some(id) = id_for_overlay {
                        manager_for_overlay.close(id);
                        window.refresh();
                    }
                },
            );

        let panel = div()
            .id(format!("{}-modal-panel", self.id))
            .w_96()
            .max_w_full()
            .bg(panel_bg)
            .border_1()
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
                    .child(
                        div()
                            .text_color(title_color)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(entry.title),
                    )
                    .child(
                        div()
                            .id(format!(
                                "{}-modal-close-{}",
                                self.id,
                                entry.id.map(|value| value.0).unwrap_or_default()
                            ))
                            .cursor_pointer()
                            .child(
                                Icon::named_outline("x")
                                    .with_id(format!(
                                        "{}-modal-close-icon-{}",
                                        self.id,
                                        entry.id.map(|value| value.0).unwrap_or_default()
                                    ))
                                    .size(14.0)
                                    .color(title_color)
                                    .registry(icons),
                            )
                            .on_click(
                                move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                                    if let Some(id) = id_for_close {
                                        manager_for_close.close(id);
                                        window.refresh();
                                    }
                                },
                            ),
                    ),
            )
            .child(div().text_color(body_color).child(entry.body))
            .with_enter_transition(format!("{}-modal-enter", self.id), self.motion);

        div()
            .id(format!("{}-modal-root", self.id))
            .size_full()
            .absolute()
            .top_0()
            .left_0()
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
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let stack = self.manager.list();
        let Some(entry) = stack.last().cloned() else {
            return div().into_any_element();
        };

        match entry.layer {
            Layer::Modal | Layer::Popover | Layer::Dropdown | Layer::Tooltip | Layer::Toast => {
                self.render_modal(entry)
            }
            Layer::Base | Layer::DragPreview => div().into_any_element(),
        }
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
