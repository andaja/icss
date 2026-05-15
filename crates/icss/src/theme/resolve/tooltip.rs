//! Tooltip style resolver.

use iced::widget::container;
use iced::{Border, Shadow, Theme as IcedTheme, Vector};

use crate::theme::resolve::Theme;

impl Theme {
    pub fn tooltip<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme) -> container::Style + 'a {
        // Tooltip uses container::Style under the hood.
        move |_iced_theme| {
            let computed = self.resolve_with_pseudo(classes, None);
            let background = Self::resolve_color(&computed, "background-color")
                .map(|c| iced::Background::Color(c.to_iced()));
            let text_color = Self::resolve_color(&computed, "color").map(|c| c.to_iced());
            let border_radius = computed.length("border-radius").unwrap_or(4.0);
            let shadow = computed
                .shadow("box-shadow")
                .map(|(x, y, blur, color)| Shadow {
                    color: color.to_iced(),
                    offset: Vector::new(x, y),
                    blur_radius: blur,
                })
                .unwrap_or_default();

            container::Style {
                background,
                text_color,
                border: Border {
                    radius: border_radius.into(),
                    ..Default::default()
                },
                shadow,
                snap: false,
            }
        }
    }
}
