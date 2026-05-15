//! Scrollable style resolver.

use iced::widget::{container, scrollable};
use iced::{Border, Color, Theme as IcedTheme, Vector};

use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    pub fn scrollable<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme, scrollable::Status) -> scrollable::Style + 'a {
        move |_iced_theme, _status| {
            let computed = self.resolve_with_pseudo(classes, None);
            build_scrollable_style(&computed)
        }
    }
}

fn build_scrollable_style(computed: &ComputedStyle) -> scrollable::Style {
    let rail_bg = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()));

    let scroller_color = Theme::resolve_color(computed, "accent-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.3));

    let border_radius = computed.length("border-radius").unwrap_or(4.0);
    let border_width = computed.length("border-width").unwrap_or(0.0);
    let border_color = Theme::resolve_color(computed, "border-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::TRANSPARENT);

    let rail = scrollable::Rail {
        background: rail_bg,
        border: Border {
            color: border_color,
            width: border_width,
            radius: border_radius.into(),
        },
        scroller: scrollable::Scroller {
            background: iced::Background::Color(scroller_color),
            border: Border {
                radius: border_radius.into(),
                ..Border::default()
            },
        },
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: iced::Background::Color(Color::TRANSPARENT),
            border: Border::default(),
            shadow: iced::Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::ZERO,
                blur_radius: 0.0,
            },
            icon: Color::TRANSPARENT,
        },
    }
}
