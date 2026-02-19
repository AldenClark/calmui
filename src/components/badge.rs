use gpui::{
    AnyElement, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, div,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

#[derive(IntoElement)]
pub struct Badge {
    id: ComponentId,
    label: Option<SharedString>,
    variant: Variant,
    size: Size,
    radius: Radius,
    left_slot: Option<SlotRenderer>,
    right_slot: Option<SlotRenderer>,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
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
            style: gpui::StyleRefinement::default(),
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
            Variant::Filled => (tokens.filled_bg.clone(), tokens.filled_fg.clone(), None),
            Variant::Light => (tokens.light_bg.clone(), tokens.light_fg.clone(), None),
            Variant::Subtle => (tokens.subtle_bg.clone(), tokens.subtle_fg.clone(), None),
            Variant::Outline => (
                gpui::transparent_black(),
                tokens.outline_fg.clone(),
                Some(tokens.outline_border.clone()),
            ),
            Variant::Ghost => (gpui::transparent_black(), tokens.outline_fg.clone(), None),
            Variant::Default => (
                tokens.default_bg.clone(),
                tokens.default_fg.clone(),
                Some(tokens.default_border.clone()),
            ),
        }
    }
}

impl Badge {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Badge {
    fn with_variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn with_radius(mut self, value: Radius) -> Self {
        self.radius = value;
        self
    }
}

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
        let bg = resolve_hsla(&self.theme, &bg_token);
        let fg = resolve_hsla(&self.theme, &fg_token);

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
            root = root.border_color(resolve_hsla(&self.theme, &border_token));
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

impl crate::contracts::ComponentThemeOverridable for Badge {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Badge {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
