use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::id::ComponentId;
use crate::style::Size;

use super::utils::resolve_hsla;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollDirection {
    Vertical,
    Horizontal,
    Both,
}

#[derive(IntoElement)]
pub struct ScrollArea {
    id: ComponentId,
    viewport_height_px: Option<f32>,
    viewport_width_px: Option<f32>,
    padding: Size,
    direction: ScrollDirection,
    show_scrollbars: bool,
    bordered: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    children: Vec<AnyElement>,
}

impl ScrollArea {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            viewport_height_px: None,
            viewport_width_px: None,
            padding: Size::Md,
            direction: ScrollDirection::Vertical,
            show_scrollbars: true,
            bordered: true,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            children: Vec::new(),
        }
    }

    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height_px = Some(value.max(1.0));
        self
    }

    pub fn viewport_width(mut self, value: f32) -> Self {
        self.viewport_width_px = Some(value.max(1.0));
        self
    }

    pub fn padding(mut self, value: Size) -> Self {
        self.padding = value;
        self
    }

    pub fn direction(mut self, value: ScrollDirection) -> Self {
        self.direction = value;
        self
    }

    pub fn show_scrollbars(mut self, value: bool) -> Self {
        self.show_scrollbars = value;
        self
    }

    pub fn bordered(mut self, value: bool) -> Self {
        self.bordered = value;
        self
    }

    pub fn child(mut self, content: impl IntoElement + 'static) -> Self {
        self.children.push(content.into_any_element());
        self
    }

    pub fn children<I, E>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: IntoElement + 'static,
    {
        self.children
            .extend(children.into_iter().map(IntoElement::into_any_element));
        self
    }
}

impl ParentElement for ScrollArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl ScrollArea {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for ScrollArea {
    fn render(mut self, window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.scroll_area;
        let content_padding = tokens.padding.for_size(self.padding);
        let mut viewport = div().id(self.id.slot("viewport")).w_full().min_h_0();

        viewport = match self.direction {
            ScrollDirection::Vertical => viewport.overflow_y_scroll(),
            ScrollDirection::Horizontal => viewport.overflow_x_scroll(),
            ScrollDirection::Both => viewport.overflow_scroll(),
        };

        if let Some(height) = self.viewport_height_px {
            viewport = viewport.h(px(height));
        } else {
            viewport = viewport.h_full();
        }

        if let Some(width) = self.viewport_width_px {
            viewport = viewport.w(px(width));
        }

        if !self.show_scrollbars {
            viewport = viewport.scrollbar_width(px(0.0));
        }

        viewport = viewport.p(content_padding).children(self.children);

        let mut root = div()
            .id(self.id)
            .w_full()
            .min_h_0()
            .h_full()
            .bg(resolve_hsla(&self.theme, &tokens.bg));

        if self.bordered {
            root = root
                .border(super::utils::quantized_stroke_px(window, 1.0))
                .border_color(resolve_hsla(&self.theme, &tokens.border));
        }

        root.child(viewport)
    }
}

impl crate::contracts::ComponentThemeOverridable for ScrollArea {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for ScrollArea {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
