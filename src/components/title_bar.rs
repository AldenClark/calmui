use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui::{
    AnyElement, ElementId, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, WindowControlArea, div,
    px, rgb,
};

use crate::id::ComponentId;
use crate::theme::ColorScheme;

use super::control;
use super::utils::{hairline_px, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type WindowCloseHandler = std::rc::Rc<dyn Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App)>;

static TITLEBAR_SHORTCUTS_INSTALLED: AtomicBool = AtomicBool::new(false);

pub fn default_title_bar_height() -> f32 {
    if cfg!(target_os = "macos") {
        30.0
    } else if cfg!(target_os = "windows") {
        32.0
    } else {
        34.0
    }
}

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

    std::mem::forget(subscription);
}

struct WindowControls {
    element: AnyElement,
    width_px: f32,
}

#[derive(IntoElement)]
pub struct TitleBar {
    id: ComponentId,
    pub(crate) visible: bool,
    pub(crate) title: Option<SharedString>,
    pub(crate) height_px: f32,
    pub(crate) immersive: bool,
    pub(crate) background: Option<Hsla>,
    pub(crate) show_window_controls: bool,
    pub(crate) on_close_window: Option<WindowCloseHandler>,
    pub(crate) slot: Option<SlotRenderer>,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
}

impl TitleBar {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            visible: true,
            title: None,
            height_px: default_title_bar_height(),
            immersive: false,
            background: None,
            show_window_controls: true,
            on_close_window: None,
            slot: None,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    /// 设置标题栏可见性。
    ///
    /// 说明：
    /// - `false` 时直接不渲染标题栏内容；
    /// - 适合在某些页面模式下彻底隐藏标题栏区域。
    pub fn visible(mut self, value: bool) -> Self {
        self.visible = value;
        self
    }

    /// 设置标题栏文本。
    ///
    /// 说明：
    /// - 若未设置则不显示文本；
    /// - macOS 全屏且存在 `slot` 时会自动隐藏该文本，避免与沉浸式内容冲突。
    pub fn title(mut self, value: impl Into<SharedString>) -> Self {
        self.title = Some(value.into());
        self
    }

    /// 设置标题栏高度（像素）。
    ///
    /// 说明：
    /// - 仅影响 `TitleBar` 组件自身高度；
    /// - 如果放入 `AppShell` 中，建议同时通过 `AppShell::title_bar_height(...)`
    ///   保持外层区域高度一致。
    pub fn height(mut self, value: f32) -> Self {
        self.height_px = value.max(0.0);
        self
    }

    /// 设置标题栏是否为沉浸模式。
    ///
    /// 说明：
    /// - `true` 时不绘制标题栏底部边框；
    /// - `false` 时绘制 1px 底部分割线。
    pub fn immersive(mut self, value: bool) -> Self {
        self.immersive = value;
        self
    }

    /// 设置标题栏背景色。
    ///
    /// 说明：
    /// - `None` 时使用透明背景（由主题与容器共同决定视觉效果）；
    /// - 传入后将强制覆盖默认背景。
    pub fn background(mut self, value: impl Into<Hsla>) -> Self {
        self.background = Some(value.into());
        self
    }

    /// 控制是否显示窗口控制按钮（交通灯 / 最小化最大化关闭）。
    pub fn show_window_controls(mut self, value: bool) -> Self {
        self.show_window_controls = value;
        self
    }

    /// 设置标题栏右侧（或平台对应位置）slot 内容。
    ///
    /// 说明：
    /// - 常用于放置工具按钮、搜索框、状态信息等；
    /// - macOS 全屏时，若存在 slot，标题文本会自动隐藏以保留操作空间。
    pub fn slot(mut self, value: impl IntoElement + 'static) -> Self {
        self.slot = Some(Box::new(|| value.into_any_element()));
        self
    }

    pub fn height_px(&self) -> f32 {
        self.height_px
    }

    pub fn has_slot_content(&self) -> bool {
        self.slot.is_some()
    }

    pub fn on_close_window(
        mut self,
        handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        if cfg!(target_os = "linux") {
            self.on_close_window = Some(std::rc::Rc::new(handler));
        }
        self
    }

    fn render_window_controls_windows(&self, window: &mut Window) -> WindowControls {
        let tokens = &self.theme.components.title_bar;
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

        let button = move |id: ElementId, text: &'static str, close: bool| {
            let (hover_bg, active_bg, hover_fg) = if close {
                let close_bg: gpui::Hsla = rgb(0xe81123).into();
                (close_bg, close_bg.opacity(0.85), gpui::white())
            } else {
                (neutral_hover_bg, neutral_active_bg, fg)
            };
            div()
                .id(id)
                .w(tokens.windows_button_width)
                .h(px(self.height_px))
                .flex()
                .items_center()
                .justify_center()
                .font_family("Segoe MDL2 Assets")
                .text_size(tokens.windows_icon_size)
                .bg(gpui::transparent_black())
                .text_color(fg)
                .cursor_pointer()
                .hover(move |style| style.bg(hover_bg).text_color(hover_fg))
                .active(move |style| style.bg(active_bg).text_color(hover_fg))
                .child(text)
        };

        let close = button(self.id.slot("win-close"), "\u{e8bb}", true)
            .window_control_area(WindowControlArea::Close);

        WindowControls {
            element: div()
                .id(self.id.slot("controls-win"))
                .flex()
                .items_center()
                .h(px(self.height_px))
                .child(
                    button(self.id.slot("win-min"), "\u{e921}", false)
                        .window_control_area(WindowControlArea::Min),
                )
                .child(
                    button(self.id.slot("win-max"), max_or_restore_icon, false)
                        .window_control_area(WindowControlArea::Max),
                )
                .child(close)
                .into_any_element(),
            width_px: f32::from(tokens.windows_button_width) * 3.0,
        }
    }

    fn render_window_controls_linux(&self) -> WindowControls {
        #[derive(Clone, Copy)]
        enum LinuxWindowAction {
            Minimize,
            Zoom,
            Close,
        }

        let tokens = &self.theme.components.title_bar;
        let fg = resolve_hsla(&self.theme, &tokens.fg);
        let bg = resolve_hsla(&self.theme, &tokens.controls_bg);

        let on_close_window = self.on_close_window.clone();
        let button =
            move |id: ElementId, text: &'static str, action: LinuxWindowAction, close: bool| {
                let hover_bg = if close {
                    rgb(0xcc3344).into()
                } else {
                    bg.opacity(0.9)
                };
                div()
                    .id(id)
                    .w(tokens.linux_button_width)
                    .h(tokens.linux_button_height)
                    .rounded(tokens.control_button_radius)
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(bg)
                    .text_color(fg)
                    .cursor_pointer()
                    .hover(move |style| style.bg(hover_bg))
                    .active(move |style| style.bg(hover_bg.opacity(0.85)))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_click({
                        let on_close_window = on_close_window.clone();
                        move |event, window, cx| {
                            cx.stop_propagation();
                            match action {
                                LinuxWindowAction::Minimize => window.minimize_window(),
                                LinuxWindowAction::Zoom => window.zoom_window(),
                                LinuxWindowAction::Close => {
                                    if let Some(handler) = on_close_window.as_ref() {
                                        (handler)(event, window, cx);
                                    } else {
                                        window.remove_window();
                                    }
                                }
                            }
                        }
                    })
                    .child(text)
            };

        WindowControls {
            element: div()
                .id(self.id.slot("controls-linux"))
                .flex()
                .items_center()
                .gap(tokens.linux_buttons_gap)
                .child(button(
                    self.id.slot("linux-min"),
                    "—",
                    LinuxWindowAction::Minimize,
                    false,
                ))
                .child(button(
                    self.id.slot("linux-max"),
                    "□",
                    LinuxWindowAction::Zoom,
                    false,
                ))
                .child(button(
                    self.id.slot("linux-close"),
                    "×",
                    LinuxWindowAction::Close,
                    true,
                ))
                .into_any_element(),
            width_px: (f32::from(tokens.linux_button_width) * 3.0)
                + (f32::from(tokens.linux_buttons_gap) * 2.0),
        }
    }

    fn render_window_controls(
        &self,
        window: &mut Window,
        fullscreen: bool,
    ) -> Option<WindowControls> {
        if !self.show_window_controls || fullscreen {
            return None;
        }

        if cfg!(target_os = "windows") {
            return Some(self.render_window_controls_windows(window));
        }

        Some(self.render_window_controls_linux())
    }
}

impl TitleBar {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(mut self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(cx);
        install_titlebar_shortcuts_once(cx);

        if !self.visible {
            return div().into_any_element();
        }

        let fullscreen = cfg!(target_os = "macos") && window.is_fullscreen();
        let has_slot = self.slot.is_some();
        // macOS 全屏下，如果没有 slot 操作区，则整条标题栏不再渲染：
        // 1) 避免只剩一条空白条带；
        // 2) 与原生沉浸式体验对齐（交通灯也已由系统隐藏）。
        if cfg!(target_os = "macos") && fullscreen && !has_slot {
            return div().into_any_element();
        }
        let immersive = self.immersive;
        let tokens = &self.theme.components.title_bar;
        let controls = self.render_window_controls(window, fullscreen);
        let controls_width = controls.as_ref().map_or(0.0, |c| c.width_px);
        let macos_controls_reserve =
            if cfg!(target_os = "macos") && self.show_window_controls && !fullscreen {
                f32::from(tokens.macos_controls_reserve)
            } else {
                0.0
            };

        let line_thickness = hairline_px(window);
        let bg_token = self
            .background
            .clone()
            .unwrap_or_else(gpui::transparent_black);
        let fg = resolve_hsla(&self.theme, &tokens.fg);
        let (padding_left, padding_right) = if cfg!(target_os = "windows") {
            (f32::from(tokens.platform_padding_left), 0.0)
        } else {
            (
                f32::from(tokens.platform_padding_left),
                f32::from(tokens.platform_padding_right),
            )
        };

        let hide_title_in_macos_fullscreen = cfg!(target_os = "macos") && fullscreen && has_slot;
        let title_element = if hide_title_in_macos_fullscreen {
            None
        } else {
            self.title.clone().map(|title| {
                div()
                    .text_size(tokens.title_size)
                    .font_weight(tokens.title_weight)
                    .text_color(fg)
                    .truncate()
                    .child(title)
            })
        };

        let mut row = div()
            .id(self.id.clone())
            .relative()
            .w_full()
            .h(px(self.height_px))
            .pl(px(padding_left))
            .pr(px(padding_right))
            .flex()
            .items_center()
            .bg(resolve_hsla(&self.theme, &bg_token))
            .text_color(fg);

        if cfg!(target_os = "macos") {
            if has_slot {
                let mut left_cluster = div()
                    .id(self.id.slot("mac-left"))
                    .flex()
                    .items_center()
                    .gap(tokens.controls_slot_gap);

                left_cluster =
                    left_cluster.child(div().w(px(macos_controls_reserve)).h(px(self.height_px)));
                if let Some(title) = title_element {
                    left_cluster = left_cluster.child(title);
                }

                row = row.child(left_cluster);
                if let Some(slot) = self.slot {
                    row = row.child(
                        div()
                            .id(self.id.slot("mac-slot"))
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_end()
                            .child(slot()),
                    );
                }
            } else {
                let left = div()
                    .id(self.id.slot("mac-left"))
                    .w(px(macos_controls_reserve))
                    .h(px(self.height_px))
                    .flex();

                let center = div()
                    .id(self.id.slot("mac-center"))
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .children(title_element);

                let right = div()
                    .id(self.id.slot("mac-right"))
                    .w(px(macos_controls_reserve))
                    .h(px(self.height_px));

                row = row.child(left).child(center).child(right);
            }
        } else if cfg!(target_os = "windows") {
            if has_slot {
                let left_title = div()
                    .id(self.id.slot("win-title"))
                    .h_full()
                    .max_w(tokens.title_max_width)
                    .min_w(tokens.title_min_width)
                    .pr(tokens.title_padding_right)
                    .flex()
                    .items_center()
                    .window_control_area(WindowControlArea::Drag)
                    .children(title_element);

                let mut middle_slot_content = div()
                    .id(self.id.slot("win-slot-content"))
                    .h_full()
                    .flex()
                    .items_center();
                if let Some(slot) = self.slot {
                    middle_slot_content = middle_slot_content.child(slot());
                }

                let middle_slot = div()
                    .id(self.id.slot("win-slot"))
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(middle_slot_content)
                    .child(
                        div()
                            .id(self.id.slot("win-slot-drag"))
                            .flex_1()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag),
                    );

                row = row.child(left_title).child(middle_slot);
                if let Some(controls) = controls {
                    row = row.child(controls.element);
                }
            } else {
                row = row
                    .child(
                        div()
                            .w(px(controls_width))
                            .h(px(self.height_px))
                            .window_control_area(WindowControlArea::Drag),
                    )
                    .child(
                        div()
                            .id(self.id.slot("win-center"))
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .window_control_area(WindowControlArea::Drag)
                            .children(title_element),
                    )
                    .child(controls.map_or_else(
                        || {
                            div()
                                .w(px(controls_width))
                                .h(px(self.height_px))
                                .into_any_element()
                        },
                        |controls| controls.element,
                    ));
            }
        } else {
            if has_slot {
                row = row.child(
                    div()
                        .id(self.id.slot("linux-title"))
                        .h_full()
                        .max_w(tokens.title_max_width)
                        .pr(tokens.title_padding_right)
                        .flex()
                        .items_center()
                        .children(title_element),
                );

                let mut slot_container = div()
                    .id(self.id.slot("linux-slot"))
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .items_center();
                if let Some(slot) = self.slot {
                    slot_container = slot_container.child(slot());
                }
                row = row.child(slot_container);

                if let Some(controls) = controls {
                    row = row.child(controls.element);
                }
            } else {
                row = row
                    .child(div().w(px(controls_width)).h(px(self.height_px)))
                    .child(
                        div()
                            .id(self.id.slot("linux-center"))
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .children(title_element),
                    )
                    .child(controls.map_or_else(
                        || {
                            div()
                                .w(px(controls_width))
                                .h(px(self.height_px))
                                .into_any_element()
                        },
                        |controls| controls.element,
                    ));
            }
        }

        let press_state_id = self.id.clone();
        let press_state_id_for_timer = self.id.clone();
        let press_state_id_for_up = self.id.clone();
        let press_state_id_for_up_out = self.id.clone();

        let mut root = row
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

        if !immersive {
            root = root.child(
                div()
                    .id(self.id.slot("bottom-border"))
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .h(line_thickness)
                    .bg(resolve_hsla(&self.theme, &tokens.border)),
            );
        }

        root.into_any_element()
    }
}

impl crate::contracts::ComponentThemeOverridable for TitleBar {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl Styled for TitleBar {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
