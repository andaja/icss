//! Text input style resolver: `ComputedStyle` → `text_input::Style`.

use iced::widget::text_input;
use iced::{Border, Color, Theme as IcedTheme};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    /// Return an iced text_input style closure for the given CSS classes.
    ///
    /// ```rust,ignore
    /// text_input("placeholder", &value)
    ///     .style(theme.text_input(&["input", "connect"]))
    /// ```
    pub fn text_input<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, text_input::Status) -> text_input::Style + 'a {
        move |_iced_theme, status| {
            let pseudo = match status {
                text_input::Status::Active => None,
                text_input::Status::Hovered => Some(PseudoClass::Hover),
                text_input::Status::Focused { .. } => Some(PseudoClass::Focus),
                text_input::Status::Disabled => Some(PseudoClass::Disabled),
            };

            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_text_input_style(&computed)
        }
    }
}

fn build_text_input_style(computed: &ComputedStyle) -> text_input::Style {
    let opacity = computed.number("opacity").unwrap_or(1.0);

    let mut background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::TRANSPARENT));

    let border_radius = computed.length("border-radius").unwrap_or(0.0);
    let border_width = computed.length("border-width").unwrap_or(0.0);
    let mut border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::TRANSPARENT);

    let mut value = Theme::resolve_color(computed, "color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::WHITE);

    let mut placeholder = Theme::resolve_color(computed, "placeholder-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.4));

    let mut icon = Theme::resolve_color(computed, "caret-color")
        .map(|c| c.to_iced())
        .unwrap_or(value);

    let selection = Theme::resolve_color(computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(0.0, 0.4, 0.8, 0.3));

    if opacity < 1.0 {
        background = background.scale_alpha(opacity);
        border_color.a *= opacity;
        value.a *= opacity;
        placeholder.a *= opacity;
        icon.a *= opacity;
    }

    text_input::Style {
        background,
        border: Border {
            color: border_color,
            width: border_width,
            radius: border_radius.into(),
        },
        icon,
        placeholder,
        value,
        selection,
    }
}
