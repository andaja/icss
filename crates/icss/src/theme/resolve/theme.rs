//! The `Theme` struct — loads a stylesheet and resolves styles for widgets.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::theme::color::ThemeColor;
use crate::theme::css::ast::{CssValue, PseudoClass, Stylesheet};
use crate::theme::css::parser::{ParseError, parse_stylesheet};
use crate::theme::resolve::computed::ComputedStyle;

/// A loaded theme. Constructed from an `.icss` source string.
///
/// Provides methods to resolve iced widget styles from CSS class lists.
pub struct Theme {
    stylesheet: Stylesheet,
    /// Index: class name → indices into `stylesheet.rules` that mention this class.
    class_index: HashMap<String, Vec<usize>>,
    /// Cache: (sorted classes key, pseudo) → ComputedStyle.
    cache: RefCell<HashMap<CacheKey, ComputedStyle>>,
    /// Default font for `theme.text(...)` helper when a class sets font-weight.
    /// Keeps family/stretch/style stable while only overriding weight.
    /// Defaults to `iced::Font::DEFAULT`; set via [`Theme::with_default_font`].
    default_font: iced::Font,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    /// Sorted, joined class names.
    classes: String,
    pseudo: Option<PseudoClass>,
}

impl Theme {
    /// Parse an `.icss` source string and build the theme.
    pub fn load(source: &str) -> Result<Self, ParseError> {
        let stylesheet = parse_stylesheet(source)?;
        let class_index = build_class_index(&stylesheet);
        Ok(Self {
            stylesheet,
            class_index,
            cache: RefCell::new(HashMap::new()),
            default_font: iced::Font::DEFAULT,
        })
    }

    /// Set the default font used by `theme.text(...)` when a class sets
    /// `font-weight`. The helper constructs an override as
    /// `iced::Font { weight: <parsed>, ..default_font }`, so family/stretch/
    /// style stay stable while only weight changes.
    ///
    /// Call this once at app boot after theme load, passing the same font you
    /// configured in `iced::application(...).default_font(...)`.
    pub fn with_default_font(mut self, font: iced::Font) -> Self {
        self.default_font = font;
        self
    }

    /// Read the default font used for font-weight overrides in `theme.text`.
    pub fn default_font(&self) -> iced::Font {
        self.default_font
    }

    /// Access the raw stylesheet.
    pub fn stylesheet(&self) -> &Stylesheet {
        &self.stylesheet
    }

    /// Resolve a `ComputedStyle` for an element with the given classes and pseudo-state.
    ///
    /// This is the core resolution algorithm:
    /// 1. Find all rules whose selector classes are a subset of `classes`
    /// 2. Filter by pseudo-class match
    /// 3. Sort by specificity, then source order
    /// 4. Merge declarations (later/more-specific wins)
    /// 5. Resolve `var()` references
    pub fn resolve(&self, classes: &[&str], pseudo: Option<PseudoClass>) -> ComputedStyle {
        // Check cache.
        let mut sorted_classes: Vec<&str> = classes.to_vec();
        sorted_classes.sort();
        let key = CacheKey {
            classes: sorted_classes.join(","),
            pseudo,
        };
        if let Some(cached) = self.cache.borrow().get(&key) {
            return cached.clone();
        }

        // Collect candidate rule indices via the class index.
        let mut candidate_indices: Vec<usize> = Vec::new();
        for class in &sorted_classes {
            if let Some(indices) = self.class_index.get(*class) {
                for &idx in indices {
                    if !candidate_indices.contains(&idx) {
                        candidate_indices.push(idx);
                    }
                }
            }
        }

        // Filter: selector classes must be a subset of element classes,
        // and pseudo must match.
        let mut matching: Vec<&crate::theme::css::ast::Rule> = candidate_indices
            .iter()
            .filter_map(|&idx| {
                let rule = &self.stylesheet.rules[idx];
                if rule.selector.matches(classes, pseudo) {
                    Some(rule)
                } else {
                    None
                }
            })
            .collect();

        // Also include base rules (no pseudo) when resolving with a pseudo-state.
        // Base rules form the foundation; pseudo rules override.
        if pseudo.is_some() {
            let base_matching: Vec<&crate::theme::css::ast::Rule> = candidate_indices
                .iter()
                .filter_map(|&idx| {
                    let rule = &self.stylesheet.rules[idx];
                    if rule.selector.pseudo.is_none() && rule.selector.matches(classes, None) {
                        Some(rule)
                    } else {
                        None
                    }
                })
                .collect();

            // Combine: base rules first, then pseudo-specific rules.
            let mut combined = base_matching;
            combined.extend(matching);
            matching = combined;
        }

        // Sort by specificity (ascending), then source order.
        matching.sort_by(|a, b| {
            let spec_a = a.selector.specificity();
            let spec_b = b.selector.specificity();
            spec_a
                .cmp(&spec_b)
                .then(a.source_order.cmp(&b.source_order))
        });

        // Merge declarations — later wins.
        let mut computed = ComputedStyle::new();
        for rule in &matching {
            for decl in &rule.declarations {
                let resolved = self.resolve_value(&decl.value);
                computed.set(decl.property.clone(), resolved);
            }
        }

        // Cache and return.
        self.cache.borrow_mut().insert(key, computed.clone());
        computed
    }

    /// Resolve a `CssValue`, substituting `var()` references.
    fn resolve_value(&self, value: &CssValue) -> CssValue {
        match value {
            CssValue::Var { name, fallback } => {
                if let Some(resolved) = self.stylesheet.custom_properties.get(name) {
                    self.resolve_value(resolved)
                } else if let Some(fb) = fallback {
                    self.resolve_value(fb)
                } else {
                    tracing::warn!("unresolved CSS variable: {name}");
                    CssValue::Keyword("unset".into())
                }
            }
            // Raw values may contain unresolved var() references (e.g. shadow
            // colors like `0 2px 8px var(--shadow)`). Substitute and re-parse.
            CssValue::Raw(raw) if raw.contains("var(") || raw.contains("var (") => {
                let substituted = self.substitute_vars_in_raw(raw);
                crate::theme::css::tokenize_and_parse(&substituted)
            }
            other => other.clone(),
        }
    }

    /// Text-level var() substitution for raw CSS values.
    fn substitute_vars_in_raw(&self, raw: &str) -> String {
        let mut result = raw.to_string();
        // Iteratively substitute var(--name) references.
        // The tokenizer may insert spaces: "var ( --name )" so we handle both forms.
        while let Some(start) = result.find("var(").or_else(|| result.find("var (")) {
            // Skip past "var" + optional space + "("
            let paren_pos = result[start..].find('(').unwrap() + start;
            let after = &result[paren_pos + 1..];
            if let Some(end) = after.find(')') {
                let var_name = after[..end].trim();
                let replacement = self
                    .stylesheet
                    .custom_properties
                    .get(var_name)
                    .map(|v| v.to_css_string())
                    .unwrap_or_else(|| "transparent".into());
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    replacement,
                    &result[paren_pos + 1 + end + 1..]
                );
            } else {
                break;
            }
        }
        result
    }

    /// Resolve with base state (no pseudo) + a specific pseudo layered on top.
    ///
    /// This is the typical pattern for widget style closures:
    /// compute base, then override with pseudo-specific properties.
    pub fn resolve_with_pseudo(
        &self,
        classes: &[&str],
        pseudo: Option<PseudoClass>,
    ) -> ComputedStyle {
        if pseudo.is_none() {
            return self.resolve(classes, None);
        }
        // The resolve() method already handles layering base + pseudo.
        self.resolve(classes, pseudo)
    }

    /// Look up a CSS custom property (e.g. `"primary-s0"`) and return as iced Color.
    /// Accepts with or without `--` prefix.
    pub fn color_var(&self, var_name: &str) -> Option<iced::Color> {
        let key = if var_name.starts_with("--") {
            var_name.to_string()
        } else {
            format!("--{var_name}")
        };
        let value = self.stylesheet.custom_properties.get(&key)?;
        match value {
            CssValue::Color(c) => Some(c.to_iced()),
            CssValue::Raw(s) => ThemeColor::from_hex(s).map(|c| c.to_iced()),
            _ => None,
        }
    }

    /// Look up a CSS custom property (e.g. `"space-400"`) and return as f32 length.
    /// Accepts with or without `--` prefix.
    pub fn length_var(&self, var_name: &str) -> Option<f32> {
        let key = if var_name.starts_with("--") {
            var_name.to_string()
        } else {
            format!("--{var_name}")
        };
        let value = self.stylesheet.custom_properties.get(&key)?;
        match value {
            CssValue::Length(v) | CssValue::Number(v) => Some(*v),
            CssValue::Raw(s) => s.trim_end_matches("px").parse().ok(),
            _ => None,
        }
    }

    /// Helper: resolve a color from computed style. Does NOT apply opacity.
    pub fn resolve_color(computed: &ComputedStyle, property: &str) -> Option<ThemeColor> {
        computed.color(property)
    }
}

/// Build an index from class names to rule indices.
fn build_class_index(stylesheet: &Stylesheet) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, rule) in stylesheet.rules.iter().enumerate() {
        for class in &rule.selector.classes {
            index.entry(class.clone()).or_default().push(i);
        }
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CSS: &str = r#"
        :root {
            --primary: #0f3460;
            --primary-light: #1a4a80;
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

        .button.primary {
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
        }

        .small {
            border-radius: 4px;
        }
    "#;

    #[test]
    fn resolve_single_class() {
        let theme = Theme::load(TEST_CSS).unwrap();
        let style = theme.resolve(&["button"], None);
        assert!(style.color("color").is_some());
        assert_eq!(style.length("border-radius"), Some(8.0));
    }

    #[test]
    fn resolve_multi_class() {
        let theme = Theme::load(TEST_CSS).unwrap();
        let style = theme.resolve(&["button", "primary"], None);
        // Should have color from .button, background-color from .primary,
        // and box-shadow from .button.primary.
        assert!(style.color("color").is_some());
        assert!(style.color("background-color").is_some());
        assert!(style.shadow("box-shadow").is_some());
    }

    #[test]
    fn resolve_pseudo_layers_on_base() {
        let theme = Theme::load(TEST_CSS).unwrap();
        let style = theme.resolve(&["button", "primary"], Some(PseudoClass::Hover));
        // hover should override background-color from :hover rule,
        // but keep other base properties.
        let bg = style.color("background-color").unwrap();
        // --primary-light = #1a4a80 → r ≈ 0.102
        assert!(bg.r > 0.09 && bg.r < 0.12);
    }

    #[test]
    fn resolve_composable_override() {
        let theme = Theme::load(TEST_CSS).unwrap();
        let style = theme.resolve(&["button", "primary", "small"], None);
        // .small has border-radius: 4px which should override .button's 8px
        // because .small comes later in source order with same specificity.
        assert_eq!(style.length("border-radius"), Some(4.0));
    }

    #[test]
    fn resolve_unmatched_class() {
        let theme = Theme::load(TEST_CSS).unwrap();
        let style = theme.resolve(&["nonexistent"], None);
        assert!(style.color("color").is_none());
    }

    #[test]
    fn var_resolution() {
        let theme = Theme::load(TEST_CSS).unwrap();
        let style = theme.resolve(&["primary"], None);
        let bg = style.color("background-color").unwrap();
        // --primary = #0f3460 → r ≈ 0.059
        assert!(bg.r > 0.05 && bg.r < 0.07);
    }

    #[test]
    fn caching_works() {
        let theme = Theme::load(TEST_CSS).unwrap();
        let s1 = theme.resolve(&["button"], None);
        let s2 = theme.resolve(&["button"], None);
        // Should be identical (from cache).
        assert_eq!(s1.length("border-radius"), s2.length("border-radius"));
    }
}
