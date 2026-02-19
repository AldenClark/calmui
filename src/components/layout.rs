use gpui::{
    AnyElement, Div, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, px,
};

use crate::id::ComponentId;
use crate::style::Size;

pub struct Stack;

impl Stack {
    #[track_caller]
    pub fn vertical() -> Div {
        div().flex().flex_col()
    }

    #[track_caller]
    pub fn horizontal() -> Div {
        div().flex().flex_row().items_center()
    }
}

#[derive(IntoElement)]
pub struct Grid {
    id: ComponentId,
    columns: usize,
    gap_x: Size,
    gap_y: Size,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    children: Vec<AnyElement>,
}

impl Grid {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            columns: 2,
            gap_x: Size::Md,
            gap_y: Size::Md,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            children: Vec::new(),
        }
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    pub fn gap(mut self, gap: Size) -> Self {
        self.gap_x = gap;
        self.gap_y = gap;
        self
    }

    pub fn gap_x(mut self, gap: Size) -> Self {
        self.gap_x = gap;
        self
    }

    pub fn gap_y(mut self, gap: Size) -> Self {
        self.gap_y = gap;
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

impl ParentElement for Grid {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Grid {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Grid {
    fn render(mut self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let gap_x = self.gap_x;
        let gap_y = self.gap_y;
        let gap_scale = self.theme.components.layout.gap;
        let columns = self.columns.max(1);
        let mut rows = Vec::new();

        let mut current_row = Vec::new();
        for child in self.children {
            current_row.push(child);
            if current_row.len() == columns {
                let mut row = div().flex().flex_row().w_full();
                row = row.gap(gap_scale.for_size(gap_x));
                let items = current_row
                    .drain(..)
                    .map(|item| div().flex_1().min_w_0().child(item))
                    .collect::<Vec<_>>();
                rows.push(row.children(items));
            }
        }

        if !current_row.is_empty() {
            let mut row = div().flex().flex_row().w_full();
            row = row.gap(gap_scale.for_size(gap_x));
            let filled_count = current_row.len();
            let mut items = current_row
                .drain(..)
                .map(|item| div().flex_1().min_w_0().child(item))
                .collect::<Vec<_>>();
            for _ in filled_count..columns {
                items.push(div().flex_1().min_w_0());
            }
            rows.push(row.children(items));
        }

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .gap(gap_scale.for_size(gap_y))
            .text_color(self.theme.resolve_hsla(&self.theme.semantic.text_primary))
            .children(rows)
    }
}

#[derive(IntoElement)]
pub struct SimpleGrid {
    id: ComponentId,
    inner: Grid,
    style: gpui::StyleRefinement,
}

impl SimpleGrid {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            inner: Grid::new(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn cols(mut self, value: usize) -> Self {
        self.inner = self.inner.columns(value);
        self
    }

    pub fn spacing(mut self, value: Size) -> Self {
        self.inner = self.inner.gap(value);
        self
    }

    pub fn spacing_x(mut self, value: Size) -> Self {
        self.inner = self.inner.gap_x(value);
        self
    }

    pub fn spacing_y(mut self, value: Size) -> Self {
        self.inner = self.inner.gap_y(value);
        self
    }

    pub fn child(mut self, content: impl IntoElement + 'static) -> Self {
        self.inner = self.inner.child(content);
        self
    }

    pub fn children<I, E>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: IntoElement + 'static,
    {
        self.inner = self.inner.children(children);
        self
    }
}

impl SimpleGrid {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for SimpleGrid {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let mut inner = self.inner.with_id(self.id);
        gpui::Refineable::refine(gpui::Styled::style(&mut inner), &self.style);
        inner.render(window, cx)
    }
}

impl ParentElement for SimpleGrid {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

#[derive(IntoElement)]
pub struct Space {
    id: ComponentId,
    width_px: Option<f32>,
    height_px: Option<f32>,
    width_size: Option<Size>,
    height_size: Option<Size>,
    theme: crate::theme::LocalTheme,
}

impl Space {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            width_px: None,
            height_px: None,
            width_size: None,
            height_size: None,
            theme: crate::theme::LocalTheme::default(),
        }
    }

    pub fn w(mut self, value: Size) -> Self {
        self.width_size = Some(value);
        self.width_px = None;
        self
    }

    pub fn h(mut self, value: Size) -> Self {
        self.height_size = Some(value);
        self.height_px = None;
        self
    }

    pub fn with_size(mut self, value: Size) -> Self {
        self.width_size = Some(value);
        self.height_size = Some(value);
        self.width_px = None;
        self.height_px = None;
        self
    }

    pub fn w_px(mut self, value: f32) -> Self {
        self.width_px = Some(value.max(0.0));
        self.width_size = None;
        self
    }

    pub fn h_px(mut self, value: f32) -> Self {
        self.height_px = Some(value.max(0.0));
        self.height_size = None;
        self
    }

    pub fn size_px(mut self, value: f32) -> Self {
        let size_px = value.max(0.0);
        self.width_px = Some(size_px);
        self.height_px = Some(size_px);
        self.width_size = None;
        self.height_size = None;
        self
    }
}

impl Space {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Space {
    fn render(mut self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let scale = self.theme.components.layout.space;
        let mut node = div().id(self.id);
        let width_px = self
            .width_px
            .or_else(|| self.width_size.map(|size| f32::from(scale.for_size(size))));
        let height_px = self
            .height_px
            .or_else(|| self.height_size.map(|size| f32::from(scale.for_size(size))));
        if let Some(width_px) = width_px {
            node = node.w(px(width_px));
        }
        if let Some(height_px) = height_px {
            node = node.h(px(height_px));
        }
        node
    }
}

crate::impl_sized_via_method!(Space);

impl crate::contracts::ComponentThemeOverridable for Grid {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemeOverridable for SimpleGrid {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        self.inner.local_theme_mut()
    }
}

impl crate::contracts::ComponentThemeOverridable for Space {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Grid {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for SimpleGrid {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
