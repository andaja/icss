//! CSS value parsing — converts token sequences into `CssValue`.

use crate::theme::color::ThemeColor;
use crate::theme::css::ast::CssValue;
use crate::theme::css::tokenizer::Token;

/// Named CSS colors (small subset of the most common ones).
pub fn named_color(name: &str) -> Option<ThemeColor> {
    let (r, g, b) = match name {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 128, 0),
        "blue" => (0, 0, 255),
        "yellow" => (255, 255, 0),
        "cyan" | "aqua" => (0, 255, 255),
        "magenta" | "fuchsia" => (255, 0, 255),
        "orange" => (255, 165, 0),
        "gray" | "grey" => (128, 128, 128),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "silver" => (192, 192, 192),
        "maroon" => (128, 0, 0),
        "olive" => (128, 128, 0),
        "lime" => (0, 255, 0),
        "teal" => (0, 128, 128),
        "navy" => (0, 0, 128),
        "purple" => (128, 0, 128),
        "indigo" => (75, 0, 130),
        "coral" => (255, 127, 80),
        "salmon" => (250, 128, 114),
        "tomato" => (255, 99, 71),
        "crimson" => (220, 20, 60),
        "gold" => (255, 215, 0),
        "khaki" => (240, 230, 140),
        "skyblue" => (135, 206, 235),
        "steelblue" => (70, 130, 180),
        "slategray" | "slategrey" => (112, 128, 144),
        _ => return None,
    };
    Some(ThemeColor::from_rgba8(r, g, b, 255))
}

/// Parse a sequence of tokens (the value part of a declaration) into a `CssValue`.
///
/// `tokens` is the slice of tokens between `:` and `;` for this declaration.
pub fn parse_value(tokens: &[Token]) -> CssValue {
    if tokens.is_empty() {
        return CssValue::Keyword("none".into());
    }

    // Single token values.
    if tokens.len() == 1 {
        return parse_single_token(&tokens[0]);
    }

    // var(--name) or var(--name, fallback)
    if matches!(&tokens[0], Token::Ident(name) if name == "var")
        && matches!(&tokens[1], Token::ParenOpen)
    {
        return parse_var(tokens);
    }

    // rgb(r, g, b) or rgba(r, g, b, a)
    if matches!(&tokens[0], Token::Ident(name) if name == "rgb" || name == "rgba")
        && matches!(&tokens[1], Token::ParenOpen)
    {
        return parse_rgb_rgba(tokens);
    }

    // box-shadow: <x> <y> <blur> <color>
    // Try to parse as shadow (4 components: num num num color).
    if let Some(shadow) = try_parse_shadow(tokens) {
        return shadow;
    }

    // Fallback: join tokens as raw string.
    let raw: String = tokens
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    CssValue::Raw(raw)
}

fn parse_single_token(token: &Token) -> CssValue {
    match token {
        Token::Hash(hex) => {
            if let Some(color) = ThemeColor::from_hex(&format!("#{hex}")) {
                CssValue::Color(color)
            } else {
                CssValue::Raw(format!("#{hex}"))
            }
        }
        Token::Number(n) => CssValue::Number(*n),
        Token::Dimension(n, unit) => {
            if unit == "px" {
                CssValue::Length(*n)
            } else {
                // Store as length anyway, we only support px.
                CssValue::Length(*n)
            }
        }
        Token::Ident(name) => {
            if name == "transparent" {
                CssValue::Color(ThemeColor::from_rgba8(0, 0, 0, 0))
            } else if name == "none" {
                CssValue::Keyword("none".into())
            } else if let Some(color) = named_color(name) {
                CssValue::Color(color)
            } else {
                CssValue::Keyword(name.clone())
            }
        }
        Token::StringLit(s) => CssValue::Raw(s.clone()),
        _ => CssValue::Raw(token.to_string()),
    }
}

fn parse_var(tokens: &[Token]) -> CssValue {
    // var( --name )  or  var( --name , fallback )
    // tokens[0] = "var", [1] = "(", [2] = "--name", ...
    let mut i = 2;

    let name = match tokens.get(i) {
        Some(Token::CustomProperty(name)) => name.clone(),
        _ => return CssValue::Raw(tokens_to_string(tokens)),
    };
    i += 1;

    // Check for comma (fallback value).
    let fallback = if matches!(tokens.get(i), Some(Token::Comma)) {
        i += 1;
        // Collect remaining tokens until ')'.
        let mut fb_tokens = Vec::new();
        while i < tokens.len() && !matches!(tokens.get(i), Some(Token::ParenClose)) {
            fb_tokens.push(tokens[i].clone());
            i += 1;
        }
        if fb_tokens.is_empty() {
            None
        } else {
            Some(Box::new(parse_value(&fb_tokens)))
        }
    } else {
        None
    };

    CssValue::Var { name, fallback }
}

fn parse_rgb_rgba(tokens: &[Token]) -> CssValue {
    // rgb( r , g , b )  or  rgba( r , g , b , a )
    let nums: Vec<f32> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Number(n) => Some(*n),
            Token::Dimension(n, _) => Some(*n),
            _ => None,
        })
        .collect();

    match nums.len() {
        3 => {
            let color = ThemeColor::from_rgba8(nums[0] as u8, nums[1] as u8, nums[2] as u8, 255);
            CssValue::Color(color)
        }
        4 => {
            // Alpha can be 0-1 float or 0-255 int.
            let a = if nums[3] <= 1.0 {
                (nums[3] * 255.0) as u8
            } else {
                nums[3] as u8
            };
            let color = ThemeColor::from_rgba8(nums[0] as u8, nums[1] as u8, nums[2] as u8, a);
            CssValue::Color(color)
        }
        _ => CssValue::Raw(tokens_to_string(tokens)),
    }
}

fn try_parse_shadow(tokens: &[Token]) -> Option<CssValue> {
    // box-shadow value: <x> <y> <blur> <color>
    // Tokens should be: num num num (hash | rgb/rgba | ident)
    // We need at least 4 logical values.

    let mut nums = Vec::new();
    let mut color_tokens = Vec::new();
    let mut in_color = false;
    let mut paren_depth = 0;

    for token in tokens {
        if in_color {
            color_tokens.push(token.clone());
            match token {
                Token::ParenOpen => paren_depth += 1,
                Token::ParenClose => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        // done with color function
                    }
                }
                _ => {}
            }
            continue;
        }

        match token {
            Token::Number(n) | Token::Dimension(n, _) if nums.len() < 3 => {
                nums.push(*n);
            }
            _ if nums.len() == 3 => {
                // Everything after the 3 numbers is the color.
                in_color = true;
                color_tokens.push(token.clone());
                if matches!(token, Token::ParenOpen) {
                    paren_depth += 1;
                }
            }
            _ => return None,
        }
    }

    if nums.len() != 3 || color_tokens.is_empty() {
        return None;
    }

    let color_value = parse_value(&color_tokens);
    let color = match color_value {
        CssValue::Color(c) => c,
        _ => return None,
    };

    Some(CssValue::Shadow {
        x: nums[0],
        y: nums[1],
        blur: nums[2],
        color,
    })
}

fn tokens_to_string(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::css::tokenizer::Tokenizer;

    fn parse(src: &str) -> CssValue {
        let mut tok = Tokenizer::new(src);
        let tokens: Vec<Token> = tok
            .tokenize_all()
            .unwrap()
            .into_iter()
            .map(|s| s.token)
            .filter(|t| !matches!(t, Token::Eof))
            .collect();
        parse_value(&tokens)
    }

    #[test]
    fn hex_color() {
        let val = parse("#ff0000");
        assert!(matches!(val, CssValue::Color(c) if c.r > 0.99));
    }

    #[test]
    fn hex_color_with_alpha() {
        let val = parse("#00000080");
        assert!(matches!(val, CssValue::Color(c) if c.r < 0.01 && (c.a - 0.502).abs() < 0.01));
    }

    #[test]
    fn rgba_function() {
        let val = parse("rgba(255, 0, 0, 0.5)");
        assert!(matches!(val, CssValue::Color(c) if c.r > 0.99 && (c.a - 0.502).abs() < 0.01));
    }

    #[test]
    fn rgb_function() {
        let val = parse("rgb(0, 128, 255)");
        assert!(matches!(val, CssValue::Color(_)));
    }

    #[test]
    fn var_reference() {
        let val = parse("var(--primary)");
        assert!(matches!(val, CssValue::Var { name, fallback: None } if name == "--primary"));
    }

    #[test]
    fn var_with_fallback() {
        let val = parse("var(--primary, #ff0000)");
        assert!(matches!(val, CssValue::Var { name, fallback: Some(_) } if name == "--primary"));
    }

    #[test]
    fn length_px() {
        let val = parse("8px");
        assert!(matches!(val, CssValue::Length(v) if (v - 8.0).abs() < 0.001));
    }

    #[test]
    fn unitless_number() {
        let val = parse("0.5");
        assert!(matches!(val, CssValue::Number(v) if (v - 0.5).abs() < 0.001));
    }

    #[test]
    fn keyword_transparent() {
        let val = parse("transparent");
        assert!(matches!(val, CssValue::Color(c) if c.a < 0.01));
    }

    #[test]
    fn keyword_none() {
        let val = parse("none");
        assert!(val.is_none());
    }

    #[test]
    fn named_color_red() {
        let val = parse("red");
        assert!(matches!(val, CssValue::Color(c) if c.r > 0.99 && c.g < 0.01));
    }

    #[test]
    fn shadow_value() {
        let val = parse("0 2px 8px #00000040");
        assert!(matches!(val, CssValue::Shadow { .. }));
    }

    #[test]
    fn shadow_with_rgba() {
        let val = parse("0 2px 8px rgba(0, 0, 0, 0.25)");
        assert!(matches!(val, CssValue::Shadow { .. }));
    }
}
