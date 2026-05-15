//! Component sizing — extracts layout properties from ICSS computed styles.
//!
//! Bridges the gap between ICSS (which can define `padding-v`, `padding-h`,
//! `font-size`, `min-width`, `min-height`, `gap`) and iced widgets (which set
//! these as widget attributes, not part of the style closure).
//!
//! # ICSS conventions
//!
//! ```css
//! .sz-md { padding-v: 8px; padding-h: 16px; font-size: 16px; min-width: 100px; min-height: 36px; gap: 6px; }
//! .sz-sm { padding-v: 6px; padding-h: 12px; font-size: 14px; min-width: 80px; min-height: 30px; gap: 6px; }
//! ```
//!
//! # Rust usage
//!
//! ```rust,ignore
//! let sz = theme.sizing(&["sz-md"]);
//!
//! // Use protect() from icss::widgets to wrap with MinSize enforcement:
//! icss::widgets::protect(&sz,
//!     button(text("OK").size(sz.font_size))
//!         .padding(sz.padding())
//!         .style(theme.button(&["button", "primary"]))
//! )
//! ```

use iced::{Length, Padding};

use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

/// Layout sizing extracted from ICSS computed style.
#[derive(Debug, Clone, Copy)]
pub struct ComponentSize {
    pub pad_v: f32,
    pub pad_h: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub min_w: Option<f32>,
    pub min_h: Option<f32>,
    pub gap: f32,
}

impl ComponentSize {
    /// Build from a computed style, falling back to sensible defaults.
    pub fn from_computed(computed: &ComputedStyle) -> Self {
        let font_size = computed.length("font-size").unwrap_or(16.0);
        Self {
            pad_v: computed.length("padding-v").unwrap_or(8.0),
            pad_h: computed.length("padding-h").unwrap_or(16.0),
            font_size,
            icon_size: computed.length("icon-size").unwrap_or(font_size),
            min_w: computed.length("min-width"),
            min_h: computed.length("min-height"),
            gap: computed.length("gap").unwrap_or(6.0),
        }
    }

    /// Return iced `Padding` — symmetric, no fractional offsets.
    ///
    /// Content sits at `pad_v` from top and bottom, `pad_h` left and right.
    /// All values come from DimTokens which are rounded to integers, so
    /// every edge lands on the pixel grid.
    pub fn padding(&self) -> Padding {
        Padding {
            top: self.pad_v,
            right: self.pad_h,
            bottom: self.pad_v,
            left: self.pad_h,
        }
    }

    /// Return symmetric `Padding` — for icon-only or non-text widgets that
    /// don't have the baseline asymmetry. Use this on icon-only square
    /// buttons, color swatches, etc.
    pub fn padding_symmetric(&self) -> Padding {
        Padding::from([self.pad_v, self.pad_h])
    }

    /// Return iced `Length` for width: `Fixed(min_w)` if set, else `Shrink`.
    pub fn min_width(&self) -> Length {
        match self.min_w {
            Some(w) => Length::Fixed(w),
            None => Length::Shrink,
        }
    }

    /// Return iced `Length` for height: `Fixed(min_h)` if set, else `Shrink`.
    pub fn min_height(&self) -> Length {
        match self.min_h {
            Some(h) => Length::Fixed(h),
            None => Length::Shrink,
        }
    }
}

impl Theme {
    /// Resolve component sizing from ICSS classes.
    ///
    /// ```rust,ignore
    /// let sz = theme.sizing(&["sz-md"]);
    /// button(content).padding(sz.padding()).width(sz.min_width())
    /// ```
    pub fn sizing(&self, classes: &[&str]) -> ComponentSize {
        let computed = self.resolve(classes, None);
        ComponentSize::from_computed(&computed)
    }
}
