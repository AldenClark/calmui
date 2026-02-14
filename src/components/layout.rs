use gpui::{
    AnyElement, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, px,
};

use crate::contracts::WithId;
use crate::id::stable_auto_id;
use crate::style::Size;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlexJustify {
    Start,
    Center,
    Between,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlexAlign {
    Start,
    Center,
}

pub struct Flex {
    id: String,
    direction: FlexDirection,
    justify: FlexJustify,
    align: FlexAlign,
    wrap: bool,
    gap: Size,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    children: Vec<AnyElement>,
}

impl Flex {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("flex"),
            direction: FlexDirection::Row,
            justify: FlexJustify::Start,
            align: FlexAlign::Start,
            wrap: false,
            gap: Size::Md,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            children: Vec::new(),
        }
    }

    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn justify(mut self, justify: FlexJustify) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: FlexAlign) -> Self {
        self.align = align;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn gap(mut self, gap: Size) -> Self {
        self.gap = gap;
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

    fn apply_gap<T: Styled>(node: T, gap: Size) -> T {
        match gap {
            Size::Xs => node.gap_1(),
            Size::Sm => node.gap_1p5(),
            Size::Md => node.gap_2(),
            Size::Lg => node.gap_3(),
            Size::Xl => node.gap_4(),
        }
    }
}

impl ParentElement for Flex {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl WithId for Flex {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Flex {
    fn render(mut self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let mut root = div().id(self.id.clone()).flex();

        root = match self.direction {
            FlexDirection::Row => root.flex_row(),
            FlexDirection::Column => root.flex_col(),
        };

        if self.wrap {
            root = root.flex_wrap();
        }

        root = Self::apply_gap(root, self.gap);

        root = match self.justify {
            FlexJustify::Start => root,
            FlexJustify::Center => root.justify_center(),
            FlexJustify::Between => root.justify_between(),
        };

        root = match self.align {
            FlexAlign::Start => root,
            FlexAlign::Center => root.items_center(),
        };

        root.text_color(self.theme.resolve_hsla(&self.theme.semantic.text_primary))
            .children(self.children)
    }
}

impl IntoElement for Flex {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StackDirection {
    Vertical,
    Horizontal,
}

pub struct Stack {
    inner: Flex,
    style: gpui::StyleRefinement,
}

impl Stack {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            inner: Flex::new().direction(FlexDirection::Column),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn direction(mut self, direction: StackDirection) -> Self {
        self.inner = self.inner.direction(match direction {
            StackDirection::Vertical => FlexDirection::Column,
            StackDirection::Horizontal => FlexDirection::Row,
        });
        self
    }

    pub fn justify(mut self, justify: FlexJustify) -> Self {
        self.inner = self.inner.justify(justify);
        self
    }

    pub fn align(mut self, align: FlexAlign) -> Self {
        self.inner = self.inner.align(align);
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.inner = self.inner.wrap(wrap);
        self
    }

    pub fn gap(mut self, gap: Size) -> Self {
        self.inner = self.inner.gap(gap);
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

impl WithId for Stack {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn id_mut(&mut self) -> &mut String {
        self.inner.id_mut()
    }
}

impl RenderOnce for Stack {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let mut inner = self.inner;
        gpui::Refineable::refine(gpui::Styled::style(&mut inner), &self.style);
        inner.render(window, cx)
    }
}

impl IntoElement for Stack {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl ParentElement for Stack {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

pub struct Grid {
    id: String,
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
            id: stable_auto_id("grid"),
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

    fn apply_gap<T: Styled>(node: T, gap: Size) -> T {
        match gap {
            Size::Xs => node.gap_1(),
            Size::Sm => node.gap_1p5(),
            Size::Md => node.gap_2(),
            Size::Lg => node.gap_3(),
            Size::Xl => node.gap_4(),
        }
    }
}

impl ParentElement for Grid {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl WithId for Grid {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Grid {
    fn render(mut self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let gap_x = self.gap_x;
        let gap_y = self.gap_y;
        let columns = self.columns.max(1);
        let mut rows = Vec::new();

        let mut current_row = Vec::new();
        for child in self.children {
            current_row.push(child);
            if current_row.len() == columns {
                let mut row = div().flex().flex_row().w_full();
                row = Self::apply_gap(row, gap_x);
                let items = current_row
                    .drain(..)
                    .map(|item| div().flex_1().min_w_0().child(item).into_any_element())
                    .collect::<Vec<_>>();
                rows.push(row.children(items).into_any_element());
            }
        }

        if !current_row.is_empty() {
            let mut row = div().flex().flex_row().w_full();
            row = Self::apply_gap(row, gap_x);
            while current_row.len() < columns {
                current_row.push(div().w_full().h_full().into_any_element());
            }
            let items = current_row
                .drain(..)
                .map(|item| div().flex_1().min_w_0().child(item).into_any_element())
                .collect::<Vec<_>>();
            rows.push(row.children(items).into_any_element());
        }

        Self::apply_gap(div().id(self.id).flex().flex_col().w_full(), gap_y)
            .text_color(self.theme.resolve_hsla(&self.theme.semantic.text_primary))
            .children(rows)
    }
}

impl IntoElement for Grid {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

pub struct SimpleGrid {
    inner: Grid,
    style: gpui::StyleRefinement,
}

impl SimpleGrid {
    #[track_caller]
    pub fn new() -> Self {
        Self {
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

impl WithId for SimpleGrid {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn id_mut(&mut self) -> &mut String {
        self.inner.id_mut()
    }
}

impl RenderOnce for SimpleGrid {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let mut inner = self.inner;
        gpui::Refineable::refine(gpui::Styled::style(&mut inner), &self.style);
        inner.render(window, cx)
    }
}

impl IntoElement for SimpleGrid {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl ParentElement for SimpleGrid {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

pub fn stack() -> Stack {
    Stack::new().direction(StackDirection::Vertical)
}

pub fn h_stack_layout() -> Stack {
    Stack::new().direction(StackDirection::Horizontal)
}

pub fn v_stack_layout() -> Stack {
    Stack::new().direction(StackDirection::Vertical)
}

pub fn fixed_spacer(width: f32, height: f32) -> impl IntoElement {
    div().w(px(width.max(0.0))).h(px(height.max(0.0)))
}

impl crate::contracts::ComponentThemePatchable for Flex {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl crate::contracts::ComponentThemePatchable for Grid {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Flex {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for Grid {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for Stack {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Styled for SimpleGrid {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
