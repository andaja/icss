//! HSV color picker: hue slider + saturation/value square + preview.

use iced::mouse;
use iced::widget::Action;
use iced::widget::canvas::{self, Canvas, Frame, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

/// HSV color state.
#[derive(Debug, Clone, Copy)]
pub struct HsvColor {
    pub h: f32, // 0–360
    pub s: f32, // 0–1
    pub v: f32, // 0–1
}

impl HsvColor {
    pub fn to_rgb(self) -> Color {
        hsv_to_rgb(self.h, self.s, self.v)
    }

    pub fn to_hex(self) -> String {
        let c = self.to_rgb();
        let r = (c.r * 255.0) as u8;
        let g = (c.g * 255.0) as u8;
        let b = (c.b * 255.0) as u8;
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() < 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(rgb_to_hsv(r, g, b))
    }
}

// ── SV Square ──

pub struct SvSquare {
    pub hue: f32,
    pub sat: f32,
    pub val: f32,
}

#[derive(Default)]
pub struct SvSquareState {
    dragging: bool,
}

#[derive(Debug, Clone)]
pub enum SvMsg {
    Changed(f32, f32),
}

impl canvas::Program<SvMsg> for SvSquare {
    type State = SvSquareState;

    fn draw(
        &self,
        _state: &SvSquareState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;
        let steps = 48;
        let cell_w = w / steps as f32;
        let cell_h = h / steps as f32;

        for xi in 0..steps {
            for yi in 0..steps {
                let s = (xi as f32 + 0.5) / steps as f32;
                let v = 1.0 - (yi as f32 + 0.5) / steps as f32;
                let color = hsv_to_rgb(self.hue, s, v);
                frame.fill_rectangle(
                    Point::new(xi as f32 * cell_w, yi as f32 * cell_h),
                    Size::new(cell_w + 0.5, cell_h + 0.5),
                    color,
                );
            }
        }

        // Crosshair at current position.
        let cx = self.sat * w;
        let cy = (1.0 - self.val) * h;
        let ring = Path::circle(Point::new(cx, cy), 5.0);
        frame.stroke(
            &ring,
            canvas::Stroke::default()
                .with_color(Color::WHITE)
                .with_width(2.0),
        );
        let ring2 = Path::circle(Point::new(cx, cy), 6.0);
        frame.stroke(
            &ring2,
            canvas::Stroke::default()
                .with_color(Color::BLACK)
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut SvSquareState,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<SvMsg>> {
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.dragging = true;
                    let s = (pos.x / bounds.width).clamp(0.0, 1.0);
                    let v = 1.0 - (pos.y / bounds.height).clamp(0.0, 1.0);
                    return Some(Action::publish(SvMsg::Changed(s, v)));
                }
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging
                    && let Some(pos) = cursor.position_in(bounds)
                {
                    let s = (pos.x / bounds.width).clamp(0.0, 1.0);
                    let v = 1.0 - (pos.y / bounds.height).clamp(0.0, 1.0);
                    return Some(Action::publish(SvMsg::Changed(s, v)));
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = false;
            }
            _ => {}
        }
        None
    }
}

pub fn sv_square(hue: f32, sat: f32, val: f32) -> Element<'static, SvMsg> {
    Canvas::new(SvSquare { hue, sat, val })
        .width(Length::Fill)
        .height(150)
        .into()
}

// ── Hue Bar ──

pub struct HueBar {
    pub hue: f32,
}

#[derive(Default)]
pub struct HueBarState {
    dragging: bool,
}

#[derive(Debug, Clone)]
pub enum HueMsg {
    Changed(f32),
}

impl canvas::Program<HueMsg> for HueBar {
    type State = HueBarState;

    fn draw(
        &self,
        _state: &HueBarState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;
        let steps = 72;
        let cell_w = w / steps as f32;

        for i in 0..steps {
            let hue = (i as f32 / steps as f32) * 360.0;
            let color = hsv_to_rgb(hue, 1.0, 1.0);
            frame.fill_rectangle(
                Point::new(i as f32 * cell_w, 0.0),
                Size::new(cell_w + 0.5, h),
                color,
            );
        }

        // Marker at current hue.
        let mx = (self.hue / 360.0) * w;
        let marker = Path::rectangle(Point::new(mx - 2.0, 0.0), Size::new(4.0, h));
        frame.stroke(
            &marker,
            canvas::Stroke::default()
                .with_color(Color::WHITE)
                .with_width(2.0),
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut HueBarState,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<HueMsg>> {
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.dragging = true;
                    let h = (pos.x / bounds.width).clamp(0.0, 1.0) * 360.0;
                    return Some(Action::publish(HueMsg::Changed(h)));
                }
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging
                    && let Some(pos) = cursor.position_in(bounds)
                {
                    let h = (pos.x / bounds.width).clamp(0.0, 1.0) * 360.0;
                    return Some(Action::publish(HueMsg::Changed(h)));
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = false;
            }
            _ => {}
        }
        None
    }
}

pub fn hue_bar(hue: f32) -> Element<'static, HueMsg> {
    Canvas::new(HueBar { hue })
        .width(Length::Fill)
        .height(20)
        .into()
}

// ── HSV ↔ RGB ──

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let h2 = h / 60.0;
    let x = c * (1.0 - (h2 % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h2 < 1.0 {
        (c, x, 0.0)
    } else if h2 < 2.0 {
        (x, c, 0.0)
    } else if h2 < 3.0 {
        (0.0, c, x)
    } else if h2 < 4.0 {
        (0.0, x, c)
    } else if h2 < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::from_rgb(r + m, g + m, b + m)
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> HsvColor {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;

    let h = if d < 0.00001 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };

    let s = if max < 0.00001 { 0.0 } else { d / max };

    HsvColor { h, s, v: max }
}

// ── Tonal Bar ──

/// Renders a horizontal bar showing all 101 tonal palette steps.
pub struct TonalBar {
    /// 101 entries of [r, g, b] in 0.0–1.0.
    pub palette: Vec<[f32; 3]>,
}

impl canvas::Program<()> for TonalBar {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;
        let n = self.palette.len().max(1);
        let cell_w = w / n as f32;

        for (i, rgb) in self.palette.iter().enumerate() {
            let color = Color::from_rgb(rgb[0], rgb[1], rgb[2]);
            frame.fill_rectangle(
                Point::new(i as f32 * cell_w, 0.0),
                Size::new(cell_w + 0.5, h),
                color,
            );
        }

        // Draw tick marks every 10 steps.
        for i in (0..=100).step_by(10) {
            let x = (i as f32 / 100.0) * w;
            let tick = Path::line(Point::new(x, h - 4.0), Point::new(x, h));
            frame.stroke(
                &tick,
                canvas::Stroke::default()
                    .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.5))
                    .with_width(1.0),
            );
        }

        vec![frame.into_geometry()]
    }
}

pub fn tonal_bar(palette: &[[f32; 3]]) -> Element<'_, ()> {
    Canvas::new(TonalBar {
        palette: palette.to_vec(),
    })
    .width(Length::Fill)
    .height(16)
    .into()
}
