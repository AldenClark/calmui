use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::Stack;
use super::control;
use super::icon::Icon;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type ChangeHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccordionItemMeta {
    pub value: SharedString,
    pub label: SharedString,
    pub description: Option<SharedString>,
    pub disabled: bool,
}

pub struct AccordionItem {
    meta: AccordionItemMeta,
    body: Option<SharedString>,
    content: Option<SlotRenderer>,
}

impl AccordionItem {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            meta: AccordionItemMeta {
                value: value.into(),
                label: label.into(),
                description: None,
                disabled: false,
            },
            body: None,
            content: None,
        }
    }

    pub fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.meta.description = Some(value.into());
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.meta.disabled = value;
        self
    }

    pub fn body(mut self, value: impl Into<SharedString>) -> Self {
        self.body = Some(value.into());
        self
    }

    pub fn content(mut self, value: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| value.into_any_element()));
        self
    }
}

#[derive(IntoElement)]
pub struct Accordion {
    id: ComponentId,
    items: Vec<AccordionItem>,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl Accordion {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            items: Vec::new(),
            value: None,
            value_controlled: false,
            default_value: None,
            size: Size::Md,
            radius: Radius::Md,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn item(mut self, item: AccordionItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self.value_controlled = true;
        self
    }

    pub fn clear_value(mut self) -> Self {
        self.value = None;
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(Option<SharedString>, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_value(&self) -> Option<SharedString> {
        control::optional_text_state(
            &self.id,
            "value",
            self.value_controlled
                .then_some(self.value.as_ref().map(|value| value.to_string())),
            self.default_value.as_ref().map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }

    pub fn id(&self) -> &ElementId {
        self.id.id()
    }

    pub fn set_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Accordion {
    fn variant(self, _value: Variant) -> Self {
        self
    }

    fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn radius(mut self, value: Radius) -> Self {
        self.radius = value;
        self
    }
}

impl MotionAware for Accordion {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Accordion {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.accordion;
        let active_value = self.resolved_value();
        let is_controlled = self.value_controlled;

        let item_views = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, mut item)| {
                let item_root_id = self.id.slot_index("item", index.to_string());
                let header_id = self.id.slot_index("header", index.to_string());
                let chevron_id = self.id.slot_index("chevron", index.to_string());
                let panel_id = self.id.slot_index("panel", index.to_string());

                let is_open = active_value
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == item.meta.value.as_ref());

                let mut root = Stack::vertical()
                    .id(item_root_id)
                    .w_full()
                    .bg(resolve_hsla(&self.theme, &tokens.item_bg))
                    .border(super::utils::quantized_stroke_px(window, 1.0))
                    .border_color(resolve_hsla(&self.theme, &tokens.item_border));
                root = apply_radius(&self.theme, root, self.radius);

                let size_text = match self.size {
                    Size::Xs | Size::Sm => div().text_sm(),
                    Size::Md => div().text_base(),
                    Size::Lg => div().text_lg(),
                    Size::Xl => div().text_xl(),
                };

                let mut header = div()
                    .id(header_id)
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .cursor_pointer()
                    .px(gpui::px(12.0))
                    .py(gpui::px(10.0))
                    .child(
                        Stack::vertical()
                            .gap_0p5()
                            .child(
                                size_text
                                    .text_color(resolve_hsla(&self.theme, &tokens.label))
                                    .child(item.meta.label),
                            )
                            .children(item.meta.description.clone().map(|description| {
                                div()
                                    .text_sm()
                                    .text_color(resolve_hsla(&self.theme, &tokens.description))
                                    .child(description)
                            })),
                    )
                    .child(
                        Icon::named(if is_open {
                            "chevron-up"
                        } else {
                            "chevron-down"
                        })
                        .with_id(chevron_id)
                        .size(14.0)
                        .color(resolve_hsla(&self.theme, &tokens.chevron)),
                    );

                if item.meta.disabled {
                    header = header.cursor_default().opacity(0.55);
                } else if let Some(handler) = self.on_change.clone() {
                    let accordion_id = self.id.to_string();
                    let value = item.meta.value.clone();
                    header = header.on_click(move |_: &ClickEvent, window, cx| {
                        let current = control::optional_text_state(
                            &accordion_id,
                            "value",
                            None,
                            None::<String>,
                        );
                        let next = if current.as_deref() == Some(value.as_ref()) {
                            None
                        } else {
                            Some(value.to_string())
                        };

                        if !is_controlled {
                            control::set_optional_text_state(&accordion_id, "value", next.clone());
                            window.refresh();
                        }
                        (handler)(next.map(SharedString::from), window, cx);
                    });
                } else if !is_controlled {
                    let accordion_id = self.id.to_string();
                    let value = item.meta.value.clone();
                    header = header.on_click(move |_: &ClickEvent, window, _cx| {
                        let current = control::optional_text_state(
                            &accordion_id,
                            "value",
                            None,
                            None::<String>,
                        );
                        let next = if current.as_deref() == Some(value.as_ref()) {
                            None
                        } else {
                            Some(value.to_string())
                        };
                        control::set_optional_text_state(&accordion_id, "value", next);
                        window.refresh();
                    });
                }

                root = root.child(header);

                if is_open {
                    let mut body = Stack::vertical()
                        .id(panel_id.clone())
                        .gap_1()
                        .px(gpui::px(12.0))
                        .pb(gpui::px(10.0))
                        .pt(gpui::px(2.0))
                        .text_color(resolve_hsla(&self.theme, &tokens.content));

                    if let Some(text) = item.body.take() {
                        body = body.child(div().text_sm().child(text));
                    }
                    if let Some(content) = item.content.take() {
                        body = body.child(content());
                    }
                    root = root.child(body.with_enter_transition((panel_id, "enter"), self.motion));
                }

                root
            })
            .collect::<Vec<_>>();

        Stack::vertical()
            .id(self.id.clone())
            .gap_2()
            .w_full()
            .children(item_views)
            .with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for Accordion {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(AccordionItem);

impl gpui::Styled for Accordion {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
