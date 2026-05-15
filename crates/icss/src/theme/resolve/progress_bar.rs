//! Progress bar style resolver.

use iced::widget::progress_bar;
use iced::{Border, Color, Theme as IcedTheme};

use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn progress_bar<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme) -> progress_bar::Style + 'a {
        move |_iced_theme| {
            let computed = self.resolve_with_pseudo(classes, None);
            build_progress_bar_style(&computed)
        }
    }
}

fn build_progress_bar_style(computed: &ComputedStyle) -> progress_bar::Style {
    let background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::from_rgba(
            1.0, 1.0, 1.0, 0.15,
        )));

    let bar = Theme::resolve_color(computed, "accent-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::from_rgb(0.0, 0.4, 0.8)));

    let border_radius = computed.length("border-radius").unwrap_or(2.0);

    progress_bar::Style {
        background,
        bar,
        border: Border {
            radius: border_radius.into(),
            ..Border::default()
        },
    }
}
