//! Dropdown menu layout with mixed item types.
//!
//! A vertical list of menu items rendered inside a styled container.
//! This is the **layout and styling** component — trigger/positioning
//! is handled by the caller (e.g. via `iced::widget::tooltip` or manual overlay).
//!
//! # Item types
//!
//! - `Text` — simple label, optional left indent
//! - `IconLabel` — icon + label
//! - `Check` — checkmark icon + label (toggle)
//! - `Custom` — arbitrary element (compound controls, sliders)
//! - `Submenu` — label + `>` arrow
//! - `Divider` — horizontal rule
//!
//! # Usage
//!
//! ```ignore
//! Menu::new()
//!     .item(MenuItem::icon_label(bootstrap::copy(), "Copy", Msg::Copy))
//!     .item(MenuItem::icon_label(bootstrap::scissors(), "Cut", Msg::Cut))
//!     .divider()
//!     .item(MenuItem::check("Bold", is_bold, Msg::ToggleBold))
//!     .item(MenuItem::submenu("Align", Msg::OpenAlign))
//!     .divider()
//!     .item(MenuItem::custom(zoom_control))
//!     .view(theme, &["sz-sm"])
//! ```

use crate::theme::Theme;
use iced::widget::{button, column, container, row, rule, text};
use iced::{Alignment, Element, Length, Padding};

/// A single menu item.
pub enum MenuItem<'a, Message> {
    /// Simple text label. `indent`: if true, left-pad to align with icon items.
    Text {
        label: &'a str,
        on_press: Message,
        indent: bool,
    },
    /// Icon + label.
    IconLabel {
        icon: Element<'a, Message>,
        label: &'a str,
        on_press: Message,
    },
    /// Checkmark toggle — shows check icon when active.
    Check {
        label: &'a str,
        checked: bool,
        on_toggle: Message,
    },
    /// Submenu trigger — label + `>` arrow on right.
    Submenu { label: &'a str, on_press: Message },
    /// Custom element (compound control, slider, etc).
    Custom { content: Element<'a, Message> },
    /// Horizontal divider.
    Divider,
}

impl<'a, Message: Clone + 'a> MenuItem<'a, Message> {
    pub fn label(label: &'a str, on_press: Message) -> Self {
        Self::Text {
            label,
            on_press,
            indent: false,
        }
    }

    pub fn label_indented(label: &'a str, on_press: Message) -> Self {
        Self::Text {
            label,
            on_press,
            indent: true,
        }
    }

    pub fn icon_label(
        icon: impl Into<Element<'a, Message>>,
        label: &'a str,
        on_press: Message,
    ) -> Self {
        Self::IconLabel {
            icon: icon.into(),
            label,
            on_press,
        }
    }

    pub fn check(label: &'a str, checked: bool, on_toggle: Message) -> Self {
        Self::Check {
            label,
            checked,
            on_toggle,
        }
    }

    pub fn submenu(label: &'a str, on_press: Message) -> Self {
        Self::Submenu { label, on_press }
    }

    pub fn custom(content: impl Into<Element<'a, Message>>) -> Self {
        Self::Custom {
            content: content.into(),
        }
    }

    pub fn divider() -> Self {
        Self::Divider
    }
}

/// Menu layout builder.
pub struct Menu<'a, Message> {
    items: Vec<MenuItem<'a, Message>>,
    width: Length,
    /// Menu container classes (e.g. `&["select-menu"]`).
    menu_classes: &'a [&'a str],
    /// Sizing classes for items.
    size_classes: &'a [&'a str],
}

impl<'a, Message: Clone + 'a> Default for Menu<'a, Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message: Clone + 'a> Menu<'a, Message> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            width: Length::Fixed(220.0),
            menu_classes: &[],
            size_classes: &[],
        }
    }

    pub fn item(mut self, item: MenuItem<'a, Message>) -> Self {
        self.items.push(item);
        self
    }

    pub fn divider(mut self) -> Self {
        self.items.push(MenuItem::Divider);
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Menu container style classes (default: `&["select-menu"]`).
    pub fn menu_style(mut self, classes: &'a [&'a str]) -> Self {
        self.menu_classes = classes;
        self
    }

    /// Item sizing classes (default: `&["sz-sm"]`).
    pub fn sizing(mut self, classes: &'a [&'a str]) -> Self {
        self.size_classes = classes;
        self
    }

    pub fn view(self, theme: &'a Theme) -> Element<'a, Message> {
        let menu_cls = if self.menu_classes.is_empty() {
            &["select-menu"] as &[&str]
        } else {
            self.menu_classes
        };
        let size_cls: &[&str] = if self.size_classes.is_empty() {
            &["sz-sm"]
        } else {
            self.size_classes
        };
        let sz = theme.sizing(size_cls);
        // Resolve the menu's own padding/gap from the .menu class — falls
        // back to zero if the host theme hasn't defined it.
        let menu_layout = theme.sizing(&["menu"]);

        // Items adopt the ghost variant + the menu's active size so
        // per-size border-radius kicks in via compose.rs compound
        // selectors. Match on size rather than building a local Vec —
        // button style closures hold onto the class slice for 'a, and a
        // local Vec would not outlive the return value.
        let item_cls: &'static [&'static str] = match size_cls {
            c if c.contains(&"sz-md") => &["button", "ghost", "sz-md"],
            c if c.contains(&"sz-xs") => &["button", "ghost", "sz-xs"],
            _ => &["button", "ghost", "sz-sm"],
        };

        // Indent for text-only items so their labels align with the
        // label column of icon items (icon glyph + row spacing).
        let label_indent = sz.font_size + sz.gap;
        let text_soft = theme.color_var("text-soft").unwrap_or(iced::Color::WHITE);
        let check_color = theme
            .color_var("surface-primary-s0")
            .unwrap_or(iced::Color::WHITE);

        let mut col = column![].spacing(menu_layout.gap).width(Length::Fill);

        for item in self.items {
            match item {
                MenuItem::Text {
                    label: lbl,
                    on_press,
                    indent,
                } => {
                    let pad_left = if indent { label_indent } else { 0.0 };
                    col = col.push(
                        button(
                            container(text(lbl).size(sz.font_size))
                                .padding(Padding {
                                    left: pad_left,
                                    ..Padding::ZERO
                                })
                                .width(Length::Fill),
                        )
                        .padding([sz.pad_v, sz.pad_h])
                        .width(Length::Fill)
                        .on_press(on_press)
                        .style(theme.button(item_cls)),
                    );
                }
                MenuItem::IconLabel {
                    icon,
                    label: lbl,
                    on_press,
                } => {
                    // Match real button internal gap: icon is rendered at
                    // font-size directly, the row uses `sz.gap` as the
                    // icon↔label spacing (same as btn_category's il_btn).
                    col = col.push(
                        button(
                            row![icon, text(lbl).size(sz.font_size),]
                                .spacing(sz.gap)
                                .align_y(Alignment::Center)
                                .width(Length::Fill),
                        )
                        .padding([sz.pad_v, sz.pad_h])
                        .width(Length::Fill)
                        .on_press(on_press)
                        .style(theme.button(item_cls)),
                    );
                }
                MenuItem::Check {
                    label: lbl,
                    checked,
                    on_toggle,
                } => {
                    let check_icon: Element<'a, Message> = if checked {
                        text("\u{2713}")
                            .size(sz.font_size)
                            .color(check_color)
                            .into()
                    } else {
                        // Fixed-width placeholder so checked and unchecked
                        // items align on the label column.
                        container(text("").size(sz.font_size))
                            .width(sz.font_size)
                            .into()
                    };
                    col = col.push(
                        button(
                            row![check_icon, text(lbl).size(sz.font_size),]
                                .spacing(sz.gap)
                                .align_y(Alignment::Center)
                                .width(Length::Fill),
                        )
                        .padding([sz.pad_v, sz.pad_h])
                        .width(Length::Fill)
                        .on_press(on_toggle)
                        .style(theme.button(item_cls)),
                    );
                }
                MenuItem::Submenu {
                    label: lbl,
                    on_press,
                } => {
                    col = col.push(
                        button(
                            row![
                                text(lbl).size(sz.font_size).width(Length::Fill),
                                text("\u{203A}").size(sz.font_size).color(text_soft),
                            ]
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                        )
                        .padding([sz.pad_v, sz.pad_h])
                        .width(Length::Fill)
                        .on_press(on_press)
                        .style(theme.button(item_cls)),
                    );
                }
                MenuItem::Custom { content } => {
                    col = col.push(
                        container(content)
                            .padding([sz.pad_v, sz.pad_h])
                            .width(Length::Fill),
                    );
                }
                MenuItem::Divider => {
                    col = col.push(
                        container(rule::horizontal(1).style(theme.rule(&["divider"])))
                            .padding(Padding::from([2.0, sz.pad_h])),
                    );
                }
            }
        }

        container(col)
            .width(self.width)
            .padding([menu_layout.pad_v, menu_layout.pad_h])
            .style(theme.container(menu_cls))
            .into()
    }
}
