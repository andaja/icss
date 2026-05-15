//! Radio button style resolver.

use iced::widget::radio;
use iced::{Color, Theme as IcedTheme};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn radio<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, radio::Status) -> radio::Style + 'a {
        move |_iced_theme, status| {
            let pseudo = match status {
                radio::Status::Active { is_selected } => {
                    if is_selected {
                        Some(PseudoClass::Checked)
                    } else {
                        None
                    }
                }
                radio::Status::Hovered { is_selected } => {
                    if is_selected {
                        Some(PseudoClass::Checked)
                    } else {
                        Some(PseudoClass::Hover)
                    }
                }
            };

            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_radio_style(&computed)
        }
    }
}

fn build_radio_style(computed: &ComputedStyle) -> radio::Style {
    let background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::TRANSPARENT));

    let dot_color = Theme::resolve_color(computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::WHITE);

    let border_width = computed.length("border-width").unwrap_or(1.0);
    let border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.3));

    let text_color = Theme::resolve_color(computed, "color").map(|c| c.to_iced());

    radio::Style {
        background,
        dot_color,
        border_width,
        border_color,
        text_color,
    }
}
