use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::icon::Icon;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>;
type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

pub struct ActionIcon {
    id: String,
    variant: Variant,
    size: Size,
    radius: Radius,
    disabled: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    content: Option<SlotRenderer>,
    on_click: Option<ClickHandler>,
}

impl ActionIcon {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("action-icon"),
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Sm,
            disabled: false,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            content: None,
            on_click: None,
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

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
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

    fn box_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 22.0,
            Size::Sm => 26.0,
            Size::Md => 30.0,
            Size::Lg => 36.0,
            Size::Xl => 42.0,
        }
    }

    fn icon_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 12.0,
            Size::Sm => 14.0,
            Size::Md => 16.0,
            Size::Lg => 18.0,
            Size::Xl => 20.0,
        }
    }
}

impl WithId for ActionIcon {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for ActionIcon {
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
        let size_px = self.box_size_px();

        let fallback = Icon::named_outline("dots")
            .with_id(format!("{}-fallback", self.id))
            .size(self.icon_size_px())
            .color(fg)
            .into_any_element();

        let content = self.content.take().map(|value| value()).unwrap_or(fallback);

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

        if self.disabled {
            root = root.opacity(0.55).cursor_default();
        } else {
            root = root.cursor_pointer();
            if let Some(handler) = self.on_click.take() {
                root = root.on_click(move |event, window, cx| {
                    (handler)(event, window, cx);
                });
            }
        }

        root.with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for ActionIcon {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for ActionIcon {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(ActionIcon);

impl gpui::Styled for ActionIcon {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
