//! Chrome-style tab bar widget with theme integration, close buttons, drag reorder,
//! and drag-to-detach support.
//!
//! Colors are derived from `crate::theme::Theme` — active tab uses the "primary" button
//! style, inactive uses "ghost", and the bar background comes from the page container.

use crate::theme::Theme as RlTheme;
use iced::advanced::layout::{self, Layout, Node};
use iced::advanced::renderer;
use iced::advanced::svg as iced_svg;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::widgets::mdi;

/// Unique tab identifier.
pub type TabId = usize;

/// Data for a single tab.
#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub closable: bool,
    /// Optional icon rendered before the title (MDI IconData).
    pub icon: Option<&'static str>,
}

/// Persistent drag state — lives in your app `State`.
#[derive(Debug, Clone, Default)]
pub struct TabDragState {
    /// Tab currently being dragged.
    pub dragging: Option<TabId>,
    /// Mouse position when drag started.
    pub drag_start_x: f32,
    pub drag_start_y: f32,
    /// Current mouse position during drag.
    pub drag_current_x: f32,
    pub drag_current_y: f32,
    /// Index the dragged tab started at.
    pub drag_origin_idx: usize,
    /// Whether the tab has been "torn off" (vertical threshold exceeded).
    pub detached: bool,
}

impl TabDragState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Vertical distance before a tab tears off into its own window.
const _DETACH_THRESHOLD: f32 = 20.0;

/// Return a square `Rectangle` of `size` centered within `outer`.
fn centered_icon_rect(outer: Rectangle, size: f32) -> Rectangle {
    Rectangle {
        x: outer.x + (outer.width - size) / 2.0,
        y: outer.y + (outer.height - size) / 2.0,
        width: size,
        height: size,
    }
}

/// Messages produced by the tab bar.
#[derive(Debug, Clone)]
pub enum TabBarAction {
    Select(TabId),
    Close(TabId),
    New,
    DragStart {
        tab: TabId,
        x: f32,
        y: f32,
    },
    DragMove {
        x: f32,
        y: f32,
    },
    DragEnd,
    /// Tab was dragged far enough vertically to detach into a new window.
    Detach {
        tab: TabId,
        x: f32,
        y: f32,
    },
    /// Mouse pressed on empty bar area — app should call window::drag().
    WindowDrag,
    /// Window button pressed.
    WindowClose,
    WindowMinimize,
    WindowMaximize,
    /// Home icon or branding clicked — navigate to home screen.
    Home,
    /// App menu icon clicked — show menu popover.
    AppMenu,
    /// Theme toggle clicked.
    ToggleTheme,
    /// Connected indicator clicked — show connection info.
    ConnectionInfo,
}

/// Layout constants — sizing only, no colors.
#[derive(Debug, Clone)]
pub struct TabBarStyle {
    pub tab_height: f32,
    pub tab_min_width: f32,
    pub tab_max_width: f32,
    pub tab_padding_h: f32,
    pub close_size: f32,
    pub font_size: f32,
    pub tab_gap: f32,
    pub tab_radius: f32,
    /// Left padding before the first tab. Only non-zero when traffic
    /// lights share the horizontal line (Chrome-style) — not the default.
    pub left_pad: f32,
    /// Top padding above the tabs (0 by default; set to the OS title-bar
    /// height if the tab bar shares its row with traffic lights).
    pub top_pad: f32,
    /// Optional branding text rendered in the left_pad area (after traffic lights).
    pub branding: Option<String>,
    /// Show a home icon button after the branding text.
    pub show_home_icon: bool,
    /// Override the home-icon x offset relative to the bar's left edge.
    /// `None` → historical behaviour (home sits right before the first tab,
    /// after the branding text). `Some(x)` → home icon's left edge goes to
    /// `bounds.x + x`, independent of `left_pad` / branding. Used to line
    /// the home icon up with the sidebar's left-edge padding when the bar
    /// doubles as the window titlebar.
    pub home_icon_left: Option<f32>,
    /// Show app menu icon on the right side (before theme toggle).
    pub show_app_menu: bool,
    /// Show dark/light theme toggle on the right side.
    pub show_theme_toggle: bool,
    /// Current dark mode state (for rendering the toggle).
    pub dark_mode: bool,
    /// Show connected indicator icon (true = connected, false = disconnected).
    pub show_connected_indicator: bool,
    /// Whether currently connected to broker.
    pub is_connected: bool,
    /// Base font for tab titles and branding text. `None` → `iced::Font::default()`
    /// (iced's built-in SansSerif). Tab titles override the weight to Medium;
    /// the family/style come from this. Set to the app's platform font so the
    /// tabbar matches the rest of the UI.
    pub font: Option<iced::Font>,
}

impl Default for TabBarStyle {
    fn default() -> Self {
        Self {
            tab_height: 36.0,
            tab_min_width: 80.0,
            tab_max_width: 240.0,
            tab_padding_h: 12.0,
            close_size: 14.0,
            font_size: 13.0,
            tab_gap: 1.0,
            tab_radius: 8.0,
            left_pad: 8.0,
            top_pad: 0.0,
            branding: None,
            show_home_icon: false,
            home_icon_left: None,
            show_app_menu: false,
            show_theme_toggle: false,
            dark_mode: true,
            show_connected_indicator: false,
            is_connected: false,
            font: None,
        }
    }
}

/// Resolved colors extracted from the theme at construction time.
struct ResolvedColors {
    bar_bg: Color,
    tab_bg: Color,
    tab_active_bg: Color,
    tab_hover_bg: Color,
    tab_text: Color,
    tab_active_text: Color,
    tab_border: Color,
    tab_hover_border: Color,
    tab_radius: f32,
    close_color: Color,
    close_hover_color: Color,
    border_color: Color,
    #[allow(dead_code)]
    new_btn_color: Color,
    icon_muted: Color,
}

fn resolve_colors(theme: &RlTheme) -> ResolvedColors {
    // Inactive tab → ghost button style
    let ghost = theme.resolve(&["button", "ghost"], None);
    let tab_bg = RlTheme::resolve_color(&ghost, "background-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::TRANSPARENT);
    let tab_text = RlTheme::resolve_color(&ghost, "color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.6));
    let tab_border = Color::TRANSPARENT;

    // Get radius from default button (used for all tab shapes)
    let default_btn = theme.resolve(&["button", "default"], None);
    let tab_radius = default_btn.length("border-radius").unwrap_or(8.0);

    // Hover → ghost:hover
    let ghost_hover = theme.resolve(
        &["button", "ghost"],
        Some(crate::theme::css::PseudoClass::Hover),
    );
    let tab_hover_bg = RlTheme::resolve_color(&ghost_hover, "background-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.06));
    let tab_hover_border = Color::TRANSPARENT;

    // Active tab → page background (surface-s0) so it merges with content
    let page = theme.resolve(&["page"], None);
    let active_bg = RlTheme::resolve_color(&page, "background-color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(0.08, 0.08, 0.12, 1.0));

    // Bar background → surface-s3 (elevated surface)
    let bar_bg = theme
        .color_var("surface-s3")
        .unwrap_or(Color::from_rgba(0.06, 0.06, 0.08, 1.0));

    // Border from divider
    let divider = theme.resolve(&["divider"], None);
    let border_color = RlTheme::resolve_color(&divider, "color")
        .map(|c| c.to_iced())
        .unwrap_or(Color::from_rgba(1.0, 1.0, 1.0, 0.08));

    ResolvedColors {
        bar_bg,
        tab_bg,
        tab_active_bg: active_bg,
        tab_hover_bg,
        tab_text,
        tab_active_text: tab_text, // same text color for active
        tab_border,
        tab_hover_border,
        tab_radius,
        close_color: Color {
            a: (tab_text.a * 0.8).min(1.0),
            ..tab_text
        },
        close_hover_color: Color { a: 1.0, ..tab_text },
        border_color,
        new_btn_color: Color {
            a: tab_text.a * 0.8,
            ..tab_text
        },
        icon_muted: Color {
            a: tab_text.a * 0.5,
            ..tab_text
        },
    }
}

/// The tab bar widget. Constructed each frame (immediate-mode style).
pub struct TabBar<'a, Message> {
    tabs: Vec<Tab>,
    active: TabId,
    drag: &'a TabDragState,
    layout: TabBarStyle,
    colors: ResolvedColors,
    on_action: Box<dyn Fn(TabBarAction) -> Message + 'a>,
}

impl<'a, Message> TabBar<'a, Message> {
    pub fn new(
        tabs: impl Into<Vec<Tab>>,
        active: TabId,
        drag: &'a TabDragState,
        theme: &RlTheme,
        on_action: impl Fn(TabBarAction) -> Message + 'a,
    ) -> Self {
        Self {
            tabs: tabs.into(),
            active,
            drag,
            layout: TabBarStyle::default(),
            colors: resolve_colors(theme),
            on_action: Box::new(on_action),
        }
    }

    pub fn layout_style(mut self, layout: TabBarStyle) -> Self {
        self.layout = layout;
        self
    }

    /// Compute the visual order of tabs (accounting for drag reordering).
    /// Tabs only shift when dragged past half a tab width to avoid jitter.
    fn visual_order(&self) -> Vec<usize> {
        let n = self.tabs.len();
        let mut order: Vec<usize> = (0..n).collect();

        if let Some(dragging_id) = self.drag.dragging
            && !self.drag.detached
            && let Some(drag_idx) = self.tabs.iter().position(|t| t.id == dragging_id)
        {
            let dx = self.drag.drag_current_x - self.drag.drag_start_x;
            let tab_w = self.layout.tab_max_width + self.layout.tab_gap;
            // Only shift when past half a tab width (dead zone)
            let shift = if dx.abs() < tab_w * 0.5 {
                0
            } else {
                ((dx - dx.signum() * tab_w * 0.5) / tab_w).round() as i32 + dx.signum() as i32
            };
            let new_idx = (drag_idx as i32 + shift).clamp(0, n as i32 - 1) as usize;
            if new_idx != drag_idx {
                order.remove(drag_idx);
                order.insert(new_idx, drag_idx);
            }
        }

        order
    }

    /// Get tab rect given its visual position index.
    fn tab_rect(&self, visual_pos: usize, bounds: Rectangle) -> Rectangle {
        let s = &self.layout;
        let x = bounds.x + s.left_pad + (visual_pos as f32) * (s.tab_max_width + s.tab_gap);
        Rectangle {
            x,
            y: bounds.y + s.top_pad,
            width: s.tab_max_width,
            height: s.tab_height,
        }
    }

    /// Expanded hit-test rect — includes gap and 1px bottom shift so clicks
    /// anywhere on the visual tab area register, not just the inner rect.
    fn tab_hit_rect(&self, visual_pos: usize, bounds: Rectangle) -> Rectangle {
        let s = &self.layout;
        let x = bounds.x + s.left_pad + (visual_pos as f32) * (s.tab_max_width + s.tab_gap);
        Rectangle {
            x: x - s.tab_gap / 2.0,
            y: bounds.y,
            width: s.tab_max_width + s.tab_gap,
            height: s.top_pad + s.tab_height + 1.0,
        }
    }

    /// Close button rect within a tab rect.
    fn close_rect(&self, tab_rect: Rectangle) -> Rectangle {
        let s = &self.layout;
        Rectangle {
            x: tab_rect.x + tab_rect.width - s.tab_padding_h - s.close_size,
            y: tab_rect.y + (tab_rect.height - s.close_size) / 2.0,
            width: s.close_size,
            height: s.close_size,
        }
    }

    /// Home icon rect (between branding and first tab, or at the
    /// `home_icon_left` override when set).
    fn home_icon_rect(&self, bounds: Rectangle) -> Rectangle {
        let s = &self.layout;
        let x = match s.home_icon_left {
            Some(left) => bounds.x + left,
            None => bounds.x + s.left_pad - 30.0,
        };
        Rectangle {
            x,
            y: bounds.y + s.top_pad + (s.tab_height - 24.0) / 2.0,
            width: 24.0,
            height: 24.0,
        }
    }

    /// How many pixels are consumed by icons to the right of the app menu.
    fn right_icons_width(&self) -> f32 {
        let mut w = 0.0;
        if self.layout.show_connected_indicator {
            w += 32.0;
        }
        if self.layout.show_theme_toggle {
            w += 36.0;
        }
        w
    }

    /// App menu rect (right side, before theme toggle).
    fn app_menu_rect(&self, bounds: Rectangle) -> Rectangle {
        let s = &self.layout;
        let x = bounds.x + bounds.width - 36.0 - self.right_icons_width();
        Rectangle {
            x,
            y: bounds.y + s.top_pad + (s.tab_height - 24.0) / 2.0,
            width: 28.0,
            height: 24.0,
        }
    }

    /// Theme toggle rect (right side of bar).
    fn theme_toggle_rect(&self, bounds: Rectangle) -> Rectangle {
        let s = &self.layout;
        let indicator_space = if self.layout.show_connected_indicator {
            32.0
        } else {
            0.0
        };
        let x = bounds.x + bounds.width - 36.0 - indicator_space;
        Rectangle {
            x,
            y: bounds.y + s.top_pad + (s.tab_height - 24.0) / 2.0,
            width: 28.0,
            height: 24.0,
        }
    }

    /// Connected indicator rect (right side of bar, icon only).
    fn indicator_rect(&self, bounds: Rectangle) -> Rectangle {
        let s = &self.layout;
        let x = bounds.x + bounds.width - 32.0;
        Rectangle {
            x,
            y: bounds.y + s.top_pad + (s.tab_height - 24.0) / 2.0,
            width: 24.0,
            height: 24.0,
        }
    }

    /// Draw a single tab at the given visual position.
    fn draw_tab(
        &self,
        renderer: &mut Renderer,
        state: &TabBarState,
        vis_pos: usize,
        data_idx: usize,
        bounds: Rectangle,
    ) {
        use iced::advanced::graphics::core::renderer::Renderer as _;
        use iced::advanced::text::Renderer as TextRenderer;
        use iced_svg::Renderer as SvgRenderer;

        let tab = &self.tabs[data_idx];
        let s = &self.layout;
        let c = &self.colors;
        let is_active = tab.id == self.active;
        let is_hovered = state.hovered_tab == Some(tab.id);
        let is_dragging = self.drag.dragging == Some(tab.id);

        // Base rect. For the dragged tab, ignore the reordered visual position
        // and follow the cursor from the ORIGINAL slot — otherwise the tab jumps
        // one slot-width whenever `visual_order()` shifts it.
        let rect = if is_dragging && !self.drag.detached {
            let mut r = self.tab_rect(self.drag.drag_origin_idx, bounds);
            r.x += self.drag.drag_current_x - self.drag.drag_start_x;
            r
        } else {
            self.tab_rect(vis_pos, bounds)
        };

        // Background color
        let bg = if is_active {
            c.tab_active_bg
        } else if is_hovered {
            c.tab_hover_bg
        } else {
            c.tab_bg
        };

        let border_color = if is_hovered && !is_active {
            c.tab_hover_border
        } else {
            c.tab_border
        };

        // All tabs: top corners rounded, bottom straight.
        let r = c.tab_radius;
        let radius = iced::border::Radius {
            top_left: r,
            top_right: r,
            bottom_right: 0.0,
            bottom_left: 0.0,
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: rect,
                border: Border {
                    radius,
                    width: if border_color.a > 0.0 { 1.0 } else { 0.0 },
                    color: border_color,
                },
                ..renderer::Quad::default()
            },
            bg,
        );

        // Tab icon (optional, rendered before title)
        let icon_space = if tab.icon.is_some() { 18.0 } else { 0.0 };
        if let Some(icon_data) = tab.icon {
            let icon_color = if is_active {
                c.tab_active_text
            } else {
                c.icon_muted
            };
            let icon_rect = Rectangle {
                x: rect.x + s.tab_padding_h,
                y: rect.y + (rect.height - 14.0) / 2.0,
                width: 14.0,
                height: 14.0,
            };
            renderer.draw_svg(
                mdi::icon_svg_sw(icon_data, 1.5, Some(icon_color)),
                icon_rect,
                rect,
            );
        }

        // Title text — reserve space for icon and close button
        let text_color = if is_active {
            c.tab_active_text
        } else {
            c.tab_text
        };
        let text_x = rect.x + s.tab_padding_h + icon_space;
        let text_w = if tab.closable {
            rect.width - s.tab_padding_h * 2.0 - s.close_size - 4.0 - icon_space
        } else {
            rect.width - s.tab_padding_h * 2.0 - icon_space
        };

        // NOTE: iced's `fill_text` treats `position` as an ANCHOR per alignment:
        //   Left → position.x is the text's left edge
        //   Center (horizontal) → position.x is the text's horizontal center
        //   Top → position.y is the text's top edge
        //   Center (vertical) → position.y is the text's vertical center
        //   Bottom → position.y is the text's bottom edge
        // So for Left + Center, we pass (left-edge, vertical-center).
        renderer.fill_text(
            iced::advanced::Text {
                content: tab.title.clone(),
                bounds: Size::new(text_w.max(0.0), rect.height),
                size: s.font_size.into(),
                line_height: iced::widget::text::LineHeight::default(),
                font: iced::Font {
                    weight: iced::font::Weight::Medium,
                    ..self.layout.font.unwrap_or_default()
                },
                align_x: iced::widget::text::Alignment::Left,
                align_y: iced::alignment::Vertical::Center,
                shaping: iced::widget::text::Shaping::Basic,
                wrapping: iced::widget::text::Wrapping::None,
            },
            Point::new(text_x, rect.y + rect.height / 2.0),
            text_color,
            rect,
        );

        // Close button (×)
        if tab.closable {
            let cr = self.close_rect(rect);
            let close_hovered = state.hovered_close == Some(tab.id);

            if close_hovered {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: cr,
                        border: Border {
                            radius: (cr.width / 2.0).into(),
                            ..Default::default()
                        },
                        ..renderer::Quad::default()
                    },
                    Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                );
            }

            let icon_size = s.close_size;
            let icon_rect = centered_icon_rect(cr, icon_size);
            let close_color = if close_hovered {
                c.close_hover_color
            } else {
                c.close_color
            };
            renderer.draw_svg(mdi::icon_svg(mdi::X, Some(close_color)), icon_rect, cr);
        }
    }
}

#[derive(Default)]
struct TabBarState {
    hovered_tab: Option<TabId>,
    hovered_close: Option<TabId>,
    hovered_home: bool,
    hovered_app_menu: bool,
    hovered_theme: bool,
    hovered_indicator: bool,
    /// Timestamp of last click on empty bar area (for double-click detection).
    last_empty_click: Option<std::time::Instant>,
}

impl<'a, Message: Clone> Widget<Message, Theme, Renderer> for TabBar<'a, Message> {
    fn size(&self) -> Size<Length> {
        let total_h = self.layout.top_pad + self.layout.tab_height;
        Size {
            width: Length::Fill,
            height: Length::Fixed(total_h),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> Node {
        let total_h = self.layout.top_pad + self.layout.tab_height;
        let limits = limits.width(Length::Fill).height(Length::Fixed(total_h));
        Node::new(Size::new(limits.max().width, total_h))
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<TabBarState>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(TabBarState::default())
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<TabBarState>();
        let order = self.visual_order();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                state.hovered_tab = None;
                state.hovered_close = None;
                state.hovered_home = false;
                state.hovered_app_menu = false;
                state.hovered_theme = false;
                state.hovered_indicator = false;

                if let Some(pos) = cursor.position_in(bounds) {
                    let abs = Point::new(bounds.x + pos.x, bounds.y + pos.y);

                    // Check home icon
                    if self.layout.show_home_icon && self.home_icon_rect(bounds).contains(abs) {
                        state.hovered_home = true;
                    }

                    // Check branding area (clickable for home)
                    if self.layout.branding.is_some() {
                        let br = Rectangle {
                            x: bounds.x + 78.0,
                            y: bounds.y + self.layout.top_pad,
                            width: self.layout.left_pad - 78.0 - 30.0,
                            height: self.layout.tab_height,
                        };
                        if br.contains(abs) {
                            state.hovered_home = true;
                        }
                    }

                    // Check app menu
                    if self.layout.show_app_menu && self.app_menu_rect(bounds).contains(abs) {
                        state.hovered_app_menu = true;
                    }

                    // Check theme toggle
                    if self.layout.show_theme_toggle && self.theme_toggle_rect(bounds).contains(abs)
                    {
                        state.hovered_theme = true;
                    }

                    // Check connected indicator
                    if self.layout.show_connected_indicator
                        && self.indicator_rect(bounds).contains(abs)
                    {
                        state.hovered_indicator = true;
                    }

                    // Check tabs — use expanded hit rect for detection
                    for (vis_pos, &data_idx) in order.iter().enumerate() {
                        let tab = &self.tabs[data_idx];
                        let hit = self.tab_hit_rect(vis_pos, bounds);
                        if hit.contains(abs) {
                            state.hovered_tab = Some(tab.id);
                            if tab.closable {
                                // Close rect uses visual tab_rect for positioning
                                let visual = self.tab_rect(vis_pos, bounds);
                                let cr = self.close_rect(visual);
                                if cr.contains(abs) {
                                    state.hovered_close = Some(tab.id);
                                }
                            }
                            break;
                        }
                    }
                }

                // Drag move — use event position (window-global) so it works
                // even when cursor has left the widget bounds during vertical drag.
                if self.drag.dragging.is_some() && !self.drag.detached {
                    shell.publish((self.on_action)(TabBarAction::DragMove {
                        x: position.x,
                        y: position.y,
                    }));
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let abs = Point::new(bounds.x + pos.x, bounds.y + pos.y);

                    // Home icon or branding click
                    if self.layout.show_home_icon && self.home_icon_rect(bounds).contains(abs) {
                        shell.publish((self.on_action)(TabBarAction::Home));
                        return;
                    }
                    if self.layout.branding.is_some() {
                        let br = Rectangle {
                            x: bounds.x + 78.0,
                            y: bounds.y + self.layout.top_pad,
                            width: self.layout.left_pad - 78.0 - 30.0,
                            height: self.layout.tab_height,
                        };
                        if br.contains(abs) {
                            shell.publish((self.on_action)(TabBarAction::Home));
                            return;
                        }
                    }

                    // App menu
                    if self.layout.show_app_menu && self.app_menu_rect(bounds).contains(abs) {
                        shell.publish((self.on_action)(TabBarAction::AppMenu));
                        return;
                    }

                    // Connected indicator
                    if self.layout.show_connected_indicator
                        && self.indicator_rect(bounds).contains(abs)
                    {
                        shell.publish((self.on_action)(TabBarAction::ConnectionInfo));
                        return;
                    }

                    // Theme toggle
                    if self.layout.show_theme_toggle && self.theme_toggle_rect(bounds).contains(abs)
                    {
                        shell.publish((self.on_action)(TabBarAction::ToggleTheme));
                        return;
                    }

                    // Tab close or select + start drag — use expanded hit rect
                    for (vis_pos, &data_idx) in order.iter().enumerate() {
                        let tab = &self.tabs[data_idx];
                        let hit = self.tab_hit_rect(vis_pos, bounds);
                        if hit.contains(abs) {
                            if tab.closable {
                                let visual = self.tab_rect(vis_pos, bounds);
                                let cr = self.close_rect(visual);
                                if cr.contains(abs) {
                                    shell.publish((self.on_action)(TabBarAction::Close(tab.id)));
                                    return;
                                }
                            }
                            shell.publish((self.on_action)(TabBarAction::Select(tab.id)));
                            shell.publish((self.on_action)(TabBarAction::DragStart {
                                tab: tab.id,
                                x: abs.x,
                                y: abs.y,
                            }));
                            return;
                        }
                    }

                    // Click on empty bar area: double-click → toggle maximize,
                    // single click → drag the window.
                    let now = std::time::Instant::now();
                    if let Some(prev) = state.last_empty_click
                        && now.duration_since(prev).as_millis() < 400
                    {
                        state.last_empty_click = None;
                        shell.publish((self.on_action)(TabBarAction::WindowMaximize));
                        return;
                    }
                    state.last_empty_click = Some(now);
                    shell.publish((self.on_action)(TabBarAction::WindowDrag));
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if self.drag.dragging.is_some() =>
            {
                shell.publish((self.on_action)(TabBarAction::DragEnd));
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::graphics::core::renderer::Renderer as _;
        use iced::advanced::text::Renderer as TextRenderer;
        use iced_svg::Renderer as SvgRenderer;

        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<TabBarState>();
        let c = &self.colors;
        let order = self.visual_order();

        // Bar background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default(),
                ..renderer::Quad::default()
            },
            c.bar_bg,
        );

        // Branding text in the left-pad area (after traffic lights).
        if let Some(ref label) = self.layout.branding {
            let label_x = bounds.x + 78.0; // after traffic lights
            let label_y = bounds.y + self.layout.top_pad;
            let label_w = self.layout.left_pad - 78.0;
            let label_h = self.layout.tab_height;
            if label_w > 10.0 {
                renderer.fill_text(
                    iced::advanced::Text {
                        content: label.clone(),
                        bounds: Size::new(label_w, label_h),
                        size: 12.0.into(),
                        line_height: iced::widget::text::LineHeight::default(),
                        font: self.layout.font.unwrap_or_default(),
                        align_x: iced::widget::text::Alignment::Left,
                        align_y: iced::alignment::Vertical::Center,
                        shaping: iced::widget::text::Shaping::Basic,
                        wrapping: iced::widget::text::Wrapping::None,
                    },
                    Point::new(label_x, label_y + label_h / 2.0),
                    Color {
                        a: c.tab_text.a * 0.5,
                        ..c.tab_text
                    },
                    Rectangle {
                        x: label_x,
                        y: label_y,
                        width: label_w,
                        height: label_h,
                    },
                );
            }
        }

        // Bottom border
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y + bounds.height - 1.0,
                    width: bounds.width,
                    height: 1.0,
                },
                border: Border::default(),
                ..renderer::Quad::default()
            },
            c.border_color,
        );

        // Window management buttons are provided by the native OS (traffic
        // lights on macOS) — the widget only reserves space via `left_pad`.

        // Draw tabs — inactive first, then active tab in a fresh layer so it
        // ALWAYS renders above the others. Without a new layer, the text-pass
        // in iced's renderer can paint inactive-tab glyphs on top of the
        // active tab when drawn/clip rects overlap (during drag/reorder).
        let active_entry = order
            .iter()
            .enumerate()
            .find(|(_, di)| self.tabs[**di].id == self.active);
        let active_vis_pos = active_entry.map(|(vp, _)| vp);

        // Pass 1: inactive tabs
        for (vis_pos, &data_idx) in order.iter().enumerate() {
            if Some(vis_pos) == active_vis_pos {
                continue;
            }
            self.draw_tab(renderer, state, vis_pos, data_idx, bounds);
        }
        // Pass 2: active tab on its own layer (z-order over inactive tabs)
        if let Some((vis_pos, &data_idx)) = active_entry {
            renderer.with_layer(bounds, |r| {
                self.draw_tab(r, state, vis_pos, data_idx, bounds);
            });
        }

        // Home icon (house symbol ⌂)
        if self.layout.show_home_icon {
            let hr = self.home_icon_rect(bounds);
            if state.hovered_home {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: hr,
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..renderer::Quad::default()
                    },
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                );
            }
            let home_color = if state.hovered_home {
                Color::WHITE
            } else {
                c.icon_muted
            };
            let icon_rect = centered_icon_rect(hr, 16.0);
            renderer.draw_svg(
                mdi::icon_svg_sw(mdi::HOUSE, 1.0, Some(home_color)),
                icon_rect,
                hr,
            );
        }

        // App menu icon (right side, before theme toggle)
        if self.layout.show_app_menu {
            let ar = self.app_menu_rect(bounds);
            if state.hovered_app_menu {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: ar,
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..renderer::Quad::default()
                    },
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                );
            }
            let menu_color = if state.hovered_app_menu {
                Color::WHITE
            } else {
                c.icon_muted
            };
            let icon_rect = centered_icon_rect(ar, 16.0);
            renderer.draw_svg(
                mdi::icon_svg(mdi::ELLIPSIS_VERTICAL, Some(menu_color)),
                icon_rect,
                ar,
            );
        }

        // Theme toggle (right side)
        if self.layout.show_theme_toggle {
            let tr = self.theme_toggle_rect(bounds);
            if state.hovered_theme {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: tr,
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..renderer::Quad::default()
                    },
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                );
            }
            let icon_data = if self.layout.dark_mode {
                mdi::MOON
            } else {
                mdi::SUN
            };
            let theme_color = if state.hovered_theme {
                Color::WHITE
            } else {
                c.icon_muted
            };
            let icon_rect = centered_icon_rect(tr, 16.0);
            renderer.draw_svg(
                mdi::icon_svg_sw(icon_data, 1.0, Some(theme_color)),
                icon_rect,
                tr,
            );
        }

        // Connected indicator (right side, icon only)
        if self.layout.show_connected_indicator {
            let ir = self.indicator_rect(bounds);
            let color = if self.layout.is_connected {
                Color::from_rgb(0.2, 0.8, 0.3)
            } else {
                c.icon_muted
            };
            let icon_rect = centered_icon_rect(ir, 16.0);
            renderer.draw_svg(mdi::icon_svg(mdi::WIFI, Some(color)), icon_rect, ir);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<TabBarState>();

        if self.drag.dragging.is_some() {
            return mouse::Interaction::Grabbing;
        }

        if cursor.position_in(bounds).is_some() {
            if state.hovered_close.is_some()
                || state.hovered_home
                || state.hovered_app_menu
                || state.hovered_theme
                || state.hovered_indicator
            {
                return mouse::Interaction::Pointer;
            }
            if state.hovered_tab.is_some() {
                return mouse::Interaction::Pointer;
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'a> From<TabBar<'a, Message>> for Element<'a, Message> {
    fn from(tab_bar: TabBar<'a, Message>) -> Self {
        Self::new(tab_bar)
    }
}
