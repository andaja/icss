//! Button style resolver: `ComputedStyle` → `button::Style`.

use iced::widget::button;
use iced::{Border, Color, Shadow, Theme as IcedTheme, Vector};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    /// Return an iced button style closure for the given CSS classes.
    ///
    /// ```rust,ignore
    /// button("Connect").style(theme.button(&["button", "primary"]))
    /// ```
    pub fn button<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, button::Status) -> button::Style + 'a {
        move |_iced_theme, status| {
            let pseudo = match status {
                button::Status::Active => None,
                button::Status::Hovered => Some(PseudoClass::Hover),
                button::Status::Pressed => Some(PseudoClass::Active),
                button::Status::Disabled => Some(PseudoClass::Disabled),
            };

            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_button_style(&computed)
        }
    }
}

fn build_button_style(computed: &ComputedStyle) -> button::Style {
    let opacity = computed.number("opacity").unwrap_or(1.0);

    let background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()));

    let text_color = Theme::resolve_color(computed, "color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::WHITE);

    let border_radius = computed.length("border-radius").unwrap_or(0.0);
    let border_width = computed.length("border-width").unwrap_or(0.0);
    let border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::TRANSPARENT);

    let shadow = computed
        .shadow("box-shadow")
        .map(|(x, y, blur, color)| Shadow {
            color: color.to_iced(),
            offset: Vector::new(x, y),
            blur_radius: blur,
        })
        .unwrap_or(Shadow {
            color: Color::TRANSPARENT,
            offset: Vector::ZERO,
            blur_radius: 0.0,
        });

    let mut style = button::Style {
        background,
        text_color,
        border: Border {
            color: border_color,
            width: border_width,
            radius: border_radius.into(),
        },
        shadow,
        snap: false,
    };

    // Disabled opacity: fade everything, no shadow.
    if opacity < 1.0 {
        if let Some(ref mut bg) = style.background {
            *bg = bg.scale_alpha(opacity);
        }
        style.text_color.a *= opacity;
        style.border.color.a *= opacity;
        style.shadow = Shadow {
            color: Color::TRANSPARENT,
            offset: Vector::ZERO,
            blur_radius: 0.0,
        };
    }

    style
}
