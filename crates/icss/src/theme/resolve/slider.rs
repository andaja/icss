//! Slider style resolver.

use iced::widget::slider;
use iced::{Border, Color, Theme as IcedTheme};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn slider<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, slider::Status) -> slider::Style + 'a {
        move |_iced_theme, status| {
            let pseudo = match status {
                slider::Status::Active => None,
                slider::Status::Hovered => Some(PseudoClass::Hover),
                slider::Status::Dragged => Some(PseudoClass::Active),
            };

            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_slider_style(&computed)
        }
    }
}

fn build_slider_style(computed: &ComputedStyle) -> slider::Style {
    let rail_bg = Theme::resolve_color(computed, "background-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.2));

    let accent = Theme::resolve_color(computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgb(0.0, 0.4, 0.8));

    let handle_border_width = computed.length("border-width").unwrap_or(0.0);
    let handle_border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::TRANSPARENT);

    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                iced::Background::Color(accent),
                iced::Background::Color(rail_bg),
            ),
            width: 4.0,
            border: Border::default(),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: iced::Background::Color(accent),
            border_width: handle_border_width,
            border_color: handle_border_color,
        },
    }
}
