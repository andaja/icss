//! CSS parser — converts a token stream into a [`Stylesheet`] AST.

use std::collections::HashMap;

use crate::theme::css::ast::{CssValue, Declaration, Rule, Selector, Stylesheet};
use crate::theme::css::selector::{SelectorError, parse_selector};
use crate::theme::css::tokenizer::{Spanned, Token, TokenError, Tokenizer};
use crate::theme::css::value::parse_value;

/// Parse error with source location.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("line {line}:{col}: {message}")]
    Syntax {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("{0}")]
    Token(#[from] TokenError),

    #[error("selector: {0}")]
    Selector(#[from] SelectorError),
}

/// Parse an `.icss` source string into a `Stylesheet`.
pub fn parse_stylesheet(source: &str) -> Result<Stylesheet, ParseError> {
    let mut tokenizer = Tokenizer::new(source);
    let tokens = tokenizer.tokenize_all()?;
    let mut parser = Parser::new(&tokens);
    parser.parse()
}

struct Parser<'a> {
    tokens: &'a [Spanned],
    pos: usize,
    source_order: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Spanned]) -> Self {
        Self {
            tokens,
            pos: 0,
            source_order: 0,
        }
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|s| &s.token)
            .unwrap_or(&Token::Eof)
    }

    fn current_span(&self) -> (usize, usize) {
        self.tokens
            .get(self.pos)
            .map(|s| (s.line, s.col))
            .unwrap_or((0, 0))
    }

    fn advance(&mut self) -> &Token {
        let token = self
            .tokens
            .get(self.pos)
            .map(|s| &s.token)
            .unwrap_or(&Token::Eof);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        let (line, col) = self.current_span();
        let actual = self.advance();
        if actual == expected {
            Ok(())
        } else {
            Err(ParseError::Syntax {
                line,
                col,
                message: format!("expected '{expected}', got '{actual}'"),
            })
        }
    }

    fn parse(&mut self) -> Result<Stylesheet, ParseError> {
        let mut custom_properties = HashMap::new();
        let mut rules = Vec::new();

        while *self.peek() != Token::Eof {
            // Read selector tokens until '{'.
            let selector_tokens = self.read_selector_tokens()?;

            // Check if this is a :root block.
            let is_root = selector_tokens.len() == 2
                && matches!(&selector_tokens[0], Token::Colon)
                && matches!(&selector_tokens[1], Token::Ident(name) if name == "root");

            self.expect(&Token::BraceOpen)?;

            if is_root {
                self.parse_custom_properties(&mut custom_properties)?;
            } else {
                let selector = parse_selector(&selector_tokens)?;
                let declarations = self.parse_declarations()?;

                // Check for nested rules inside this block.
                // We collect nested rules after processing declarations.
                let nested_rules = self.parse_nested_rules(&selector)?;

                if !declarations.is_empty() {
                    let order = self.source_order;
                    self.source_order += 1;
                    rules.push(Rule {
                        selector: selector.clone(),
                        declarations,
                        source_order: order,
                    });
                }

                rules.extend(nested_rules);
            }

            self.expect(&Token::BraceClose)?;
        }

        Ok(Stylesheet {
            custom_properties,
            rules,
        })
    }

    fn read_selector_tokens(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        loop {
            match self.peek() {
                Token::BraceOpen | Token::Eof => break,
                _ => {
                    tokens.push(self.advance().clone());
                }
            }
        }
        if tokens.is_empty() {
            let (line, col) = self.current_span();
            return Err(ParseError::Syntax {
                line,
                col,
                message: "expected selector before '{'".into(),
            });
        }
        Ok(tokens)
    }

    fn parse_custom_properties(
        &mut self,
        props: &mut HashMap<String, CssValue>,
    ) -> Result<(), ParseError> {
        loop {
            match self.peek() {
                Token::BraceClose | Token::Eof => break,
                Token::CustomProperty(_) => {
                    let name = match self.advance().clone() {
                        Token::CustomProperty(name) => name,
                        _ => unreachable!(),
                    };
                    self.expect(&Token::Colon)?;
                    let value_tokens = self.read_value_tokens()?;
                    let value = parse_value(&value_tokens);
                    self.skip_semicolon();
                    props.insert(name, value);
                }
                _ => {
                    // Skip unexpected tokens in :root.
                    self.advance();
                }
            }
        }
        Ok(())
    }

    fn parse_declarations(&mut self) -> Result<Vec<Declaration>, ParseError> {
        let mut decls = Vec::new();

        loop {
            match self.peek() {
                Token::BraceClose | Token::Eof => break,
                // Nested rule starts with Dot — stop parsing declarations.
                Token::Dot => break,
                Token::Ident(_) => {
                    let property = match self.advance().clone() {
                        Token::Ident(name) => name,
                        _ => unreachable!(),
                    };
                    self.expect(&Token::Colon)?;
                    let value_tokens = self.read_value_tokens()?;
                    let value = parse_value(&value_tokens);
                    self.skip_semicolon();
                    decls.push(Declaration { property, value });
                }
                _ => {
                    // Skip unexpected tokens.
                    self.advance();
                }
            }
        }

        Ok(decls)
    }

    fn parse_nested_rules(&mut self, parent_selector: &Selector) -> Result<Vec<Rule>, ParseError> {
        let mut rules = Vec::new();

        while matches!(self.peek(), Token::Dot) {
            let selector_tokens = self.read_selector_tokens()?;
            let mut child_selector = parse_selector(&selector_tokens)?;

            // Merge parent classes into child selector (flattening).
            for pc in &parent_selector.classes {
                if !child_selector.classes.contains(pc) {
                    child_selector.classes.push(pc.clone());
                }
            }
            child_selector.classes.sort();

            self.expect(&Token::BraceOpen)?;
            let declarations = self.parse_declarations()?;
            self.expect(&Token::BraceClose)?;

            if !declarations.is_empty() {
                let order = self.source_order;
                self.source_order += 1;
                rules.push(Rule {
                    selector: child_selector,
                    declarations,
                    source_order: order,
                });
            }
        }

        Ok(rules)
    }

    fn read_value_tokens(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        let mut paren_depth = 0u32;

        loop {
            match self.peek() {
                Token::Semicolon if paren_depth == 0 => break,
                Token::BraceClose if paren_depth == 0 => break,
                Token::Eof => break,
                Token::ParenOpen => {
                    paren_depth += 1;
                    tokens.push(self.advance().clone());
                }
                Token::ParenClose => {
                    paren_depth = paren_depth.saturating_sub(1);
                    tokens.push(self.advance().clone());
                }
                _ => {
                    tokens.push(self.advance().clone());
                }
            }
        }

        Ok(tokens)
    }

    fn skip_semicolon(&mut self) {
        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::css::ast::PseudoClass;

    #[test]
    fn parse_root_and_rule() {
        let css = r#"
            :root {
                --primary: #0f3460;
                --text: #eaeaea;
            }

            .button {
                background-color: var(--primary);
                color: #ffffff;
                border-radius: 8px;
            }
        "#;

        let sheet = parse_stylesheet(css).unwrap();
        assert_eq!(sheet.custom_properties.len(), 2);
        assert!(sheet.custom_properties.contains_key("--primary"));
        assert!(sheet.custom_properties.contains_key("--text"));

        assert_eq!(sheet.rules.len(), 1);
        let rule = &sheet.rules[0];
        assert_eq!(rule.selector.classes, vec!["button"]);
        assert_eq!(rule.declarations.len(), 3);
        assert_eq!(rule.declarations[0].property, "background-color");
        assert_eq!(rule.declarations[1].property, "color");
        assert_eq!(rule.declarations[2].property, "border-radius");
    }

    #[test]
    fn parse_multi_class() {
        let css = r#"
            .button.primary {
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
            }
        "#;

        let sheet = parse_stylesheet(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selector.classes, vec!["button", "primary"]);
    }

    #[test]
    fn parse_pseudo_class() {
        let css = r#"
            .button:hover {
                background-color: #1a4a80;
            }

            .button.primary:disabled {
                background-color: #333355;
            }
        "#;

        let sheet = parse_stylesheet(css).unwrap();
        assert_eq!(sheet.rules.len(), 2);
        assert_eq!(sheet.rules[0].selector.pseudo, Some(PseudoClass::Hover));
        assert_eq!(sheet.rules[1].selector.pseudo, Some(PseudoClass::Disabled));
        assert_eq!(sheet.rules[1].selector.classes, vec!["button", "primary"]);
    }

    #[test]
    fn parse_nested_rules() {
        let css = r#"
            .sidebar {
                background-color: #1a1a2e;

                .action {
                    border-radius: 4px;
                }
            }
        "#;

        let sheet = parse_stylesheet(css).unwrap();
        assert_eq!(sheet.rules.len(), 2);
        // Parent rule.
        assert_eq!(sheet.rules[0].selector.classes, vec!["sidebar"]);
        // Nested rule flattened: .sidebar + .action = .action.sidebar (sorted).
        assert_eq!(sheet.rules[1].selector.classes, vec!["action", "sidebar"]);
    }

    #[test]
    fn parse_source_order() {
        let css = r#"
            .a { color: red; }
            .b { color: blue; }
            .c { color: green; }
        "#;

        let sheet = parse_stylesheet(css).unwrap();
        assert_eq!(sheet.rules[0].source_order, 0);
        assert_eq!(sheet.rules[1].source_order, 1);
        assert_eq!(sheet.rules[2].source_order, 2);
    }

    #[test]
    fn parse_var_with_fallback() {
        let css = r#"
            .button {
                color: var(--text, #ffffff);
            }
        "#;

        let sheet = parse_stylesheet(css).unwrap();
        let val = &sheet.rules[0].declarations[0].value;
        assert!(matches!(
            val,
            CssValue::Var {
                name,
                fallback: Some(_)
            } if name == "--text"
        ));
    }

    #[test]
    fn parse_comments() {
        let css = r#"
            /* Block comment */
            .button {
                // Line comment
                color: red;
            }
        "#;

        let sheet = parse_stylesheet(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].declarations.len(), 1);
    }

    #[test]
    fn parse_full_theme() {
        let css = r#"
            :root {
                --primary: #0f3460;
                --primary-light: #1a4a80;
                --primary-dark: #0a2440;
                --success: #16c79a;
                --danger: #e94560;
                --text: #eaeaea;
                --text-dim: #888888;
                --text-muted: #707070;
                --surface: #1a1a2e;
                --surface-raised: #25254a;
            }

            .surface {
                background-color: var(--surface);
                color: var(--text);
            }

            .button {
                color: #ffffff;
                border-radius: 8px;
            }

            .primary {
                background-color: var(--primary);
            }

            .primary:hover {
                background-color: var(--primary-light);
            }

            .primary:active {
                background-color: var(--primary-dark);
            }

            .primary:disabled {
                background-color: #333355;
                color: rgba(255, 255, 255, 0.38);
            }

            .button.primary {
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
            }

            .button.primary:hover {
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.37);
            }

            .danger {
                background-color: var(--danger);
            }

            .success {
                background-color: var(--success);
            }
        "#;

        let sheet = parse_stylesheet(css).unwrap();
        assert_eq!(sheet.custom_properties.len(), 10);
        // Count rules: surface, button, primary, primary:hover, primary:active,
        // primary:disabled, button.primary, button.primary:hover, danger, success = 10
        assert_eq!(sheet.rules.len(), 10);
    }
}
