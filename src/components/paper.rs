use gpui::InteractiveElement;
use gpui::{AnyElement, IntoElement, ParentElement, RenderOnce, Styled, div, px};

use crate::id::ComponentId;
use crate::style::{Radius, Size};

use super::utils::{apply_radius, resolve_hsla};

#[derive(IntoElement)]
pub struct Paper {
    pub(crate) id: ComponentId,
    padding: Size,
    radius: Radius,
    bordered: bool,
    with_shadow: bool,
    pub(crate) theme: crate::theme::LocalTheme,
    children: Vec<AnyElement>,
}

impl Paper {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            padding: Size::Md,
            radius: Radius::Md,
            bordered: true,
            with_shadow: false,
            theme: crate::theme::LocalTheme::default(),
            children: Vec::new(),
        }
    }

    pub fn padding(mut self, value: Size) -> Self {
        self.padding = value;
        self
    }
    pub fn bordered(mut self, value: bool) -> Self {
        self.bordered = value;
        self
    }

    pub fn with_shadow(mut self, value: bool) -> Self {
        self.with_shadow = value;
        self
    }

    pub fn child(mut self, content: impl IntoElement + 'static) -> Self {
        self.children.push(content.into_any_element());
        self
    }

    pub fn children<I, E>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: IntoElement + 'static,
    {
        self.children
            .extend(values.into_iter().map(IntoElement::into_any_element));
        self
    }
}

impl ParentElement for Paper {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Paper {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.paper;
        let root_id = self.id.clone();
        let padding = tokens.padding.for_size(self.padding);
        let mut root = div()
            .id(root_id)
            .bg(resolve_hsla(&self.theme, tokens.bg))
            .w_full();
        root = apply_radius(&self.theme, root, self.radius);
        root = root.p(padding);

        if self.bordered {
            root = root
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(&self.theme, tokens.border));
        }

        if self.with_shadow {
            root = root.shadow_sm();
        }

        root.children(self.children).min_h(px(1.0))
    }
}

crate::impl_radiused_via_method!(Paper, radius);
