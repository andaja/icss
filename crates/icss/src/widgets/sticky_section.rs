//! Custom widget: sticky header within a scrollable.
//!
//! Uses the Widget::overlay() method to render the header above the
//! scrollable's clip region when it's been scrolled past.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse};

pub struct StickySection<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    header: Element<'a, Message, Theme, Renderer>,
    body: Element<'a, Message, Theme, Renderer>,
    width: Length,
}

impl<'a, Message, Theme, Renderer> StickySection<'a, Message, Theme, Renderer> {
    pub fn new(
        header: impl Into<Element<'a, Message, Theme, Renderer>>,
        body: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            header: header.into(),
            body: body.into(),
            width: Length::Fill,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for StickySection<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.header), Tree::new(&self.body)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.header, &self.body]);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);

        let header_node =
            self.header
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits);
        let header_h = header_node.size().height;

        let body_node = self
            .body
            .as_widget_mut()
            .layout(&mut tree.children[1], renderer, &limits);

        let total_w = header_node.size().width.max(body_node.size().width);
        let total_h = header_h + body_node.size().height;

        layout::Node::with_children(
            Size::new(total_w, total_h),
            vec![
                header_node.move_to(Point::new(0.0, 0.0)),
                body_node.move_to(Point::new(0.0, header_h)),
            ],
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        let header_layout = children.next().unwrap();
        let body_layout = children.next().unwrap();

        // Always draw header at its normal position.
        // When sticky, the overlay will draw it again on top.
        // The normal header is clipped by the scrollable when above viewport — invisible.
        self.header.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            header_layout,
            cursor,
            viewport,
        );

        self.body.as_widget().draw(
            &tree.children[1],
            renderer,
            theme,
            style,
            body_layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = layout.children();
        let header_layout = children.next().unwrap();
        let body_layout = children.next().unwrap();

        let header_bounds = header_layout.bounds();
        let body_bounds = body_layout.bounds();

        // Screen position = content position + translation.
        let header_screen_y = header_bounds.y + translation.y;
        let body_bottom_screen = body_bounds.y + body_bounds.height + translation.y;

        // Sticky when header is above the visible viewport top AND body still visible.
        // Using viewport.y (not 0.0) so the header docks below any fixed UI above
        // the scrollable (tabs, toolbars, etc.) rather than overlapping them.
        let dock_y = viewport.y;
        let should_stick =
            header_screen_y < dock_y && body_bottom_screen > dock_y + header_bounds.height;

        if should_stick {
            let overlay_x = header_bounds.x + translation.x;

            // Pre-compute the layout for the overlay header.
            let limits =
                layout::Limits::new(Size::ZERO, Size::new(header_bounds.width, f32::INFINITY))
                    .width(Length::Fixed(header_bounds.width));
            let node = self
                .header
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits);
            let node = node.move_to(Point::new(overlay_x, dock_y));

            Some(overlay::Element::new(Box::new(StickyOverlay {
                header: &self.header,
                tree: &tree.children[0],
                cached_node: node,
            })))
        } else {
            None
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        let header_layout = children.next().unwrap();
        let body_layout = children.next().unwrap();

        self.header.as_widget_mut().update(
            &mut tree.children[0],
            event,
            header_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        self.body.as_widget_mut().update(
            &mut tree.children[1],
            event,
            body_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children();
        let header_layout = children.next().unwrap();
        let body_layout = children.next().unwrap();

        let h = self.header.as_widget().mouse_interaction(
            &tree.children[0],
            header_layout,
            cursor,
            viewport,
            renderer,
        );

        if h != mouse::Interaction::None {
            return h;
        }

        self.body.as_widget().mouse_interaction(
            &tree.children[1],
            body_layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut children = layout.children();
        let header_layout = children.next().unwrap();
        let body_layout = children.next().unwrap();

        self.header.as_widget_mut().operate(
            &mut tree.children[0],
            header_layout,
            renderer,
            operation,
        );
        self.body
            .as_widget_mut()
            .operate(&mut tree.children[1], body_layout, renderer, operation);
    }
}

// ── Overlay that draws the header at a fixed screen position ──

struct StickyOverlay<'a, Message, Theme, Renderer> {
    header: &'a Element<'a, Message, Theme, Renderer>,
    tree: &'a Tree,
    /// Pre-computed layout node (with children) from the widget's own layout pass.
    cached_node: layout::Node,
}

impl<'a, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for StickyOverlay<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        self.cached_node.clone()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.header.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }
}

impl<'a, Message, Theme, Renderer> From<StickySection<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(sticky: StickySection<'a, Message, Theme, Renderer>) -> Self {
        Element::new(sticky)
    }
}
