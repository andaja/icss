//! `ComputedStyle` — a resolved property bag with typed accessors.

use std::collections::HashMap;

use crate::theme::color::ThemeColor;
use crate::theme::css::ast::CssValue;

/// A set of resolved CSS declarations for a specific element + state.
#[derive(Debug, Clone, Default)]
pub struct ComputedStyle {
    props: HashMap<String, CssValue>,
}

impl ComputedStyle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a property.
    pub fn set(&mut self, property: String, value: CssValue) {
        self.props.insert(property, value);
    }

    /// Get a raw value.
    pub fn get(&self, property: &str) -> Option<&CssValue> {
        self.props.get(property)
    }

    /// Get a color property.
    pub fn color(&self, property: &str) -> Option<ThemeColor> {
        match self.props.get(property)? {
            CssValue::Color(c) => Some(*c),
            _ => None,
        }
    }

    /// Get a length property (px).
    pub fn length(&self, property: &str) -> Option<f32> {
        match self.props.get(property)? {
            CssValue::Length(v) | CssValue::Number(v) => Some(*v),
            _ => None,
        }
    }

    /// Get a shadow property.
    pub fn shadow(&self, property: &str) -> Option<(f32, f32, f32, ThemeColor)> {
        match self.props.get(property)? {
            CssValue::Shadow { x, y, blur, color } => Some((*x, *y, *blur, *color)),
            _ => None,
        }
    }

    /// Get a number property (e.g., opacity).
    pub fn number(&self, property: &str) -> Option<f32> {
        match self.props.get(property)? {
            CssValue::Number(v) | CssValue::Length(v) => Some(*v),
            _ => None,
        }
    }

    /// Get a keyword property (e.g., `text-align: center`).
    pub fn keyword(&self, property: &str) -> Option<&str> {
        match self.props.get(property)? {
            CssValue::Keyword(k) => Some(k.as_str()),
            CssValue::Raw(r) => Some(r.as_str()),
            _ => None,
        }
    }

    /// Check if a property is set to `none`.
    pub fn is_none(&self, property: &str) -> bool {
        matches!(self.props.get(property), Some(v) if v.is_none())
    }
}
