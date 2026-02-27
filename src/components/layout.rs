use gpui::{
    AlignItems, AnyElement, Div, Hsla, InteractiveElement, Interactivity, IntoElement,
    JustifyContent, ParentElement, Pixels, RenderOnce, StatefulInteractiveElement, Styled, Window,
    div, px,
};

use crate::id::ComponentId;
use crate::style::Size;

#[derive(IntoElement)]
pub struct Stack {
    pub(crate) id: ComponentId,
    inner: Div,
}

impl Stack {
    #[track_caller]
    pub fn vertical() -> Self {
        Self {
            id: ComponentId::default(),
            inner: div().flex().flex_col(),
        }
    }

    #[track_caller]
    pub fn horizontal() -> Self {
        Self {
            id: ComponentId::default(),
            inner: div().flex().flex_row().items_center(),
        }
    }

    pub fn id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn into_div(self) -> Div {
        self.inner
    }

    pub(crate) fn style_mut(&mut self) -> &mut gpui::StyleRefinement {
        self.inner.style()
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

    pub fn gap(mut self, value: Pixels) -> Self {
        self.inner = self.inner.gap(value);
        self
    }

    pub fn gap_x(mut self, value: Pixels) -> Self {
        self.inner = self.inner.gap_x(value);
        self
    }

    pub fn gap_y(mut self, value: Pixels) -> Self {
        self.inner = self.inner.gap_y(value);
        self
    }

    pub fn p(mut self, value: Pixels) -> Self {
        self.inner = self.inner.p(value);
        self
    }

    pub fn pb(mut self, value: Pixels) -> Self {
        self.inner = self.inner.pb(value);
        self
    }

    pub fn pt(mut self, value: Pixels) -> Self {
        self.inner = self.inner.pt(value);
        self
    }

    pub fn w(mut self, value: Pixels) -> Self {
        self.inner = self.inner.w(value);
        self
    }

    pub fn min_w(mut self, value: Pixels) -> Self {
        self.inner = self.inner.min_w(value);
        self
    }

    pub fn max_w_full(mut self) -> Self {
        self.inner = self.inner.max_w_full();
        self
    }

    pub fn max_w(mut self, value: Pixels) -> Self {
        self.inner = self.inner.max_w(value);
        self
    }

    pub fn h(mut self, value: Pixels) -> Self {
        self.inner = self.inner.h(value);
        self
    }

    pub fn min_h(mut self, value: Pixels) -> Self {
        self.inner = self.inner.min_h(value);
        self
    }

    pub fn max_h(mut self, value: Pixels) -> Self {
        self.inner = self.inner.max_h(value);
        self
    }

    pub fn items_start(mut self) -> Self {
        self.inner = self.inner.items_start();
        self
    }

    pub fn items_center(mut self) -> Self {
        self.inner = self.inner.items_center();
        self
    }

    pub fn items_end(mut self) -> Self {
        self.inner = self.inner.items_end();
        self
    }

    pub fn items_stretch(mut self) -> Self {
        self.inner.style().align_items = Some(AlignItems::Stretch);
        self
    }

    pub fn justify_start(mut self) -> Self {
        self.inner = self.inner.justify_start();
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.inner = self.inner.justify_center();
        self
    }

    pub fn justify_end(mut self) -> Self {
        self.inner = self.inner.justify_end();
        self
    }

    pub fn justify_between(mut self) -> Self {
        self.inner = self.inner.justify_between();
        self
    }

    pub fn justify_around(mut self) -> Self {
        self.inner = self.inner.justify_around();
        self
    }

    pub fn justify_evenly(mut self) -> Self {
        self.inner.style().justify_content = Some(JustifyContent::SpaceEvenly);
        self
    }

    pub fn w_full(mut self) -> Self {
        self.inner = self.inner.w_full();
        self
    }

    pub fn h_full(mut self) -> Self {
        self.inner = self.inner.h_full();
        self
    }

    pub fn min_w_0(mut self) -> Self {
        self.inner = self.inner.min_w_0();
        self
    }

    pub fn min_h_0(mut self) -> Self {
        self.inner = self.inner.min_h_0();
        self
    }

    pub fn size_full(mut self) -> Self {
        self.inner = self.inner.size_full();
        self
    }

    pub fn max_h_full(mut self) -> Self {
        self.inner = self.inner.max_h_full();
        self
    }

    pub fn flex_1(mut self) -> Self {
        self.inner = self.inner.flex_1();
        self
    }

    pub fn flex_none(mut self) -> Self {
        self.inner = self.inner.flex_none();
        self
    }

    pub fn overflow_hidden(mut self) -> Self {
        self.inner = self.inner.overflow_hidden();
        self
    }

    pub fn overflow_scroll(self) -> Self {
        <Self as StatefulInteractiveElement>::overflow_scroll(self)
    }

    pub fn overflow_x_scroll(self) -> Self {
        <Self as StatefulInteractiveElement>::overflow_x_scroll(self)
    }

    pub fn overflow_y_scroll(self) -> Self {
        <Self as StatefulInteractiveElement>::overflow_y_scroll(self)
    }

    pub fn flex_wrap(mut self) -> Self {
        self.inner = self.inner.flex_wrap();
        self
    }

    pub fn flex_nowrap(mut self) -> Self {
        self.inner = self.inner.flex_nowrap();
        self
    }

    pub fn relative(mut self) -> Self {
        self.inner = self.inner.relative();
        self
    }

    pub fn absolute(mut self) -> Self {
        self.inner = self.inner.absolute();
        self
    }

    pub fn inset_0(mut self) -> Self {
        self.inner = self.inner.top_0().right_0().bottom_0().left_0();
        self
    }

    pub fn bg(mut self, value: Hsla) -> Self {
        self.inner = self.inner.bg(value);
        self
    }

    pub fn border(mut self, value: Pixels) -> Self {
        self.inner = self.inner.border(value);
        self
    }

    pub fn border_color(mut self, value: Hsla) -> Self {
        self.inner = self.inner.border_color(value);
        self
    }

    pub fn py(mut self, value: Pixels) -> Self {
        self.inner = self.inner.py(value);
        self
    }

    pub fn px(mut self, value: Pixels) -> Self {
        self.inner = self.inner.px(value);
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.inner = self.inner.opacity(value);
        self
    }

    pub fn shadow_sm(mut self) -> Self {
        self.inner = self.inner.shadow_sm();
        self
    }

    pub fn rounded(mut self, value: Pixels) -> Self {
        self.inner = self.inner.rounded(value);
        self
    }

    pub fn with_enter_transition(
        self,
        id: impl Into<gpui::ElementId>,
        motion: crate::motion::MotionConfig,
    ) -> gpui::AnimationElement<gpui::Stateful<Div>> {
        use super::transition::TransitionExt;
        self.inner.id(self.id).with_enter_transition(id, motion)
    }
}

impl crate::contracts::WithId for Stack {
    fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl InteractiveElement for Stack {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Stack {}

impl RenderOnce for Stack {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.inner.id(self.id)
    }
}

#[derive(IntoElement)]
pub struct Grid {
    pub(crate) id: ComponentId,
    columns: usize,
    gap_x: Size,
    gap_y: Size,
    pub(crate) theme: crate::theme::LocalTheme,
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

impl Grid {}

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
            .text_color(self.theme.resolve_hsla(self.theme.semantic.text_primary))
            .children(rows)
    }
}

#[derive(IntoElement)]
pub struct SimpleGrid {
    pub(crate) id: ComponentId,
    pub(crate) inner: Grid,
}

impl SimpleGrid {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            inner: Grid::new(),
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

impl SimpleGrid {}

impl RenderOnce for SimpleGrid {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let inner = self.id.ctx().root(self.inner);
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
    pub(crate) id: ComponentId,
    width_px: Option<f32>,
    height_px: Option<f32>,
    width_size: Option<Size>,
    height_size: Option<Size>,
    pub(crate) theme: crate::theme::LocalTheme,
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

impl Space {}

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

crate::impl_sized_via_method!(Space, |this, value| {
    this.width_size = Some(value);
    this.height_size = Some(value);
    this.width_px = None;
    this.height_px = None;
});
