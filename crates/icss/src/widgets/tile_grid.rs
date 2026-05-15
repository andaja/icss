//! Responsive tile grid widget.
//!
//! A grid of interactive card tiles that supports three layout modes:
//! - **Flow**: responsive wrapping grid (uses iced's `Grid` with `fluid()`)
//! - **Horizontal**: single-row horizontal scroll
//! - **Vertical**: single-column vertical scroll
//!
//! Each tile is a styled button with native hover highlight, selection, and click.
//! Hover is handled by iced's button widget in the draw pass (no messages needed).
//! Styling comes from the ICSS theme (`.tile`, `.tile:hover`, `.tile.selected`).

use std::collections::HashSet;

use crate::theme::Theme;
use iced::widget::{Grid, button, column, row, scrollable};
use iced::{Element, Length, Padding};

/// Layout mode for the tile grid.
#[derive(Debug, Clone)]
pub enum TileLayout {
    /// Responsive wrapping grid with a minimum tile width.
    Flow { min_tile_width: f32 },
    /// Single-row horizontal scroll.
    Horizontal,
    /// Single-column vertical scroll.
    Vertical,
}

impl Default for TileLayout {
    fn default() -> Self {
        Self::Flow {
            min_tile_width: 200.0,
        }
    }
}

/// A single tile in the grid.
pub struct TileItem<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Message>,
}

/// A responsive grid of interactive card tiles.
///
/// Hover highlighting is handled natively by iced's button widget
/// (no messages, no re-renders during scroll).
///
/// # Usage
///
/// ```rust,ignore
/// TileGrid::new()
///     .layout(TileLayout::Flow { min_tile_width: 240.0 })
///     .spacing(12.0)
///     .selected(&state.selected_tiles)
///     .push(tile_content, MyMessage::TilePressed(0))
///     .push(tile_content2, MyMessage::TilePressed(1))
///     .view(&theme)
/// ```
pub struct TileGrid<'a, Message> {
    tiles: Vec<TileItem<'a, Message>>,
    layout: TileLayout,
    spacing: f32,
    padding: f32,
    selected: HashSet<usize>,
}

impl<'a, Message: Clone + 'a> Default for TileGrid<'a, Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message: Clone + 'a> TileGrid<'a, Message> {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            layout: TileLayout::default(),
            spacing: 12.0,
            padding: 16.0,
            selected: HashSet::new(),
        }
    }

    /// Set the layout mode.
    pub fn layout(mut self, layout: TileLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set spacing between tiles (default: 12.0).
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding inside each tile card (default: 16.0).
    pub fn tile_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set the selected tile indices.
    pub fn selected(mut self, indices: &HashSet<usize>) -> Self {
        self.selected = indices.clone();
        self
    }

    /// Add a tile with content and an on-press message.
    pub fn push(mut self, content: impl Into<Element<'a, Message>>, on_press: Message) -> Self {
        self.tiles.push(TileItem {
            content: content.into(),
            on_press: Some(on_press),
        });
        self
    }

    /// Add a tile with content and no press handler.
    pub fn push_passive(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.tiles.push(TileItem {
            content: content.into(),
            on_press: None,
        });
        self
    }

    /// Build the final `Element`.
    ///
    /// Hover is handled natively by iced's button in the draw pass.
    pub fn view(self, theme: &'a Theme) -> Element<'a, Message> {
        let TileGrid {
            tiles,
            layout,
            spacing,
            padding,
            selected,
        } = self;

        let pad = Padding::from(padding);

        let elements: Vec<Element<'a, Message>> = tiles
            .into_iter()
            .enumerate()
            .map(|(i, tile)| {
                let is_selected = selected.contains(&i);

                let classes: &[&str] = if is_selected {
                    &["tile", "selected"]
                } else {
                    &["tile"]
                };

                let mut btn = button(tile.content)
                    .padding(pad)
                    .width(Length::Fill)
                    .style(theme.button(classes));

                if let Some(msg) = tile.on_press {
                    btn = btn.on_press(msg);
                }

                btn.into()
            })
            .collect();

        let is_horizontal = matches!(layout, TileLayout::Horizontal);

        let inner: Element<'a, Message> = match layout {
            TileLayout::Flow { min_tile_width } => Grid::with_children(elements)
                .fluid(min_tile_width)
                .spacing(spacing)
                .height(Length::Shrink)
                .into(),

            TileLayout::Horizontal => {
                let r = elements
                    .into_iter()
                    .fold(row![], |r, tile| r.push(tile))
                    .spacing(spacing);
                r.into()
            }

            TileLayout::Vertical => {
                let c = elements
                    .into_iter()
                    .fold(column![], |c, tile| c.push(tile))
                    .spacing(spacing);
                c.into()
            }
        };

        if is_horizontal {
            scrollable(inner)
                .style(theme.scrollable(&["scroll"]))
                .horizontal()
                .into()
        } else {
            scrollable(inner)
                .style(theme.scrollable(&["scroll"]))
                .into()
        }
    }
}
