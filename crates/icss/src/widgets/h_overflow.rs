//! Horizontal overflow scroll wrapper — CSS `overflow-x: auto` equivalent.
//!
//! When the available width is at least `min_content_width`, the child fills
//! the viewport normally (Fill columns expand). When the available width is
//! smaller, the child is laid out at exactly `min_content_width` and scrolls
//! horizontally via mouse wheel / trackpad. A thin scrollbar appears at the
//! bottom of the viewport whenever the content overflows.
//!
//! This exists because iced's `scrollable().horizontal()` passes infinite
//! width to its child, collapsing `Length::Fill` content. `HOverflow` instead
//! gives the child `max(viewport_w, min_content_width)`, which preserves the
//! "stretch when room, scroll when not" behavior of CSS overflow-x: auto.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay};
use iced::{
    Background, Color, Element, Event, Length, Point, Rectangle, Size, Vector, border, mouse,
};

const SCROLLBAR_HEIGHT: f32 = 6.0;
const SCROLLBAR_MARGIN: f32 = 2.0;
const WHEEL_LINE_PIXELS: f32 = 32.0;

/// A wrapper that gives its child at least `min_content_width` of layout
/// width and clips/scrolls horizontally when the viewport is narrower.
pub struct HOverflow<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    child: Element<'a, Message, Theme, Renderer>,
    min_content_width: f32,
}

impl<'a, Message, Theme, Renderer> HOverflow<'a, Message, Theme, Renderer> {
    pub fn new(
        child: impl Into<Element<'a, Message, Theme, Renderer>>,
        min_content_width: f32,
    ) -> Self {
        Self {
            child: child.into(),
            min_content_width,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct State {
    scroll_x: f32,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for HOverflow<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.child]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let viewport_w = limits.max().width;
        let content_w = viewport_w.max(self.min_content_width);

        let child_limits = layout::Limits::new(
            Size::new(content_w, limits.min().height),
            Size::new(content_w, limits.max().height),
        );

        let child_node =
            self.child
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &child_limits);
        let child_h = child_node.size().height;

        // Clamp the persistent scroll offset to the new valid range so a
        // resize never leaves us scrolled past the end.
        let state = tree.state.downcast_mut::<State>();
        let max_scroll = (content_w - viewport_w).max(0.0);
        if state.scroll_x > max_scroll {
            state.scroll_x = max_scroll;
        }
        if state.scroll_x < 0.0 {
            state.scroll_x = 0.0;
        }

        let positioned = child_node.move_to(Point::new(-state.scroll_x, 0.0));

        layout::Node::with_children(Size::new(viewport_w, child_h), vec![positioned])
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
        let bounds = layout.bounds();
        let child_layout = layout.children().next().unwrap();
        let state = tree.state.downcast_ref::<State>();

        // Clip child drawing to our viewport so overflow stays hidden.
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        renderer.with_layer(clip, |renderer| {
            self.child.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                &clip,
            );
        });

        // Thin scrollbar at the bottom — only when overflow exists.
        let content_w = child_layout.bounds().width;
        if content_w > bounds.width + 0.5 {
            let track_w = bounds.width;
            let ratio = (bounds.width / content_w).clamp(0.0, 1.0);
            let thumb_w = (track_w * ratio).max(20.0);
            let max_scroll = content_w - bounds.width;
            let progress = if max_scroll > 0.0 {
                (state.scroll_x / max_scroll).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let thumb_x = bounds.x + progress * (track_w - thumb_w);
            let thumb_y = bounds.y + bounds.height - SCROLLBAR_HEIGHT - SCROLLBAR_MARGIN;

            renderer.with_layer(clip, |renderer| {
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: thumb_x,
                            y: thumb_y,
                            width: thumb_w,
                            height: SCROLLBAR_HEIGHT,
                        },
                        border: border::rounded(SCROLLBAR_HEIGHT / 2.0),
                        ..Quad::default()
                    },
                    Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.28)),
                );
            });
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
        let bounds = layout.bounds();
        let child_layout = layout.children().next().unwrap();
        let content_w = child_layout.bounds().width;
        let max_scroll = (content_w - bounds.width).max(0.0);

        // Intercept horizontal wheel events when content overflows and the
        // cursor is over us. We only consume horizontal deltas — vertical
        // wheel scrolling stays available for an outer scrollable.
        if max_scroll > 0.0
            && let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event
            && cursor.is_over(bounds)
        {
            let dx = match delta {
                mouse::ScrollDelta::Lines { x, .. } => -x * WHEEL_LINE_PIXELS,
                mouse::ScrollDelta::Pixels { x, .. } => -x,
            };
            if dx.abs() > 0.0 {
                let state = tree.state.downcast_mut::<State>();
                let new_scroll = (state.scroll_x + dx).clamp(0.0, max_scroll);
                if (new_scroll - state.scroll_x).abs() > 0.01 {
                    state.scroll_x = new_scroll;
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }
            }
        }

        // Forward all other events. Hide the cursor from the child when
        // it's outside the visible viewport so clipped cells don't receive
        // hits via their (translated) absolute bounds.
        let child_cursor = if cursor.is_over(bounds) {
            cursor
        } else {
            mouse::Cursor::Unavailable
        };

        self.child.as_widget_mut().update(
            &mut tree.children[0],
            event,
            child_layout,
            child_cursor,
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
        let bounds = layout.bounds();
        if !cursor.is_over(bounds) {
            return mouse::Interaction::None;
        }
        let child_layout = layout.children().next().unwrap();
        self.child.as_widget().mouse_interaction(
            &tree.children[0],
            child_layout,
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
        let child_layout = layout.children().next().unwrap();
        self.child.as_widget_mut().operate(
            &mut tree.children[0],
            child_layout,
            renderer,
            operation,
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
        let child_layout = layout.children().next().unwrap();
        self.child.as_widget_mut().overlay(
            &mut tree.children[0],
            child_layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<HOverflow<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(w: HOverflow<'a, Message, Theme, Renderer>) -> Self {
        Element::new(w)
    }
}
