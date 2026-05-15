//! Text input with optional leading icon and trailing button.
//!
//! Uses the same sizing tokens as buttons for consistency.
//! The leading icon is decorative; the trailing element is a clickable
//! button (search, clear, reveal password, etc.).
//!
//! The outer container takes the input's border/background styling.
//! The inner text_input is rendered borderless and transparent.

use crate::theme::Theme;
use iced::widget::{container, row, text_input};
use iced::{Border, Color, Element, Length, Padding, Theme as IcedTheme};

/// Text input with icon slots, sized consistently with buttons.
pub struct IconInput<'a, Message> {
    placeholder: &'a str,
    value: &'a str,
    /// Decorative leading icon (sized to match font).
    leading: Option<Element<'a, Message>>,
    /// Trailing button element (search, clear, chevron, etc.).
    trailing: Option<Element<'a, Message>>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    /// Input styling (border, background, colors) — applied to outer container.
    input_classes: &'a [&'a str],
    /// Sizing classes (sz-md, sz-sm, etc.).
    size_classes: &'a [&'a str],
    width: Length,
}

impl<'a, Message: Clone + 'a> IconInput<'a, Message> {
    pub fn new(placeholder: &'a str, value: &'a str) -> Self {
        Self {
            placeholder,
            value,
            leading: None,
            trailing: None,
            on_input: None,
            on_submit: None,
            input_classes: &[],
            size_classes: &[],
            width: Length::Fill,
        }
    }

    /// Set decorative leading icon.
    pub fn leading(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(icon.into());
        self
    }

    /// Set trailing button element.
    pub fn trailing(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(icon.into());
        self
    }

    pub fn on_input(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(f));
        self
    }

    pub fn on_submit(mut self, msg: Message) -> Self {
        self.on_submit = Some(msg);
        self
    }

    /// Set input style classes (e.g. `&["input"]` or `&["input", "error"]`).
    pub fn input_style(mut self, classes: &'a [&'a str]) -> Self {
        self.input_classes = classes;
        self
    }

    /// Set sizing classes (e.g. `&["sz-md"]` or `&["sz-sm"]`).
    pub fn sizing(mut self, classes: &'a [&'a str]) -> Self {
        self.size_classes = classes;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Build the final element.
    pub fn view(self, theme: &'a Theme) -> Element<'a, Message> {
        let size_cls = if self.size_classes.is_empty() {
            &["sz-md"] as &[&str]
        } else {
            self.size_classes
        };
        let sz = theme.sizing(size_cls);

        // Resolve the input style for the Active state to get colors. Merge in
        // the size class so size-keyed properties (e.g. border-radius) apply.
        let mut merged: Vec<&str> = self.input_classes.to_vec();
        merged.extend_from_slice(size_cls);
        let computed = theme.resolve(&merged, None);
        let bg_color = Theme::resolve_color(&computed, "background-color")
            .map(|c| c.to_iced())
            .unwrap_or(Color::TRANSPARENT);
        let text_color = Theme::resolve_color(&computed, "color")
            .map(|c| c.to_iced())
            .unwrap_or(Color::WHITE);
        let placeholder_color = Theme::resolve_color(&computed, "placeholder-color")
            .map(|c| c.to_iced())
            .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.4));
        let border_radius = computed.length("border-radius").unwrap_or(0.0);
        let border_width = computed.length("border-width").unwrap_or(0.0);
        let border_color = Theme::resolve_color(&computed, "border-color")
            .map(|c| c.to_iced())
            .unwrap_or(Color::TRANSPARENT);
        let selection = Theme::resolve_color(&computed, "accent-color")
            .map(|c| c.to_iced())
            .unwrap_or(Color::from_rgba(0.0, 0.4, 0.8, 0.3));

        // Text input — borderless, transparent bg, inherits text colors
        let tc = text_color;
        let pc = placeholder_color;
        let sel = selection;
        let mut input = text_input(self.placeholder, self.value)
            .padding(Padding::from([sz.pad_v, 0.0]))
            .size(sz.font_size)
            .style(
                move |_: &IcedTheme, _status| iced::widget::text_input::Style {
                    background: iced::Background::Color(Color::TRANSPARENT),
                    border: Border::default(),
                    icon: tc,
                    placeholder: pc,
                    value: tc,
                    selection: sel,
                },
            );

        if let Some(f) = self.on_input {
            input = input.on_input(f);
        }
        if let Some(msg) = self.on_submit {
            input = input.on_submit(msg);
        }

        // Assemble row: [leading] [input] [trailing]
        // Use slightly more spacing than button gap for readability
        let input_gap = (sz.gap * 1.5).round();
        let mut content = row![].spacing(input_gap).align_y(iced::Alignment::Center);

        if let Some(icon) = self.leading {
            content = content.push(icon);
        }

        content = content.push(container(input).width(Length::Fill));

        if let Some(trailing) = self.trailing {
            content = content.push(trailing);
        }

        // Outer container — horizontal padding tighter than buttons
        let h_pad = (sz.pad_v * 1.5).round();
        container(content)
            .padding(Padding::from([0.0, h_pad]))
            .width(self.width)
            .style(move |_: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: Border {
                    color: border_color,
                    width: border_width,
                    radius: border_radius.into(),
                },
                ..Default::default()
            })
            .into()
    }
}
