//! Control group — label + input + button/icon-button composition.
//!
//! Layouts:
//! - Horizontal: `[Label] [Input] [Button]`
//! - Vertical: `[Label]` above `[Input] [Button]`
//!
//! Optional error message shown below the input.

use crate::theme::Theme;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Color, Element, Length};

/// Layout direction for the control group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlLayout {
    Horizontal,
    Vertical,
}

/// A label + input + optional button composition.
pub struct ControlGroup<'a, Message> {
    label: Option<&'a str>,
    input: Element<'a, Message>,
    trailing: Option<Element<'a, Message>>,
    leading: Option<Element<'a, Message>>,
    error: Option<&'a str>,
    layout: ControlLayout,
    label_width: Option<f32>,
    gap: f32,
    font_size: f32,
}

impl<'a, Message: Clone + 'a> ControlGroup<'a, Message> {
    /// Create with an input element.
    pub fn new(input: impl Into<Element<'a, Message>>) -> Self {
        Self {
            label: None,
            input: input.into(),
            trailing: None,
            leading: None,
            error: None,
            layout: ControlLayout::Vertical,
            label_width: None,
            gap: 6.0,
            font_size: 14.0,
        }
    }

    /// Set label text.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Add a trailing element (button, icon button).
    pub fn trailing(mut self, el: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(el.into());
        self
    }

    /// Add a leading element (before the input, after the label).
    pub fn leading(mut self, el: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(el.into());
        self
    }

    /// Set an error message shown below the input.
    pub fn error(mut self, msg: &'a str) -> Self {
        self.error = Some(msg);
        self
    }

    /// Set layout direction (default: Vertical).
    pub fn layout(mut self, layout: ControlLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set fixed label width for horizontal alignment.
    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = Some(width);
        self
    }

    /// Set gap between elements.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set label font size.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Build the final element.
    pub fn view(self, theme: &'a Theme) -> Element<'a, Message> {
        let error_color = theme
            .color_var("surface-danger-s0")
            .unwrap_or(Color::from_rgb(1.0, 0.3, 0.3));
        let label_color = theme.color_var("text-default").unwrap_or(Color::WHITE);

        // Input row: [leading] [input] [trailing]
        let mut input_row = row![].spacing(self.gap).align_y(Alignment::Center);

        if let Some(leading) = self.leading {
            input_row = input_row.push(leading);
        }

        input_row = input_row.push(container(self.input).width(Length::Fill));

        if let Some(trailing) = self.trailing {
            input_row = input_row.push(trailing);
        }

        // Input + error column
        let mut input_col = column![input_row].spacing(2);
        if let Some(err) = self.error {
            input_col = input_col.push(
                text(err)
                    .size((self.font_size * 0.75).round())
                    .color(error_color),
            );
        }

        // Assemble with label
        match self.layout {
            ControlLayout::Vertical => {
                let mut col = column![].spacing(self.gap * 0.5);
                if let Some(lbl) = self.label {
                    col = col.push(text(lbl).size(self.font_size).color(label_color));
                }
                col = col.push(input_col);
                col.into()
            }
            ControlLayout::Horizontal => {
                let mut r = row![].spacing(self.gap).align_y(Alignment::Center);
                if let Some(lbl) = self.label {
                    let label_el = text(lbl).size(self.font_size).color(label_color);
                    if let Some(w) = self.label_width {
                        r = r.push(container(label_el).width(w));
                    } else {
                        r = r.push(label_el);
                    }
                }
                r = r.push(container(input_col).width(Length::Fill));
                r.into()
            }
        }
    }
}
