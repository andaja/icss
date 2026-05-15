//! Selector parsing for multi-class conjunctive selectors.
//!
//! Parses token sequences like `.button.primary:hover` into a [`Selector`].

use crate::theme::css::ast::{PseudoClass, Selector};
use crate::theme::css::tokenizer::Token;

/// Parse a selector from a slice of tokens (consumed before the `{`).
///
/// Expected patterns:
/// - `.class` → single class
/// - `.class1.class2` → multi-class conjunctive
/// - `.class:pseudo` → class with pseudo-class
/// - `.class1.class2:pseudo` → multi-class with pseudo
/// - `:root` → special case for custom properties
pub fn parse_selector(tokens: &[Token]) -> Result<Selector, SelectorError> {
    if tokens.is_empty() {
        return Err(SelectorError("empty selector".into()));
    }

    // Special case: `:root`
    if tokens.len() == 2
        && matches!(&tokens[0], Token::Colon)
        && matches!(&tokens[1], Token::Ident(name) if name == "root")
    {
        return Ok(Selector {
            classes: vec!["root".into()],
            pseudo: None,
        });
    }

    let mut classes = Vec::new();
    let mut pseudo = None;
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Dot => {
                // Expect an ident after the dot.
                i += 1;
                match tokens.get(i) {
                    Some(Token::Ident(name)) => {
                        classes.push(name.clone());
                        i += 1;
                    }
                    _ => {
                        return Err(SelectorError("expected class name after '.'".into()));
                    }
                }
            }
            Token::Colon => {
                // Pseudo-class.
                i += 1;
                match tokens.get(i) {
                    Some(Token::Ident(name)) => {
                        if let Some(pc) = PseudoClass::parse(name) {
                            pseudo = Some(pc);
                            i += 1;
                        } else {
                            return Err(SelectorError(format!("unknown pseudo-class ':{name}'")));
                        }
                    }
                    _ => {
                        return Err(SelectorError("expected pseudo-class name after ':'".into()));
                    }
                }
            }
            other => {
                return Err(SelectorError(format!(
                    "unexpected token in selector: {other}"
                )));
            }
        }
    }

    if classes.is_empty() {
        return Err(SelectorError("selector has no classes".into()));
    }

    // Sort classes for canonical form (enables consistent hashing/comparison).
    classes.sort();

    Ok(Selector { classes, pseudo })
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct SelectorError(pub String);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::css::tokenizer::Token;

    fn sel(tokens: &[Token]) -> Selector {
        parse_selector(tokens).unwrap()
    }

    #[test]
    fn single_class() {
        let s = sel(&[Token::Dot, Token::Ident("button".into())]);
        assert_eq!(s.classes, vec!["button"]);
        assert_eq!(s.pseudo, None);
        assert_eq!(s.specificity(), (0, 1));
    }

    #[test]
    fn multi_class() {
        let s = sel(&[
            Token::Dot,
            Token::Ident("button".into()),
            Token::Dot,
            Token::Ident("primary".into()),
        ]);
        // Sorted: button, primary
        assert_eq!(s.classes, vec!["button", "primary"]);
        assert_eq!(s.specificity(), (0, 2));
    }

    #[test]
    fn class_with_pseudo() {
        let s = sel(&[
            Token::Dot,
            Token::Ident("button".into()),
            Token::Colon,
            Token::Ident("hover".into()),
        ]);
        assert_eq!(s.classes, vec!["button"]);
        assert_eq!(s.pseudo, Some(PseudoClass::Hover));
        assert_eq!(s.specificity(), (1, 1));
    }

    #[test]
    fn multi_class_with_pseudo() {
        let s = sel(&[
            Token::Dot,
            Token::Ident("button".into()),
            Token::Dot,
            Token::Ident("primary".into()),
            Token::Colon,
            Token::Ident("disabled".into()),
        ]);
        assert_eq!(s.classes, vec!["button", "primary"]);
        assert_eq!(s.pseudo, Some(PseudoClass::Disabled));
        assert_eq!(s.specificity(), (1, 2));
    }

    #[test]
    fn root_selector() {
        let s = sel(&[Token::Colon, Token::Ident("root".into())]);
        assert_eq!(s.classes, vec!["root"]);
        assert_eq!(s.pseudo, None);
    }

    #[test]
    fn matches_subset() {
        let s = sel(&[
            Token::Dot,
            Token::Ident("button".into()),
            Token::Dot,
            Token::Ident("primary".into()),
        ]);
        // Element has button + primary + small → should match.
        assert!(s.matches(&["button", "primary", "small"], None));
        // Element has only button → should not match.
        assert!(!s.matches(&["button", "small"], None));
    }

    #[test]
    fn matches_pseudo() {
        let s = sel(&[
            Token::Dot,
            Token::Ident("button".into()),
            Token::Colon,
            Token::Ident("hover".into()),
        ]);
        assert!(s.matches(&["button"], Some(PseudoClass::Hover)));
        assert!(!s.matches(&["button"], None));
        assert!(!s.matches(&["button"], Some(PseudoClass::Active)));
    }
}
