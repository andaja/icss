//! Menu (dropdown overlay) style resolver.
//!
//! Used via `pick_list(...).menu_style(theme.menu(&["select-menu"]))`.

use iced::overlay::menu;
use iced::{Border, Color, Shadow, Theme as IcedTheme, Vector};

use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn menu<'a>(&'a self, classes: &'a [&str]) -> impl Fn(&IcedTheme) -> menu::Style + 'a {
        move |_iced_theme| {
            let computed = self.resolve_with_pseudo(classes, None);
            build_menu_style(&computed)
        }
    }
}

pub(crate) fn build_menu_style(computed: &ComputedStyle) -> menu::Style {
    let background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::from_rgba(
            0.1, 0.1, 0.15, 1.0,
        )));

    let text_color = Theme::resolve_color(computed, "color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::WHITE);

    let selected_text_color = Theme::resolve_color(computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::WHITE);

    let selected_background = Theme::resolve_color(computed, "border-color")
        .map(|c| iced::Background::Color(c.to_iced()))
        .unwrap_or(iced::Background::Color(Color::from_rgba(
            0.0, 0.3, 0.6, 1.0,
        )));

    let border_radius = computed.length("border-radius").unwrap_or(4.0);

    let shadow = computed
        .shadow("box-shadow")
        .map(|(x, y, blur, color)| Shadow {
            color: color.to_iced(),
            offset: Vector::new(x, y),
            blur_radius: blur,
        })
        .unwrap_or_default();

    menu::Style {
        background,
        border: Border {
            radius: border_radius.into(),
            ..Default::default()
        },
        text_color,
        selected_text_color,
        selected_background,
        shadow,
    }
}
