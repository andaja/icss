//! Text editor style resolver.

use iced::widget::text_editor;
use iced::{Border, Color, Theme as IcedTheme};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn text_editor<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, text_editor::Status) -> text_editor::Style + 'a {
        move |_iced_theme, status| {
            let pseudo = match status {
                text_editor::Status::Active => None,
                text_editor::Status::Hovered => Some(PseudoClass::Hover),
                text_editor::Status::Focused { .. } => Some(PseudoClass::Focus),
                text_editor::Status::Disabled => Some(PseudoClass::Disabled),
            };

            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_text_editor_style(&computed)
        }
    }
}

fn build_text_editor_style(computed: &ComputedStyle) -> text_editor::Style {
    let background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::TRANSPARENT));

    let border_radius = computed.length("border-radius").unwrap_or(0.0);
    let border_width = computed.length("border-width").unwrap_or(0.0);
    let border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::TRANSPARENT);

    let value = Theme::resolve_color(computed, "color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::WHITE);

    let placeholder = Theme::resolve_color(computed, "placeholder-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.4));

    let selection = Theme::resolve_color(computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(0.0, 0.4, 0.8, 0.3));

    text_editor::Style {
        background,
        border: Border {
            color: border_color,
            width: border_width,
            radius: border_radius.into(),
        },
        placeholder,
        value,
        selection,
    }
}
