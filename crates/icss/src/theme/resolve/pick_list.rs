//! Pick list (dropdown) style resolver.

use iced::widget::pick_list;
use iced::{Border, Color, Theme as IcedTheme};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn pick_list<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, pick_list::Status) -> pick_list::Style + 'a {
        move |_iced_theme, status| {
            let pseudo = match status {
                pick_list::Status::Active => None,
                pick_list::Status::Hovered => Some(PseudoClass::Hover),
                pick_list::Status::Opened { .. } => Some(PseudoClass::Focus),
            };

            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_pick_list_style(&computed)
        }
    }
}

fn build_pick_list_style(computed: &ComputedStyle) -> pick_list::Style {
    let opacity = computed.number("opacity").unwrap_or(1.0);

    let mut text_color = Theme::resolve_color(computed, "color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::WHITE);

    let mut placeholder_color = Theme::resolve_color(computed, "placeholder-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.4));

    let mut handle_color = Theme::resolve_color(computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(text_color);

    let mut background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::TRANSPARENT));

    let border_radius = computed.length("border-radius").unwrap_or(0.0);
    let border_width = computed.length("border-width").unwrap_or(0.0);
    let mut border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::TRANSPARENT);

    if opacity < 1.0 {
        background = background.scale_alpha(opacity);
        border_color.a *= opacity;
        text_color.a *= opacity;
        placeholder_color.a *= opacity;
        handle_color.a *= opacity;
    }

    pick_list::Style {
        text_color,
        placeholder_color,
        handle_color,
        background,
        border: Border {
            color: border_color,
            width: border_width,
            radius: border_radius.into(),
        },
    }
}
