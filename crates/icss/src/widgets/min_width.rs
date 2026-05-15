//! Wrapper widget that enforces a minimum width on its child.
//!
//! Iced's flex layout sets `min == max` for `Fill` children, so
//! `Limits::min_width()` alone has no effect. This widget overrides
//! the limits to force both min and max to the floor value when flex
//! allocates less than the minimum. The resulting node is wider than
//! what flex allocated, causing the parent Row to overflow — which
//! the DataTable handles with horizontal scrolling.
//!
//! ```rust,ignore
//! MinWidth::new(my_element, 180.0)
//! ```

use iced::advanced::layout::{self, Layout, Node};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size, Theme, Vector};

/// A wrapper that enforces a minimum width on its child element.
///
/// Fully transparent to the widget tree — delegates tag, state, and children
/// directly to the inner widget so iced's tree diffing sees no extra layer.
pub struct MinWidth<'a, Message> {
    child: Element<'a, Message>,
    min: f32,
}

impl<'a, Message> MinWidth<'a, Message> {
    pub fn new(child: impl Into<Element<'a, Message>>, min: f32) -> Self {
        Self {
            child: child.into(),
            min,
        }
    }
}

impl<'a, Message> Widget<Message, Theme, iced::Renderer> for MinWidth<'a, Message> {
    fn tag(&self) -> widget::tree::Tag {
        self.child.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.child.as_widget().state()
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.child.as_widget().children()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        self.child.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.child.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> Node {
        let constrained = if limits.max().width < self.min {
            // Flex gave us less than the minimum — override both min and max
            // so the child resolves to exactly `self.min`. The oversized node
            // causes the parent Row to overflow (handled by horizontal scroll).
            layout::Limits::new(
                Size::new(self.min, limits.min().height),
                Size::new(self.min, limits.max().height),
            )
        } else {
            // Enough space — just raise the floor (mostly a no-op for Fill).
            limits.min_width(self.min)
        };
        self.child
            .as_widget_mut()
            .layout(tree, renderer, &constrained)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.child
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.child
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.child.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.child
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.child
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message: 'a> From<MinWidth<'a, Message>> for Element<'a, Message> {
    fn from(w: MinWidth<'a, Message>) -> Self {
        Element::new(w)
    }
}
