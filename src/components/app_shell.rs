use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, WindowControlArea,
    WindowOptions, div, px, rgb,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;
use crate::theme::{ColorScheme, ColorValue};

use super::control;
use super::overlay::{Overlay, OverlayMaterialMode};
use super::scroll_area::{ScrollArea, ScrollDirection};
use super::utils::resolve_hsla;

fn default_title_bar_height() -> f32 {
    if cfg!(target_os = "macos") {
        30.0
    } else if cfg!(target_os = "windows") {
        32.0
    } else {
        34.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TitleBarControlPlacement {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitleTextSlot {
    Left,
    Center,
    Right,
}

fn default_control_placement() -> TitleBarControlPlacement {
    if cfg!(target_os = "macos") {
        TitleBarControlPlacement::Left
    } else {
        TitleBarControlPlacement::Right
    }
}

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
static TITLEBAR_SHORTCUTS_INSTALLED: AtomicBool = AtomicBool::new(false);

fn install_titlebar_shortcuts_once(cx: &mut gpui::App) {
    if TITLEBAR_SHORTCUTS_INSTALLED.swap(true, Ordering::AcqRel) {
        return;
    }

    let subscription = cx.observe_keystrokes(|event, window, _cx| {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        let secondary_only =
            modifiers.secondary() && !modifiers.alt && !modifiers.shift && !modifiers.function;

        if secondary_only && key == "m" {
            window.minimize_window();
            return;
        }
        if secondary_only && key == "w" {
            window.remove_window();
            return;
        }

        if cfg!(target_os = "macos")
            && key == "f"
            && modifiers.platform
            && modifiers.control
            && !modifiers.alt
            && !modifiers.shift
            && !modifiers.function
        {
            window.toggle_fullscreen();
            return;
        }

        if !cfg!(target_os = "macos") && key == "f11" && !modifiers.modified() {
            window.toggle_fullscreen();
        }
    });

    // Keep this global observer for app lifetime.
    std::mem::forget(subscription);
}

pub struct TitleBar {
    id: String,
    visible: bool,
    title: Option<SharedString>,
    title_slot: TitleTextSlot,
    height_px: f32,
    immersive: bool,
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
            title_slot: TitleTextSlot::Center,
            height_px: default_title_bar_height(),
            immersive: false,
            background: None,
            show_window_controls: true,
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

    pub fn clear_title(mut self) -> Self {
        self.title = None;
        self
    }

    pub fn title_slot(mut self, value: TitleTextSlot) -> Self {
        self.title_slot = value;
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

    pub fn immersive(mut self, value: bool) -> Self {
        self.immersive = value;
        self
    }

    pub fn show_window_controls(mut self, value: bool) -> Self {
        self.show_window_controls = value;
        self
    }

    pub fn height_px(&self) -> f32 {
        self.height_px
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

    pub fn has_any_slot_content(&self) -> bool {
        self.title.is_some()
            || !self.left_slots.is_empty()
            || !self.center_slots.is_empty()
            || !self.right_slots.is_empty()
    }

    fn render_window_controls_windows(&self, window: &mut Window) -> (AnyElement, f32) {
        let fg = resolve_hsla(&self.theme, &self.theme.components.title_bar.fg);
        let (neutral_hover_bg, neutral_active_bg) = match self.theme.color_scheme {
            ColorScheme::Dark => (gpui::white().opacity(0.14), gpui::white().opacity(0.22)),
            ColorScheme::Light => (gpui::black().opacity(0.08), gpui::black().opacity(0.14)),
        };
        let max_or_restore_icon = if window.is_maximized() {
            "\u{e923}"
        } else {
            "\u{e922}"
        };

        let button = |id: String, text: &'static str, area: WindowControlArea, close: bool| {
            let (hover_bg, active_bg, hover_fg) = if close {
                let close_bg: gpui::Hsla = rgb(0xe81123).into();
                (close_bg, close_bg.opacity(0.85), gpui::white())
            } else {
                (neutral_hover_bg, neutral_active_bg, fg)
            };
            div()
                .id(id)
                .w(px(45.0))
                .h(px(self.height_px))
                .flex()
                .items_center()
                .justify_center()
                .font_family("Segoe MDL2 Assets")
                .text_size(px(10.0))
                .bg(gpui::transparent_black())
                .text_color(fg)
                .cursor_pointer()
                .hover(move |style| style.bg(hover_bg).text_color(hover_fg))
                .active(move |style| style.bg(active_bg).text_color(hover_fg))
                .window_control_area(area)
                .child(text)
        };

        (
            div()
                .id(format!("{}-controls-win", self.id))
                .flex()
                .items_center()
                .h(px(self.height_px))
                .child(button(
                    format!("{}-win-min", self.id),
                    "\u{e921}",
                    WindowControlArea::Min,
                    false,
                ))
                .child(button(
                    format!("{}-win-max", self.id),
                    max_or_restore_icon,
                    WindowControlArea::Max,
                    false,
                ))
                .child(button(
                    format!("{}-win-close", self.id),
                    "\u{e8bb}",
                    WindowControlArea::Close,
                    true,
                ))
                .into_any_element(),
            135.0,
        )
    }

    fn render_window_controls_macos(&self) -> (AnyElement, f32) {
        let circle =
            |id: String,
             color: gpui::Hsla,
             area: WindowControlArea,
             handler: Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>| {
                div()
                    .id(id)
                    .w(px(12.0))
                    .h(px(12.0))
                    .rounded_full()
                    .bg(color)
                    .border_1()
                    .border_color(color.opacity(0.75))
                    .cursor_pointer()
                    .hover(move |style| style.bg(color.opacity(0.92)))
                    .active(move |style| style.bg(color.opacity(0.84)))
                    .window_control_area(area)
                    .on_click(move |event, window, cx| (handler)(event, window, cx))
            };

        let on_close: Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)> =
            Rc::new(|_, window, _| window.remove_window());
        let on_minimize: Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)> =
            Rc::new(|_, window, _| window.minimize_window());
        let on_zoom: Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)> =
            Rc::new(|_, window, _| window.toggle_fullscreen());

        (
            div()
                .id(format!("{}-controls-macos", self.id))
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(circle(
                    format!("{}-mac-close", self.id),
                    rgb(0xff5f57).into(),
                    WindowControlArea::Close,
                    on_close,
                ))
                .child(circle(
                    format!("{}-mac-min", self.id),
                    rgb(0xfebc2e).into(),
                    WindowControlArea::Min,
                    on_minimize,
                ))
                .child(circle(
                    format!("{}-mac-max", self.id),
                    rgb(0x28c840).into(),
                    WindowControlArea::Max,
                    on_zoom,
                ))
                .into_any_element(),
            52.0,
        )
    }

    fn render_window_controls_linux(&self) -> (AnyElement, f32) {
        let tokens = &self.theme.components.title_bar;
        let fg = resolve_hsla(&self.theme, &tokens.fg);
        let bg = resolve_hsla(&self.theme, &tokens.controls_bg);

        let button = |id: String, text: &'static str, area: WindowControlArea, close: bool| {
            let hover_bg = if close {
                rgb(0xcc3344).into()
            } else {
                bg.opacity(0.9)
            };
            div()
                .id(id)
                .w(px(28.0))
                .h(px(24.0))
                .rounded_sm()
                .flex()
                .items_center()
                .justify_center()
                .bg(bg)
                .text_color(fg)
                .cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .active(move |style| style.bg(hover_bg.opacity(0.85)))
                .window_control_area(area)
                .child(text)
        };

        (
            div()
                .id(format!("{}-controls-linux", self.id))
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(button(
                    format!("{}-linux-min", self.id),
                    "—",
                    WindowControlArea::Min,
                    false,
                ))
                .child(button(
                    format!("{}-linux-max", self.id),
                    "□",
                    WindowControlArea::Max,
                    false,
                ))
                .child(button(
                    format!("{}-linux-close", self.id),
                    "×",
                    WindowControlArea::Close,
                    true,
                ))
                .into_any_element(),
            96.0,
        )
    }

    fn render_window_controls(
        &self,
        window: &mut Window,
    ) -> Option<(AnyElement, TitleBarControlPlacement, f32)> {
        if !self.show_window_controls {
            return None;
        }

        let (controls, width) = if cfg!(target_os = "macos") {
            self.render_window_controls_macos()
        } else if cfg!(target_os = "windows") {
            self.render_window_controls_windows(window)
        } else {
            self.render_window_controls_linux()
        };

        Some((controls, default_control_placement(), width))
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
    fn render(mut self, window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        install_titlebar_shortcuts_once(_cx);
        if !self.visible {
            return div().into_any_element();
        }
        let macos_fullscreen = cfg!(target_os = "macos") && window.is_fullscreen();

        let tokens = &self.theme.components.title_bar;
        let bg_token = if self.immersive && self.background.is_none() {
            ColorValue::Custom("#00000000".to_string())
        } else {
            self.background.clone().unwrap_or_else(|| tokens.bg.clone())
        };
        let fg = resolve_hsla(&self.theme, &tokens.fg);
        let (padding_left, padding_right) = if cfg!(target_os = "windows") {
            (12.0, 0.0)
        } else {
            (12.0, 12.0)
        };

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
        //.window_control_area(WindowControlArea::Drag);
        let mut right = div()
            .id(format!("{}-right", self.id))
            .flex()
            .items_center()
            .gap_2();

        if cfg!(target_os = "macos") && !macos_fullscreen && !self.show_window_controls {
            // Reserve native traffic-light area on macOS. Native controls are provided by system titlebar.
            left = left.child(
                div()
                    .w(px(76.0))
                    .h(px(self.height_px))
                    .flex_none()
                    .invisible(),
            );
        }

        if let Some(title) = self.title.clone() {
            let title_element = div()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(fg)
                .child(title);
            match self.title_slot {
                TitleTextSlot::Left => {
                    left = left.child(title_element);
                }
                TitleTextSlot::Center => {
                    center = center.child(title_element);
                }
                TitleTextSlot::Right => {
                    right = right.child(title_element);
                }
            }
        }

        let window_controls = self.render_window_controls(window);

        for slot in self.left_slots {
            left = left.child(slot());
        }
        for slot in self.center_slots {
            center = center.child(slot());
        }
        for slot in self.right_slots {
            right = right.child(slot());
        }

        if let Some((controls, placement, width_px)) = window_controls {
            match placement {
                TitleBarControlPlacement::Left => {
                    left = left.child(controls);
                    right = right.child(div().w(px(width_px)).h(px(1.0)).invisible());
                }
                TitleBarControlPlacement::Right => {
                    left = left.child(div().w(px(width_px)).h(px(1.0)).invisible());
                    right = right.child(controls);
                }
            }
        }

        let root_id = self.id.clone();
        let mut root = div()
            .id(root_id)
            .w_full()
            .h(px(self.height_px))
            .pl(px(padding_left))
            .pr(px(padding_right))
            .flex()
            .items_center()
            .justify_between()
            .bg(resolve_hsla(&self.theme, &bg_token))
            .text_color(fg)
            .child(left)
            .child(center)
            .child(right);

        let press_state_id = self.id.clone();
        let press_state_id_for_timer = self.id.clone();
        let press_state_id_for_up = self.id.clone();
        let press_state_id_for_up_out = self.id.clone();
        root = root
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                control::set_bool_state(&press_state_id, "mouse-pressing", true);
                control::set_bool_state(&press_state_id, "mouse-long-press-fired", false);

                if event.click_count >= 2 {
                    control::set_bool_state(&press_state_id, "mouse-pressing", false);
                    window.titlebar_double_click();
                    window.refresh();
                    return;
                }

                let window_handle = window.window_handle();
                let id = press_state_id_for_timer.clone();
                cx.spawn(async move |cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(520))
                        .await;
                    let _ = window_handle.update(cx, |_, window, _| {
                        let pressing = control::bool_state(&id, "mouse-pressing", None, false);
                        let already_fired =
                            control::bool_state(&id, "mouse-long-press-fired", None, false);
                        if pressing && !already_fired {
                            control::set_bool_state(&id, "mouse-long-press-fired", true);
                            control::set_bool_state(&id, "mouse-pressing", false);
                            window.titlebar_double_click();
                            window.refresh();
                        }
                    });
                })
                .detach();
            })
            .on_mouse_up(MouseButton::Left, move |_, window, _| {
                control::set_bool_state(&press_state_id_for_up, "mouse-pressing", false);
                window.refresh();
            })
            .on_mouse_up_out(MouseButton::Left, move |_, window, _| {
                control::set_bool_state(&press_state_id_for_up_out, "mouse-pressing", false);
                window.refresh();
            });

        if !self.immersive {
            root = root
                .border_1()
                .border_color(resolve_hsla(&self.theme, &tokens.border));
        }

        root.into_any_element()
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

    pub fn title_bar_immersive(mut self, value: bool) -> Self {
        self.title_bar_immersive = value;
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
        let macos_fullscreen = cfg!(target_os = "macos") && _window.is_fullscreen();
        let mut title_bar = self.title_bar.take();
        let titlebar_height_px = title_bar
            .as_ref()
            .map(TitleBar::height_px)
            .unwrap_or_else(default_title_bar_height);
        let hide_titlebar_on_macos_fullscreen = macos_fullscreen
            && title_bar
                .as_ref()
                .is_some_and(|titlebar| !titlebar.has_any_slot_content());
        let show_title_bar = title_bar.is_some() && !hide_titlebar_on_macos_fullscreen;
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
