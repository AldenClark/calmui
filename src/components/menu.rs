use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, canvas, div, px,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;

use super::Stack;
use super::control;
use super::icon::Icon;
use super::popup::{PopupPlacement, PopupState, anchored_host};
use super::transition::TransitionExt;
use super::utils::{
    InteractionStyles, PressHandler, PressableBehavior, apply_interaction_styles,
    interaction_style, resolve_hsla, wire_pressable,
};

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

#[derive(IntoElement)]
pub struct Menu {
    id: ComponentId,
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
            id: ComponentId::default(),
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

    fn dropdown_width_px(id: &str) -> f32 {
        let width = control::f32_state(id, "dropdown-width-px", None, 0.0);
        if width >= 1.0 {
            width.max(180.0)
        } else {
            220.0
        }
    }

    fn render_dropdown(&self, is_controlled: bool, window: &gpui::Window) -> AnyElement {
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
                    .id(self.id.slot_index("item", (item.value).to_string()))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(10.0))
                    .py(px(8.0))
                    .rounded_sm()
                    .text_sm()
                    .text_color(resolve_hsla(&self.theme, &tokens.item_fg));

                if let Some(icon) = item.left_icon.clone() {
                    row = row.child(
                        Icon::named(icon.to_string())
                            .with_id(self.id.slot_index("item-icon", (item.value).to_string()))
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
                    let hover_bg = resolve_hsla(&self.theme, &tokens.item_hover_bg);
                    let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                    let click_handler: PressHandler = Rc::new(
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
                    row = apply_interaction_styles(
                        row.cursor_pointer(),
                        InteractionStyles::new()
                            .hover(interaction_style(move |style| style.bg(hover_bg)))
                            .active(interaction_style(move |style| style.bg(press_bg)))
                            .focus(interaction_style(move |style| style.bg(hover_bg))),
                    );
                    row =
                        wire_pressable(row, PressableBehavior::new().on_click(Some(click_handler)));
                }

                row
            })
            .collect::<Vec<_>>();

        let mut dropdown = Stack::vertical()
            .id(self.id.slot("dropdown"))
            .w(px(Self::dropdown_width_px(&self.id)))
            .max_w_full()
            .p_1p5()
            .gap_1()
            .rounded_md()
            .border(super::utils::quantized_stroke_px(window, 1.0))
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
            .with_enter_transition(self.id.slot("dropdown-enter"), self.motion)
            .into_any_element()
    }
}

impl Menu {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl MotionAware for Menu {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Menu {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let popup_state = PopupState::resolve(&self.id, self.opened, self.default_opened);
        let opened = if self.disabled {
            false
        } else {
            popup_state.opened
        };
        let is_controlled = popup_state.controlled;

        let mut trigger = div()
            .id(self.id.slot("trigger"))
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
        } else {
            let click_handler = if let Some(handler) = self.on_open_change.clone() {
                let id = self.id.clone();
                let next = !opened;
                Some(Rc::new(
                    move |_: &ClickEvent, window: &mut Window, cx: &mut gpui::App| {
                        if !is_controlled {
                            control::set_bool_state(&id, "opened", next);
                            window.refresh();
                        }
                        (handler)(next, window, cx);
                    },
                ) as PressHandler)
            } else if !is_controlled {
                let id = self.id.clone();
                let next = !opened;
                Some(Rc::new(
                    move |_: &ClickEvent, window: &mut Window, _cx: &mut gpui::App| {
                        control::set_bool_state(&id, "opened", next);
                        window.refresh();
                    },
                ) as PressHandler)
            } else {
                None
            };

            if let Some(click_handler) = click_handler {
                trigger = trigger.cursor_pointer();
                trigger = wire_pressable(
                    trigger,
                    PressableBehavior::new().on_click(Some(click_handler)),
                );
            } else {
                trigger = trigger.cursor_default();
            }
        }

        if opened {
            let dropdown = self.render_dropdown(is_controlled, window);
            let anchor_host = anchored_host(
                &self.id,
                "anchor-host",
                PopupPlacement::Bottom,
                self.offset_px,
                dropdown,
                22,
                true,
                false,
            );
            trigger = trigger.child(anchor_host);
        }

        div().id(self.id.clone()).relative().child(trigger)
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
