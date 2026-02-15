use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, WindowOptions, div, px,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;

use super::control;
use super::overlay::{Overlay, OverlayMaterialMode};
use super::scroll_area::{ScrollArea, ScrollDirection};
use super::title_bar::{TitleBar, default_title_bar_height};
use super::utils::resolve_hsla;

#[derive(Clone, Debug)]
pub struct AppShellWindowConfig {
    macos_traffic_light_position: Option<gpui::Point<gpui::Pixels>>,
}

impl Default for AppShellWindowConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AppShellWindowConfig {
    pub fn new() -> Self {
        Self {
            macos_traffic_light_position: None,
        }
    }

    pub fn macos_traffic_light_position(mut self, value: gpui::Point<gpui::Pixels>) -> Self {
        self.macos_traffic_light_position = Some(value);
        self
    }

    pub fn apply_to_window_options(&self, mut options: WindowOptions) -> WindowOptions {
        if cfg!(target_os = "macos") {
            let mut titlebar = options.titlebar.unwrap_or_default();
            titlebar.appears_transparent = true;
            titlebar.title = None;
            titlebar.traffic_light_position = self.macos_traffic_light_position;
            options.titlebar = Some(titlebar);
            return options;
        }

        if cfg!(target_os = "windows") {
            let mut titlebar = options.titlebar.unwrap_or_default();
            titlebar.appears_transparent = true;
            options.titlebar = Some(titlebar);
            return options;
        }

        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            options.window_decorations = Some(gpui::WindowDecorations::Client);
        }

        options
    }
}

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type OverlaySidebarChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SidebarPosition {
    Left,
    Right,
}

struct Sidebar {
    id: String,
    width_px: f32,
    position: SidebarPosition,
    background: Option<Hsla>,
    header: Option<SlotRenderer>,
    content: Option<SlotRenderer>,
    footer: Option<SlotRenderer>,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
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
            style: gpui::StyleRefinement::default(),
        }
    }

    fn position(mut self, value: SidebarPosition) -> Self {
        self.position = value;
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
                    .text_color(resolve_hsla(&self.theme, &tokens.content_fg))
                    .child(
                        ScrollArea::new()
                            .with_id(format!("{}-sidebar-scroll", sidebar_id))
                            .direction(ScrollDirection::Vertical)
                            .bordered(false)
                            .padding(crate::style::Size::Md)
                            .child(content()),
                    ),
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
    title_bar_immersive: bool,
    sidebar: Option<Sidebar>,
    secondary_sidebar: Option<Sidebar>,
    content: Option<SlotRenderer>,
    secondary_content: Option<SlotRenderer>,
    split_secondary_width_px: f32,
    overlay_sidebar_opened: Option<bool>,
    overlay_sidebar_default_opened: bool,
    on_overlay_sidebar_change: Option<OverlaySidebarChangeHandler>,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
}

impl AppShell {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("app-shell"),
            layout: AppShellLayout::Standard,
            title_bar: None,
            title_bar_immersive: false,
            sidebar: None,
            secondary_sidebar: None,
            content: None,
            secondary_content: None,
            split_secondary_width_px: 320.0,
            overlay_sidebar_opened: None,
            overlay_sidebar_default_opened: false,
            on_overlay_sidebar_change: None,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn layout(mut self, value: AppShellLayout) -> Self {
        self.layout = value;
        self
    }

    pub fn title_bar_immersive(mut self, value: bool) -> Self {
        self.title_bar_immersive = value;
        self
    }

    pub fn show_title_bar(mut self, value: bool) -> Self {
        if value {
            self.ensure_title_bar();
        } else {
            self.title_bar = None;
        }
        self
    }

    pub fn title_bar_visible(mut self, value: bool) -> Self {
        self.ensure_title_bar().visible = value;
        self
    }

    pub fn title_bar_title(mut self, value: impl Into<SharedString>) -> Self {
        self.ensure_title_bar().title = Some(value.into());
        self
    }

    pub fn title_bar_clear_title(mut self) -> Self {
        self.ensure_title_bar().title = None;
        self
    }

    pub fn title_bar_height(mut self, value: f32) -> Self {
        self.ensure_title_bar().height_px = value.max(20.0);
        self
    }

    pub fn title_bar_background(mut self, value: impl Into<Hsla>) -> Self {
        self.ensure_title_bar().background = Some(value.into());
        self
    }

    pub fn title_bar_show_window_controls(mut self, value: bool) -> Self {
        self.ensure_title_bar().show_window_controls = value;
        self
    }

    pub fn title_bar_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.ensure_title_bar().slot = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn title_bar_clear_slot(mut self) -> Self {
        self.ensure_title_bar().slot = None;
        self
    }

    pub fn show_sidebar(mut self, value: bool) -> Self {
        if value {
            self.ensure_sidebar();
        } else {
            self.sidebar = None;
        }
        self
    }

    pub fn sidebar_width(mut self, value: f32) -> Self {
        self.ensure_sidebar().width_px = value.max(140.0);
        self
    }

    pub fn sidebar_background(mut self, value: impl Into<Hsla>) -> Self {
        self.ensure_sidebar().background = Some(value.into());
        self
    }

    pub fn sidebar_header(mut self, content: impl IntoElement + 'static) -> Self {
        self.ensure_sidebar().header = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn sidebar_content(mut self, content: impl IntoElement + 'static) -> Self {
        self.ensure_sidebar().content = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn sidebar_footer(mut self, content: impl IntoElement + 'static) -> Self {
        self.ensure_sidebar().footer = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn show_secondary_sidebar(mut self, value: bool) -> Self {
        if value {
            self.ensure_secondary_sidebar();
        } else {
            self.secondary_sidebar = None;
        }
        self
    }

    pub fn secondary_sidebar_width(mut self, value: f32) -> Self {
        self.ensure_secondary_sidebar().width_px = value.max(140.0);
        self
    }

    pub fn secondary_sidebar_background(mut self, value: impl Into<Hsla>) -> Self {
        self.ensure_secondary_sidebar().background = Some(value.into());
        self
    }

    pub fn secondary_sidebar_header(mut self, content: impl IntoElement + 'static) -> Self {
        self.ensure_secondary_sidebar().header = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn secondary_sidebar_content(mut self, content: impl IntoElement + 'static) -> Self {
        self.ensure_secondary_sidebar().content = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn secondary_sidebar_footer(mut self, content: impl IntoElement + 'static) -> Self {
        self.ensure_secondary_sidebar().footer = Some(Box::new(|| content.into_any_element()));
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

    fn ensure_title_bar(&mut self) -> &mut TitleBar {
        self.title_bar.get_or_insert_with(TitleBar::new)
    }

    fn ensure_sidebar(&mut self) -> &mut Sidebar {
        self.sidebar.get_or_insert_with(Sidebar::new)
    }

    fn ensure_secondary_sidebar(&mut self) -> &mut Sidebar {
        self.secondary_sidebar.get_or_insert_with(Sidebar::new)
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
        let macos_fullscreen = cfg!(target_os = "macos") && _window.is_fullscreen();
        let mut title_bar = self.title_bar.take();
        let hide_title_bar = macos_fullscreen
            && title_bar
                .as_ref()
                .is_some_and(|title_bar| !title_bar.has_slot_content());
        let titlebar_height_px = title_bar
            .as_ref()
            .map(TitleBar::height_px)
            .unwrap_or_else(default_title_bar_height);
        let show_title_bar = title_bar.is_some() && !hide_title_bar;
        let content_top_padding = if show_title_bar && !self.title_bar_immersive {
            titlebar_height_px
        } else {
            0.0
        };

        if !show_title_bar {
            title_bar = None;
        }

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
            .relative()
            .bg(body_bg)
            .text_color(body_text);

        let mut body = div()
            .id(format!("{}-body", self.id))
            .flex_1()
            .min_h_0()
            .w_full()
            .pt(px(content_top_padding));

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

        if let Some(title_bar) = title_bar {
            root = root.child(body).child(
                div()
                    .id(format!("{}-titlebar-overlay", self.id))
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .child(title_bar),
            );
        } else {
            root = root.child(body);
        }

        root.into_any_element()
    }
}

impl IntoElement for AppShell {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Sidebar {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for AppShell {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for AppShell {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for Sidebar {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
