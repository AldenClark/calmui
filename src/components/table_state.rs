use super::control;

pub struct TableStateInput<'a> {
    pub id: &'a str,
    pub total_rows: usize,
    pub page_size: usize,
    pub page_size_options: Vec<usize>,
    pub pagination_enabled: bool,
    pub page_controlled: bool,
    pub page: Option<usize>,
    pub default_page: usize,
    pub max_height_px: Option<f32>,
    pub sticky_header: bool,
    pub has_headers: bool,
    pub virtual_window: Option<(usize, usize)>,
    pub auto_virtualization: bool,
    pub virtualization_min_rows: usize,
    pub virtualization_overscan_rows: usize,
    pub virtual_row_height_px: Option<f32>,
    pub default_row_height_px: f32,
    pub line_thickness_px: f32,
}

#[derive(Clone, Debug)]
pub struct TableState {
    pub page_size_options: Vec<usize>,
    pub resolved_page_size: usize,
    pub page_count: usize,
    pub resolved_page: usize,
    pub resolved_scroll_height: Option<f32>,
    pub auto_virtualization_enabled: bool,
    pub row_extent: f32,
    pub scroll_y: f32,
    pub max_scroll_y: f32,
    pub window_start: usize,
    pub window_count: usize,
}

impl TableState {
    pub fn resolve(input: TableStateInput<'_>) -> Self {
        let mut page_size_options = input
            .page_size_options
            .into_iter()
            .map(|value| value.max(1))
            .collect::<Vec<_>>();
        page_size_options.sort_unstable();
        page_size_options.dedup();
        if page_size_options.is_empty() {
            page_size_options.push(input.page_size);
        }

        let resolved_page_size = if input.pagination_enabled {
            let size = control::usize_state(input.id, "page-size", None, input.page_size).max(1);
            if !page_size_options.contains(&size) {
                page_size_options.push(size);
                page_size_options.sort_unstable();
                page_size_options.dedup();
            }
            size
        } else {
            input.page_size
        };

        let page_count = if input.pagination_enabled {
            ((input.total_rows + resolved_page_size.saturating_sub(1)) / resolved_page_size).max(1)
        } else {
            1
        };

        let resolved_page = if input.pagination_enabled {
            let controlled_page = input
                .page
                .unwrap_or(input.default_page)
                .clamp(1, page_count);
            control::usize_state(
                input.id,
                "page",
                input.page_controlled.then_some(controlled_page),
                input.default_page.clamp(1, page_count),
            )
            .clamp(1, page_count)
        } else {
            1
        };

        let resolved_scroll_height = input.max_height_px.map(|max_height| {
            if input.sticky_header && input.has_headers {
                (max_height - 42.0).max(40.0)
            } else {
                max_height
            }
        });

        let auto_virtualization_enabled = input.auto_virtualization
            && !input.pagination_enabled
            && input.virtual_window.is_none()
            && resolved_scroll_height.is_some()
            && input.total_rows >= input.virtualization_min_rows;

        let default_row_extent = input.default_row_height_px + input.line_thickness_px;
        let measured_row_height = control::f32_state(
            input.id,
            "virtual-row-height",
            None,
            input.default_row_height_px,
        )
        .max(1.0);
        let row_extent = if auto_virtualization_enabled {
            (input.virtual_row_height_px.unwrap_or(measured_row_height) + input.line_thickness_px)
                .max(1.0)
        } else {
            default_row_extent
        };

        let scroll_height_for_virtual = resolved_scroll_height.unwrap_or(0.0);
        let max_scroll_y = if auto_virtualization_enabled {
            ((input.total_rows as f32 * row_extent)
                - input.line_thickness_px
                - scroll_height_for_virtual)
                .max(0.0)
        } else {
            0.0
        };

        let scroll_y = if auto_virtualization_enabled {
            control::f32_state(input.id, "virtual-scroll-y", None, 0.0).clamp(0.0, max_scroll_y)
        } else {
            0.0
        };

        let (window_start, window_count) = if input.pagination_enabled {
            (
                (resolved_page - 1) * resolved_page_size,
                resolved_page_size.max(1),
            )
        } else if auto_virtualization_enabled {
            let overscan = input.virtualization_overscan_rows.max(1);
            let visible_count = ((scroll_height_for_virtual / row_extent).ceil() as usize)
                .saturating_add(overscan.saturating_mul(2))
                .saturating_add(2)
                .max(1);
            let start = ((scroll_y / row_extent).floor() as usize).saturating_sub(overscan);
            (start, visible_count)
        } else {
            input
                .virtual_window
                .map(|(start, count)| (start, count.max(1)))
                .unwrap_or((0, input.total_rows.saturating_add(1)))
        };

        if auto_virtualization_enabled {
            control::set_usize_state(input.id, "virtual-window-start", window_start);
            control::set_f32_state(input.id, "virtual-scroll-y", scroll_y);
        }

        Self {
            page_size_options,
            resolved_page_size,
            page_count,
            resolved_page,
            resolved_scroll_height,
            auto_virtualization_enabled,
            row_extent,
            scroll_y,
            max_scroll_y,
            window_start,
            window_count,
        }
    }

    pub fn top_spacer_height(&self) -> f32 {
        if self.auto_virtualization_enabled {
            (self.window_start as f32 * self.row_extent).max(0.0)
        } else {
            0.0
        }
    }

    pub fn bottom_spacer_height(&self, total_rows: usize, visible_rows: usize) -> f32 {
        if self.auto_virtualization_enabled {
            let remaining_rows =
                total_rows.saturating_sub(self.window_start.saturating_add(visible_rows));
            (remaining_rows as f32 * self.row_extent).max(0.0)
        } else {
            0.0
        }
    }
}

pub fn on_page_change(id: &str, page_controlled: bool, next_page: usize) -> bool {
    if page_controlled {
        return false;
    }
    control::set_usize_state(id, "page", next_page.max(1));
    true
}

pub fn on_page_size_change(id: &str, next_page_size: usize) {
    control::set_usize_state(id, "page-size", next_page_size.max(1));
    control::set_usize_state(id, "page", 1);
}

pub fn on_row_height_measured(id: &str, measured: f32) -> bool {
    let measured = measured.max(1.0);
    let previous = control::f32_state(id, "virtual-row-height", None, 0.0);
    if (measured - previous).abs() > 0.5 {
        control::set_f32_state(id, "virtual-row-height", measured);
        true
    } else {
        false
    }
}

pub fn on_virtual_scroll(id: &str, next_y: f32, row_extent: f32, overscan_rows: usize) -> bool {
    let mut should_refresh = false;
    let current_y = control::f32_state(id, "virtual-scroll-y", None, 0.0);
    if (next_y - current_y).abs() > 0.5 {
        control::set_f32_state(id, "virtual-scroll-y", next_y);
    }

    let next_start =
        ((next_y / row_extent.max(1.0)).floor() as usize).saturating_sub(overscan_rows);
    let prev_start = control::usize_state(id, "virtual-window-start", None, 0);
    if next_start != prev_start {
        control::set_usize_state(id, "virtual-window-start", next_start);
        should_refresh = true;
    }

    should_refresh
}
