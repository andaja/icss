//! CSS subset parser for `.icss` theme files.
//!
//! This module has **no iced dependency**. It parses CSS-like syntax into
//! a [`Stylesheet`] AST that the `resolve` module converts into iced styles.

pub mod ast;
pub mod parser;
pub mod selector;
pub mod tokenizer;
pub mod value;

pub use ast::{CssValue, Declaration, PseudoClass, Rule, Selector, Stylesheet};
pub use parser::{ParseError, parse_stylesheet};

/// Tokenize and parse a single CSS value string (e.g. after var substitution).
pub fn tokenize_and_parse(input: &str) -> CssValue {
    let mut lexer = tokenizer::Tokenizer::new(input);
    let spanned = lexer.tokenize_all().unwrap_or_default();
    let tokens: Vec<tokenizer::Token> = spanned
        .into_iter()
        .map(|s| s.token)
        .filter(|t| !matches!(t, tokenizer::Token::Eof))
        .collect();
    value::parse_value(&tokens)
}
