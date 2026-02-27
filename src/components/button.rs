use super::transition::TransitionExt;
use std::rc::Rc;

use gpui::InteractiveElement;
use gpui::{
    AnyElement, ClickEvent, ElementId, FocusHandle, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, px,
};

use crate::contracts::Disableable as _;
use crate::contracts::{MotionAware, Radiused, Sized, Varianted};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::interaction_adapter::{PressAdapter, bind_press_adapter};
use super::loader::{Loader, LoaderElement, LoaderVariant};
use super::selection_state;
use super::utils::{
    PressHandler, apply_interaction_styles, apply_radius, default_pressable_surface_styles,
    resolve_hsla, variant_text_weight,
};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type LoaderRenderer = Box<dyn FnOnce(Size, Hsla, ElementId) -> AnyElement>;

#[derive(IntoElement)]
pub struct Button {
    pub(crate) id: ComponentId,
    label: Option<SharedString>,
    variant: Variant,
    size: Size,
    radius: Radius,
    disabled: bool,
    loading: bool,
    loading_variant: LoaderVariant,
    loader: Option<LoaderRenderer>,
    left_slot: Option<SlotRenderer>,
    right_slot: Option<SlotRenderer>,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_click: Option<PressHandler>,
    focus_handle: Option<FocusHandle>,
}

impl Button {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            label: None,
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
            motion: MotionConfig::default(),
            on_click: None,
            focus_handle: None,
        }
    }

    #[track_caller]
    pub fn labeled(label: impl Into<SharedString>) -> Self {
        Self::new().label(label)
    }

    #[track_caller]
    pub fn without_label() -> Self {
        Self::new()
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn clear_label(mut self) -> Self {
        self.label = None;
        self
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
                .with_size(size)
                .color(color)
                .into_any_element()
        }));
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn focus_handle(mut self, value: FocusHandle) -> Self {
        self.focus_handle = Some(value);
        self
    }

    fn variant_tokens(&self) -> (Hsla, Hsla, Option<Hsla>) {
        let tokens = &self.theme.components.button;
        match self.variant {
            Variant::Filled => (tokens.filled_bg, tokens.filled_fg, None),
            Variant::Light => (tokens.light_bg, tokens.light_fg, None),
            Variant::Subtle => (tokens.subtle_bg, tokens.subtle_fg, None),
            Variant::Outline => (
                gpui::transparent_black(),
                tokens.outline_fg,
                Some(tokens.outline_border),
            ),
            Variant::Ghost => (gpui::transparent_black(), tokens.ghost_fg, None),
            Variant::Default => (
                self.theme.semantic.bg_surface,
                self.theme.semantic.text_primary,
                Some(self.theme.semantic.border_subtle),
            ),
        }
    }

    fn size_preset(&self) -> crate::theme::ButtonSizePreset {
        self.theme.components.button.sizes.for_size(self.size)
    }

    fn render_content(&mut self) -> AnyElement {
        let (_, fg_token, _) = self.variant_tokens();
        let fg = resolve_hsla(&self.theme, fg_token);
        let size_preset = self.size_preset();

        if self.loading {
            let loader_id = self.id.slot("loader");
            let loader = if let Some(custom_loader) = self.loader.take() {
                custom_loader(self.size, fg_token, loader_id)
            } else {
                crate::id::IdCtx::new(loader_id)
                    .root(Loader::new())
                    .variant(self.loading_variant)
                    .with_size(self.size)
                    .color(fg_token)
                    .into_any_element()
            };

            let mut placeholder = Stack::horizontal().gap(size_preset.content_gap);
            if let Some(left) = self.left_slot.take() {
                placeholder = placeholder.child(left());
            }
            if let Some(label) = self.label.clone() {
                placeholder = placeholder.child(
                    div()
                        .font_weight(variant_text_weight(self.variant))
                        .child(label),
                );
            }
            if let Some(right) = self.right_slot.take() {
                placeholder = placeholder.child(right());
            }

            return div()
                .relative()
                .child(div().text_color(fg).invisible().child(placeholder))
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

        let mut row = Stack::horizontal().gap(size_preset.content_gap);
        if let Some(left) = self.left_slot.take() {
            row = row.child(left());
        }

        if let Some(label) = self.label.clone() {
            row = row.child(
                div()
                    .font_weight(variant_text_weight(self.variant))
                    .child(label),
            );
        }

        if let Some(right) = self.right_slot.take() {
            row = row.child(right());
        }

        div().text_color(fg).child(row).into_any_element()
    }
}

impl Button {}

crate::impl_variant_size_radius_via_methods!(Button, variant, size, radius);

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
        let bg = resolve_hsla(&self.theme, bg_token);
        let fg = resolve_hsla(&self.theme, fg_token);
        let size_preset = self.size_preset();

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .text_color(fg)
            .bg(bg)
            .text_size(size_preset.font_size)
            .line_height(size_preset.line_height)
            .py(size_preset.padding_y)
            .px(size_preset.padding_x)
            .min_h(px(
                f32::from(size_preset.line_height) + f32::from(size_preset.padding_y) * 2.0
            ))
            .border(super::utils::quantized_stroke_px(window, 1.0));

        root = apply_radius(&self.theme, root, self.radius);

        if let Some(border_token) = border_token {
            root = root.border_color(resolve_hsla(&self.theme, border_token));
        } else {
            root = root.border_color(bg);
        }

        if self.disabled || self.loading {
            root = root.cursor_default().opacity(0.55);
        } else if self.on_click.is_some() {
            root = root.cursor_pointer();
            root = apply_interaction_styles(
                root,
                default_pressable_surface_styles(
                    bg,
                    resolve_hsla(&self.theme, self.theme.semantic.focus_ring),
                ),
            );
            root = bind_press_adapter(
                root,
                PressAdapter::new(self.id.clone())
                    .on_click(self.on_click.clone())
                    .focus_handle(self.focus_handle.clone()),
            );
        } else {
            root = root.cursor_default();
        }

        root.child(self.render_content())
            .with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

type GroupChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonGroupItem {
    pub value: SharedString,
    pub label: Option<SharedString>,
    pub disabled: bool,
}

impl ButtonGroupItem {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: None,
            disabled: false,
        }
    }

    pub fn labeled(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(value).label(label)
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }
}

#[derive(IntoElement)]
pub struct ButtonGroup {
    pub(crate) id: ComponentId,
    items: Vec<ButtonGroupItem>,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    orientation: GroupOrientation,
    size: Size,
    radius: Radius,
    active_variant: Variant,
    inactive_variant: Variant,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<GroupChangeHandler>,
}

impl ButtonGroup {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
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
        selection_state::resolve_optional_text(
            &self.id,
            "value",
            self.value_controlled,
            self.value.as_ref().map(|value| value.to_string()),
            self.default_value.as_ref().map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }
}

impl ButtonGroup {}

impl Varianted for ButtonGroup {
    fn with_variant(mut self, value: Variant) -> Self {
        self.active_variant = value;
        self.inactive_variant = value;
        self
    }
}

impl Sized for ButtonGroup {
    fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }
}

impl Radiused for ButtonGroup {
    fn with_radius(mut self, value: Radius) -> Self {
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
        let group_gap = self
            .theme
            .components
            .button
            .sizes
            .for_size(self.size)
            .content_gap;
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

                let mut button = self
                    .id
                    .ctx()
                    .child_index("item", index.to_string(), Button::new())
                    .with_variant(variant);
                if let Some(label) = item.label.clone() {
                    button = button.label(label);
                }
                button = Sized::with_size(button, self.size);
                button = Radiused::with_radius(button, self.radius);
                button = button.motion(self.motion);

                if item.disabled {
                    button = button.disabled(true);
                } else if let Some(handler) = self.on_change.clone() {
                    let value = item.value.clone();
                    let id = self.id.clone();
                    button = button.on_click(move |_, window, cx| {
                        if selection_state::apply_optional_text(
                            &id,
                            "value",
                            is_controlled,
                            Some(value.to_string()),
                        ) {
                            window.refresh();
                        }
                        (handler)(value.clone(), window, cx);
                    });
                } else if !is_controlled {
                    let value = item.value.clone();
                    let id = self.id.clone();
                    button = button.on_click(move |_, window, _cx| {
                        if selection_state::apply_optional_text(
                            &id,
                            "value",
                            false,
                            Some(value.to_string()),
                        ) {
                            window.refresh();
                        }
                    });
                }

                div().group(self.id.clone()).child(button)
            })
            .collect::<Vec<_>>();

        match self.orientation {
            GroupOrientation::Horizontal => Stack::horizontal()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .gap(group_gap)
                .children(children)
                .into_any_element(),
            GroupOrientation::Vertical => div()
                .id(self.id.clone())
                .group(self.id.clone())
                .tab_group()
                .flex()
                .flex_col()
                .gap(group_gap)
                .children(children)
                .into_any_element(),
        }
    }
}

crate::impl_disableable!(Button, |this, value| this.disabled = value);
crate::impl_clickable!(Button);
crate::impl_focusable!(Button);
crate::impl_disableable!(ButtonGroupItem, |this, value| this.disabled = value);
