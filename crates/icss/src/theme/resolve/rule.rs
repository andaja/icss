//! Rule (divider) style resolver.

use iced::widget::rule;
use iced::{Color, Theme as IcedTheme};

use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn rule<'a>(&'a self, classes: &'a [&str]) -> impl Fn(&IcedTheme) -> rule::Style + 'a {
        move |_iced_theme| {
            let computed = self.resolve_with_pseudo(classes, None);
            build_rule_style(&computed)
        }
    }
}

fn build_rule_style(computed: &ComputedStyle) -> rule::Style {
    let color = Theme::resolve_color(computed, "color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.15));

    let border_radius = computed.length("border-radius").unwrap_or(0.0);

    rule::Style {
        color,
        radius: border_radius.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}
