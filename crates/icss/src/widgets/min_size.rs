//! Wrapper widget that enforces minimum width **and** height on its child.
//!
//! Iced's flex layout sets `min == max` for `Fill` children, so
//! `Limits::min_width()` / `min_height()` alone have no effect. This widget
//! overrides the limits to force both min and max to the floor values when
//! flex allocates less than the minimum. The resulting node is larger than
//! what flex allocated, causing the parent to overflow — which a `Scrollable`
//! or `HOverflow` parent can handle gracefully.
//!
//! Use this to protect interactive controls (buttons, inputs, pick-lists)
//! from being squished to unusable sizes by their container.
//!
//! ```rust,ignore
//! MinSize::new(my_element, 100.0, 32.0)
//! ```

use iced::advanced::layout::{self, Layout, Node};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size, Theme, Vector};

/// A wrapper that enforces minimum width and height on its child element.
///
/// Fully transparent to the widget tree — delegates tag, state, and children
/// directly to the inner widget so iced's tree diffing sees no extra layer.
pub struct MinSize<'a, Message> {
    child: Element<'a, Message>,
    min_w: f32,
    min_h: f32,
}

impl<'a, Message> MinSize<'a, Message> {
    pub fn new(child: impl Into<Element<'a, Message>>, min_w: f32, min_h: f32) -> Self {
        Self {
            child: child.into(),
            min_w,
            min_h,
        }
    }

    /// Enforce only minimum width (height unconstrained).
    pub fn width(child: impl Into<Element<'a, Message>>, min_w: f32) -> Self {
        Self::new(child, min_w, 0.0)
    }

    /// Enforce only minimum height (width unconstrained).
    pub fn height(child: impl Into<Element<'a, Message>>, min_h: f32) -> Self {
        Self::new(child, 0.0, min_h)
    }
}

impl<'a, Message> Widget<Message, Theme, iced::Renderer> for MinSize<'a, Message> {
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
        let cur_min = limits.min();
        let cur_max = limits.max();

        let new_min_w = cur_min.width.max(self.min_w);
        let new_max_w = if cur_max.width < self.min_w {
            self.min_w
        } else {
            cur_max.width
        };

        let new_min_h = cur_min.height.max(self.min_h);
        let new_max_h = if cur_max.height < self.min_h {
            self.min_h
        } else {
            cur_max.height
        };

        let constrained = layout::Limits::new(
            Size::new(new_min_w, new_min_h),
            Size::new(new_max_w, new_max_h),
        );

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

impl<'a, Message: 'a> From<MinSize<'a, Message>> for Element<'a, Message> {
    fn from(w: MinSize<'a, Message>) -> Self {
        Element::new(w)
    }
}
