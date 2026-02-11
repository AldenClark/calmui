use gpui::{
    AnyElement, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, div, px,
};

use crate::contracts::{MotionAware, ThemeScoped, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size};
use crate::theme::Theme;

use super::primitives::v_stack;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type CellRenderer = Box<dyn FnOnce() -> AnyElement>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableAlign {
    Left,
    Center,
    Right,
}

pub struct TableCell {
    content: CellRenderer,
    align: TableAlign,
}

impl TableCell {
    pub fn new(content: impl IntoElement + 'static) -> Self {
        Self {
            content: Box::new(|| content.into_any_element()),
            align: TableAlign::Left,
        }
    }

    pub fn align(mut self, value: TableAlign) -> Self {
        self.align = value;
        self
    }
}

pub struct TableRow {
    cells: Vec<TableCell>,
}

impl TableRow {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn cell(mut self, cell: TableCell) -> Self {
        self.cells.push(cell);
        self
    }

    pub fn cells(mut self, cells: impl IntoIterator<Item = TableCell>) -> Self {
        self.cells.extend(cells);
        self
    }
}

pub struct Table {
    id: String,
    headers: Vec<SharedString>,
    rows: Vec<TableRow>,
    caption: Option<SharedString>,
    striped: bool,
    highlight_on_hover: bool,
    with_outer_border: bool,
    with_column_borders: bool,
    size: Size,
    radius: Radius,
    theme: Theme,
    motion: MotionConfig,
}

impl Table {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("table"),
            headers: Vec::new(),
            rows: Vec::new(),
            caption: None,
            striped: true,
            highlight_on_hover: true,
            with_outer_border: true,
            with_column_borders: false,
            size: Size::Md,
            radius: Radius::Sm,
            theme: Theme::default(),
            motion: MotionConfig::default(),
        }
    }

    pub fn header(mut self, text: impl Into<SharedString>) -> Self {
        self.headers.push(text.into());
        self
    }

    pub fn headers(mut self, headers: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.headers.extend(headers.into_iter().map(Into::into));
        self
    }

    pub fn row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    pub fn rows(mut self, rows: impl IntoIterator<Item = TableRow>) -> Self {
        self.rows.extend(rows);
        self
    }

    pub fn caption(mut self, text: impl Into<SharedString>) -> Self {
        self.caption = Some(text.into());
        self
    }

    pub fn striped(mut self, value: bool) -> Self {
        self.striped = value;
        self
    }

    pub fn highlight_on_hover(mut self, value: bool) -> Self {
        self.highlight_on_hover = value;
        self
    }

    pub fn with_outer_border(mut self, value: bool) -> Self {
        self.with_outer_border = value;
        self
    }

    pub fn with_column_borders(mut self, value: bool) -> Self {
        self.with_column_borders = value;
        self
    }

    pub fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    pub fn radius(mut self, value: Radius) -> Self {
        self.radius = value;
        self
    }

    fn apply_cell_size<T: Styled>(size: Size, node: T) -> T {
        match size {
            Size::Xs => node.text_xs().px_2().py_1(),
            Size::Sm => node.text_sm().px_2p5().py_1p5(),
            Size::Md => node.text_base().px_3().py_2(),
            Size::Lg => node.text_lg().px_3p5().py_2(),
            Size::Xl => node.text_xl().px_4().py_2p5(),
        }
    }

    fn column_count(&self) -> usize {
        let row_max = self
            .rows
            .iter()
            .map(|row| row.cells.len())
            .max()
            .unwrap_or(0);
        self.headers.len().max(row_max).max(1)
    }
}

impl WithId for Table {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl MotionAware for Table {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl ThemeScoped for Table {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for Table {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let tokens = self.theme.components.table.clone();
        let column_count = self.column_count();
        let size = self.size;
        let table_id = self.id.clone();
        let caption = self.caption;
        let headers = self.headers;
        let rows = self.rows;
        let striped = self.striped;
        let highlight_on_hover = self.highlight_on_hover;
        let with_column_borders = self.with_column_borders;
        let motion = self.motion;

        let mut root = v_stack()
            .id(table_id.clone())
            .w_full()
            .gap_0()
            .bg(resolve_hsla(&self.theme, &tokens.row_bg));

        if self.with_outer_border {
            root = root
                .border_1()
                .border_color(resolve_hsla(&self.theme, &tokens.row_border));
            root = apply_radius(root, self.radius);
        }

        if let Some(caption) = caption {
            let caption = Self::apply_cell_size(
                size,
                div()
                    .id(format!("{}-caption", table_id))
                    .text_color(resolve_hsla(&self.theme, &tokens.caption))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(caption),
            );
            root = root.child(caption);
        }

        if !headers.is_empty() {
            let mut header_row = div()
                .id(format!("{}-header", table_id))
                .w_full()
                .flex()
                .items_center()
                .bg(resolve_hsla(&self.theme, &tokens.header_bg))
                .text_color(resolve_hsla(&self.theme, &tokens.header_fg))
                .border_1()
                .border_color(resolve_hsla(&self.theme, &tokens.row_border));

            for index in 0..column_count {
                if index > 0 && with_column_borders {
                    header_row = header_row.child(
                        div()
                            .w(px(1.0))
                            .h_full()
                            .bg(resolve_hsla(&self.theme, &tokens.row_border)),
                    );
                }

                let text = headers.get(index).cloned().unwrap_or_default();
                let cell = Self::apply_cell_size(
                    size,
                    div()
                        .id(format!("{}-header-cell-{index}", table_id))
                        .flex_1()
                        .min_w_0()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .truncate()
                        .child(text),
                );
                header_row = header_row.child(cell);
            }

            root = root.child(header_row);
        }

        for (row_index, row) in rows.into_iter().enumerate() {
            let row_bg = if striped && row_index % 2 == 1 {
                resolve_hsla(&self.theme, &tokens.row_alt_bg)
            } else {
                resolve_hsla(&self.theme, &tokens.row_bg)
            };

            let mut row_node = div()
                .id(format!("{}-row-{row_index}", table_id))
                .w_full()
                .flex()
                .items_center()
                .bg(row_bg)
                .text_color(resolve_hsla(&self.theme, &tokens.cell_fg))
                .border_1()
                .border_color(resolve_hsla(&self.theme, &tokens.row_border));

            if highlight_on_hover {
                let hover_bg = resolve_hsla(&self.theme, &tokens.row_hover_bg);
                row_node = row_node.hover(move |style| style.bg(hover_bg));
            }

            let mut cells = row.cells.into_iter();
            for column in 0..column_count {
                if column > 0 && with_column_borders {
                    row_node = row_node.child(
                        div()
                            .w(px(1.0))
                            .h_full()
                            .bg(resolve_hsla(&self.theme, &tokens.row_border)),
                    );
                }

                let next_cell = cells.next();
                let mut cell = Self::apply_cell_size(
                    size,
                    div()
                        .id(format!("{}-row-{row_index}-cell-{column}", table_id))
                        .flex_1()
                        .min_w_0(),
                );

                if let Some(cell_data) = next_cell {
                    cell = match cell_data.align {
                        TableAlign::Left => cell.items_start().justify_start(),
                        TableAlign::Center => cell.items_center().justify_center(),
                        TableAlign::Right => cell.items_end().justify_end(),
                    }
                    .child((cell_data.content)());
                } else {
                    cell = cell.child(div().child(""));
                }

                row_node = row_node.child(cell);
            }

            root = root.child(row_node);
        }

        root.with_enter_transition(format!("{}-enter", table_id), motion)
    }
}

impl IntoElement for Table {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}
