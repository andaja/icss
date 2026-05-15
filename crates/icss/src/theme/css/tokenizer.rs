//! Hand-rolled CSS tokenizer for the `.icss` subset.
//!
//! Produces a flat stream of [`Token`]s from source text.
//! Designed to be swappable with Mozilla's `cssparser` crate later if needed.

use std::fmt;

/// A token produced by the tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// `.` (dot — starts a class selector)
    Dot,
    /// `{`
    BraceOpen,
    /// `}`
    BraceClose,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `(`
    ParenOpen,
    /// `)`
    ParenClose,
    /// An identifier: `background-color`, `hover`, `root`, `var`, `rgb`, etc.
    Ident(String),
    /// A numeric literal: `8`, `0.5`, `-2`.
    Number(f32),
    /// A dimension: `8px`.
    Dimension(f32, String),
    /// A hash value (without `#`): `ff0000`, `RRGGBBAA`.
    Hash(String),
    /// `--custom-property-name` (custom property name including the `--` prefix).
    CustomProperty(String),
    /// A string literal (contents without quotes).
    StringLit(String),
    /// End of input.
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Dot => write!(f, "."),
            Token::BraceOpen => write!(f, "{{"),
            Token::BraceClose => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::ParenOpen => write!(f, "("),
            Token::ParenClose => write!(f, ")"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Number(n) => write!(f, "{n}"),
            Token::Dimension(n, u) => write!(f, "{n}{u}"),
            Token::Hash(h) => write!(f, "#{h}"),
            Token::CustomProperty(name) => write!(f, "{name}"),
            Token::StringLit(s) => write!(f, "\"{s}\""),
            Token::Eof => write!(f, "<EOF>"),
        }
    }
}

/// Tokenizer state.
pub struct Tokenizer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

/// A token with its source position.
#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Tokenize the entire source into a Vec of spanned tokens.
    pub fn tokenize_all(&mut self) -> Result<Vec<Spanned>, TokenError> {
        let mut tokens = Vec::new();
        loop {
            let sp = self.next_token()?;
            if sp.token == Token::Eof {
                tokens.push(sp);
                break;
            }
            tokens.push(sp);
        }
        Ok(tokens)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.bytes.get(self.pos).copied()?;
        self.pos += 1;
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(b)
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), TokenError> {
        loop {
            // Skip whitespace.
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.advance();
            }
            // Skip block comments /* ... */.
            if self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos] == b'/'
                && self.bytes[self.pos + 1] == b'*'
            {
                let start_line = self.line;
                self.advance(); // /
                self.advance(); // *
                loop {
                    if self.pos + 1 >= self.bytes.len() {
                        return Err(TokenError {
                            line: start_line,
                            col: 1,
                            message: "unterminated block comment".into(),
                        });
                    }
                    if self.bytes[self.pos] == b'*' && self.bytes[self.pos + 1] == b'/' {
                        self.advance(); // *
                        self.advance(); // /
                        break;
                    }
                    self.advance();
                }
                continue;
            }
            // Skip line comments // ... (non-standard CSS but we support them).
            if self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos] == b'/'
                && self.bytes[self.pos + 1] == b'/'
            {
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                    self.advance();
                }
                continue;
            }
            break;
        }
        Ok(())
    }

    /// Produce the next token.
    pub fn next_token(&mut self) -> Result<Spanned, TokenError> {
        self.skip_whitespace_and_comments()?;

        let line = self.line;
        let col = self.col;

        let Some(b) = self.peek_byte() else {
            return Ok(Spanned {
                token: Token::Eof,
                line,
                col,
            });
        };

        let token = match b {
            b'.' => {
                self.advance();
                Token::Dot
            }
            b'{' => {
                self.advance();
                Token::BraceOpen
            }
            b'}' => {
                self.advance();
                Token::BraceClose
            }
            b':' => {
                self.advance();
                Token::Colon
            }
            b';' => {
                self.advance();
                Token::Semicolon
            }
            b',' => {
                self.advance();
                Token::Comma
            }
            b'(' => {
                self.advance();
                Token::ParenOpen
            }
            b')' => {
                self.advance();
                Token::ParenClose
            }
            b'#' => {
                self.advance();
                let hash = self.read_hex_or_ident();
                Token::Hash(hash)
            }
            b'-' if self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'-' => {
                // Custom property: --name
                self.advance(); // first -
                self.advance(); // second -
                let name = self.read_ident_chars();
                Token::CustomProperty(format!("--{name}"))
            }
            b'-' | b'0'..=b'9' => self.read_number_or_dimension()?,
            b'"' | b'\'' => self.read_string()?,
            _ if is_ident_start(b) => {
                let ident = self.read_ident_chars();
                Token::Ident(ident)
            }
            _ => {
                return Err(TokenError {
                    line,
                    col,
                    message: format!("unexpected character: '{}'", b as char),
                });
            }
        };

        Ok(Spanned { token, line, col })
    }

    fn read_ident_chars(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.bytes.len() && is_ident_char(self.bytes[self.pos]) {
            self.pos += 1;
            self.col += 1;
        }
        self.source[start..self.pos].to_string()
    }

    fn read_hex_or_ident(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.bytes.len() && is_hex_or_ident(self.bytes[self.pos]) {
            self.pos += 1;
            self.col += 1;
        }
        self.source[start..self.pos].to_string()
    }

    fn read_number_or_dimension(&mut self) -> Result<Token, TokenError> {
        let start = self.pos;
        let line = self.line;
        let col = self.col;

        // Optional leading minus.
        if self.peek_byte() == Some(b'-') {
            // Check if this is actually a negative number (next char is digit or dot).
            if self.pos + 1 < self.bytes.len()
                && (self.bytes[self.pos + 1].is_ascii_digit() || self.bytes[self.pos + 1] == b'.')
            {
                self.advance();
            } else {
                // Not a number — might be an ident starting with -.
                return Ok(Token::Ident(self.read_ident_chars()));
            }
        }

        // Integer part.
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.advance();
        }

        // Decimal part.
        if self.pos < self.bytes.len()
            && self.bytes[self.pos] == b'.'
            && self.pos + 1 < self.bytes.len()
            && self.bytes[self.pos + 1].is_ascii_digit()
        {
            self.advance(); // .
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.advance();
            }
        }

        let num_str = &self.source[start..self.pos];
        let value: f32 = num_str.parse().map_err(|_| TokenError {
            line,
            col,
            message: format!("invalid number: '{num_str}'"),
        })?;

        // Check for unit suffix (e.g., `px`).
        if self.pos < self.bytes.len() && is_ident_start(self.bytes[self.pos]) {
            let unit = self.read_ident_chars();
            Ok(Token::Dimension(value, unit))
        } else {
            Ok(Token::Number(value))
        }
    }

    fn read_string(&mut self) -> Result<Token, TokenError> {
        let quote = self.advance().unwrap(); // consume opening quote
        let line = self.line;
        let col = self.col;
        let mut s = String::new();

        loop {
            match self.advance() {
                None => {
                    return Err(TokenError {
                        line,
                        col,
                        message: "unterminated string".into(),
                    });
                }
                Some(b) if b == quote => break,
                Some(b'\\') => {
                    // Simple escape: just take the next char.
                    if let Some(escaped) = self.advance() {
                        s.push(escaped as char);
                    }
                }
                Some(b) => s.push(b as char),
            }
        }

        Ok(Token::StringLit(s))
    }
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn is_hex_or_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

/// Tokenizer error.
#[derive(Debug, Clone, thiserror::Error)]
#[error("line {line}:{col}: {message}")]
pub struct TokenError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(src: &str) -> Vec<Token> {
        Tokenizer::new(src)
            .tokenize_all()
            .unwrap()
            .into_iter()
            .map(|s| s.token)
            .filter(|t| !matches!(t, Token::Eof))
            .collect()
    }

    #[test]
    fn simple_rule() {
        let tokens = tokenize(".button { color: #ff0000; }");
        assert_eq!(
            tokens,
            vec![
                Token::Dot,
                Token::Ident("button".into()),
                Token::BraceOpen,
                Token::Ident("color".into()),
                Token::Colon,
                Token::Hash("ff0000".into()),
                Token::Semicolon,
                Token::BraceClose,
            ]
        );
    }

    #[test]
    fn multi_class_selector() {
        let tokens = tokenize(".button.primary:hover { }");
        assert_eq!(
            tokens,
            vec![
                Token::Dot,
                Token::Ident("button".into()),
                Token::Dot,
                Token::Ident("primary".into()),
                Token::Colon,
                Token::Ident("hover".into()),
                Token::BraceOpen,
                Token::BraceClose,
            ]
        );
    }

    #[test]
    fn custom_properties() {
        let tokens = tokenize(":root { --primary: #0f3460; }");
        assert_eq!(
            tokens,
            vec![
                Token::Colon,
                Token::Ident("root".into()),
                Token::BraceOpen,
                Token::CustomProperty("--primary".into()),
                Token::Colon,
                Token::Hash("0f3460".into()),
                Token::Semicolon,
                Token::BraceClose,
            ]
        );
    }

    #[test]
    fn var_reference() {
        let tokens = tokenize("background-color: var(--primary);");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("background-color".into()),
                Token::Colon,
                Token::Ident("var".into()),
                Token::ParenOpen,
                Token::CustomProperty("--primary".into()),
                Token::ParenClose,
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn rgba_function() {
        let tokens = tokenize("color: rgba(255, 0, 0, 0.5);");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("color".into()),
                Token::Colon,
                Token::Ident("rgba".into()),
                Token::ParenOpen,
                Token::Number(255.0),
                Token::Comma,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(0.5),
                Token::ParenClose,
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn dimensions() {
        let tokens = tokenize("border-radius: 8px;");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("border-radius".into()),
                Token::Colon,
                Token::Dimension(8.0, "px".into()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn negative_number() {
        let tokens = tokenize("margin: -4px;");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("margin".into()),
                Token::Colon,
                Token::Dimension(-4.0, "px".into()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn block_comment() {
        let tokens = tokenize("/* comment */ .btn { }");
        assert_eq!(
            tokens,
            vec![
                Token::Dot,
                Token::Ident("btn".into()),
                Token::BraceOpen,
                Token::BraceClose,
            ]
        );
    }

    #[test]
    fn line_comment() {
        let tokens = tokenize("// comment\n.btn { }");
        assert_eq!(
            tokens,
            vec![
                Token::Dot,
                Token::Ident("btn".into()),
                Token::BraceOpen,
                Token::BraceClose,
            ]
        );
    }
}
