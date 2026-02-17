use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, Corner, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, anchored, canvas,
    deferred, div, point, px,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;

use super::Stack;
use super::control;
use super::icon::Icon;
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type ItemClickHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;
type OpenChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuItem {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
    pub left_icon: Option<SharedString>,
}

impl MenuItem {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            left_icon: None,
        }
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn left_icon(mut self, value: impl Into<SharedString>) -> Self {
        self.left_icon = Some(value.into());
        self
    }
}

pub struct Menu {
    id: String,
    opened: Option<bool>,
    default_opened: bool,
    disabled: bool,
    offset_px: f32,
    close_on_click_outside: bool,
    close_on_item_click: bool,
    trigger: Option<SlotRenderer>,
    items: Vec<MenuItem>,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_item_click: Option<ItemClickHandler>,
    on_open_change: Option<OpenChangeHandler>,
}

impl Menu {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("menu"),
            opened: None,
            default_opened: false,
            disabled: false,
            offset_px: 4.0,
            close_on_click_outside: true,
            close_on_item_click: true,
            trigger: None,
            items: Vec::new(),
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_item_click: None,
            on_open_change: None,
        }
    }

    pub fn opened(mut self, value: bool) -> Self {
        self.opened = Some(value);
        self
    }

    pub fn default_opened(mut self, value: bool) -> Self {
        self.default_opened = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn offset(mut self, value: f32) -> Self {
        self.offset_px = value.max(0.0);
        self
    }

    pub fn close_on_click_outside(mut self, value: bool) -> Self {
        self.close_on_click_outside = value;
        self
    }

    pub fn close_on_item_click(mut self, value: bool) -> Self {
        self.close_on_item_click = value;
        self
    }

    pub fn trigger(mut self, value: impl IntoElement + 'static) -> Self {
        self.trigger = Some(Box::new(|| value.into_any_element()));
        self
    }

    pub fn item(mut self, value: MenuItem) -> Self {
        self.items.push(value);
        self
    }

    pub fn items(mut self, values: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(values);
        self
    }

    pub fn on_item_click(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_item_click = Some(Rc::new(handler));
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }

    fn resolved_opened(&self) -> bool {
        control::bool_state(&self.id, "opened", self.opened, self.default_opened)
    }

    fn dropdown_width_px(id: &str) -> f32 {
        control::text_state(id, "dropdown-width-px", None, String::new())
            .parse::<f32>()
            .ok()
            .filter(|width| *width >= 1.0)
            .map(|width| width.max(180.0))
            .unwrap_or(220.0)
    }

    fn render_dropdown(&self, is_controlled: bool) -> AnyElement {
        let tokens = &self.theme.components.menu;
        let on_item_click = self.on_item_click.clone();
        let on_open_change = self.on_open_change.clone();
        let close_on_item_click = self.close_on_item_click;
        let menu_id = self.id.clone();

        let rows = self
            .items
            .iter()
            .cloned()
            .map(|item| {
                let mut row = div()
                    .id(format!("{}-item-{}", self.id, item.value))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(10.0))
                    .py(px(8.0))
                    .rounded_sm()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.item_fg))
                    .hover({
                        let hover_bg = resolve_hsla(&self.theme, &tokens.item_hover_bg);
                        move |style| style.bg(hover_bg)
                    });

                if let Some(icon) = item.left_icon.clone() {
                    row = row.child(
                        Icon::named_outline(icon.to_string())
                            .with_id(format!("{}-item-icon-{}", self.id, item.value))
                            .size(14.0)
                            .color(resolve_hsla(&self.theme, &tokens.icon)),
                    );
                }
                row = row.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .child(item.label.clone()),
                );

                if item.disabled {
                    row = row
                        .cursor_default()
                        .text_color(resolve_hsla(&self.theme, &tokens.item_disabled_fg))
                        .hover(|style| style);
                } else {
                    let value = item.value.clone();
                    let on_item_click = on_item_click.clone();
                    let on_open_change = on_open_change.clone();
                    let menu_id = menu_id.clone();
                    row = row.cursor_pointer().on_click(
                        move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                            if let Some(handler) = on_item_click.as_ref() {
                                (handler)(value.clone(), window, cx);
                            }

                            if close_on_item_click {
                                if !is_controlled {
                                    control::set_bool_state(&menu_id, "opened", false);
                                    window.refresh();
                                }
                                if let Some(handler) = on_open_change.as_ref() {
                                    (handler)(false, window, cx);
                                }
                            }
                        },
                    );
                }

                row.into_any_element()
            })
            .collect::<Vec<_>>();

        let mut dropdown = Stack::vertical()
            .id(format!("{}-dropdown", self.id))
            .w(px(Self::dropdown_width_px(&self.id)))
            .max_w_full()
            .p_1p5()
            .gap_1()
            .rounded_md()
            .border_1()
            .border_color(resolve_hsla(&self.theme, &tokens.dropdown_border))
            .bg(resolve_hsla(&self.theme, &tokens.dropdown_bg))
            .shadow_sm()
            .children(rows);

        if self.close_on_click_outside {
            if let Some(handler) = self.on_open_change.clone() {
                let menu_id = self.id.clone();
                dropdown = dropdown.on_mouse_down_out(move |_, window, cx| {
                    if !is_controlled {
                        control::set_bool_state(&menu_id, "opened", false);
                        window.refresh();
                    }
                    (handler)(false, window, cx);
                });
            } else if !is_controlled {
                let menu_id = self.id.clone();
                dropdown = dropdown.on_mouse_down_out(move |_, window, _cx| {
                    control::set_bool_state(&menu_id, "opened", false);
                    window.refresh();
                });
            }
        }

        dropdown
            .with_enter_transition(format!("{}-dropdown-enter", self.id), self.motion)
            .into_any_element()
    }
}

impl WithId for Menu {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl MotionAware for Menu {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Menu {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let opened = if self.disabled {
            false
        } else {
            self.resolved_opened()
        };
        let is_controlled = self.opened.is_some();

        let mut trigger = div()
            .id(format!("{}-trigger", self.id))
            .relative()
            .cursor_pointer()
            .child(
                self.trigger
                    .take()
                    .map(|content| content())
                    .unwrap_or_else(|| div().child("Menu").into_any_element()),
            )
            .child({
                let id_for_width = self.id.clone();
                canvas(
                    move |bounds, _, _cx| {
                        control::set_text_state(
                            &id_for_width,
                            "dropdown-width-px",
                            format!("{:.2}", f32::from(bounds.size.width)),
                        );
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
            });

        if self.disabled {
            trigger = trigger.opacity(0.55).cursor_default();
        } else if let Some(handler) = self.on_open_change.clone() {
            let id = self.id.clone();
            let next = !opened;
            trigger = trigger.on_click(move |_, window, cx| {
                if !is_controlled {
                    control::set_bool_state(&id, "opened", next);
                    window.refresh();
                }
                (handler)(next, window, cx);
            });
        } else if !is_controlled {
            let id = self.id.clone();
            let next = !opened;
            trigger = trigger.on_click(move |_, window, _cx| {
                control::set_bool_state(&id, "opened", next);
                window.refresh();
            });
        }

        if opened {
            let dropdown = self.render_dropdown(is_controlled);
            let anchor_host = div()
                .id(format!("{}-anchor-host", self.id))
                .absolute()
                .bottom_0()
                .left_0()
                .w(px(0.0))
                .h(px(0.0))
                .child(
                    deferred(
                        anchored()
                            .anchor(Corner::TopLeft)
                            .offset(point(px(0.0), px(self.offset_px)))
                            .snap_to_window_with_margin(px(8.0))
                            .child(dropdown),
                    )
                    .priority(22),
                );
            trigger = trigger.child(anchor_host);
        }

        div().id(self.id.clone()).relative().child(trigger)
    }
}

impl IntoElement for Menu {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Menu {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(MenuItem);
crate::impl_disableable!(Menu);
crate::impl_openable!(Menu);

impl gpui::Styled for Menu {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
