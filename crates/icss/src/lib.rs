//! # ICSS — a CSS-like theme engine for [iced](https://github.com/iced-rs/iced)
//!
//! ICSS lets you style an iced application the way you'd style a web app:
//! write a `.icss` file with classes, variables, and pseudo-states, then
//! attach class lists to your widgets.
//!
//! ```rust,ignore
//! let theme = icss::Theme::load(include_str!("theme.icss"))?;
//! button("Connect").style(theme.button(&["button", "primary"]));
//! ```
//!
//! The crate is organised into three modules:
//!
//! - [`theme`] — parses `.icss` and resolves widget styles for iced.
//! - [`engine`] — generates a complete `.icss` theme from a handful of base
//!   variables (OKLCH tonal palettes, dimensional tokens, semantic mapping).
//! - [`widgets`] — theme-aware iced widgets that don't ship with iced core
//!   (`DataTable`, `TileGrid`, `ButtonGroup`, `StickySection`, …).

pub mod engine;
pub mod theme;
pub mod widgets;

// Convenience re-exports for the most common entry points.
pub use engine::{ThemeInputs, ThemeOutput, generate};
pub use theme::{ParseError, Stylesheet, Theme, parse_stylesheet};
