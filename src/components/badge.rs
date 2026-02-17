use gpui::{
    AnyElement, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

#[derive(IntoElement)]
pub struct Badge {
    id: String,
    label: SharedString,
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
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("badge"),
            label: label.into(),
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

impl WithId for Badge {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for Badge {
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
        let bg = resolve_hsla(&self.theme, &bg_token);
        let fg = resolve_hsla(&self.theme, &fg_token);

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .bg(bg)
            .text_color(fg)
            .border(super::utils::quantized_stroke_px(window, 1.0));
        root = apply_radius(&self.theme, root, self.radius);

        root = match self.size {
            Size::Xs => root.text_xs().py(px(1.0)).px(px(6.0)),
            Size::Sm => root.text_xs().py(px(2.0)).px(px(8.0)),
            Size::Md => root.text_sm().py(px(3.0)).px(px(10.0)),
            Size::Lg => root.text_base().py(px(4.0)).px(px(12.0)),
            Size::Xl => root.text_lg().py(px(5.0)).px(px(14.0)),
        };

        if let Some(border_token) = border_token {
            root = root.border_color(resolve_hsla(&self.theme, &border_token));
        } else {
            root = root.border_color(bg);
        }

        if let Some(left) = self.left_slot.take() {
            root = root.child(left());
        }
        root = root.child(self.label);
        if let Some(right) = self.right_slot.take() {
            root = root.child(right());
        }

        root.with_enter_transition(format!("{}-enter", self.id), self.motion)
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
