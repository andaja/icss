//! Reusable iced widgets.
//!
//! - [`animation`] — tick-driven animation primitives (fade, slide, easing)
//! - [`tile_grid::TileGrid`] — responsive grid of interactive card tiles
//! - [`data_table::DataTable`] — full-featured sortable/paginated data table

pub mod animation;
pub mod button_group;
pub mod control_group;
pub mod data_table;
pub mod h_overflow;
pub mod icon_input;
pub mod mdi;
pub mod menu;
pub mod min_size;
pub mod min_width;
pub mod sticky_section;
pub mod tab_bar;
pub mod tile_grid;

pub use animation::{AnimKind, Animation, Easing, Edge};
pub use data_table::{DataColumn, DataTable, SortDirection, SortState};
pub use h_overflow::HOverflow;
pub use min_size::MinSize;
pub use min_width::MinWidth;
pub use sticky_section::StickySection;
pub use tab_bar::{Tab, TabBar, TabBarAction, TabBarStyle, TabDragState, TabId};
pub use tile_grid::{TileGrid, TileLayout};

use crate::theme::resolve::sizing::ComponentSize;
use iced::Element;

/// Wrap an element in a [`MinSize`] widget using the minimum dimensions from
/// a [`ComponentSize`]. If neither `min_w` nor `min_h` is set, returns the
/// element unchanged.
///
/// This is the recommended way to protect interactive controls (buttons,
/// inputs, pick-lists) from being squished by flex layout.
///
/// ```rust,ignore
/// let sz = theme.sizing(&["sz-md"]);
/// protect(&sz,
///     button(text("OK").size(sz.font_size))
///         .padding(sz.padding())
///         .style(theme.button(&["button", "primary"]))
/// )
/// ```
pub fn protect<'a, M: 'a>(
    sz: &ComponentSize,
    element: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    let min_w = sz.min_w.unwrap_or(0.0);
    let min_h = sz.min_h.unwrap_or(0.0);
    if min_w > 0.0 || min_h > 0.0 {
        MinSize::new(element, min_w, min_h).into()
    } else {
        element.into()
    }
}
