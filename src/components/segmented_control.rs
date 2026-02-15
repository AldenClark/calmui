use std::rc::Rc;

use gpui::{
    ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::{MotionAware, VariantConfigurable, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::control;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

pub struct SegmentedControlItem {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl SegmentedControlItem {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

pub struct SegmentedControl {
    id: String,
    items: Vec<SegmentedControlItem>,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    full_width: bool,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl SegmentedControl {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("segmented-control"),
            items: Vec::new(),
            value: None,
            value_controlled: false,
            default_value: None,
            full_width: true,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Md,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn item(mut self, item: SegmentedControlItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SegmentedControlItem>) -> Self {
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

    pub fn full_width(mut self, value: bool) -> Self {
        self.full_width = value;
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
        control::optional_text_state(
            &self.id,
            "value",
            self.value_controlled
                .then_some(self.value.as_ref().map(|value| value.to_string())),
            self.default_value.as_ref().map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }

    fn apply_item_size<T: Styled>(size: Size, node: T) -> T {
        match size {
            Size::Xs => node.text_xs().py_0p5().px_2(),
            Size::Sm => node.text_sm().py_1().px_2p5(),
            Size::Md => node.text_base().py_1p5().px_3(),
            Size::Lg => node.text_lg().py_2().px_3p5(),
            Size::Xl => node.text_xl().py_2p5().px_4(),
        }
    }

    fn active_bg(&self) -> gpui::Hsla {
        let token = &self.theme.components.segmented_control.item_active_bg;
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

impl WithId for SegmentedControl {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for SegmentedControl {
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

impl MotionAware for SegmentedControl {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = self.theme.components.segmented_control.clone();
        let selected = self.resolved_value();
        let active_bg = self.active_bg();
        let size = self.size;
        let full_width = self.full_width;
        let theme = self.theme.clone();
        let on_change = self.on_change.clone();
        let controlled = self.value_controlled;
        let control_id = self.id.clone();
        let root_id = self.id.clone();
        let enter_id = self.id.clone();
        let motion = self.motion;

        let items = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let is_active = selected
                    .as_ref()
                    .is_some_and(|value| value.as_ref() == item.value.as_ref());

                let mut segment = div()
                    .id(format!("{}-item-{index}", self.id))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(if item.disabled {
                        resolve_hsla(&theme, &tokens.item_disabled_fg)
                    } else if is_active {
                        resolve_hsla(&theme, &tokens.item_active_fg)
                    } else {
                        resolve_hsla(&theme, &tokens.item_fg)
                    })
                    .bg(if is_active {
                        active_bg
                    } else {
                        resolve_hsla(&theme, &gpui::transparent_black())
                    })
                    .child(item.label.clone());

                segment = Self::apply_item_size(size, segment);

                if full_width {
                    segment = segment.flex_1();
                }

                if !item.disabled {
                    let on_change = on_change.clone();
                    let value = item.value.clone();
                    let id = control_id.clone();
                    let hover_bg = resolve_hsla(&theme, &tokens.item_hover_bg);
                    segment = segment
                        .cursor_pointer()
                        .hover(move |style| style.bg(hover_bg))
                        .on_click(move |_: &ClickEvent, window, cx| {
                            if !controlled {
                                control::set_optional_text_state(
                                    &id,
                                    "value",
                                    Some(value.to_string()),
                                );
                                window.refresh();
                            }
                            if let Some(handler) = on_change.as_ref() {
                                (handler)(value.clone(), window, cx);
                            }
                        });
                } else {
                    segment = segment.opacity(0.5).cursor_default();
                }

                segment.into_any_element()
            })
            .collect::<Vec<_>>();

        let mut root = div()
            .id(root_id)
            .flex()
            .items_center()
            .gap_1()
            .p_1()
            .bg(resolve_hsla(&theme, &tokens.bg))
            .border_1()
            .border_color(resolve_hsla(&theme, &tokens.border))
            .children(items);

        root = apply_radius(&self.theme, root, self.radius);

        root.with_enter_transition(format!("{}-enter", enter_id), motion)
    }
}

impl IntoElement for SegmentedControl {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for SegmentedControl {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(SegmentedControlItem);

impl gpui::Styled for SegmentedControl {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
