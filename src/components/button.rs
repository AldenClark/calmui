use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::WithId;
use crate::contracts::{MotionAware, Radiusable, Sizeable, VariantConfigurable};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::loader::{Loader, LoaderElement, LoaderVariant};
use super::transition::TransitionExt;
use super::utils::{apply_button_size, apply_radius, resolve_hsla, variant_text_weight};

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>;
type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type LoaderRenderer = Box<dyn FnOnce(Size, Hsla, String) -> AnyElement>;

pub struct Button {
    id: String,
    label: SharedString,
    variant: Variant,
    size: Size,
    radius: Radius,
    disabled: bool,
    loading: bool,
    loading_variant: LoaderVariant,
    loader: Option<LoaderRenderer>,
    left_slot: Option<SlotRenderer>,
    right_slot: Option<SlotRenderer>,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_click: Option<ClickHandler>,
}

impl Button {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("button"),
            label: label.into(),
            variant: Variant::Filled,
            size: Size::Md,
            radius: Radius::Sm,
            disabled: false,
            loading: false,
            loading_variant: LoaderVariant::Dots,
            loader: None,
            left_slot: None,
            right_slot: None,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_click: None,
        }
    }

    pub fn left_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.left_slot = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn right_slot(mut self, content: impl IntoElement + 'static) -> Self {
        self.right_slot = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn loading_variant(mut self, variant: LoaderVariant) -> Self {
        self.loading_variant = variant;
        self
    }

    pub fn loader<L>(mut self, loader: L) -> Self
    where
        L: LoaderElement,
    {
        self.loader = Some(Box::new(move |size, color, id| {
            loader
                .with_id(id)
                .size(size)
                .color(color)
                .into_any_element()
        }));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    fn variant_tokens(&self) -> (Hsla, Hsla, Option<Hsla>) {
        let tokens = &self.theme.components.button;
        match self.variant {
            Variant::Filled => (tokens.filled_bg.clone(), tokens.filled_fg.clone(), None),
            Variant::Light => (tokens.light_bg.clone(), tokens.light_fg.clone(), None),
            Variant::Subtle => (tokens.subtle_bg.clone(), tokens.subtle_fg.clone(), None),
            Variant::Outline => (
                gpui::transparent_black(),
                tokens.outline_fg.clone(),
                Some(tokens.outline_border.clone()),
            ),
            Variant::Ghost => (gpui::transparent_black(), tokens.ghost_fg.clone(), None),
            Variant::Default => (
                self.theme.semantic.bg_surface.clone(),
                self.theme.semantic.text_primary.clone(),
                Some(self.theme.semantic.border_subtle.clone()),
            ),
        }
    }

    fn render_content(&mut self) -> AnyElement {
        let (bg_token, fg_token, _border_token) = self.variant_tokens();
        let fg = resolve_hsla(&self.theme, &fg_token);

        if self.loading {
            let loader_id = format!("{}-loader", self.id);
            let loader = if let Some(custom_loader) = self.loader.take() {
                custom_loader(self.size, fg_token.clone(), loader_id)
            } else {
                Loader::new()
                    .with_id(loader_id)
                    .variant(self.loading_variant)
                    .size(self.size)
                    .color(fg_token)
                    .into_any_element()
            };

            let mut placeholder = Stack::horizontal().gap_2();
            if let Some(left) = self.left_slot.take() {
                placeholder = placeholder.child(div().text_color(fg).child(left()));
            }
            placeholder = placeholder.child(
                div()
                    .font_weight(variant_text_weight(self.variant))
                    .text_color(fg)
                    .child(self.label.clone()),
            );
            if let Some(right) = self.right_slot.take() {
                placeholder = placeholder.child(div().text_color(fg).child(right()));
            }

            return div()
                .relative()
                .child(placeholder.invisible())
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(loader),
                )
                .into_any_element();
        }

        let mut row = Stack::horizontal().gap_2();
        if let Some(left) = self.left_slot.take() {
            row = row.child(div().text_color(fg).child(left()));
        }

        row = row.child(
            div()
                .font_weight(variant_text_weight(self.variant))
                .text_color(fg)
                .child(self.label.clone()),
        );

        if let Some(right) = self.right_slot.take() {
            row = row.child(div().text_color(fg).child(right()));
        }

        let _ = bg_token;
        row.into_any_element()
    }
}

impl WithId for Button {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for Button {
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

impl MotionAware for Button {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Button {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let (bg_token, fg_token, border_token) = self.variant_tokens();
        let bg = resolve_hsla(&self.theme, &bg_token);
        let fg = resolve_hsla(&self.theme, &fg_token);

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .text_color(fg)
            .bg(bg)
            .border(super::utils::quantized_stroke_px(window, 1.0));

        root = apply_button_size(root, self.size);
        root = apply_radius(&self.theme, root, self.radius);

        if let Some(border_token) = border_token {
            root = root.border_color(resolve_hsla(&self.theme, &border_token));
        } else {
            root = root.border_color(bg);
        }

        if self.disabled {
            root = root.cursor_default().opacity(0.55);
        }

        if !self.disabled && !self.loading {
            if let Some(handler) = self.on_click.clone() {
                root = root.on_click(move |event, window, cx| {
                    (handler)(event, window, cx);
                });
            }
        }

        root.child(self.render_content())
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Button {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

type GroupChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonGroupItem {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl ButtonGroupItem {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct ButtonGroup {
    id: String,
    items: Vec<ButtonGroupItem>,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    orientation: GroupOrientation,
    size: Size,
    radius: Radius,
    active_variant: Variant,
    inactive_variant: Variant,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<GroupChangeHandler>,
}

impl ButtonGroup {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("button-group"),
            items: Vec::new(),
            value: None,
            value_controlled: false,
            default_value: None,
            orientation: GroupOrientation::Horizontal,
            size: Size::Md,
            radius: Radius::Sm,
            active_variant: Variant::Filled,
            inactive_variant: Variant::Light,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn item(mut self, item: ButtonGroupItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ButtonGroupItem>) -> Self {
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

    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn active_variant(mut self, variant: Variant) -> Self {
        self.active_variant = variant;
        self
    }

    pub fn inactive_variant(mut self, variant: Variant) -> Self {
        self.inactive_variant = variant;
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
}

impl WithId for ButtonGroup {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for ButtonGroup {
    fn variant(mut self, value: Variant) -> Self {
        self.active_variant = value;
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

impl MotionAware for ButtonGroup {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for ButtonGroup {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let selected_value = self.resolved_value();
        let is_controlled = self.value_controlled;
        let children = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let selected = selected_value
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == item.value.as_ref());
                let variant = if selected {
                    self.active_variant
                } else {
                    self.inactive_variant
                };

                let mut button = Button::new(item.label.clone())
                    .with_id(format!("{}-item-{index}", self.id))
                    .variant(variant);
                button = Sizeable::size(button, self.size);
                button = Radiusable::radius(button, self.radius);
                button = button.motion(self.motion);

                if item.disabled {
                    button = button.disabled(true);
                } else if let Some(handler) = self.on_change.clone() {
                    let value = item.value.clone();
                    let id = self.id.clone();
                    button = button.on_click(move |_, window, cx| {
                        if !is_controlled {
                            control::set_optional_text_state(&id, "value", Some(value.to_string()));
                            window.refresh();
                        }
                        (handler)(value.clone(), window, cx);
                    });
                } else if !is_controlled {
                    let value = item.value.clone();
                    let id = self.id.clone();
                    button = button.on_click(move |_, window, _cx| {
                        control::set_optional_text_state(&id, "value", Some(value.to_string()));
                        window.refresh();
                    });
                }

                div()
                    .group(self.id.clone())
                    .child(button)
                    .into_any_element()
            })
            .collect::<Vec<_>>();

        let root = match self.orientation {
            GroupOrientation::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap_1()
                .children(children)
                .into_any_element(),
            GroupOrientation::Vertical => div()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .flex()
                .flex_col()
                .gap_1()
                .children(children)
                .into_any_element(),
        };
        root
    }
}

impl IntoElement for ButtonGroup {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Button {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for ButtonGroup {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Button);
crate::impl_disableable!(ButtonGroupItem);

impl gpui::Styled for Button {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for ButtonGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
