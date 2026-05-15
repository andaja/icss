//! Column resolver — returns an `iced::widget::Column` preconfigured with gap.
//!
//! ```rust,ignore
//! theme.column(&["stack"])         // space_100 gap
//! theme.column(&["stack-tight"])   // space_75 gap
//! theme.column(&["field-col"])     // space_50 gap (tight label+control)
//! ```
//!
//! Pulls `gap` (maps to `Column::spacing`). Padding is intentionally NOT
//! applied — wrap in `theme.frame(...)` if you need a padded column.

use iced::Theme as IcedTheme;
use iced::widget::{Column, column};

use crate::theme::resolve::Theme;

impl Theme {
    /// Build a preconfigured `iced::widget::Column` with gap from classes.
    ///
    /// The caller adds children via `.push(...)` and can still chain
    /// `.align_x(...)`, `.width(...)`, `.height(...)` as usual.
    pub fn column<'a, Message>(&self, classes: &[&str]) -> Column<'a, Message, IcedTheme>
    where
        Message: 'a,
    {
        let computed = self.resolve(classes, None);
        let mut c = column![];
        if let Some(gap) = computed.length("gap") {
            c = c.spacing(gap);
        }
        c
    }
}
