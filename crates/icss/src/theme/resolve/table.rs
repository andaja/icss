//! Table style resolver.

use iced::widget::table;
use iced::{Background, Theme as IcedTheme};

use crate::theme::resolve::Theme;

impl Theme {
    pub fn table<'a>(&'a self, classes: &'a [&str]) -> impl Fn(&IcedTheme) -> table::Style + 'a {
        move |_iced_theme| {
            let computed = self.resolve_with_pseudo(classes, None);
            let sep_color = Self::resolve_color(&computed, "border-color")
                .map(|c| c.to_iced())
                .unwrap_or(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08));
            let sep = Background::Color(sep_color);
            table::Style {
                separator_x: sep,
                separator_y: sep,
            }
        }
    }
}
