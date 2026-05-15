//! CSS AST types — no iced dependency.

use std::collections::HashMap;
use std::fmt;

use crate::theme::color::ThemeColor;

/// A parsed stylesheet.
#[derive(Debug, Clone)]
pub struct Stylesheet {
    /// Custom properties from `:root { --name: value; }`.
    pub custom_properties: HashMap<String, CssValue>,
    /// All rules in source order.
    pub rules: Vec<Rule>,
}

/// A single CSS rule: selector + declarations.
#[derive(Debug, Clone)]
pub struct Rule {
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
    /// Position in source file for specificity tie-breaking.
    pub source_order: usize,
}

/// A multi-class conjunctive selector, optionally with a pseudo-class.
///
/// `.button.primary:hover` → classes: ["button", "primary"], pseudo: Some(Hover)
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    /// Class names (without the leading dot). Sorted for canonical form.
    pub classes: Vec<String>,
    /// Optional pseudo-class.
    pub pseudo: Option<PseudoClass>,
}

impl Selector {
    /// Compute specificity as (pseudo_count, class_count).
    pub fn specificity(&self) -> (u8, u8) {
        let pseudo = if self.pseudo.is_some() { 1 } else { 0 };
        let classes = self.classes.len().min(255) as u8;
        (pseudo, classes)
    }

    /// Check if this selector matches an element with the given classes and pseudo-state.
    ///
    /// The selector's classes must be a subset of `element_classes`.
    /// If the selector has a pseudo-class, `active_pseudo` must match it.
    /// If the selector has no pseudo-class, it matches regardless of `active_pseudo`.
    pub fn matches(&self, element_classes: &[&str], active_pseudo: Option<PseudoClass>) -> bool {
        // All selector classes must be present in element classes.
        for sc in &self.classes {
            if !element_classes.iter().any(|ec| ec == sc) {
                return false;
            }
        }
        // Pseudo-class check.
        match &self.pseudo {
            None => true,
            Some(sp) => active_pseudo.as_ref() == Some(sp),
        }
    }
}

/// Widget interaction pseudo-classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PseudoClass {
    Hover,
    Active,
    Disabled,
    Focus,
    Checked,
}

impl PseudoClass {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "hover" => Some(Self::Hover),
            "active" => Some(Self::Active),
            "disabled" => Some(Self::Disabled),
            "focus" => Some(Self::Focus),
            "checked" => Some(Self::Checked),
            _ => None,
        }
    }
}

impl fmt::Display for PseudoClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hover => write!(f, "hover"),
            Self::Active => write!(f, "active"),
            Self::Disabled => write!(f, "disabled"),
            Self::Focus => write!(f, "focus"),
            Self::Checked => write!(f, "checked"),
        }
    }
}

/// A property declaration: `property: value;`.
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: CssValue,
}

/// A CSS value.
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    /// A resolved color (hex, rgb, rgba, named, transparent).
    Color(ThemeColor),
    /// A length in pixels (e.g. `8px` or `8`).
    Length(f32),
    /// A plain number (e.g. opacity `0.5`).
    Number(f32),
    /// A box-shadow value.
    Shadow {
        x: f32,
        y: f32,
        blur: f32,
        color: ThemeColor,
    },
    /// A `var(--name)` or `var(--name, fallback)` reference.
    Var {
        name: String,
        fallback: Option<Box<CssValue>>,
    },
    /// A keyword like `transparent`, `none`, `inherit`.
    Keyword(String),
    /// Raw unparsed value (for unknown properties we store as-is).
    Raw(String),
}

impl CssValue {
    /// Try to extract a color, returning None if this isn't a Color value.
    pub fn as_color(&self) -> Option<&ThemeColor> {
        match self {
            CssValue::Color(c) => Some(c),
            _ => None,
        }
    }

    /// Try to extract a length in pixels.
    pub fn as_length(&self) -> Option<f32> {
        match self {
            CssValue::Length(v) | CssValue::Number(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a number.
    pub fn as_number(&self) -> Option<f32> {
        match self {
            CssValue::Number(v) | CssValue::Length(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a shadow.
    pub fn as_shadow(&self) -> Option<(f32, f32, f32, &ThemeColor)> {
        match self {
            CssValue::Shadow { x, y, blur, color } => Some((*x, *y, *blur, color)),
            _ => None,
        }
    }

    /// Check if this is the keyword "none".
    pub fn is_none(&self) -> bool {
        matches!(self, CssValue::Keyword(k) if k == "none")
    }

    /// Serialize back to a CSS string (for var substitution in raw values).
    pub fn to_css_string(&self) -> String {
        match self {
            CssValue::Color(c) => c.to_css_string(),
            CssValue::Length(v) => format!("{v}px"),
            CssValue::Number(v) => format!("{v}"),
            CssValue::Keyword(k) => k.clone(),
            CssValue::Raw(r) => r.clone(),
            CssValue::Var { name, .. } => format!("var({name})"),
            CssValue::Shadow { x, y, blur, color } => {
                format!("{x} {y}px {blur}px {}", color.to_css_string())
            }
        }
    }
}
