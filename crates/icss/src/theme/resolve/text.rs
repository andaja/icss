//! Text resolver — content-taking helper for styled text widgets.
//!
//! ```rust,ignore
//! theme.text("Section Title", &["title-small"])
//! theme.text("Error occurred", &["label-small", "text-danger"])
//! theme.text(format!("{n} items"), &["caption"])
//! ```
//!
//! Pulls `font-size`, `color`, and `font-weight` from the merged class set.
//! Font family is NOT set from classes — it comes from the app-level default
//! font configured via [`Theme::with_default_font`], so weight overrides
//! preserve the configured family.

use iced::widget::text as text_fn;
use iced::widget::text::{IntoFragment, Text};
use iced::{Font, Theme as IcedTheme};

use crate::theme::resolve::Theme;

impl Theme {
    /// Build a styled `iced::widget::Text` from class list.
    ///
    /// The returned `Text` is a plain iced widget — callers can still chain
    /// `.width(...)`, `.height(...)`, `.align_x(...)`, etc. as usual, and
    /// even override color/size per call if needed.
    pub fn text<'a>(
        &self,
        content: impl IntoFragment<'a>,
        classes: &[&str],
    ) -> Text<'a, IcedTheme> {
        let computed = self.resolve(classes, None);
        let mut t = text_fn(content);

        if let Some(size) = computed.length("font-size") {
            t = t.size(size);
        }

        if let Some(color) = computed.color("color") {
            t = t.color(color.to_iced());
        }

        if let Some(weight) = computed.number("font-weight") {
            t = t.font(Font {
                weight: weight_from_css(weight),
                ..self.default_font()
            });
        }

        t
    }
}

/// Map a CSS numeric font-weight (100–900) to `iced::font::Weight`.
pub(crate) fn weight_from_css(v: f32) -> iced::font::Weight {
    use iced::font::Weight;
    match v.round() as i32 {
        100 => Weight::Thin,
        200 => Weight::ExtraLight,
        300 => Weight::Light,
        400 => Weight::Normal,
        500 => Weight::Medium,
        600 => Weight::Semibold,
        700 => Weight::Bold,
        800 => Weight::ExtraBold,
        900 => Weight::Black,
        _ => Weight::Normal,
    }
}
