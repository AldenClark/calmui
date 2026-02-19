use std::rc::Rc;

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, canvas, div, px,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;

use super::Stack;
use super::icon::Icon;
use super::interaction_adapter::{ActivateHandler, PressAdapter, bind_press_adapter};
use super::menu_state::{self, MenuState, MenuStateInput};
use super::popup::{PopupPlacement, anchored_host};
use super::transition::TransitionExt;
use super::utils::{InteractionStyles, apply_interaction_styles, interaction_style, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type ItemClickHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;
type OpenChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuItem {
    pub value: SharedString,
    pub label: Option<SharedString>,
    pub disabled: bool,
    pub left_icon: Option<SharedString>,
}

impl MenuItem {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: None,
            disabled: false,
            left_icon: None,
        }
    }

    pub fn labeled(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(value).label(label)
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
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

    fn render_dropdown(
        &self,
        is_controlled: bool,
        dropdown_width_px: f32,
        window: &gpui::Window,
    ) -> AnyElement {
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
                let row_id = self.id.slot_index("item", item.value.to_string());
                let mut row = div()
                    .id(row_id.clone())
                    .flex()
                    .items_center()
                    .gap(tokens.item_gap)
                    .px(tokens.item_padding_x)
                    .py(tokens.item_padding_y)
                    .rounded(tokens.item_radius)
                    .text_size(tokens.item_size)
                    .text_color(resolve_hsla(&self.theme, &tokens.item_fg));

                if let Some(icon) = item.left_icon.clone() {
                    row = row.child(
                        Icon::named(icon.to_string())
                            .with_id(self.id.slot_index("item-icon", (item.value).to_string()))
                            .size(f32::from(tokens.item_icon_size))
                            .color(resolve_hsla(&self.theme, &tokens.icon)),
                    );
                }
                let mut label_node = div().flex_1().min_w_0().truncate();
                if let Some(label) = item.label.clone() {
                    label_node = label_node.child(label);
                }
                row = row.child(label_node);

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
                    let activate_handler: ActivateHandler =
                        Rc::new(move |window: &mut Window, cx: &mut gpui::App| {
                            if let Some(handler) = on_item_click.as_ref() {
                                (handler)(value.clone(), window, cx);
                            }

                            if close_on_item_click {
                                if menu_state::on_item_click(
                                    &menu_id,
                                    is_controlled,
                                    close_on_item_click,
                                ) {
                                    window.refresh();
                                }
                                if let Some(handler) = on_open_change.as_ref() {
                                    (handler)(false, window, cx);
                                }
                            }
                        });
                    row = apply_interaction_styles(
                        row.cursor_pointer(),
                        InteractionStyles::new()
                            .hover(interaction_style(move |style| style.bg(hover_bg)))
                            .active(interaction_style(move |style| style.bg(press_bg)))
                            .focus(interaction_style(move |style| style.bg(hover_bg))),
                    );
                    row = bind_press_adapter(
                        row,
                        PressAdapter::new(row_id.clone()).on_activate(Some(activate_handler)),
                    );
                }

                row
            })
            .collect::<Vec<_>>();

        let mut dropdown = Stack::vertical()
            .id(self.id.slot("dropdown"))
            .w(px(dropdown_width_px))
            .max_w_full()
            .p(tokens.dropdown_padding)
            .gap(tokens.dropdown_gap)
            .rounded(tokens.dropdown_radius)
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&self.theme, &tokens.dropdown_border))
            .bg(resolve_hsla(&self.theme, &tokens.dropdown_bg))
            .shadow_sm()
            .children(rows);

        if self.close_on_click_outside {
            if let Some(handler) = self.on_open_change.clone() {
                let menu_id = self.id.clone();
                dropdown = dropdown.on_mouse_down_out(move |_, window, cx| {
                    if menu_state::on_close_request(&menu_id, is_controlled) {
                        window.refresh();
                    }
                    (handler)(false, window, cx);
                });
            } else if !is_controlled {
                let menu_id = self.id.clone();
                dropdown = dropdown.on_mouse_down_out(move |_, window, _cx| {
                    if menu_state::on_close_request(&menu_id, false) {
                        window.refresh();
                    }
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
        let tokens = &self.theme.components.menu;
        let state = MenuState::resolve(MenuStateInput {
            id: &self.id,
            opened: self.opened,
            default_opened: self.default_opened,
            disabled: self.disabled,
            dropdown_width_fallback: f32::from(tokens.dropdown_width_fallback),
            dropdown_min_width: f32::from(tokens.dropdown_min_width),
        });
        let opened = state.opened;
        let is_controlled = state.controlled;
        let dropdown_width_px = state.dropdown_width_px;

        let mut trigger = div()
            .id(self.id.slot("trigger"))
            .relative()
            .cursor_pointer();
        if let Some(content) = self.trigger.take() {
            trigger = trigger.child(content());
        } else {
            trigger = trigger.child("Menu");
        }
        trigger = trigger.child({
            let id_for_width = self.id.clone();
            canvas(
                move |bounds, _, _cx| {
                    menu_state::set_dropdown_width(&id_for_width, f32::from(bounds.size.width));
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        });

        if self.disabled {
            trigger = trigger.opacity(0.55).cursor_default();
        } else {
            let activate_handler = if let Some(handler) = self.on_open_change.clone() {
                let id = self.id.clone();
                let next = !opened;
                Some(Rc::new(move |window: &mut Window, cx: &mut gpui::App| {
                    if menu_state::on_trigger_toggle(&id, is_controlled, next) {
                        window.refresh();
                    }
                    (handler)(next, window, cx);
                }) as ActivateHandler)
            } else if !is_controlled {
                let id = self.id.clone();
                let next = !opened;
                Some(Rc::new(move |window: &mut Window, _cx: &mut gpui::App| {
                    if menu_state::on_trigger_toggle(&id, false, next) {
                        window.refresh();
                    }
                }) as ActivateHandler)
            } else {
                None
            };

            if let Some(activate_handler) = activate_handler {
                trigger = trigger.cursor_pointer();
                trigger = bind_press_adapter(
                    trigger,
                    PressAdapter::new(self.id.slot("trigger")).on_activate(Some(activate_handler)),
                );
            } else {
                trigger = trigger.cursor_default();
            }
        }

        if opened {
            let dropdown = self.render_dropdown(is_controlled, dropdown_width_px, window);
            let anchor_host = anchored_host(
                &self.id,
                "anchor-host",
                PopupPlacement::Bottom,
                self.offset_px,
                self.theme.components.layout.popup_snap_margin,
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
