use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::icon::Icon;
use super::interaction_adapter::{PressAdapter, bind_press_adapter};
use super::loader::{Loader, LoaderVariant};
use super::transition::TransitionExt;
use super::utils::{
    PressHandler, apply_interaction_styles, apply_radius, default_pressable_surface_styles,
    resolve_hsla,
};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

#[derive(IntoElement)]
pub struct ActionIcon {
    id: ComponentId,
    variant: Variant,
    size: Size,
    radius: Radius,
    disabled: bool,
    loading: bool,
    loading_variant: LoaderVariant,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    content: Option<SlotRenderer>,
    on_click: Option<PressHandler>,
    focus_handle: Option<FocusHandle>,
}

impl ActionIcon {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Sm,
            disabled: false,
            loading: false,
            loading_variant: LoaderVariant::Dots,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            content: None,
            on_click: None,
            focus_handle: None,
        }
    }

    pub fn child(mut self, value: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| value.into_any_element()));
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn loading(mut self, value: bool) -> Self {
        self.loading = value;
        self
    }

    pub fn loading_variant(mut self, value: LoaderVariant) -> Self {
        self.loading_variant = value;
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
        let tokens = &self.theme.components.action_icon;
        if self.disabled {
            return (
                tokens.disabled_bg.clone(),
                tokens.disabled_fg.clone(),
                Some(tokens.disabled_border.clone()),
            );
        }

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
                tokens.default_bg.clone(),
                tokens.default_fg.clone(),
                Some(tokens.default_border.clone()),
            ),
        }
    }

    fn size_preset(&self) -> crate::theme::ActionIconSizePreset {
        self.theme.components.action_icon.sizes.for_size(self.size)
    }
}

impl ActionIcon {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for ActionIcon {
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

impl MotionAware for ActionIcon {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for ActionIcon {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let (bg_token, fg_token, border_token) = self.variant_tokens();
        let bg = resolve_hsla(&self.theme, &bg_token);
        let fg = resolve_hsla(&self.theme, &fg_token);
        let size_preset = self.size_preset();
        let size_px = f32::from(size_preset.box_size);

        let fallback = Icon::named("dots")
            .with_id(self.id.slot("fallback"))
            .size(f32::from(size_preset.icon_size))
            .color(fg)
            .into_any_element();

        let content = if self.loading {
            Loader::new()
                .with_id(self.id.slot("loader"))
                .variant(self.loading_variant)
                .with_size(self.size)
                .color(fg_token)
                .into_any_element()
        } else {
            self.content.take().map(|value| value()).unwrap_or(fallback)
        };

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .items_center()
            .justify_center()
            .w(px(size_px))
            .h(px(size_px))
            .bg(bg)
            .text_color(fg)
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .child(content);

        root = apply_radius(&self.theme, root, self.radius);

        if let Some(border) = border_token {
            root = root.border_color(resolve_hsla(&self.theme, &border));
        } else {
            root = root.border_color(bg);
        }

        if self.disabled || self.loading {
            root = root.opacity(0.55).cursor_default();
        } else if self.on_click.is_some() {
            root = root.cursor_pointer();
            root = apply_interaction_styles(
                root,
                default_pressable_surface_styles(
                    bg,
                    resolve_hsla(&self.theme, &self.theme.semantic.focus_ring),
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

        root.with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for ActionIcon {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(ActionIcon);
crate::impl_clickable!(ActionIcon);
crate::impl_focusable!(ActionIcon);

impl gpui::Styled for ActionIcon {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
