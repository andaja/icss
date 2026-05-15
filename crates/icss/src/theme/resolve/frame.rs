//! Frame resolver — content-taking container helper.
//!
//! `theme.frame(content, &[classes])` wraps content in an `iced::Container`
//! with both the **style** (background, border, radius, shadow) from
//! [`Theme::container`] AND the **padding** pulled from the merged class set.
//!
//! ```rust,ignore
//! // Section card: bg + radius + padding + gap inside
//! theme.frame(
//!     theme.column(&["subsection"])
//!         .push(theme.text("Buttons", &["title-small"]))
//!         .push(buttons_content),
//!     &["section", "section-body"],
//! )
//!
//! // Sidebar with scrollable content
//! theme.frame(
//!     scrollable(sidebar_content).height(Length::Fill),
//!     &["sidebar", "section-body"],
//! )
//! .width(280)
//! .height(Length::Fill)
//! ```
//!
//! This is the ergonomic counterpart to [`Theme::container`] (which returns
//! a style closure for use with `iced::widget::container(c).style(...)`).
//! Use `frame` when you want padding baked in; use `container` when you only
//! need the style closure.

use iced::widget::{Container, container};
use iced::{Element, Padding, Theme as IcedTheme};

use crate::theme::resolve::Theme;

impl Theme {
    /// Wrap content in a styled, padded `iced::widget::Container`.
    ///
    /// Pulls `padding-v` / `padding-h` from the class set (via the same
    /// mechanism as [`Theme::sizing`]) and applies the container style
    /// closure from [`Theme::container`] for bg/border/radius/shadow.
    pub fn frame<'a, Message>(
        &'a self,
        content: impl Into<Element<'a, Message, IcedTheme>>,
        classes: &'a [&'a str],
    ) -> Container<'a, Message, IcedTheme>
    where
        Message: 'a,
    {
        let computed = self.resolve(classes, None);

        // Padding — fall back to 0 if not specified.
        let pad_v = computed.length("padding-v").unwrap_or(0.0);
        let pad_h = computed.length("padding-h").unwrap_or(0.0);

        container(content)
            .padding(Padding::from([pad_v, pad_h]))
            .style(self.container(classes))
    }
}
