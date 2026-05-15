//! Toggler style resolver.

use iced::widget::toggler;
use iced::{Color, Theme as IcedTheme};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn toggler<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, toggler::Status) -> toggler::Style + 'a {
        move |_iced_theme, status| {
            let pseudo = match status {
                toggler::Status::Active { is_toggled } => {
                    if is_toggled {
                        Some(PseudoClass::Checked)
                    } else {
                        None
                    }
                }
                toggler::Status::Hovered { is_toggled } => {
                    if is_toggled {
                        Some(PseudoClass::Checked)
                    } else {
                        Some(PseudoClass::Hover)
                    }
                }
                toggler::Status::Disabled { .. } => Some(PseudoClass::Disabled),
            };

            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_toggler_style(&computed)
        }
    }
}

fn build_toggler_style(computed: &ComputedStyle) -> toggler::Style {
    let opacity = computed.number("opacity").unwrap_or(1.0);

    // background-color = track
    let mut background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::from_rgba(
            1.0, 1.0, 1.0, 0.2,
        )));

    // accent-color = knob (foreground circle)
    let mut foreground = Theme::resolve_color(computed, "accent-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::WHITE));

    let background_border_width = computed.length("border-width").unwrap_or(0.0);
    let mut background_border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::TRANSPARENT);

    let border_radius = computed.length("border-radius").unwrap_or(f32::MAX);

    // color = label text (independent from knob)
    let mut text_color = Theme::resolve_color(computed, "color").map(|c| c.to_iced());

    if opacity < 1.0 {
        background = background.scale_alpha(opacity);
        foreground = foreground.scale_alpha(opacity);
        background_border_color.a *= opacity;
        if let Some(ref mut tc) = text_color {
            tc.a *= opacity;
        }
    }

    toggler::Style {
        background,
        background_border_width,
        background_border_color,
        foreground,
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        border_radius: Some(border_radius.into()),
        padding_ratio: 0.1,
        text_color,
    }
}
