//! Full-featured data table widget.
//!
//! Composes iced primitives to provide:
//! - Sticky header (stays visible during body scroll, leaves with last row)
//! - Horizontal scrolling when container is narrower than `min_width`
//! - Frozen left columns (stay fixed during horizontal scroll via Stack overlay)
//! - Sortable columns with direction indicators
//! - Row selection (click or checkbox, with select-all)
//! - Row hover highlighting (native via button, no messages)
//! - Horizontal separators between rows
//! - Pagination with page size selector
//! - Quick search input
//! - Variable-height rows (multi-line content supported)
//!
//! Styled via ICSS theme classes (`.table-header`, `.table-row`, etc.).
//!
//! Hover is handled by iced's button widget in the draw pass — no messages
//! are fired during scroll, eliminating the jitter caused by message-based
//! hover tracking.

use std::collections::HashSet;

use crate::theme::Theme;
use iced::alignment::Horizontal;
use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, rule, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length, Padding};

use crate::widgets::h_overflow::HOverflow;
use crate::widgets::min_width::MinWidth;

// ── Column definition ──────────────────────────────────────────────

/// Definition of a single column in the data table.
pub struct DataColumn<'a, T, Message> {
    header: String,
    width: Length,
    /// Minimum column width — wraps the cell in [`MinWidth`] when set.
    col_min_width: Option<f32>,
    cell: Box<dyn Fn(&T, usize) -> Element<'a, Message> + 'a>,
    sort_key: Option<String>,
    align_x: Horizontal,
}

impl<'a, T, Message> DataColumn<'a, T, Message> {
    /// Create a column with a header label and a cell renderer function.
    ///
    /// The cell function receives `(row_data, row_index)` and returns an Element.
    /// Since the Element lifetime is independent of the row reference, clone or
    /// format data in the closure rather than borrowing from the row.
    pub fn new(
        header: impl Into<String>,
        cell: impl Fn(&T, usize) -> Element<'a, Message> + 'a,
    ) -> Self {
        Self {
            header: header.into(),
            width: Length::Fill,
            col_min_width: None,
            cell: Box::new(cell),
            sort_key: None,
            align_x: Horizontal::Left,
        }
    }

    /// Set the column width (default: `Fill`).
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Set a minimum column width. The column still fills available space
    /// (when width is `Fill`) but never shrinks below this value.
    pub fn col_min_width(mut self, min: f32) -> Self {
        self.col_min_width = Some(min);
        self
    }

    /// Make this column sortable with the given key.
    pub fn sortable(mut self, key: impl Into<String>) -> Self {
        self.sort_key = Some(key.into());
        self
    }

    /// Set horizontal alignment for cell content (default: `Left`).
    pub fn align(mut self, align: Horizontal) -> Self {
        self.align_x = align;
        self
    }
}

// ── Sort state ─────────────────────────────────────────────────────

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    pub fn arrow(self) -> &'static str {
        match self {
            Self::Ascending => " \u{25B2}",
            Self::Descending => " \u{25BC}",
        }
    }
}

/// Current sort state for the table.
#[derive(Debug, Clone)]
pub struct SortState {
    pub key: String,
    pub direction: SortDirection,
}

// ── DataTable builder ──────────────────────────────────────────────

const PAGE_SIZES: &[usize] = &[10, 25, 50, 100];
const SELECT_COL_WIDTH: f32 = 40.0;

/// A full-featured data table, generic over row type `T`.
///
/// All state (selection, sort, pagination, search) is owned by the
/// caller and passed in. The DataTable is a pure view builder.
///
/// ## Sticky header
///
/// The header row sits outside the body scroll area, staying visible
/// as long as the table is in the viewport. It scrolls away only when
/// the entire table leaves view.
///
/// ## Horizontal scroll
///
/// Set `.min_width(px)` to enable horizontal scrolling. When the
/// container is narrower than `min_width`, a horizontal scrollbar
/// appears. Header and body scroll together (same scrollable).
///
/// ## Frozen columns
///
/// Set `.frozen_columns(n)` to pin the first `n` data columns (plus
/// the select column if enabled) on the left. Uses a Stack overlay.
/// Works in pagination mode (no body_height). The frozen column and
/// the main table use the same cell closures, so row content matches.
///
/// **Note:** if a non-frozen column produces taller content than the
/// frozen columns for a given row, the overlay may not perfectly align.
/// Keep the most descriptive (tallest) columns in the frozen set.
pub struct DataTable<'a, T, Message> {
    columns: Vec<DataColumn<'a, T, Message>>,
    rows: &'a [T],
    selected: Option<&'a HashSet<usize>>,
    sort: Option<&'a SortState>,
    page: usize,
    page_size: usize,
    show_select_column: bool,
    show_separators: bool,
    search_query: &'a str,
    /// Minimum total table width — enables horizontal scroll.
    min_width: Option<f32>,
    /// Number of left data columns to freeze during horizontal scroll.
    frozen_columns: usize,
    /// Fixed body height — enables inner vertical scroll. When `None`,
    /// pagination controls visible rows and no inner scroll is used.
    body_height: Option<f32>,
}

impl<'a, T, Message: Clone + 'a> DataTable<'a, T, Message> {
    /// Create a data table with column definitions and a row slice.
    ///
    /// The caller is responsible for sorting/filtering the rows before
    /// passing them in. The DataTable handles pagination over the given slice.
    pub fn new(columns: Vec<DataColumn<'a, T, Message>>, rows: &'a [T]) -> Self {
        Self {
            columns,
            rows,
            selected: None,
            sort: None,
            page: 0,
            page_size: 25,
            show_select_column: false,
            show_separators: true,
            search_query: "",
            min_width: None,
            frozen_columns: 0,
            body_height: None,
        }
    }

    /// Set selected row indices.
    pub fn selected(mut self, indices: &'a HashSet<usize>) -> Self {
        self.selected = Some(indices);
        self
    }

    /// Set the current sort state (for displaying sort indicators).
    pub fn sort_state(mut self, state: &'a SortState) -> Self {
        self.sort = Some(state);
        self
    }

    /// Set the current page (0-indexed).
    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    /// Set items per page (default: 25).
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size;
        self
    }

    /// Show a checkbox column for row selection (default: false).
    pub fn select_column(mut self, show: bool) -> Self {
        self.show_select_column = show;
        self
    }

    /// Show horizontal separators between rows (default: true).
    pub fn separators(mut self, show: bool) -> Self {
        self.show_separators = show;
        self
    }

    /// Set the search query (displayed in the search input).
    pub fn search(mut self, query: &'a str) -> Self {
        self.search_query = query;
        self
    }

    /// Set minimum total table width. Enables horizontal scrolling when
    /// the container is narrower than this value.
    pub fn min_width(mut self, min: f32) -> Self {
        self.min_width = Some(min);
        self
    }

    /// Freeze the first `n` data columns on the left during horizontal
    /// scroll. The select column (if enabled) is always frozen.
    /// Only active in pagination mode (no `body_height`).
    pub fn frozen_columns(mut self, n: usize) -> Self {
        self.frozen_columns = n;
        self
    }

    /// Set a fixed body height with inner vertical scrolling.
    /// When set, frozen columns are disabled (would require scroll sync).
    pub fn body_height(mut self, h: f32) -> Self {
        self.body_height = Some(h);
        self
    }

    fn is_selected(&self, idx: usize) -> bool {
        self.selected.is_some_and(|s| s.contains(&idx))
    }

    /// Resolve the effective minimum total content width.
    ///
    /// Uses an explicit `min_width()` override when set; otherwise sums each
    /// column's `Fixed` width (or `col_min_width` for `Fill` columns) plus
    /// the select column when enabled. Columns with neither contribute 0.
    /// Minimum width for HOverflow wrapping. Public so callers of
    /// `view_split` can wrap header+body in a shared HOverflow.
    pub fn overflow_min_width(&self) -> f32 {
        self.effective_min_width()
    }

    fn effective_min_width(&self) -> f32 {
        if let Some(m) = self.min_width {
            return m;
        }
        let mut total = 0.0;
        if self.show_select_column {
            total += SELECT_COL_WIDTH;
        }
        for col in &self.columns {
            total += match col.width {
                Length::Fixed(px) => px,
                _ => col.col_min_width.unwrap_or(0.0),
            };
        }
        total
    }

    /// Build as two separate elements: `(header, body_with_footer)`.
    ///
    /// Use this with [`StickySection`] to pin the header at the viewport top:
    /// ```rust,ignore
    /// let (header, body) = table.view_split(theme, ...);
    /// StickySection::new(header, body)
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn view_split(
        self,
        theme: &'a Theme,
        on_row_press: impl Fn(usize) -> Message + 'a,
        on_sort: impl Fn(String) -> Message + 'a,
        on_select: impl Fn(usize) -> Message + 'a,
        on_select_all: Message,
        on_page_change: impl Fn(usize) -> Message + 'a,
        on_page_size_change: impl Fn(usize) -> Message + 'a,
        on_search: impl Fn(String) -> Message + 'a,
    ) -> (
        Element<'a, Message>,
        Element<'a, Message>,
        Element<'a, Message>,
    ) {
        let total_rows = self.rows.len();
        let total_pages = if total_rows == 0 {
            1
        } else {
            total_rows.div_ceil(self.page_size)
        };
        let page = self.page.min(total_pages.saturating_sub(1));
        let start = page * self.page_size;
        let end = (start + self.page_size).min(total_rows);
        let page_rows = &self.rows[start..end];
        // Rows use Fill — table stretches to 100%. MinWidth on cells
        // enforces per-column floors. Caller handles h-scroll if needed.
        let content_width = Length::Fill;

        let min_w = self.effective_min_width();

        let header_el = self.build_header(
            theme,
            &on_sort,
            &on_select_all,
            0..self.columns.len(),
            self.show_select_column,
            content_width,
        );

        let body_el = self.build_body(
            theme,
            page_rows,
            start,
            &on_row_press,
            &on_select,
            0..self.columns.len(),
            self.show_select_column,
            content_width,
        );

        // Wrap header and body in their own HOverflow regions so each
        // clips and scrolls horizontally when the viewport is narrower
        // than the natural minimum width.
        //
        // NOTE: header and body have independent scroll offsets in this
        // mode (StickySection composition prevents sharing a single
        // HOverflow). Callers expecting locked horizontal scroll between
        // a sticky header and body should use `view()` instead.
        let (header_el, body_el): (Element<'a, Message>, Element<'a, Message>) = if min_w > 0.0 {
            (
                HOverflow::new(header_el, min_w).into(),
                HOverflow::new(body_el, min_w).into(),
            )
        } else {
            (header_el, body_el)
        };

        let divider = || rule::horizontal(1).style(theme.rule(&["divider"]));

        let footer = {
            let showing = if total_rows == 0 {
                text("No rows").size(12)
            } else {
                text(format!("{}\u{2013}{} of {}", start + 1, end, total_rows)).size(12)
            };
            let size_options: Vec<String> = PAGE_SIZES.iter().map(|s| s.to_string()).collect();
            let current_size = self.page_size.to_string();
            let page_size_picker =
                pick_list(size_options, Some(current_size), move |val: String| {
                    on_page_size_change(val.parse::<usize>().unwrap_or(25))
                })
                .text_size(12)
                .style(theme.pick_list(&["select"]))
                .menu_style(theme.menu(&["select-menu"]));

            let mut prev_btn = button(text("<").size(12)).style(theme.button(&["button", "ghost"]));
            if page > 0 {
                prev_btn = prev_btn.on_press(on_page_change(page - 1));
            }
            let mut next_btn = button(text(">").size(12)).style(theme.button(&["button", "ghost"]));
            if page + 1 < total_pages {
                next_btn = next_btn.on_press(on_page_change(page + 1));
            }

            container(
                row![
                    showing,
                    Space::new().width(Length::Fill),
                    page_size_picker,
                    row![
                        prev_btn,
                        text(format!("Page {} / {}", page + 1, total_pages)).size(12),
                        next_btn
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            )
            .style(theme.container(&["table-footer"]))
            .padding([8, 12])
            .width(Length::Fill)
        };

        let _ = on_search;

        let footer_el: Element<'a, Message> = column![divider(), footer]
            .width(Length::Fill)
            .spacing(0)
            .into();

        (header_el, body_el, footer_el)
    }

    /// Build the final `Element`.
    ///
    /// Hover highlighting is handled natively by iced's button widget —
    /// no messages are fired during scroll or mouse movement.
    #[allow(clippy::too_many_arguments)]
    pub fn view(
        self,
        theme: &'a Theme,
        on_row_press: impl Fn(usize) -> Message + 'a,
        on_sort: impl Fn(String) -> Message + 'a,
        on_select: impl Fn(usize) -> Message + 'a,
        on_select_all: Message,
        on_page_change: impl Fn(usize) -> Message + 'a,
        on_page_size_change: impl Fn(usize) -> Message + 'a,
        on_search: impl Fn(String) -> Message + 'a,
    ) -> Element<'a, Message> {
        let total_rows = self.rows.len();
        let total_pages = if total_rows == 0 {
            1
        } else {
            total_rows.div_ceil(self.page_size)
        };
        let page = self.page.min(total_pages.saturating_sub(1));
        let start = page * self.page_size;
        let end = (start + self.page_size).min(total_rows);
        let page_rows = &self.rows[start..end];

        // Table always uses Fill — stretches to 100%. MinWidth on cells
        // enforces per-column floors. Caller handles h-scroll if needed.
        let content_width = Length::Fill;

        // ── Search bar ──
        let search_bar = container(
            text_input("Search...", self.search_query)
                .on_input(on_search)
                .padding(6)
                .style(theme.text_input(&["input"])),
        )
        .padding([0, 8]);

        // ── Build FULL header row (all columns) ──
        let full_header = self.build_header(
            theme,
            &on_sort,
            &on_select_all,
            0..self.columns.len(),
            self.show_select_column,
            content_width,
        );

        // ── Build FULL body (all columns) ──
        let full_body = self.build_body(
            theme,
            page_rows,
            start,
            &on_row_press,
            &on_select,
            0..self.columns.len(),
            self.show_select_column,
            content_width,
        );

        // ── Body wrapping ──
        let body_area: Element<'a, Message> = if let Some(h) = self.body_height {
            scrollable(full_body)
                .style(theme.scrollable(&["scroll"]))
                .height(Length::Fixed(h))
                .into()
        } else {
            // No inner scroll — body flows in page. Sticky header is
            // the caller's responsibility (Stack overlay on the page).
            full_body
        };

        // ── Footer ──
        let footer = {
            let showing = if total_rows == 0 {
                text("No rows").size(12)
            } else {
                text(format!("{}\u{2013}{} of {}", start + 1, end, total_rows)).size(12)
            };

            let size_options: Vec<String> = PAGE_SIZES.iter().map(|s| s.to_string()).collect();
            let current_size = self.page_size.to_string();
            let page_size_picker =
                pick_list(size_options, Some(current_size), move |val: String| {
                    let size = val.parse::<usize>().unwrap_or(25);
                    on_page_size_change(size)
                })
                .text_size(12)
                .style(theme.pick_list(&["select"]))
                .menu_style(theme.menu(&["select-menu"]));

            let mut prev_btn = button(text("<").size(12)).style(theme.button(&["button", "ghost"]));
            if page > 0 {
                prev_btn = prev_btn.on_press(on_page_change(page - 1));
            }

            let mut next_btn = button(text(">").size(12)).style(theme.button(&["button", "ghost"]));
            if page + 1 < total_pages {
                next_btn = next_btn.on_press(on_page_change(page + 1));
            }

            let page_label = text(format!("Page {} / {}", page + 1, total_pages)).size(12);

            let nav = row![prev_btn, page_label, next_btn]
                .spacing(8)
                .align_y(Alignment::Center);

            container(
                row![
                    showing,
                    Space::new().width(Length::Fill),
                    page_size_picker,
                    nav
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            )
            .style(theme.container(&["table-footer"]))
            .padding([8, 12])
            .width(Length::Fill)
        };

        // ── Assemble ──
        let divider = || rule::horizontal(1).style(theme.rule(&["divider"]));

        let min_w = self.effective_min_width();

        // Header + body share a single HOverflow so they scroll together
        // horizontally as one unit. Search and footer stay outside the
        // overflow region (they don't have columns to clip).
        let table_core: Element<'a, Message> = column![full_header, divider(), body_area]
            .width(Length::Fill)
            .spacing(0)
            .into();

        let table_core: Element<'a, Message> = if min_w > 0.0 {
            HOverflow::new(table_core, min_w).into()
        } else {
            table_core
        };

        column![search_bar, table_core, divider(), footer]
            .width(Length::Fill)
            .spacing(0)
            .into()
    }

    // ── Helpers ─────────────────────────────────────────────────────

    fn build_header(
        &self,
        theme: &'a Theme,
        on_sort: &(impl Fn(String) -> Message + 'a),
        on_select_all: &Message,
        col_range: std::ops::Range<usize>,
        include_select: bool,
        row_width: Length,
    ) -> Element<'a, Message> {
        let mut header_row = row![].spacing(0).width(row_width);

        if include_select && self.show_select_column {
            let total_rows = self.rows.len();
            let total_pages = if total_rows == 0 {
                1
            } else {
                total_rows.div_ceil(self.page_size)
            };
            let page = self.page.min(total_pages.saturating_sub(1));
            let start = page * self.page_size;
            let end = (start + self.page_size).min(total_rows);
            let page_rows = &self.rows[start..end];
            let all_on_page = !page_rows.is_empty() && (start..end).all(|i| self.is_selected(i));
            let sa_msg = on_select_all.clone();
            header_row = header_row.push(
                container(
                    checkbox(all_on_page)
                        .on_toggle(move |_| sa_msg.clone())
                        .style(theme.checkbox(&["checkbox"])),
                )
                .width(Length::Fixed(SELECT_COL_WIDTH))
                .center_y(Length::Shrink)
                .padding([4, 8]),
            );
        }

        for i in col_range {
            let col = &self.columns[i];
            let sort_arrow = self
                .sort
                .filter(|s| col.sort_key.as_deref() == Some(s.key.as_str()))
                .map(|s| s.direction.arrow())
                .unwrap_or("");

            let header_text = text(format!("{}{}", col.header, sort_arrow)).size(13);

            let header_cell = container(header_text)
                .width(col.width)
                .padding([8, 12])
                .align_x(col.align_x);

            // MinWidth must wrap the outermost element (button or cell)
            // so flex sees the minimum, not buried inside a Shrink button.
            if let Some(ref key) = col.sort_key {
                let msg = on_sort(key.clone());
                let btn = button(header_cell)
                    .on_press(msg)
                    .padding(0)
                    .width(col.width)
                    .style(theme.button(&["table-header"]));

                if let Some(min) = col.col_min_width {
                    header_row = header_row.push(MinWidth::new(btn, min));
                } else {
                    header_row = header_row.push(btn);
                }
            } else if let Some(min) = col.col_min_width {
                header_row = header_row.push(MinWidth::new(header_cell, min));
            } else {
                header_row = header_row.push(header_cell);
            }
        }

        container(header_row)
            .style(theme.container(&["table-header"]))
            .width(row_width)
            .into()
    }

    fn build_body(
        &self,
        theme: &'a Theme,
        page_rows: &'a [T],
        start: usize,
        on_row_press: &(impl Fn(usize) -> Message + 'a),
        on_select: &(impl Fn(usize) -> Message + 'a),
        col_range: std::ops::Range<usize>,
        include_select: bool,
        row_width: Length,
    ) -> Element<'a, Message> {
        let mut body_col = column![].spacing(0).width(row_width);

        for (page_idx, row_data) in page_rows.iter().enumerate() {
            let abs_idx = start + page_idx;
            let is_selected = self.is_selected(abs_idx);

            let mut data_row = row![].spacing(0).width(row_width);

            if include_select && self.show_select_column {
                let sel_msg = on_select(abs_idx);
                data_row = data_row.push(
                    container(
                        checkbox(is_selected)
                            .on_toggle(move |_| sel_msg.clone())
                            .style(theme.checkbox(&["checkbox"])),
                    )
                    .width(Length::Fixed(SELECT_COL_WIDTH))
                    .center_y(Length::Shrink)
                    .padding([4, 8]),
                );
            }

            for i in col_range.clone() {
                let col = &self.columns[i];
                let cell_content = (col.cell)(row_data, abs_idx);
                let cell = container(cell_content)
                    .width(col.width)
                    .padding([8, 12])
                    .align_x(col.align_x);

                if let Some(min) = col.col_min_width {
                    data_row = data_row.push(MinWidth::new(cell, min));
                } else {
                    data_row = data_row.push(cell);
                }
            }

            let row_classes: &[&str] = if is_selected {
                &["table-row", "selected"]
            } else {
                &["table-row"]
            };

            body_col = body_col.push(
                button(data_row)
                    .on_press(on_row_press(abs_idx))
                    .padding(Padding::ZERO)
                    .width(row_width)
                    .style(theme.button(row_classes)),
            );

            if self.show_separators && page_idx < page_rows.len() - 1 {
                body_col = body_col.push(rule::horizontal(1).style(theme.rule(&["divider"])));
            }
        }

        body_col.into()
    }
}
