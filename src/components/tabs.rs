use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::Stack;
use super::control;
use super::transition::TransitionExt;
use super::utils::{
    InteractionStyles, PressHandler, PressableBehavior, apply_interaction_styles, apply_radius,
    interaction_style, resolve_hsla, wire_pressable,
};

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;
type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

pub struct TabItem {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
    panel: Option<SlotRenderer>,
}

impl TabItem {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            panel: None,
        }
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn panel(mut self, content: impl IntoElement + 'static) -> Self {
        self.panel = Some(Box::new(|| content.into_any_element()));
        self
    }
}

#[derive(IntoElement)]
pub struct Tabs {
    id: ComponentId,
    items: Vec<TabItem>,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl Tabs {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            items: Vec::new(),
            value: None,
            value_controlled: false,
            default_value: None,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Md,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn item(mut self, item: TabItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabItem>) -> Self {
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
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_value(&self) -> Option<SharedString> {
        let default = self
            .default_value
            .clone()
            .or_else(|| self.items.first().map(|item| item.value.clone()));

        control::optional_text_state(
            &self.id,
            "value",
            self.value_controlled
                .then_some(self.value.as_ref().map(|value| value.to_string())),
            default.map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }

    fn apply_tab_size<T: Styled>(size: Size, node: T) -> T {
        match size {
            Size::Xs => node.text_xs().py_0p5().px_2(),
            Size::Sm => node.text_sm().py_1().px_2p5(),
            Size::Md => node.text_base().py_1p5().px_3(),
            Size::Lg => node.text_lg().py_2().px_3p5(),
            Size::Xl => node.text_xl().py_2p5().px_4(),
        }
    }

    fn active_bg(&self) -> gpui::Hsla {
        let token = &self.theme.components.tabs.tab_active_bg;
        let base = resolve_hsla(&self.theme, token);
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.8),
            Variant::Subtle => base.alpha(0.72),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.64),
        }
    }
}

impl Tabs {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Tabs {
    fn variant(mut self, value: Variant) -> Self {
        self.variant = value;
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

impl MotionAware for Tabs {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Tabs {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = self.theme.components.tabs.clone();
        let selected = self.resolved_value();
        let theme = self.theme.clone();
        let on_change = self.on_change.clone();
        let controlled = self.value_controlled;
        let control_id = self.id.clone();
        let active_bg = self.active_bg();
        let size = self.size;
        let motion = self.motion;
        let panel_fallback_fg = resolve_hsla(&self.theme, &self.theme.semantic.text_muted);
        let transparent = resolve_hsla(&theme, &gpui::transparent_black());

        let mut selected_panel: Option<AnyElement> = None;
        let mut first_panel: Option<AnyElement> = None;
        let mut triggers: Vec<AnyElement> = Vec::new();

        for (index, mut item) in self.items.into_iter().enumerate() {
            let is_active = selected
                .as_ref()
                .is_some_and(|value| value.as_ref() == item.value.as_ref());

            if let Some(panel) = item.panel.take() {
                if is_active {
                    selected_panel = Some(panel());
                } else if first_panel.is_none() {
                    first_panel = Some(panel());
                }
            }

            let mut trigger = div()
                .id(self.id.slot_index("tab", index.to_string()))
                .min_w_0()
                .cursor_pointer()
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(if is_active {
                    resolve_hsla(&theme, &tokens.list_border)
                } else {
                    transparent
                })
                .text_color(if item.disabled {
                    resolve_hsla(&theme, &tokens.tab_disabled_fg)
                } else if is_active {
                    resolve_hsla(&theme, &tokens.tab_active_fg)
                } else {
                    resolve_hsla(&theme, &tokens.tab_fg)
                })
                .bg(if is_active {
                    active_bg
                } else {
                    resolve_hsla(&theme, &gpui::transparent_black())
                })
                .child(item.label.clone());

            trigger = Self::apply_tab_size(size, trigger);
            trigger = apply_radius(&self.theme, trigger, self.radius);
            if is_active {
                trigger = trigger.shadow_sm();
            }

            if !item.disabled {
                let on_change = on_change.clone();
                let value = item.value.clone();
                let id = control_id.clone();
                let hover_bg = resolve_hsla(&theme, &tokens.tab_hover_bg);
                let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                let focus_bg = if is_active {
                    active_bg.blend(gpui::white().opacity(0.04))
                } else {
                    hover_bg
                };
                let focus_ring = resolve_hsla(&theme, &theme.semantic.focus_ring);
                let click_handler: PressHandler = Rc::new(move |_: &ClickEvent, window, cx| {
                    if !controlled {
                        control::set_optional_text_state(&id, "value", Some(value.to_string()));
                        window.refresh();
                    }
                    if let Some(handler) = on_change.as_ref() {
                        (handler)(value.clone(), window, cx);
                    }
                });

                let mut interaction_styles =
                    InteractionStyles::new().focus(interaction_style(move |style| {
                        style.bg(focus_bg).border_color(focus_ring)
                    }));
                if !is_active {
                    interaction_styles = interaction_styles
                        .hover(interaction_style(move |style| style.bg(hover_bg)))
                        .active(interaction_style(move |style| style.bg(press_bg)));
                }

                trigger = apply_interaction_styles(trigger.cursor_pointer(), interaction_styles);
                trigger = wire_pressable(
                    trigger,
                    PressableBehavior::new().on_click(Some(click_handler)),
                );
            } else {
                trigger = trigger.opacity(0.55).cursor_default();
            }

            triggers.push(trigger.into_any_element());
        }

        let panel_content = selected_panel.or(first_panel).unwrap_or_else(|| {
            div()
                .text_color(panel_fallback_fg)
                .child("No panel")
                .into_any_element()
        });

        let mut list = Stack::horizontal()
            .id(self.id.slot("list"))
            .w_full()
            .gap_0p5()
            .p_0p5()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .bg(resolve_hsla(&theme, &tokens.list_bg))
            .border_color(resolve_hsla(&theme, &tokens.list_border))
            .children(triggers);
        list = apply_radius(&self.theme, list, self.radius);

        let mut panel = div()
            .id(self.id.slot("panel"))
            .w_full()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&theme, &tokens.panel_border))
            .bg(resolve_hsla(&theme, &tokens.panel_bg))
            .text_color(resolve_hsla(&theme, &tokens.panel_fg))
            .p_4()
            .child(panel_content);
        panel = apply_radius(&self.theme, panel, self.radius);

        Stack::vertical()
            .id(self.id.clone())
            .w_full()
            .gap_2()
            .child(list)
            .child(panel)
            .with_enter_transition(self.id.slot("enter"), motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for Tabs {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(TabItem);

impl gpui::Styled for Tabs {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
