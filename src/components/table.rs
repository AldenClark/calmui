use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    ScrollHandle, SharedString, StatefulInteractiveElement, Styled, canvas, div, point, px,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size};

use super::Stack;
use super::control;
use super::pagination::Pagination;
use super::scroll_area::{ScrollArea, ScrollDirection};
use super::transition::TransitionExt;
use super::utils::{apply_radius, hairline_px, resolve_hsla};

type CellRenderer = Box<dyn FnOnce() -> AnyElement>;
type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type PageChangeHandler = Rc<dyn Fn(usize, &mut gpui::Window, &mut gpui::App)>;
type PageSizeChangeHandler = Rc<dyn Fn(usize, &mut gpui::Window, &mut gpui::App)>;
type RowClickHandler = Rc<dyn Fn(usize, &mut gpui::Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableSortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableSort {
    pub column: usize,
    pub direction: TableSortDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TablePaginationPosition {
    Top,
    Bottom,
    Both,
}

pub struct TableCell {
    content: CellRenderer,
    align: TableAlign,
    sort_value: Option<SharedString>,
    filter_value: Option<SharedString>,
}

impl TableCell {
    pub fn new(content: impl IntoElement + 'static) -> Self {
        Self {
            content: Box::new(|| content.into_any_element()),
            align: TableAlign::Left,
            sort_value: None,
            filter_value: None,
        }
    }

    pub fn align(mut self, value: TableAlign) -> Self {
        self.align = value;
        self
    }

    pub fn sort_value(mut self, value: impl Into<SharedString>) -> Self {
        self.sort_value = Some(value.into());
        self
    }

    pub fn filter_value(mut self, value: impl Into<SharedString>) -> Self {
        self.filter_value = Some(value.into());
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
    footer: Option<SlotRenderer>,
    empty: Option<SlotRenderer>,
    striped: bool,
    highlight_on_hover: bool,
    max_height_px: Option<f32>,
    sticky_header: bool,
    sort: Option<TableSort>,
    filter_query: Option<SharedString>,
    filter_column: Option<usize>,
    virtual_window: Option<(usize, usize)>,
    auto_virtualization: bool,
    virtualization_overscan_rows: usize,
    virtual_row_height_px: Option<f32>,
    virtualization_min_rows: usize,
    pagination_enabled: bool,
    page_size: usize,
    page: Option<usize>,
    page_controlled: bool,
    default_page: usize,
    pagination_siblings: usize,
    pagination_boundaries: usize,
    pagination_position: TablePaginationPosition,
    show_pagination: bool,
    show_page_size_selector: bool,
    page_size_options: Vec<usize>,
    with_outer_border: bool,
    with_column_borders: bool,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_page_change: Option<PageChangeHandler>,
    on_page_size_change: Option<PageSizeChangeHandler>,
    on_row_click: Option<RowClickHandler>,
}

impl Table {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("table"),
            headers: Vec::new(),
            rows: Vec::new(),
            caption: None,
            footer: None,
            empty: None,
            striped: true,
            highlight_on_hover: true,
            max_height_px: None,
            sticky_header: false,
            sort: None,
            filter_query: None,
            filter_column: None,
            virtual_window: None,
            auto_virtualization: false,
            virtualization_overscan_rows: 6,
            virtual_row_height_px: None,
            virtualization_min_rows: 120,
            pagination_enabled: false,
            page_size: 20,
            page: None,
            page_controlled: false,
            default_page: 1,
            pagination_siblings: 1,
            pagination_boundaries: 1,
            pagination_position: TablePaginationPosition::Bottom,
            show_pagination: true,
            show_page_size_selector: true,
            page_size_options: vec![10, 20, 50, 100],
            with_outer_border: true,
            with_column_borders: false,
            size: Size::Md,
            radius: Radius::Sm,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_page_change: None,
            on_page_size_change: None,
            on_row_click: None,
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

    pub fn footer(mut self, content: impl IntoElement + 'static) -> Self {
        self.footer = Some(Box::new(|| content.into_any_element()));
        self
    }

    pub fn empty(mut self, content: impl IntoElement + 'static) -> Self {
        self.empty = Some(Box::new(|| content.into_any_element()));
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

    pub fn max_height(mut self, value: f32) -> Self {
        self.max_height_px = Some(value.max(80.0));
        self
    }

    pub fn sticky_header(mut self, value: bool) -> Self {
        self.sticky_header = value;
        self
    }

    pub fn sort(mut self, column: usize, direction: TableSortDirection) -> Self {
        self.sort = Some(TableSort { column, direction });
        self
    }

    pub fn clear_sort(mut self) -> Self {
        self.sort = None;
        self
    }

    pub fn filter(mut self, query: impl Into<SharedString>) -> Self {
        self.filter_query = Some(query.into());
        self
    }

    pub fn filter_column(mut self, column: usize) -> Self {
        self.filter_column = Some(column);
        self
    }

    pub fn clear_filter(mut self) -> Self {
        self.filter_query = None;
        self
    }

    pub fn virtual_window(mut self, start: usize, count: usize) -> Self {
        self.virtual_window = Some((start, count.max(1)));
        self
    }

    pub fn auto_virtualization(mut self, value: bool) -> Self {
        self.auto_virtualization = value;
        self
    }

    pub fn virtualization_overscan_rows(mut self, value: usize) -> Self {
        self.virtualization_overscan_rows = value.max(1);
        self
    }

    pub fn virtual_row_height(mut self, value: f32) -> Self {
        self.virtual_row_height_px = Some(value.max(1.0));
        self
    }

    pub fn virtualization_min_rows(mut self, value: usize) -> Self {
        self.virtualization_min_rows = value.max(1);
        self
    }

    pub fn pagination(mut self, value: bool) -> Self {
        self.pagination_enabled = value;
        self
    }

    pub fn page_size(mut self, value: usize) -> Self {
        self.page_size = value.max(1);
        self
    }

    pub fn page(mut self, value: usize) -> Self {
        self.page = Some(value.max(1));
        self.page_controlled = true;
        self
    }

    pub fn default_page(mut self, value: usize) -> Self {
        self.default_page = value.max(1);
        self
    }

    pub fn pagination_siblings(mut self, value: usize) -> Self {
        self.pagination_siblings = value.min(4);
        self
    }

    pub fn pagination_boundaries(mut self, value: usize) -> Self {
        self.pagination_boundaries = value.min(4);
        self
    }

    pub fn pagination_position(mut self, value: TablePaginationPosition) -> Self {
        self.pagination_position = value;
        self
    }

    pub fn show_pagination(mut self, value: bool) -> Self {
        self.show_pagination = value;
        self
    }

    pub fn show_page_size_selector(mut self, value: bool) -> Self {
        self.show_page_size_selector = value;
        self
    }

    pub fn page_size_options(mut self, values: impl IntoIterator<Item = usize>) -> Self {
        let mut items = values
            .into_iter()
            .map(|value| value.max(1))
            .collect::<Vec<_>>();
        items.sort_unstable();
        items.dedup();
        if !items.is_empty() {
            self.page_size_options = items;
        }
        self
    }

    pub fn on_page_change(
        mut self,
        handler: impl Fn(usize, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_page_change = Some(Rc::new(handler));
        self
    }

    pub fn on_page_size_change(
        mut self,
        handler: impl Fn(usize, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_page_size_change = Some(Rc::new(handler));
        self
    }

    pub fn on_row_click(
        mut self,
        handler: impl Fn(usize, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_row_click = Some(Rc::new(handler));
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

    fn default_row_height_px(size: Size) -> f32 {
        match size {
            Size::Xs => 22.0,
            Size::Sm => 26.0,
            Size::Md => 32.0,
            Size::Lg => 38.0,
            Size::Xl => 44.0,
        }
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

impl RenderOnce for Table {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = self.theme.components.table.clone();
        let line_thickness = hairline_px(window);
        let line_thickness_px = f32::from(line_thickness);
        let column_count = self.column_count();
        let size = self.size;
        let table_id = self.id.clone();
        let caption = self.caption;
        let headers = self.headers;
        let striped = self.striped;
        let highlight_on_hover = self.highlight_on_hover;
        let with_column_borders = self.with_column_borders;
        let motion = self.motion;
        let max_height_px = self.max_height_px;
        let sticky_header = self.sticky_header;
        let filter_query = self
            .filter_query
            .as_ref()
            .map(|value| value.to_string().to_ascii_lowercase())
            .filter(|value| !value.trim().is_empty());
        let filter_column = self.filter_column;
        let sort = self.sort;
        let virtual_window = self.virtual_window;
        let pagination_enabled = self.pagination_enabled;
        let page_size = self.page_size.max(1);
        let page_controlled = self.page_controlled;
        let on_page_change = self.on_page_change.clone();
        let on_page_size_change = self.on_page_size_change.clone();
        let on_row_click = self.on_row_click.clone();
        let pagination_position = self.pagination_position;
        let show_page_size_selector = self.show_page_size_selector;
        let separator = || {
            div()
                .w_full()
                .h(line_thickness)
                .bg(resolve_hsla(&self.theme, &tokens.row_border))
        };

        let mut rows_with_meta = self
            .rows
            .into_iter()
            .enumerate()
            .map(|(source_index, row)| {
                let meta = (0..column_count)
                    .map(|index| {
                        row.cells
                            .get(index)
                            .and_then(|cell| {
                                cell.filter_value
                                    .clone()
                                    .or_else(|| cell.sort_value.clone())
                            })
                            .map(|value| value.to_string())
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>();
                (source_index, meta, row)
            })
            .collect::<Vec<_>>();

        if let Some(query) = filter_query {
            rows_with_meta.retain(|(_, meta, _)| {
                if let Some(column) = filter_column {
                    meta.get(column)
                        .map(|value| value.to_ascii_lowercase().contains(&query))
                        .unwrap_or(false)
                } else {
                    meta.iter()
                        .any(|value| value.to_ascii_lowercase().contains(&query))
                }
            });
        }

        if let Some(sort) = sort {
            let column = sort.column;
            rows_with_meta.sort_by(|(_, left_meta, _), (_, right_meta, _)| {
                let left = left_meta
                    .get(column)
                    .map(String::as_str)
                    .unwrap_or_default();
                let right = right_meta
                    .get(column)
                    .map(String::as_str)
                    .unwrap_or_default();
                match sort.direction {
                    TableSortDirection::Asc => left.cmp(right),
                    TableSortDirection::Desc => right.cmp(left),
                }
            });
        }

        let total_rows = rows_with_meta.len();
        let mut page_size_options = self
            .page_size_options
            .iter()
            .copied()
            .map(|value| value.max(1))
            .collect::<Vec<_>>();
        page_size_options.sort_unstable();
        page_size_options.dedup();
        if page_size_options.is_empty() {
            page_size_options.push(page_size);
        }
        let resolved_page_size = if pagination_enabled {
            let default = page_size.to_string();
            let size = control::text_state(&table_id, "page-size", None, default)
                .parse::<usize>()
                .ok()
                .unwrap_or(page_size)
                .max(1);
            if !page_size_options.contains(&size) {
                page_size_options.push(size);
                page_size_options.sort_unstable();
                page_size_options.dedup();
            }
            size
        } else {
            page_size
        };
        let page_count = if pagination_enabled {
            ((total_rows + resolved_page_size.saturating_sub(1)) / resolved_page_size).max(1)
        } else {
            1
        };
        let resolved_page = if pagination_enabled {
            let controlled = page_controlled.then_some(
                self.page
                    .unwrap_or(self.default_page)
                    .clamp(1, page_count)
                    .to_string(),
            );
            let default = self.default_page.clamp(1, page_count).to_string();
            control::text_state(&table_id, "page", controlled, default)
                .parse::<usize>()
                .ok()
                .unwrap_or(1)
                .clamp(1, page_count)
        } else {
            1
        };

        let resolved_scroll_height = max_height_px.map(|max_height| {
            if sticky_header && !headers.is_empty() {
                (max_height - 42.0).max(40.0)
            } else {
                max_height
            }
        });

        let auto_virtualization_enabled = self.auto_virtualization
            && !pagination_enabled
            && virtual_window.is_none()
            && resolved_scroll_height.is_some()
            && total_rows >= self.virtualization_min_rows;

        let default_row_extent = self
            .virtual_row_height_px
            .unwrap_or_else(|| Self::default_row_height_px(size))
            + line_thickness_px;
        let measured_row_height = control::text_state(
            &table_id,
            "virtual-row-height",
            None,
            Self::default_row_height_px(size).to_string(),
        )
        .parse::<f32>()
        .ok()
        .filter(|value| *value > 1.0)
        .unwrap_or_else(|| Self::default_row_height_px(size));
        let row_extent = if auto_virtualization_enabled {
            (self.virtual_row_height_px.unwrap_or(measured_row_height) + line_thickness_px).max(1.0)
        } else {
            default_row_extent
        };

        let scroll_height_for_virtual = resolved_scroll_height.unwrap_or(0.0);
        let max_scroll_y = if auto_virtualization_enabled {
            ((total_rows as f32 * row_extent) - line_thickness_px - scroll_height_for_virtual)
                .max(0.0)
        } else {
            0.0
        };
        let scroll_y = if auto_virtualization_enabled {
            control::text_state(&table_id, "virtual-scroll-y", None, "0".to_string())
                .parse::<f32>()
                .ok()
                .unwrap_or(0.0)
                .clamp(0.0, max_scroll_y)
        } else {
            0.0
        };

        let (window_start, window_count) = if pagination_enabled {
            (
                ((resolved_page - 1) * resolved_page_size),
                resolved_page_size,
            )
        } else if auto_virtualization_enabled {
            let overscan = self.virtualization_overscan_rows.max(1);
            let visible_count = ((scroll_height_for_virtual / row_extent).ceil() as usize)
                .saturating_add(overscan.saturating_mul(2))
                .saturating_add(2);
            let start = ((scroll_y / row_extent).floor() as usize).saturating_sub(overscan);
            (start, visible_count.max(1))
        } else {
            virtual_window.unwrap_or((0, total_rows.saturating_add(1)))
        };
        let rows = rows_with_meta
            .into_iter()
            .skip(window_start.min(total_rows))
            .take(window_count.max(1))
            .map(|(source_index, _, row)| (source_index, row))
            .collect::<Vec<_>>();
        if auto_virtualization_enabled {
            control::set_text_state(&table_id, "virtual-window-start", window_start.to_string());
            control::set_text_state(&table_id, "virtual-scroll-y", scroll_y.to_string());
        }

        let mut root = Stack::vertical()
            .id(table_id.clone())
            .w_full()
            .gap_0()
            .bg(resolve_hsla(&self.theme, &tokens.row_bg));

        if self.with_outer_border {
            root = root
                .border(line_thickness)
                .border_color(resolve_hsla(&self.theme, &tokens.row_border));
            root = apply_radius(&self.theme, root, self.radius);
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
            root = root.child(separator());
        }

        let mut header_row_any = None;
        if !headers.is_empty() {
            let mut header_row = div()
                .id(format!("{}-header", table_id))
                .w_full()
                .flex()
                .items_center()
                .bg(resolve_hsla(&self.theme, &tokens.header_bg))
                .text_color(resolve_hsla(&self.theme, &tokens.header_fg));

            for index in 0..column_count {
                if index > 0 && with_column_borders {
                    header_row = header_row.child(
                        div()
                            .w(line_thickness)
                            .h_full()
                            .bg(resolve_hsla(&self.theme, &tokens.row_border)),
                    );
                }

                let text = headers.get(index).cloned().unwrap_or_else(|| {
                    if index >= headers.len() {
                        SharedString::from(format!("Col {}", index + 1))
                    } else {
                        SharedString::default()
                    }
                });
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

            header_row_any = Some(header_row.into_any_element());
        }

        let visible_row_count = rows.len();
        let has_rows = visible_row_count > 0;
        let top_spacer_height = if auto_virtualization_enabled {
            (window_start as f32 * row_extent).max(0.0)
        } else {
            0.0
        };
        let remaining_rows =
            total_rows.saturating_sub(window_start.saturating_add(visible_row_count));
        let bottom_spacer_height = if auto_virtualization_enabled {
            (remaining_rows as f32 * row_extent).max(0.0)
        } else {
            0.0
        };
        let mut rows_root = Stack::vertical()
            .id(format!("{}-rows", table_id))
            .w_full()
            .gap_0();
        if top_spacer_height > 0.0 {
            rows_root = rows_root.child(
                div()
                    .id(format!("{}-virtual-top-spacer", table_id))
                    .w_full()
                    .h(px(top_spacer_height)),
            );
        }
        for (row_index, (source_index, row)) in rows.into_iter().enumerate() {
            if row_index > 0 {
                rows_root = rows_root.child(separator());
            }
            let striped_index = window_start + row_index;
            let row_bg = if striped && striped_index % 2 == 1 {
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
                .text_color(resolve_hsla(&self.theme, &tokens.cell_fg));

            if highlight_on_hover {
                let hover_bg = resolve_hsla(&self.theme, &tokens.row_hover_bg);
                row_node = row_node.hover(move |style| style.bg(hover_bg));
            }
            if let Some(handler) = on_row_click.as_ref() {
                let on_row_click = handler.clone();
                row_node = row_node.cursor_pointer().on_click(
                    move |_: &ClickEvent, window: &mut gpui::Window, cx: &mut gpui::App| {
                        (on_row_click)(source_index, window, cx);
                    },
                );
            }
            if auto_virtualization_enabled && row_index == 0 {
                let table_id_for_height = table_id.clone();
                row_node = row_node.relative().child(
                    canvas(
                        move |bounds, window, _cx| {
                            let measured = f32::from(bounds.size.height).max(1.0);
                            let previous = control::text_state(
                                &table_id_for_height,
                                "virtual-row-height",
                                None,
                                "0".to_string(),
                            )
                            .parse::<f32>()
                            .ok()
                            .unwrap_or(0.0);
                            if (measured - previous).abs() > 0.5 {
                                control::set_text_state(
                                    &table_id_for_height,
                                    "virtual-row-height",
                                    measured.to_string(),
                                );
                                window.refresh();
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                );
            }

            let mut cells = row.cells.into_iter();
            for column in 0..column_count {
                if column > 0 && with_column_borders {
                    row_node = row_node.child(
                        div()
                            .w(line_thickness)
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

            rows_root = rows_root.child(row_node);
        }
        if bottom_spacer_height > 0.0 {
            rows_root = rows_root.child(
                div()
                    .id(format!("{}-virtual-bottom-spacer", table_id))
                    .w_full()
                    .h(px(bottom_spacer_height)),
            );
        }

        if !has_rows {
            rows_root = rows_root.child(Self::apply_cell_size(
                size,
                div()
                    .id(format!("{}-empty", table_id))
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(resolve_hsla(&self.theme, &tokens.caption))
                    .child(
                        self.empty
                            .take()
                            .map(|slot| slot())
                            .unwrap_or_else(|| div().child("No data").into_any_element()),
                    ),
            ));
        }

        let render_pagination_bar = |suffix: &str| {
            let page_summary = format!(
                "Page {} / {} · {} rows",
                resolved_page, page_count, total_rows
            );
            let table_id_for_page = table_id.clone();
            let table_id_for_page_size = table_id.clone();
            let on_page_change = on_page_change.clone();
            let on_page_size_change = on_page_size_change.clone();
            let page_size_options = page_size_options.clone();
            let size_selector = if show_page_size_selector {
                let mut items = div()
                    .id(format!("{}-page-size-selector-{suffix}", table_id))
                    .flex()
                    .items_center()
                    .gap_1();
                for option in page_size_options {
                    let is_active = option == resolved_page_size;
                    let mut item = div()
                        .id(format!("{}-page-size-{}-{suffix}", table_id, option))
                        .px_2()
                        .py_1()
                        .text_sm()
                        .rounded_sm()
                        .border(super::utils::quantized_stroke_px(window, 1.0))
                        .border_color(resolve_hsla(&self.theme, &tokens.row_border))
                        .bg(if is_active {
                            resolve_hsla(&self.theme, &tokens.header_bg)
                        } else {
                            resolve_hsla(&self.theme, &tokens.row_bg)
                        })
                        .text_color(if is_active {
                            resolve_hsla(&self.theme, &tokens.header_fg)
                        } else {
                            resolve_hsla(&self.theme, &tokens.cell_fg)
                        })
                        .child(format!("{option} / page"));
                    if !is_active {
                        let on_page_size_change = on_page_size_change.clone();
                        let table_id_for_page_size = table_id_for_page_size.clone();
                        item = item.cursor_pointer().on_click(
                            move |_: &ClickEvent, window: &mut gpui::Window, cx| {
                                control::set_text_state(
                                    &table_id_for_page_size,
                                    "page-size",
                                    option.to_string(),
                                );
                                control::set_text_state(
                                    &table_id_for_page_size,
                                    "page",
                                    "1".to_string(),
                                );
                                window.refresh();
                                if let Some(handler) = on_page_size_change.as_ref() {
                                    (handler)(option, window, cx);
                                }
                            },
                        );
                    } else {
                        item = item.cursor_default();
                    }
                    items = items.child(item);
                }
                Some(items.into_any_element())
            } else {
                None
            };

            let mut right = div().flex().items_center().gap_2().child(
                Pagination::new()
                    .with_id(format!("{}-pagination-{suffix}", table_id))
                    .total(page_count)
                    .value(resolved_page)
                    .siblings(self.pagination_siblings)
                    .boundaries(self.pagination_boundaries)
                    .on_change(move |next_page: usize, window: &mut gpui::Window, cx| {
                        if !page_controlled {
                            control::set_text_state(
                                &table_id_for_page,
                                "page",
                                next_page.to_string(),
                            );
                            window.refresh();
                        }
                        if let Some(handler) = on_page_change.as_ref() {
                            (handler)(next_page, window, cx);
                        }
                    }),
            );
            if let Some(selector) = size_selector {
                right = right.child(selector);
            }

            div()
                .id(format!("{}-pagination-bar-{suffix}", table_id))
                .w_full()
                .px_3()
                .py_2()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .bg(resolve_hsla(&self.theme, &tokens.row_bg))
                .child(
                    div()
                        .text_sm()
                        .text_color(resolve_hsla(&self.theme, &tokens.caption))
                        .child(page_summary),
                )
                .child(right)
                .into_any_element()
        };

        if pagination_enabled
            && self.show_pagination
            && matches!(
                pagination_position,
                TablePaginationPosition::Top | TablePaginationPosition::Both
            )
        {
            root = root.child(separator()).child(render_pagination_bar("top"));
        }

        if let Some(scroll_height) = resolved_scroll_height {
            if sticky_header {
                if let Some(header_row) = header_row_any.take() {
                    root = root.child(header_row);
                    if has_rows {
                        root = root.child(separator());
                    }
                }
            }

            let mut scroll_content = Stack::vertical()
                .id(format!("{}-scroll-content", table_id))
                .w_full()
                .gap_0();
            if !sticky_header && let Some(header_row) = header_row_any.take() {
                scroll_content = scroll_content.child(header_row);
                if has_rows {
                    scroll_content = scroll_content.child(separator());
                }
            }
            scroll_content = scroll_content.child(rows_root);
            if auto_virtualization_enabled {
                let scroll_handle = ScrollHandle::new();
                scroll_handle.set_offset(point(px(0.0), px(-scroll_y)));
                let table_id_for_scroll = table_id.clone();
                let handle_for_monitor = scroll_handle.clone();
                let overscan_rows = self.virtualization_overscan_rows.max(1);
                let row_extent_for_monitor = row_extent;
                let max_scroll_for_monitor = max_scroll_y;
                root = root.child(
                    div()
                        .id(format!("{}-virtual-scroll", table_id))
                        .relative()
                        .w_full()
                        .h(px(scroll_height))
                        .overflow_y_scroll()
                        .track_scroll(&scroll_handle)
                        .p_1()
                        .child(scroll_content)
                        .child(
                            canvas(
                                move |_bounds, window, _cx| {
                                    let next_y = (-f32::from(handle_for_monitor.offset().y))
                                        .clamp(0.0, max_scroll_for_monitor);
                                    let current_y = control::text_state(
                                        &table_id_for_scroll,
                                        "virtual-scroll-y",
                                        None,
                                        "0".to_string(),
                                    )
                                    .parse::<f32>()
                                    .ok()
                                    .unwrap_or(0.0);
                                    if (next_y - current_y).abs() > 0.5 {
                                        control::set_text_state(
                                            &table_id_for_scroll,
                                            "virtual-scroll-y",
                                            next_y.to_string(),
                                        );
                                    }

                                    let next_start = ((next_y / row_extent_for_monitor).floor()
                                        as usize)
                                        .saturating_sub(overscan_rows);
                                    let prev_start = control::text_state(
                                        &table_id_for_scroll,
                                        "virtual-window-start",
                                        None,
                                        "0".to_string(),
                                    )
                                    .parse::<usize>()
                                    .ok()
                                    .unwrap_or(0);
                                    if next_start != prev_start {
                                        control::set_text_state(
                                            &table_id_for_scroll,
                                            "virtual-window-start",
                                            next_start.to_string(),
                                        );
                                        window.refresh();
                                    }
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .size_full(),
                        ),
                );
            } else {
                root = root.child(
                    ScrollArea::new()
                        .with_id(format!("{}-scroll", table_id))
                        .direction(ScrollDirection::Vertical)
                        .bordered(false)
                        .padding(Size::Xs)
                        .viewport_height(scroll_height)
                        .child(scroll_content),
                );
            }
        } else {
            if let Some(header_row) = header_row_any.take() {
                root = root.child(header_row);
                if has_rows {
                    root = root.child(separator());
                }
            }
            root = root.child(rows_root);
        }

        if let Some(footer) = self.footer.take() {
            root = root.child(separator()).child(footer());
        }

        if pagination_enabled
            && self.show_pagination
            && matches!(
                pagination_position,
                TablePaginationPosition::Bottom | TablePaginationPosition::Both
            )
        {
            root = root
                .child(separator())
                .child(render_pagination_bar("bottom"));
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

impl crate::contracts::ComponentThemeOverridable for Table {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Table {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
