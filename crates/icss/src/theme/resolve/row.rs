//! Row resolver — returns an `iced::widget::Row` preconfigured with gap.
//!
//! ```rust,ignore
//! theme.row(&["row"])              // space_100 gap
//! theme.row(&["row-tight"])        // space_75 gap
//! theme.row(&["cluster"])          // space_50 gap (inline cluster)
//! ```
//!
//! Pulls `gap` (maps to `Row::spacing`). Padding is intentionally NOT
//! applied — wrap in `theme.frame(...)` if you need a padded row.
//! Rows stay pure layout primitives.

use iced::Theme as IcedTheme;
use iced::widget::{Row, row};

use crate::theme::resolve::Theme;

impl Theme {
    /// Build a preconfigured `iced::widget::Row` with gap from classes.
    ///
    /// The caller adds children via `.push(...)` and can still chain
    /// `.align_y(...)`, `.width(...)`, `.height(...)` as usual.
    pub fn row<'a, Message>(&self, classes: &[&str]) -> Row<'a, Message, IcedTheme>
    where
        Message: 'a,
    {
        let computed = self.resolve(classes, None);
        let mut r = row![];
        if let Some(gap) = computed.length("gap") {
            r = r.spacing(gap);
        }
        r
    }
}
