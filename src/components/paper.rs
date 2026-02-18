use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div, px,
};

use crate::id::ComponentId;
use crate::style::{Radius, Size};

use super::utils::{apply_radius, resolve_hsla};

#[derive(IntoElement)]
pub struct Paper {
    id: ComponentId,
    padding: Size,
    radius: Radius,
    bordered: bool,
    with_shadow: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
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
            style: gpui::StyleRefinement::default(),
            children: Vec::new(),
        }
    }

    pub fn padding(mut self, value: Size) -> Self {
        self.padding = value;
        self
    }

    pub fn with_radius(mut self, value: Radius) -> Self {
        self.radius = value;
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

    fn apply_padding<T: Styled>(padding: Size, node: T) -> T {
        match padding {
            Size::Xs => node.p_1(),
            Size::Sm => node.p_2(),
            Size::Md => node.p_3(),
            Size::Lg => node.p_4(),
            Size::Xl => node.p_5(),
        }
    }
}

impl Paper {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
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
        let padding = self.padding;
        let mut root = div()
            .id(root_id)
            .bg(resolve_hsla(&self.theme, &tokens.bg))
            .w_full();
        root = apply_radius(&self.theme, root, self.radius);
        root = Self::apply_padding(padding, root);

        if self.bordered {
            root = root
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(&self.theme, &tokens.border));
        }

        if self.with_shadow {
            root = root.shadow_sm();
        }

        root.children(self.children).min_h(px(1.0))
    }
}

impl crate::contracts::ComponentThemeOverridable for Paper {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_radiused_via_method!(Paper);

impl gpui::Styled for Paper {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
