//! Container style resolver: `ComputedStyle` → `container::Style`.

use iced::widget::container;
use iced::{Border, Color, Shadow, Theme as IcedTheme, Vector};

use crate::theme::css::ast::PseudoClass;
use crate::theme::resolve::Theme;
use crate::theme::resolve::computed::ComputedStyle;

impl Theme {
    /// Return an iced container style closure for the given CSS classes.
    ///
    /// ```rust,ignore
    /// container(content).style(theme.container(&["surface", "main"]))
    /// ```
    pub fn container<'a>(
        &'a self,
        classes: &'a [&str],
    ) -> impl Fn(&IcedTheme) -> container::Style + 'a {
        move |_iced_theme| {
            let computed = self.resolve_with_pseudo(classes, None);
            build_container_style(&computed)
        }
    }

    /// Return a container style with an explicit pseudo-class (hover, active, etc.).
    ///
    /// Use this for interactive containers like tiles and table rows where
    /// the hover/selected state is tracked by the caller.
    ///
    /// ```rust,ignore
    /// container(content).style(theme.container_with_pseudo(
    ///     &["tile", "selected"],
    ///     Some(PseudoClass::Hover),
    /// ))
    /// ```
    pub fn container_with_pseudo<'a>(
        &'a self,
        classes: &'a [&str],
        pseudo: Option<PseudoClass>,
    ) -> impl Fn(&IcedTheme) -> container::Style + 'a {
        move |_iced_theme| {
            let computed = self.resolve_with_pseudo(classes, pseudo);
            build_container_style(&computed)
        }
    }
}

fn build_container_style(computed: &ComputedStyle) -> container::Style {
    let background = Theme::resolve_color(computed, "background-color")
        .map(|c| iced::Background::Color(c.to_iced()));

    let text_color = Theme::resolve_color(computed, "color").map(|c| c.to_iced());

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

    container::Style {
        background,
        text_color,
        border: Border {
            color: border_color,
            width: border_width,
            radius: border_radius.into(),
        },
        shadow,
        snap: false,
    }
}
