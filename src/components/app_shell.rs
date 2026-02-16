use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Refineable, RenderOnce, Styled, Window, WindowControlArea, div, px,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;

use super::control;
use super::overlay::{Overlay, OverlayMaterialMode};
use super::scroll_area::{ScrollArea, ScrollDirection};
use super::utils::resolve_hsla;

/// AppShell 内部用于存储“侧边栏 overlay 开关”的状态 key。
const SIDEBAR_OVERLAY_STATE_SLOT: &str = "sidebar-overlay-opened";
/// AppShell 内部用于存储“属性面板 overlay 开关”的状态 key。
const INSPECTOR_OVERLAY_STATE_SLOT: &str = "inspector-overlay-opened";

/// AppShell 区域插槽渲染器。
type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
/// AppShell overlay 区域开关变化回调。
type OverlayOpenChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

/// 侧边栏容器组件。
///
/// 该组件用于承载“顶部 / 主体 / 底部”三个区域，并提供统一主题 token。
/// `AppShell` 只负责摆放该组件，不关心它的内部内容结构。
pub struct Sidebar {
    /// 组件唯一 id。
    id: String,
    /// 侧边栏固定宽度（像素）。
    /// `None` 表示占满父容器宽度（推荐与 `AppShell.sidebar_width` 搭配）。
    width_px: Option<f32>,
    /// 自定义背景色；`None` 时使用 `theme.components.sidebar.bg`。
    background: Option<Hsla>,
    /// 顶部区域内容。
    header: Option<SlotRenderer>,
    /// 主体内容区域。
    content: Option<SlotRenderer>,
    /// 底部区域内容。
    footer: Option<SlotRenderer>,
    /// 局部主题（用于读取 token 以及组件级主题覆盖）。
    theme: crate::theme::LocalTheme,
    /// 通用样式精修。
    style: gpui::StyleRefinement,
}

impl Sidebar {
    /// 创建侧边栏组件。
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("sidebar"),
            width_px: None,
            background: None,
            header: None,
            content: None,
            footer: None,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    /// 设置侧边栏宽度。
    pub fn width(mut self, value: f32) -> Self {
        self.width_px = Some(value.max(120.0));
        self
    }

    /// 设置侧边栏背景色。
    pub fn background(mut self, value: impl Into<Hsla>) -> Self {
        self.background = Some(value.into());
        self
    }

    /// 设置顶部区域内容。
    pub fn header(mut self, value: impl IntoElement + 'static) -> Self {
        self.header = Some(Box::new(|| value.into_any_element()));
        self
    }

    /// 设置主体区域内容。
    pub fn content(mut self, value: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| value.into_any_element()));
        self
    }

    /// 设置底部区域内容。
    pub fn footer(mut self, value: impl IntoElement + 'static) -> Self {
        self.footer = Some(Box::new(|| value.into_any_element()));
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
        let tokens = &self.theme.components.sidebar;
        let bg_token = self.background.unwrap_or_else(|| tokens.bg.clone());

        let mut root = div()
            .id(self.id.clone())
            .h_full()
            .flex()
            .flex_col()
            .bg(resolve_hsla(&self.theme, &bg_token))
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.border));

        root = if let Some(width_px) = self.width_px {
            root.w(px(width_px))
        } else {
            root.w_full()
        };

        if let Some(header) = self.header.take() {
            root = root.child(
                div()
                    .p_3()
                    .text_color(resolve_hsla(&self.theme, &tokens.header_fg))
                    .child(header()),
            );
        }

        if let Some(content) = self.content.take() {
            root = root.child(
                div()
                    .id(format!("{}-content", self.id))
                    .flex_1()
                    .min_h_0()
                    .text_color(resolve_hsla(&self.theme, &tokens.content_fg))
                    .child(
                        ScrollArea::new()
                            .with_id(format!("{}-scroll", self.id))
                            .direction(ScrollDirection::Vertical)
                            .bordered(false)
                            .padding(crate::style::Size::Md)
                            .child(content()),
                    ),
            );
        } else {
            root = root.child(div().flex_1().min_h_0());
        }

        if let Some(footer) = self.footer.take() {
            root = root.child(
                div()
                    .p_3()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.footer_fg))
                    .child(footer()),
            );
        }

        root.style().refine(&self.style);
        root
    }
}

impl IntoElement for Sidebar {
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

impl gpui::Styled for Sidebar {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

/// 侧边区域的展示模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelMode {
    /// 内联模式：区域参与主布局占位。
    Inline,
    /// 浮层模式：区域悬浮在内容之上。
    Overlay,
}

/// 区域容器的基础外观配置。
#[derive(Clone, Debug)]
pub struct PaneChrome {
    /// 背景色；`None` 时使用区域默认背景策略。
    background: Option<Hsla>,
    /// 是否绘制 1px 边框。
    bordered: bool,
    /// 区域圆角半径（像素）；`None` 表示不设置。
    radius_px: Option<f32>,
}

impl Default for PaneChrome {
    fn default() -> Self {
        Self {
            background: None,
            bordered: false,
            radius_px: None,
        }
    }
}

impl PaneChrome {
    /// 创建默认外观配置。
    pub fn new() -> Self {
        Self::default()
    }

    /// 覆盖背景色。
    pub fn background(mut self, value: impl Into<Hsla>) -> Self {
        self.background = Some(value.into());
        self
    }

    /// 将背景设置为透明。
    pub fn transparent(mut self) -> Self {
        self.background = Some(gpui::transparent_black());
        self
    }

    /// 控制边框显示。
    pub fn bordered(mut self, value: bool) -> Self {
        self.bordered = value;
        self
    }

    /// 设置圆角半径（像素）。
    pub fn radius(mut self, value: f32) -> Self {
        self.radius_px = Some(value.max(0.0));
        self
    }
}

/// 应用级壳层布局组件。
///
/// 设计目标：
/// 1) 只负责区域编排与 overlay 行为；
/// 2) 各区域内容由调用方自行提供（例如 `TitleBar` / `Sidebar` / 自定义组件）；
/// 3) 提供少量高频基础样式能力（尺寸、背景、边框、圆角）。
pub struct AppShell {
    /// 组件唯一 id，用于受控/非受控状态 key。
    id: String,
    /// 顶部区域内容。
    title_bar: Option<SlotRenderer>,
    /// 顶部区域是否采用沉浸模式。
    /// `true` 时标题栏悬浮在主体上方，不为主体留出高度。
    /// `false` 时标题栏占据普通布局高度。
    title_bar_immersive: bool,
    /// 左侧区域内容。
    sidebar: Option<SlotRenderer>,
    /// 中央主内容区域（必填）。
    content: SlotRenderer,
    /// 右侧属性面板内容。
    inspector: Option<SlotRenderer>,
    /// 内容区底部面板。
    bottom_panel: Option<SlotRenderer>,
    /// 顶部区域高度。
    title_bar_height_px: f32,
    /// 左侧区域宽度。
    sidebar_width_px: f32,
    /// 右侧属性面板宽度。
    inspector_width_px: f32,
    /// 底部面板高度。
    bottom_panel_height_px: f32,
    /// 左侧区域展示模式。
    sidebar_mode: PanelMode,
    /// 右侧属性面板展示模式。
    inspector_mode: PanelMode,
    /// 左侧 overlay 开关（受控值）。
    sidebar_overlay_opened: Option<bool>,
    /// 左侧 overlay 开关（非受控初始值）。
    sidebar_overlay_default_opened: bool,
    /// 左侧 overlay 开关变化回调。
    on_sidebar_overlay_open_change: Option<OverlayOpenChangeHandler>,
    /// 右侧 overlay 开关（受控值）。
    inspector_overlay_opened: Option<bool>,
    /// 右侧 overlay 开关（非受控初始值）。
    inspector_overlay_default_opened: bool,
    /// 右侧 overlay 开关变化回调。
    on_inspector_overlay_open_change: Option<OverlayOpenChangeHandler>,
    /// 顶部区域外观。
    title_bar_chrome: PaneChrome,
    /// 左侧区域外观。
    sidebar_chrome: PaneChrome,
    /// 右侧属性面板外观。
    inspector_chrome: PaneChrome,
    /// 底部面板外观。
    bottom_panel_chrome: PaneChrome,
    /// 局部主题（用于读取 token 以及组件级主题覆盖）。
    theme: crate::theme::LocalTheme,
    /// 通用样式精修。
    style: gpui::StyleRefinement,
}

impl AppShell {
    /// 创建 AppShell。`content` 为必填区域。
    #[track_caller]
    pub fn new(content: impl IntoElement + 'static) -> Self {
        Self {
            id: stable_auto_id("app-shell"),
            title_bar: None,
            title_bar_immersive: false,
            sidebar: None,
            content: Box::new(|| content.into_any_element()),
            inspector: None,
            bottom_panel: None,
            title_bar_height_px: 44.0,
            sidebar_width_px: 260.0,
            inspector_width_px: 320.0,
            bottom_panel_height_px: 180.0,
            sidebar_mode: PanelMode::Inline,
            inspector_mode: PanelMode::Inline,
            sidebar_overlay_opened: None,
            sidebar_overlay_default_opened: false,
            on_sidebar_overlay_open_change: None,
            inspector_overlay_opened: None,
            inspector_overlay_default_opened: false,
            on_inspector_overlay_open_change: None,
            title_bar_chrome: PaneChrome::default(),
            sidebar_chrome: PaneChrome::default(),
            inspector_chrome: PaneChrome::default(),
            bottom_panel_chrome: PaneChrome::default(),
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    /// 替换主内容区域。
    pub fn content(mut self, value: impl IntoElement + 'static) -> Self {
        self.content = Box::new(|| value.into_any_element());
        self
    }

    /// 设置顶部区域内容。
    pub fn title_bar(mut self, value: impl IntoElement + 'static) -> Self {
        self.title_bar = Some(Box::new(|| value.into_any_element()));
        self
    }

    /// 设置顶部区域是否为沉浸模式。
    pub fn title_bar_immersive(mut self, value: bool) -> Self {
        self.title_bar_immersive = value;
        self
    }

    /// 设置左侧区域内容。
    pub fn sidebar(mut self, value: impl IntoElement + 'static) -> Self {
        self.sidebar = Some(Box::new(|| value.into_any_element()));
        self
    }

    /// 设置右侧属性面板内容。
    pub fn inspector(mut self, value: impl IntoElement + 'static) -> Self {
        self.inspector = Some(Box::new(|| value.into_any_element()));
        self
    }

    /// 设置底部面板内容。
    pub fn bottom_panel(mut self, value: impl IntoElement + 'static) -> Self {
        self.bottom_panel = Some(Box::new(|| value.into_any_element()));
        self
    }

    /// 设置顶部区域高度。
    pub fn title_bar_height(mut self, value: f32) -> Self {
        self.title_bar_height_px = value.max(0.0);
        self
    }

    /// 设置左侧区域宽度。
    pub fn sidebar_width(mut self, value: f32) -> Self {
        self.sidebar_width_px = value.max(120.0);
        self
    }

    /// 设置右侧属性面板宽度。
    pub fn inspector_width(mut self, value: f32) -> Self {
        self.inspector_width_px = value.max(120.0);
        self
    }

    /// 设置底部面板高度。
    pub fn bottom_panel_height(mut self, value: f32) -> Self {
        self.bottom_panel_height_px = value.max(80.0);
        self
    }

    /// 设置左侧区域展示模式。
    pub fn sidebar_mode(mut self, value: PanelMode) -> Self {
        self.sidebar_mode = value;
        self
    }

    /// 设置右侧属性面板展示模式。
    pub fn inspector_mode(mut self, value: PanelMode) -> Self {
        self.inspector_mode = value;
        self
    }

    /// 设置左侧 overlay 开关（受控）。
    pub fn sidebar_overlay_opened(mut self, value: bool) -> Self {
        self.sidebar_overlay_opened = Some(value);
        self
    }

    /// 设置左侧 overlay 开关默认值（非受控）。
    pub fn sidebar_overlay_default_opened(mut self, value: bool) -> Self {
        self.sidebar_overlay_default_opened = value;
        self
    }

    /// 监听左侧 overlay 开关变化。
    pub fn on_sidebar_overlay_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_sidebar_overlay_open_change = Some(Rc::new(handler));
        self
    }

    /// 设置右侧 overlay 开关（受控）。
    pub fn inspector_overlay_opened(mut self, value: bool) -> Self {
        self.inspector_overlay_opened = Some(value);
        self
    }

    /// 设置右侧 overlay 开关默认值（非受控）。
    pub fn inspector_overlay_default_opened(mut self, value: bool) -> Self {
        self.inspector_overlay_default_opened = value;
        self
    }

    /// 监听右侧 overlay 开关变化。
    pub fn on_inspector_overlay_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_inspector_overlay_open_change = Some(Rc::new(handler));
        self
    }

    /// 配置顶部区域基础外观。
    pub fn title_bar_chrome(mut self, configure: impl FnOnce(PaneChrome) -> PaneChrome) -> Self {
        self.title_bar_chrome = configure(self.title_bar_chrome);
        self
    }

    /// 配置左侧区域基础外观。
    pub fn sidebar_chrome(mut self, configure: impl FnOnce(PaneChrome) -> PaneChrome) -> Self {
        self.sidebar_chrome = configure(self.sidebar_chrome);
        self
    }

    /// 配置右侧属性面板基础外观。
    pub fn inspector_chrome(mut self, configure: impl FnOnce(PaneChrome) -> PaneChrome) -> Self {
        self.inspector_chrome = configure(self.inspector_chrome);
        self
    }

    /// 配置底部面板基础外观。
    pub fn bottom_panel_chrome(mut self, configure: impl FnOnce(PaneChrome) -> PaneChrome) -> Self {
        self.bottom_panel_chrome = configure(self.bottom_panel_chrome);
        self
    }

    /// 解析左侧 overlay 的最终可见状态。
    fn resolved_sidebar_overlay_opened(&self) -> bool {
        control::bool_state(
            &self.id,
            SIDEBAR_OVERLAY_STATE_SLOT,
            self.sidebar_overlay_opened,
            self.sidebar_overlay_default_opened,
        )
    }

    /// 解析右侧 overlay 的最终可见状态。
    fn resolved_inspector_overlay_opened(&self) -> bool {
        control::bool_state(
            &self.id,
            INSPECTOR_OVERLAY_STATE_SLOT,
            self.inspector_overlay_opened,
            self.inspector_overlay_default_opened,
        )
    }

    /// 为区域容器应用背景和圆角。
    fn apply_surface<T: Styled>(mut node: T, chrome: &PaneChrome, default_bg: Hsla) -> T {
        let bg = chrome.background.unwrap_or(default_bg);
        node = node.bg(bg);

        if let Some(radius_px) = chrome.radius_px {
            node = node.rounded(px(radius_px));
        }

        node
    }

    /// 将一个区域包装为统一的容器结构。
    fn wrap_region(
        &self,
        id: String,
        content: AnyElement,
        chrome: &PaneChrome,
        default_bg: Hsla,
    ) -> gpui::Stateful<gpui::Div> {
        let mut region =
            Self::apply_surface(div().id(id).size_full(), chrome, default_bg).child(content);

        if chrome.bordered {
            region = region.border_1().border_color(resolve_hsla(
                &self.theme,
                &self.theme.semantic.border_subtle,
            ));
        }

        region
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

        let app_tokens = &self.theme.components.app_shell;
        let title_tokens = &self.theme.components.title_bar;
        let body_bg = resolve_hsla(&self.theme, &app_tokens.bg);
        let text_color = resolve_hsla(&self.theme, &self.theme.semantic.text_primary);

        let has_sidebar = self.sidebar.is_some();
        let has_inspector = self.inspector.is_some();

        let sidebar_overlay_visible = has_sidebar
            && self.sidebar_mode == PanelMode::Overlay
            && self.resolved_sidebar_overlay_opened();
        let inspector_overlay_visible = has_inspector
            && self.inspector_mode == PanelMode::Overlay
            && self.resolved_inspector_overlay_opened();

        // 根容器：固定为“顶部 + 主体”的结构。
        let mut root = div()
            .id(self.id.clone())
            .size_full()
            .flex()
            .flex_col()
            .relative()
            .bg(body_bg)
            .text_color(text_color);

        // 顶部区域（可选）。
        // - 非沉浸：标题栏参与正常布局，占据固定高度。
        // - 沉浸：标题栏悬浮在主体上层，不占据主体高度。
        let mut title_bar_overlay: Option<AnyElement> = None;
        if let Some(title_bar) = self.title_bar.take() {
            let title_default_bg = resolve_hsla(&self.theme, &title_tokens.bg);
            let title_region = self
                .wrap_region(
                    format!("{}-title-bar", self.id),
                    title_bar(),
                    &self.title_bar_chrome,
                    title_default_bg,
                )
                .h(px(self.title_bar_height_px.max(0.0)))
                .w_full()
                .flex_none();

            if self.title_bar_immersive {
                let mut overlay = div()
                    .id(format!("{}-title-bar-overlay", self.id))
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(self.title_bar_height_px.max(0.0)))
                    .child(title_region);

                if cfg!(any(
                    target_os = "windows",
                    target_os = "linux",
                    target_os = "freebsd"
                )) {
                    overlay = overlay.window_control_area(WindowControlArea::Drag);
                }

                if cfg!(any(target_os = "linux", target_os = "freebsd")) {
                    let drag_state_id = format!("{}-titlebar-drag", self.id);
                    let drag_state_id_move = drag_state_id.clone();
                    let drag_state_id_up = drag_state_id.clone();
                    let drag_state_id_up_out = drag_state_id.clone();
                    overlay = overlay
                        .on_mouse_down(MouseButton::Left, move |event, window, _| {
                            if event.click_count >= 2 {
                                return;
                            }
                            control::set_bool_state(&drag_state_id, "pressing", true);
                            window.refresh();
                        })
                        .on_mouse_up(MouseButton::Left, move |_, window, _| {
                            control::set_bool_state(&drag_state_id_up, "pressing", false);
                            window.refresh();
                        })
                        .on_mouse_up_out(MouseButton::Left, move |_, window, _| {
                            control::set_bool_state(&drag_state_id_up_out, "pressing", false);
                            window.refresh();
                        })
                        .on_mouse_move(move |_, window, _| {
                            let pressing =
                                control::bool_state(&drag_state_id_move, "pressing", None, false);
                            if pressing {
                                control::set_bool_state(&drag_state_id_move, "pressing", false);
                                window.start_window_move();
                                window.refresh();
                            }
                        });
                }

                title_bar_overlay = Some(overlay.into_any_element());
            } else {
                root = root.child(title_region);
            }
        }

        // 主体容器：用于承载 inline 布局与 overlay 浮层。
        let mut body_host = div()
            .id(format!("{}-body", self.id))
            .flex_1()
            .min_h_0()
            .w_full()
            .relative();

        // 主体行布局：sidebar(可选) + 中心列(content 必填 + bottom_panel 可选) + inspector(可选)。
        let mut row = div()
            .id(format!("{}-row", self.id))
            .size_full()
            .flex()
            .flex_row()
            .min_h_0();

        if self.sidebar_mode == PanelMode::Inline {
            if let Some(sidebar) = self.sidebar.take() {
                let sidebar_default_bg = resolve_hsla(&self.theme, &self.theme.semantic.bg_soft);
                let sidebar_region = self
                    .wrap_region(
                        format!("{}-sidebar-inline", self.id),
                        sidebar(),
                        &self.sidebar_chrome,
                        sidebar_default_bg,
                    )
                    .w(px(self.sidebar_width_px))
                    .h_full()
                    .flex_none();
                row = row.child(sidebar_region);
            }
        }

        let mut center = div()
            .id(format!("{}-center", self.id))
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col();

        // `content` 为 `FnOnce`，这里先取出再调用，避免对 `self` 产生部分移动。
        let content_renderer =
            std::mem::replace(&mut self.content, Box::new(|| div().into_any_element()));
        let content_element = content_renderer();

        center = center.child(
            div()
                .id(format!("{}-content", self.id))
                .flex_1()
                .min_h_0()
                .min_w_0()
                .child(content_element),
        );

        if let Some(bottom_panel) = self.bottom_panel.take() {
            let bottom_default_bg = resolve_hsla(&self.theme, &self.theme.semantic.bg_surface);
            let bottom_region = self
                .wrap_region(
                    format!("{}-bottom-panel", self.id),
                    bottom_panel(),
                    &self.bottom_panel_chrome,
                    bottom_default_bg,
                )
                .h(px(self.bottom_panel_height_px))
                .w_full()
                .flex_none();
            center = center.child(bottom_region);
        }

        row = row.child(center);

        if self.inspector_mode == PanelMode::Inline {
            if let Some(inspector) = self.inspector.take() {
                let inspector_default_bg = resolve_hsla(&self.theme, &self.theme.semantic.bg_soft);
                let inspector_region = self
                    .wrap_region(
                        format!("{}-inspector-inline", self.id),
                        inspector(),
                        &self.inspector_chrome,
                        inspector_default_bg,
                    )
                    .w(px(self.inspector_width_px))
                    .h_full()
                    .flex_none();
                row = row.child(inspector_region);
            }
        }

        body_host = body_host.child(row);

        // overlay 模式：如果任一区域开启，则绘制统一遮罩。
        if sidebar_overlay_visible || inspector_overlay_visible {
            let shell_id = self.id.clone();
            let sidebar_controlled = self.sidebar_overlay_opened.is_some();
            let inspector_controlled = self.inspector_overlay_opened.is_some();
            let on_sidebar_change = self.on_sidebar_overlay_open_change.clone();
            let on_inspector_change = self.on_inspector_overlay_open_change.clone();

            body_host = body_host.child(
                Overlay::new()
                    .with_id(format!("{}-panels-overlay", self.id))
                    .material_mode(OverlayMaterialMode::TintOnly)
                    .frosted(false)
                    .opacity(1.0)
                    .on_click(
                        move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                            let mut need_refresh = false;

                            if sidebar_overlay_visible {
                                if !sidebar_controlled {
                                    control::set_bool_state(
                                        &shell_id,
                                        SIDEBAR_OVERLAY_STATE_SLOT,
                                        false,
                                    );
                                    need_refresh = true;
                                }
                                if let Some(handler) = on_sidebar_change.as_ref() {
                                    (handler)(false, window, cx);
                                }
                            }

                            if inspector_overlay_visible {
                                if !inspector_controlled {
                                    control::set_bool_state(
                                        &shell_id,
                                        INSPECTOR_OVERLAY_STATE_SLOT,
                                        false,
                                    );
                                    need_refresh = true;
                                }
                                if let Some(handler) = on_inspector_change.as_ref() {
                                    (handler)(false, window, cx);
                                }
                            }

                            if need_refresh {
                                window.refresh();
                            }
                        },
                    ),
            );
        }

        // overlay 左侧区域。
        if self.sidebar_mode == PanelMode::Overlay && sidebar_overlay_visible {
            if let Some(sidebar) = self.sidebar.take() {
                let sidebar_default_bg = gpui::transparent_black();
                let sidebar_region = self
                    .wrap_region(
                        format!("{}-sidebar-overlay", self.id),
                        sidebar(),
                        &self.sidebar_chrome,
                        sidebar_default_bg,
                    )
                    .w(px(self.sidebar_width_px))
                    .h_full()
                    .flex_none();

                body_host = body_host.child(
                    div()
                        .id(format!("{}-sidebar-overlay-host", self.id))
                        .absolute()
                        .top_0()
                        .left_0()
                        .h_full()
                        .child(sidebar_region),
                );
            }
        }

        // overlay 右侧属性面板。
        if self.inspector_mode == PanelMode::Overlay && inspector_overlay_visible {
            if let Some(inspector) = self.inspector.take() {
                let inspector_default_bg = gpui::transparent_black();
                let inspector_region = self
                    .wrap_region(
                        format!("{}-inspector-overlay", self.id),
                        inspector(),
                        &self.inspector_chrome,
                        inspector_default_bg,
                    )
                    .w(px(self.inspector_width_px))
                    .h_full()
                    .flex_none();

                body_host = body_host.child(
                    div()
                        .id(format!("{}-inspector-overlay-host", self.id))
                        .absolute()
                        .top_0()
                        .right_0()
                        .h_full()
                        .child(inspector_region),
                );
            }
        }

        root = root.child(body_host);
        if let Some(title_bar_overlay) = title_bar_overlay {
            root = root.child(title_bar_overlay);
        }
        root.style().refine(&self.style);

        root.into_any_element()
    }
}

impl IntoElement for AppShell {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
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
