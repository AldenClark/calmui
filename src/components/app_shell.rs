use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;
use crate::theme::ColorValue;

use super::control;
use super::overlay::{Overlay, OverlayMaterialMode};
use super::utils::resolve_hsla;

fn default_title_bar_height() -> f32 {
    if cfg!(target_os = "macos") {
        28.0
    } else if cfg!(target_os = "windows") {
        32.0
    } else {
        30.0
    }
}

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type OverlaySidebarChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

pub struct TitleBar {
    id: String,
    visible: bool,
    title: Option<SharedString>,
    height_px: f32,
    background: Option<ColorValue>,
    show_window_controls: bool,
    left_slots: Vec<SlotRenderer>,
    center_slots: Vec<SlotRenderer>,
    right_slots: Vec<SlotRenderer>,
    theme: crate::theme::LocalTheme,
}

impl TitleBar {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("title-bar"),
            visible: true,
            title: None,
            height_px: default_title_bar_height(),
            background: None,
            show_window_controls: !cfg!(target_os = "macos"),
            left_slots: Vec::new(),
            center_slots: Vec::new(),
            right_slots: Vec::new(),
            theme: crate::theme::LocalTheme::default(),
        }
    }

    pub fn visible(mut self, value: bool) -> Self {
        self.visible = value;
        self
    }

    pub fn title(mut self, value: impl Into<SharedString>) -> Self {
        self.title = Some(value.into());
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.height_px = value.max(20.0);
        self
    }

    pub fn background(mut self, value: ColorValue) -> Self {
        self.background = Some(value);
        self
    }

    pub fn show_window_controls(mut self, value: bool) -> Self {
        self.show_window_controls = value;
        self
    }

    pub fn left_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.left_slots
            .push(Box::new(|| content.into_any_element()));
        self
    }

    pub fn center_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.center_slots
            .push(Box::new(|| content.into_any_element()));
        self
    }

    pub fn right_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.right_slots
            .push(Box::new(|| content.into_any_element()));
        self
    }

    fn render_window_controls_windows(&self) -> AnyElement {
        let tokens = &self.theme.components.title_bar;
        let fg = resolve_hsla(&self.theme, &self.theme.components.title_bar.fg);
        let controls_bg = resolve_hsla(&self.theme, &tokens.controls_bg);

        let button = |id: String, text: &'static str| {
            div()
                .id(id)
                .w(px(36.0))
                .h(px(self.height_px.max(24.0)))
                .flex()
                .items_center()
                .justify_center()
                .bg(controls_bg)
                .text_color(fg)
                .child(text)
        };

        div()
            .id(format!("{}-controls-win", self.id))
            .flex()
            .items_center()
            .child(button(format!("{}-win-min", self.id), "-"))
            .child(button(format!("{}-win-max", self.id), "□"))
            .child(button(format!("{}-win-close", self.id), "×"))
            .into_any_element()
    }
}

impl WithId for TitleBar {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for TitleBar {
    fn render(mut self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        if !self.visible {
            return div().into_any_element();
        }

        let tokens = &self.theme.components.title_bar;
        let bg_token = self.background.clone().unwrap_or_else(|| tokens.bg.clone());
        let fg = resolve_hsla(&self.theme, &tokens.fg);

        let mut left = div()
            .id(format!("{}-left", self.id))
            .flex()
            .items_center()
            .gap_2();
        let mut center = div()
            .id(format!("{}-center", self.id))
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .gap_2();
        let mut right = div()
            .id(format!("{}-right", self.id))
            .flex()
            .items_center()
            .gap_2();

        if cfg!(target_os = "macos") {
            // On macOS, rely on native traffic lights to avoid duplicate controls.
            left = left.child(div().w(px(64.0)).h(px(1.0)).flex_none());
        }

        if let Some(title) = self.title.clone() {
            center = center.child(
                div()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(fg)
                    .child(title),
            );
        }

        let windows_controls = if self.show_window_controls && !cfg!(target_os = "macos") {
            Some(self.render_window_controls_windows())
        } else {
            None
        };

        for slot in self.left_slots {
            left = left.child(slot());
        }
        for slot in self.center_slots {
            center = center.child(slot());
        }
        for slot in self.right_slots {
            right = right.child(slot());
        }

        if let Some(controls) = windows_controls {
            right = right.child(controls);
        }

        div()
            .id(self.id)
            .w_full()
            .h(px(self.height_px))
            .px(px(12.0))
            .flex()
            .items_center()
            .justify_between()
            .bg(resolve_hsla(&self.theme, &bg_token))
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.border))
            .text_color(fg)
            .child(left)
            .child(center)
            .child(right)
            .into_any_element()
    }
}

impl IntoElement for TitleBar {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidebarPosition {
    Left,
    Right,
}

pub struct Sidebar {
    id: String,
    width_px: f32,
    position: SidebarPosition,
    background: Option<ColorValue>,
    header: Option<SlotRenderer>,
    content: Option<SlotRenderer>,
    footer: Option<SlotRenderer>,
    theme: crate::theme::LocalTheme,
}

impl Sidebar {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("sidebar"),
            width_px: 248.0,
            position: SidebarPosition::Left,
            background: None,
            header: None,
            content: None,
            footer: None,
            theme: crate::theme::LocalTheme::default(),
        }
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width_px = value.max(140.0);
        self
    }

    pub fn position(mut self, value: SidebarPosition) -> Self {
        self.position = value;
        self
    }

    pub fn background(mut self, value: ColorValue) -> Self {
        self.background = Some(value);
        self
    }

    pub fn header(mut self, content: impl IntoElement + 'static) -> Self {
        self.header = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn content(mut self, content: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn footer(mut self, content: impl IntoElement + 'static) -> Self {
        self.footer = Some(Box::new(|| content.into_any_element()));
        self
    }
}

impl WithId for Sidebar {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Sidebar {
    fn render(mut self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let sidebar_id = self.id.clone();
        let tokens = &self.theme.components.sidebar;
        let bg_token = self.background.clone().unwrap_or_else(|| tokens.bg.clone());

        let mut root = div()
            .id(sidebar_id.clone())
            .w(px(self.width_px))
            .h_full()
            .flex()
            .flex_col()
            .bg(resolve_hsla(&self.theme, &bg_token))
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.border));

        if let Some(header) = self.header {
            root = root.child(
                div()
                    .p_3()
                    .text_color(resolve_hsla(&self.theme, &tokens.header_fg))
                    .child(header()),
            );
        }

        if let Some(content) = self.content {
            root = root.child(
                div()
                    .id(format!("{}-sidebar-content", sidebar_id))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_3()
                    .text_color(resolve_hsla(&self.theme, &tokens.content_fg))
                    .child(content()),
            );
        } else {
            root = root.child(div().flex_1().min_h_0());
        }

        if let Some(footer) = self.footer {
            root = root.child(
                div()
                    .p_3()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.footer_fg))
                    .child(footer()),
            );
        }

        root.into_any_element()
    }
}

impl IntoElement for Sidebar {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppShellLayout {
    Standard,
    SidebarOverlay,
    DualSidebar,
    Inspector,
    SplitView,
    Focus,
}

pub struct AppShell {
    id: String,
    layout: AppShellLayout,
    title_bar: Option<TitleBar>,
    sidebar: Option<Sidebar>,
    secondary_sidebar: Option<Sidebar>,
    content: Option<SlotRenderer>,
    secondary_content: Option<SlotRenderer>,
    split_secondary_width_px: f32,
    overlay_sidebar_opened: Option<bool>,
    overlay_sidebar_default_opened: bool,
    on_overlay_sidebar_change: Option<OverlaySidebarChangeHandler>,
    theme: crate::theme::LocalTheme,
}

impl AppShell {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("app-shell"),
            layout: AppShellLayout::Standard,
            title_bar: None,
            sidebar: None,
            secondary_sidebar: None,
            content: None,
            secondary_content: None,
            split_secondary_width_px: 320.0,
            overlay_sidebar_opened: None,
            overlay_sidebar_default_opened: false,
            on_overlay_sidebar_change: None,
            theme: crate::theme::LocalTheme::default(),
        }
    }

    pub fn layout(mut self, value: AppShellLayout) -> Self {
        self.layout = value;
        self
    }

    pub fn title_bar(mut self, value: TitleBar) -> Self {
        self.title_bar = Some(value);
        self
    }

    pub fn sidebar(mut self, value: Sidebar) -> Self {
        self.sidebar = Some(value);
        self
    }

    pub fn secondary_sidebar(mut self, value: Sidebar) -> Self {
        self.secondary_sidebar = Some(value);
        self
    }

    pub fn content(mut self, value: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| value.into_any_element()));
        self
    }

    pub fn secondary_content(mut self, value: impl IntoElement + 'static) -> Self {
        self.secondary_content = Some(Box::new(|| value.into_any_element()));
        self
    }

    pub fn split_secondary_width(mut self, value: f32) -> Self {
        self.split_secondary_width_px = value.max(160.0);
        self
    }

    pub fn overlay_sidebar_opened(mut self, value: bool) -> Self {
        self.overlay_sidebar_opened = Some(value);
        self
    }

    pub fn overlay_sidebar_default_opened(mut self, value: bool) -> Self {
        self.overlay_sidebar_default_opened = value;
        self
    }

    pub fn on_overlay_sidebar_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_overlay_sidebar_change = Some(Rc::new(handler));
        self
    }

    fn resolved_overlay_sidebar_opened(&self) -> bool {
        control::bool_state(
            &self.id,
            "overlay-sidebar-opened",
            self.overlay_sidebar_opened,
            self.overlay_sidebar_default_opened,
        )
    }
}

impl WithId for AppShell {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for AppShell {
    fn render(mut self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.app_shell;
        let content = self
            .content
            .take()
            .map(|content| content())
            .unwrap_or_else(|| div().into_any_element());

        let body_text = self.theme.resolve_hsla(&self.theme.semantic.text_primary);
        let body_bg = resolve_hsla(&self.theme, &tokens.bg);

        let mut root = div()
            .id(self.id.clone())
            .size_full()
            .flex()
            .flex_col()
            .bg(body_bg)
            .text_color(body_text);

        if let Some(title_bar) = self.title_bar.take() {
            root = root.child(title_bar);
        }

        let mut body = div()
            .id(format!("{}-body", self.id))
            .flex_1()
            .min_h_0()
            .w_full();

        match self.layout {
            AppShellLayout::Focus => {
                body = body.child(div().size_full().child(content));
            }
            AppShellLayout::Standard => {
                let mut row = div().size_full().flex().flex_row().min_h_0();
                if let Some(sidebar) = self.sidebar.take() {
                    row = row.child(sidebar);
                }
                row = row.child(div().flex_1().min_w_0().min_h_0().child(content));
                body = body.child(row);
            }
            AppShellLayout::DualSidebar => {
                let mut row = div().size_full().flex().flex_row().min_h_0();
                if let Some(sidebar) = self.sidebar.take() {
                    row = row.child(sidebar);
                }
                row = row.child(div().flex_1().min_w_0().min_h_0().child(content));
                if let Some(sidebar) = self.secondary_sidebar.take() {
                    row = row.child(sidebar);
                }
                body = body.child(row);
            }
            AppShellLayout::Inspector => {
                let mut row = div().size_full().flex().flex_row().min_h_0();
                if let Some(sidebar) = self.sidebar.take() {
                    row = row.child(sidebar);
                }
                row = row.child(div().flex_1().min_w_0().min_h_0().child(content));
                if let Some(inspector) = self.secondary_sidebar.take() {
                    row = row.child(inspector.position(SidebarPosition::Right));
                }
                body = body.child(row);
            }
            AppShellLayout::SplitView => {
                let mut row = div().size_full().flex().flex_row().min_h_0();
                row = row.child(div().flex_1().min_w_0().min_h_0().child(content));
                if let Some(secondary) = self.secondary_content.take() {
                    row = row.child(
                        div()
                            .w(px(self.split_secondary_width_px))
                            .h_full()
                            .border_1()
                            .border_color(resolve_hsla(
                                &self.theme,
                                &self.theme.semantic.border_subtle,
                            ))
                            .bg(resolve_hsla(&self.theme, &self.theme.semantic.bg_surface))
                            .child(secondary()),
                    );
                }
                body = body.child(row);
            }
            AppShellLayout::SidebarOverlay => {
                let opened = self.resolved_overlay_sidebar_opened();
                let is_controlled = self.overlay_sidebar_opened.is_some();
                let handler = self.on_overlay_sidebar_change.clone();
                let id_for_overlay = self.id.clone();

                let mut host = div()
                    .id(format!("{}-overlay-host", self.id))
                    .relative()
                    .size_full()
                    .child(div().size_full().child(content));

                if opened {
                    if self.sidebar.is_some() {
                        host = host.child(
                            Overlay::new()
                                .with_id(format!("{}-sidebar-overlay-mask", self.id))
                                .material_mode(OverlayMaterialMode::TintOnly)
                                .frosted(false)
                                .opacity(1.0)
                                .on_click(
                                    move |_: &ClickEvent,
                                          window: &mut Window,
                                          cx: &mut gpui::App| {
                                        if !is_controlled {
                                            control::set_bool_state(
                                                &id_for_overlay,
                                                "overlay-sidebar-opened",
                                                false,
                                            );
                                            window.refresh();
                                        }
                                        if let Some(on_change) = handler.as_ref() {
                                            (on_change)(false, window, cx);
                                        }
                                    },
                                ),
                        );
                    }

                    if let Some(sidebar) = self.sidebar.take() {
                        host =
                            host.child(div().absolute().top_0().left_0().h_full().child(sidebar));
                    }
                }

                body = body.child(host);
            }
        }

        root.child(body).into_any_element()
    }
}

impl IntoElement for AppShell {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for TitleBar {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemePatchable for Sidebar {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemePatchable for AppShell {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
