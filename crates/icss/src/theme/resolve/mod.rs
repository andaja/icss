//! Resolution engine — converts CSS rules into iced widget styles.
//!
//! This module depends on iced and is gated behind the `iced-resolve` feature.

pub mod computed;

pub mod button;
pub mod checkbox;
pub mod column;
pub mod container;
pub mod frame;
pub mod menu;
pub mod pick_list;
pub mod progress_bar;
pub mod radio;
pub mod row;
pub mod rule;
pub mod scrollable;
pub mod slider;
pub mod table;
pub mod text;
pub mod text_editor;
pub mod text_input;
pub mod toggler;
pub mod tooltip;

pub mod sizing;
pub use sizing::ComponentSize;

pub mod builders;

mod theme;
pub use theme::Theme;
