use std::rc::Rc;

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, RenderOnce, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, canvas, div, point, px,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size};

use super::Stack;
use super::interaction_adapter::{ActivateHandler, PressAdapter, bind_press_adapter};
use super::pagination::Pagination;
use super::scroll_area::{ScrollArea, ScrollDirection};
use super::table_state::{self, TableState, TableStateInput};
use super::transition::TransitionExt;
use super::utils::{
    InteractionStyles, apply_interaction_styles, apply_radius, hairline_px, interaction_style,
    resolve_hsla,
};

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

#[derive(IntoElement)]
pub struct Table {
    id: ComponentId,
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
            id: ComponentId::default(),
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
        self.max_height_px = Some(value.max(0.0));
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

    pub fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    pub fn with_radius(mut self, value: Radius) -> Self {
        self.radius = value;
        self
    }

    fn apply_cell_size<T: Styled>(preset: crate::theme::TableSizePreset, node: T) -> T {
        node.text_size(preset.font_size)
            .px(preset.padding_x)
            .py(preset.padding_y)
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

    fn default_row_height_px(preset: crate::theme::TableSizePreset) -> f32 {
        f32::from(preset.row_height)
    }
}

impl Table {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
        let table_size_preset = tokens.sizes.for_size(self.size);
        let line_thickness = hairline_px(window);
        let line_thickness_px = f32::from(line_thickness);
        let column_count = self.column_count();
        let table_id = self.id.clone();
        let caption = self.caption;
        let headers = self.headers;
        let striped = self.striped;
        let highlight_on_hover = self.highlight_on_hover;
        let with_column_borders = self.with_column_borders;
        let motion = self.motion;
        let max_height_px = self
            .max_height_px
            .map(|value| value.max(f32::from(tokens.min_viewport_height)));
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
        let state = TableState::resolve(TableStateInput {
            id: &table_id,
            total_rows,
            page_size,
            page_size_options: self.page_size_options.clone(),
            pagination_enabled,
            page_controlled,
            page: self.page,
            default_page: self.default_page,
            max_height_px,
            sticky_header,
            has_headers: !headers.is_empty(),
            virtual_window,
            auto_virtualization: self.auto_virtualization,
            virtualization_min_rows: self.virtualization_min_rows,
            virtualization_overscan_rows: self.virtualization_overscan_rows,
            virtual_row_height_px: self.virtual_row_height_px,
            default_row_height_px: Self::default_row_height_px(table_size_preset),
            line_thickness_px,
        });
        let page_size_options = state.page_size_options.clone();
        let page_count = state.page_count;
        let resolved_page = state.resolved_page;
        let resolved_page_size = state.resolved_page_size;
        let resolved_scroll_height = state.resolved_scroll_height;
        let auto_virtualization_enabled = state.auto_virtualization_enabled;
        let row_extent = state.row_extent;
        let scroll_y = state.scroll_y;
        let max_scroll_y = state.max_scroll_y;
        let window_start = state.window_start;
        let rows = rows_with_meta
            .into_iter()
            .skip(state.window_start.min(total_rows))
            .take(state.window_count.max(1))
            .map(|(source_index, _, row)| (source_index, row))
            .collect::<Vec<_>>();

        let mut root = Stack::vertical()
            .id(table_id.clone())
            .w_full()
            .gap(tokens.row_gap)
            .bg(resolve_hsla(&self.theme, &tokens.row_bg));

        if self.with_outer_border {
            root = root
                .border(line_thickness)
                .border_color(resolve_hsla(&self.theme, &tokens.row_border));
            root = apply_radius(&self.theme, root, self.radius);
        }

        if let Some(caption) = caption {
            let caption = Self::apply_cell_size(
                table_size_preset,
                div()
                    .id(table_id.slot("caption"))
                    .text_size(tokens.caption_size)
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
                .id(table_id.slot("header"))
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
                    table_size_preset,
                    div()
                        .id(table_id.slot_index("header-cell", index.to_string()))
                        .flex_1()
                        .min_w_0()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .truncate()
                        .child(text),
                );
                header_row = header_row.child(cell);
            }

            header_row_any = Some(header_row);
        }

        let visible_row_count = rows.len();
        let has_rows = visible_row_count > 0;
        let top_spacer_height = state.top_spacer_height();
        let bottom_spacer_height = state.bottom_spacer_height(total_rows, visible_row_count);
        let mut rows_root = Stack::vertical()
            .id(table_id.slot("rows"))
            .w_full()
            .gap(tokens.row_gap);
        if top_spacer_height > 0.0 {
            rows_root = rows_root.child(
                div()
                    .id(table_id.slot("virtual-top-spacer"))
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
                .id(table_id.slot_index("row", row_index.to_string()))
                .w_full()
                .flex()
                .items_center()
                .bg(row_bg)
                .text_color(resolve_hsla(&self.theme, &tokens.cell_fg));

            if let Some(handler) = on_row_click.as_ref() {
                let on_row_click = handler.clone();
                let hover_bg = resolve_hsla(&self.theme, &tokens.row_hover_bg);
                let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                let mut interaction_styles = InteractionStyles::new()
                    .active(interaction_style(move |style| style.bg(press_bg)))
                    .focus(interaction_style(move |style| style.bg(hover_bg)));
                if highlight_on_hover {
                    interaction_styles = interaction_styles
                        .hover(interaction_style(move |style| style.bg(hover_bg)));
                }
                let activate_handler: ActivateHandler =
                    Rc::new(move |window: &mut gpui::Window, cx: &mut gpui::App| {
                        (on_row_click)(source_index, window, cx)
                    });
                row_node = apply_interaction_styles(row_node.cursor_pointer(), interaction_styles);
                row_node = bind_press_adapter(
                    row_node,
                    PressAdapter::new(table_id.slot_index("row", row_index.to_string()))
                        .on_activate(Some(activate_handler)),
                );
            } else if highlight_on_hover {
                let hover_bg = resolve_hsla(&self.theme, &tokens.row_hover_bg);
                row_node = row_node.hover(move |style| style.bg(hover_bg));
            }
            if auto_virtualization_enabled && row_index == 0 {
                let table_id_for_height = table_id.clone();
                row_node = row_node.relative().child(
                    canvas(
                        move |bounds, window, _cx| {
                            let measured = f32::from(bounds.size.height).max(1.0);
                            if table_state::on_row_height_measured(&table_id_for_height, measured) {
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
                    table_size_preset,
                    div()
                        .id(table_id.slot_index("row-cell", format!("{row_index}-{column}")))
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
                    .id(table_id.slot("virtual-bottom-spacer"))
                    .w_full()
                    .h(px(bottom_spacer_height)),
            );
        }

        if !has_rows {
            rows_root = rows_root.child(Self::apply_cell_size(
                table_size_preset,
                div()
                    .id(table_id.slot("empty"))
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
                    .id(table_id.slot_index("page-size-selector", suffix))
                    .flex()
                    .items_center()
                    .gap(tokens.page_chip_gap);
                for option in page_size_options {
                    let is_active = option == resolved_page_size;
                    let mut item = div()
                        .id(table_id.slot_index("page-size", format!("{option}-{suffix}")))
                        .px(tokens.page_chip_padding_x)
                        .py(tokens.page_chip_padding_y)
                        .text_size(tokens.page_chip_size)
                        .rounded(tokens.page_chip_radius)
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
                        let hover_bg = resolve_hsla(&self.theme, &tokens.row_hover_bg);
                        let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                        let focus_ring = resolve_hsla(&self.theme, &self.theme.semantic.focus_ring);
                        let activate_handler: ActivateHandler =
                            Rc::new(move |window: &mut gpui::Window, cx| {
                                table_state::on_page_size_change(&table_id_for_page_size, option);
                                window.refresh();
                                if let Some(handler) = on_page_size_change.as_ref() {
                                    (handler)(option, window, cx);
                                }
                            });
                        item = apply_interaction_styles(
                            item.cursor_pointer(),
                            InteractionStyles::new()
                                .hover(interaction_style(move |style| style.bg(hover_bg)))
                                .active(interaction_style(move |style| style.bg(press_bg)))
                                .focus(interaction_style(move |style| {
                                    style.border_color(focus_ring)
                                })),
                        );
                        item = bind_press_adapter(
                            item,
                            PressAdapter::new(
                                table_id.slot_index("page-size", format!("{option}-{suffix}")),
                            )
                            .on_activate(Some(activate_handler)),
                        );
                    } else {
                        item = item.cursor_default();
                    }
                    items = items.child(item);
                }
                Some(items)
            } else {
                None
            };

            let mut right = div()
                .flex()
                .items_center()
                .gap(tokens.pagination_items_gap)
                .child(
                    Pagination::new()
                        .with_id(table_id.slot_index("pagination", suffix))
                        .total(page_count)
                        .value(resolved_page)
                        .siblings(self.pagination_siblings)
                        .boundaries(self.pagination_boundaries)
                        .on_change(move |next_page: usize, window: &mut gpui::Window, cx| {
                            if table_state::on_page_change(
                                &table_id_for_page,
                                page_controlled,
                                next_page,
                            ) {
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
                .id(table_id.slot_index("pagination-bar", suffix))
                .w_full()
                .px(tokens.pagination_padding_x)
                .py(tokens.pagination_padding_y)
                .flex()
                .items_center()
                .justify_between()
                .gap(tokens.pagination_gap)
                .bg(resolve_hsla(&self.theme, &tokens.row_bg))
                .child(
                    div()
                        .text_size(tokens.pagination_summary_size)
                        .text_color(resolve_hsla(&self.theme, &tokens.caption))
                        .child(page_summary),
                )
                .child(right)
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
                .id(table_id.slot("scroll-content"))
                .w_full()
                .gap(tokens.row_gap);
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
                        .id(table_id.slot("virtual-scroll"))
                        .relative()
                        .w_full()
                        .h(px(scroll_height))
                        .overflow_y_scroll()
                        .track_scroll(&scroll_handle)
                        .p(tokens.virtualization_padding)
                        .child(scroll_content)
                        .child(
                            canvas(
                                move |_bounds, window, _cx| {
                                    let next_y = (-f32::from(handle_for_monitor.offset().y))
                                        .clamp(0.0, max_scroll_for_monitor);
                                    if table_state::on_virtual_scroll(
                                        &table_id_for_scroll,
                                        next_y,
                                        row_extent_for_monitor,
                                        overscan_rows,
                                    ) {
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
                        .with_id(table_id.slot("scroll"))
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

        root.with_enter_transition(table_id.slot("enter"), motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for Table {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_sized_via_method!(Table);
crate::impl_radiused_via_method!(Table);

impl gpui::Styled for Table {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
