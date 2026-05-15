//! Checkbox style resolver.

use iced::widget::checkbox;
use iced::{Border, Color, Theme as IcedTheme};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn checkbox<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, checkbox::Status) -> checkbox::Style + 'a {
        move |_iced_theme, status| {
            let pseudo = match status {
                checkbox::Status::Active { is_checked } => {
                    if is_checked {
                        Some(PseudoClass::Checked)
                    } else {
                        None
                    }
                }
                checkbox::Status::Hovered { is_checked } => {
                    // Hover takes priority; checked state handled via class.
                    if is_checked {
                        Some(PseudoClass::Checked)
                    } else {
                        Some(PseudoClass::Hover)
                    }
                }
                checkbox::Status::Disabled { .. } => Some(PseudoClass::Disabled),
            };

            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_checkbox_style(&computed)
        }
    }
}

fn build_checkbox_style(computed: &ComputedStyle) -> checkbox::Style {
    let opacity = computed.number("opacity").unwrap_or(1.0);

    let mut background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::TRANSPARENT));

    let mut icon_color = Theme::resolve_color(computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::WHITE);

    let border_radius = computed.length("border-radius").unwrap_or(2.0);
    let border_width = computed.length("border-width").unwrap_or(1.0);
    let mut border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.3));

    let mut text_color = Theme::resolve_color(computed, "color").map(|c| c.to_iced());

    if opacity < 1.0 {
        background = background.scale_alpha(opacity);
        icon_color.a *= opacity;
        border_color.a *= opacity;
        if let Some(ref mut tc) = text_color {
            tc.a *= opacity;
        }
    }

    checkbox::Style {
        background,
        icon_color,
        border: Border {
            color: border_color,
            width: border_width,
            radius: border_radius.into(),
        },
        text_color,
    }
}
