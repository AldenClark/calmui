use super::transition::TransitionExt;
use gpui::InteractiveElement;
use gpui::{AnyElement, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, div};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::utils::{apply_radius, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

#[derive(IntoElement)]
pub struct Badge {
    pub(crate) id: ComponentId,
    label: Option<SharedString>,
    variant: Variant,
    size: Size,
    radius: Radius,
    left_slot: Option<SlotRenderer>,
    right_slot: Option<SlotRenderer>,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
}

impl Badge {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            label: None,
            variant: Variant::Filled,
            size: Size::Sm,
            radius: Radius::Pill,
            left_slot: None,
            right_slot: None,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
        }
    }

    #[track_caller]
    pub fn labeled(label: impl Into<SharedString>) -> Self {
        Self::new().label(label)
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
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

    fn variant_tokens(&self) -> (Hsla, Hsla, Option<Hsla>) {
        let tokens = &self.theme.components.badge;
        match self.variant {
            Variant::Filled => (tokens.filled_bg, tokens.filled_fg, None),
            Variant::Light => (tokens.light_bg, tokens.light_fg, None),
            Variant::Subtle => (tokens.subtle_bg, tokens.subtle_fg, None),
            Variant::Outline => (
                gpui::transparent_black(),
                tokens.outline_fg,
                Some(tokens.outline_border),
            ),
            Variant::Ghost => (gpui::transparent_black(), tokens.outline_fg, None),
            Variant::Default => (
                tokens.default_bg,
                tokens.default_fg,
                Some(tokens.default_border),
            ),
        }
    }
}

impl Badge {}

crate::impl_variant_size_radius_via_methods!(Badge, variant, size, radius);

impl MotionAware for Badge {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Badge {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let (bg_token, fg_token, border_token) = self.variant_tokens();
        let size_preset = self.theme.components.badge.sizes.for_size(self.size);
        let bg = resolve_hsla(&self.theme, bg_token);
        let fg = resolve_hsla(&self.theme, fg_token);

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(size_preset.gap)
            .bg(bg)
            .text_color(fg)
            .text_size(size_preset.font_size)
            .py(size_preset.padding_y)
            .px(size_preset.padding_x)
            .border(super::utils::quantized_stroke_px(window, 1.0));
        root = apply_radius(&self.theme, root, self.radius);

        if let Some(border_token) = border_token {
            root = root.border_color(resolve_hsla(&self.theme, border_token));
        } else {
            root = root.border_color(bg);
        }

        if let Some(left) = self.left_slot.take() {
            root = root.child(left());
        }
        if let Some(label) = self.label {
            root = root.child(label);
        }
        if let Some(right) = self.right_slot.take() {
            root = root.child(right());
        }

        root.with_enter_transition(self.id.slot("enter"), self.motion)
    }
}
