//! Segmented button group — horizontal row of joined buttons.
//!
//! First button has left radius, last has right radius, middle buttons have none.
//! One button can be "active" (primary style), others use default/ghost.
//!
//! # Usage
//!
//! ```ignore
//! ButtonGroup::new(&["View", "Edit", "Share"], active_index)
//!     .on_press(|i| Msg::TabChanged(i))
//!     .view(theme, &["sz-sm"])
//! ```

use crate::theme::Theme;
use iced::widget::{button, container, row, text};
use iced::{Element, Font, Length, Theme as IcedTheme};

/// A single item in the button group.
pub struct GroupItem<'a, Message> {
    content: Element<'a, Message>,
}

/// Horizontal segmented button group.
pub struct ButtonGroup<'a, Message> {
    items: Vec<GroupItem<'a, Message>>,
    active: Option<usize>,
    on_press: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    size_classes: &'a [&'a str],
    active_classes: &'a [&'a str],
    inactive_classes: &'a [&'a str],
}

impl<'a, Message: Clone + 'a> ButtonGroup<'a, Message> {
    /// Create from string labels. Font size is set from sizing tokens in `view()`.
    pub fn from_labels(labels: &[&'a str], active: Option<usize>, font: Font) -> Self {
        // Store labels as-is; font size will be applied in view() from sizing tokens
        let items = labels
            .iter()
            .map(|label| GroupItem {
                content: text(*label).font(font).into(),
            })
            .collect();
        Self {
            items,
            active,
            on_press: None,
            size_classes: &[],
            active_classes: &[],
            inactive_classes: &[],
        }
    }

    /// Create from custom elements.
    pub fn new(items: Vec<Element<'a, Message>>, active: Option<usize>) -> Self {
        let items = items
            .into_iter()
            .map(|content| GroupItem { content })
            .collect();
        Self {
            items,
            active,
            on_press: None,
            size_classes: &[],
            active_classes: &[],
            inactive_classes: &[],
        }
    }

    pub fn on_press(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_press = Some(Box::new(f));
        self
    }

    /// Sizing classes (e.g. `&["sz-md"]`).
    pub fn sizing(mut self, classes: &'a [&'a str]) -> Self {
        self.size_classes = classes;
        self
    }

    /// Style classes for the active button (e.g. `&["button", "primary"]`).
    pub fn active_style(mut self, classes: &'a [&'a str]) -> Self {
        self.active_classes = classes;
        self
    }

    /// Style classes for inactive buttons (e.g. `&["button", "default"]`).
    pub fn inactive_style(mut self, classes: &'a [&'a str]) -> Self {
        self.inactive_classes = classes;
        self
    }

    pub fn view(self, theme: &'a Theme) -> Element<'a, Message> {
        let sz = theme.sizing(if self.size_classes.is_empty() {
            &["sz-md"]
        } else {
            self.size_classes
        });
        let active_cls = if self.active_classes.is_empty() {
            &["button", "primary"] as &[&str]
        } else {
            self.active_classes
        };
        let inactive_cls = if self.inactive_classes.is_empty() {
            &["button", "default"] as &[&str]
        } else {
            self.inactive_classes
        };

        let count = self.items.len();
        // Derive corner radius from `button` + size class so the group's
        // outer corners match the size-keyed radius (sz-md/sm/xs).
        let mut radius_classes: Vec<&str> = vec!["button"];
        if !self.size_classes.is_empty() {
            radius_classes.extend_from_slice(self.size_classes);
        } else {
            radius_classes.push("sz-md");
        }
        let r = theme
            .resolve(&radius_classes, None)
            .length("border-radius")
            .unwrap_or(4.0);

        let mut group_row = row![].spacing(0);

        for (i, item) in self.items.into_iter().enumerate() {
            let is_active = self.active == Some(i);
            let classes = if is_active { active_cls } else { inactive_cls };

            // Compute per-position radius
            let (tl, tr, bl, br) = if count == 1 {
                (r, r, r, r)
            } else if i == 0 {
                (r, 0.0, r, 0.0)
            } else if i == count - 1 {
                (0.0, r, 0.0, r)
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

            let style_fn = theme.button(classes);
            let idx = i;

            let mut btn = button(
                container(item.content)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            )
            .padding(sz.padding())
            .style(move |iced_theme: &IcedTheme, status| {
                let mut s = style_fn(iced_theme, status);
                s.border.radius = iced::border::Radius {
                    top_left: tl,
                    top_right: tr,
                    bottom_left: bl,
                    bottom_right: br,
                };
                s
            });

            if let Some(ref on_press) = self.on_press {
                btn = btn.on_press(on_press(idx));
            }

            group_row = group_row.push(btn);
        }

        group_row.into()
    }
}
