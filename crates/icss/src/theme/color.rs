/// Parse a hex color string into (r, g, b, a).
///
/// Supports: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`
pub fn parse_hex_color(s: &str) -> Option<(u8, u8, u8, u8)> {
    let s = s.strip_prefix('#')?;
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some((r, g, b, 255))
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            let a = u8::from_str_radix(&s[3..4], 16).ok()? * 17;
            Some((r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some((r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some((r, g, b, a))
        }
        _ => None,
    }
}

/// Color with alpha, convertible to iced::Color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ThemeColor {
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let (r, g, b, a) = parse_hex_color(hex)?;
        Some(Self::from_rgba8(r, g, b, a))
    }

    pub fn to_iced(self) -> iced::Color {
        iced::Color::from_rgba(self.r, self.g, self.b, self.a)
    }

    /// Construct from float RGBA (0.0–1.0 each).
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// The transparent color.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Serialize to a CSS rgba() string.
    pub fn to_css_string(self) -> String {
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        if (self.a - 1.0).abs() < f32::EPSILON {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!("rgba({r}, {g}, {b}, {:.2})", self.a)
        }
    }

    /// Darken by a percentage (0.0–1.0).
    pub fn darken(self, amount: f32) -> Self {
        Self {
            r: (self.r * (1.0 - amount)).max(0.0),
            g: (self.g * (1.0 - amount)).max(0.0),
            b: (self.b * (1.0 - amount)).max(0.0),
            a: self.a,
        }
    }

    /// Lighten by a percentage (0.0–1.0).
    pub fn lighten(self, amount: f32) -> Self {
        Self {
            r: (self.r + (1.0 - self.r) * amount).min(1.0),
            g: (self.g + (1.0 - self.g) * amount).min(1.0),
            b: (self.b + (1.0 - self.b) * amount).min(1.0),
            a: self.a,
        }
    }

    /// Set alpha.
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_6_digit_hex() {
        assert_eq!(parse_hex_color("#ff0000"), Some((255, 0, 0, 255)));
    }

    #[test]
    fn parse_8_digit_hex_with_alpha() {
        assert_eq!(parse_hex_color("#00000080"), Some((0, 0, 0, 128)));
    }

    #[test]
    fn parse_3_digit_shorthand() {
        assert_eq!(parse_hex_color("#f00"), Some((255, 0, 0, 255)));
    }
}
