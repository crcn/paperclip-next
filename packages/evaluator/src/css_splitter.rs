//! CSS splitting - separate vendor/global styles from component styles

use crate::vdom::CssRule;
use serde::{Deserialize, Serialize};

/// Split CSS rules into categories for better caching and loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitCss {
    /// Global styles (tokens, resets, etc.) - rarely change
    pub global: Vec<CssRule>,

    /// Component-specific styles - change frequently
    pub components: Vec<CssRule>,

    /// Critical styles (above the fold) - load first
    pub critical: Vec<CssRule>,

    /// Deferred styles (below the fold) - load later
    pub deferred: Vec<CssRule>,
}

impl SplitCss {
    pub fn new() -> Self {
        Self {
            global: Vec::new(),
            components: Vec::new(),
            critical: Vec::new(),
            deferred: Vec::new(),
        }
    }

    /// Total number of rules across all categories
    pub fn total_rules(&self) -> usize {
        self.global.len() + self.components.len() + self.critical.len() + self.deferred.len()
    }
}

/// Split CSS rules by type and priority
pub fn split_css_rules(rules: Vec<CssRule>) -> SplitCss {
    let mut split = SplitCss::new();

    for rule in rules {
        // Check if it's a global style (CSS variables, resets)
        if is_global_style(&rule) {
            split.global.push(rule);
        }
        // Check if it's critical (above the fold)
        else if is_critical_style(&rule) {
            split.critical.push(rule);
        }
        // Everything else is component-specific
        else {
            split.components.push(rule);
        }
    }

    split
}

/// Determine if a style is global (tokens, CSS variables, resets)
fn is_global_style(rule: &CssRule) -> bool {
    // CSS variables and token definitions
    if rule.selector.starts_with(":root") || rule.selector.contains("--") {
        return true;
    }

    // Reset/normalize styles (body, html, *)
    if matches!(rule.selector.as_str(), "body" | "html" | "*" | "*, *::before, *::after") {
        return true;
    }

    // Global utility classes
    if rule.selector.starts_with(".u-") || rule.selector.starts_with(".utility-") {
        return true;
    }

    false
}

/// Determine if a style is critical (above the fold)
fn is_critical_style(rule: &CssRule) -> bool {
    // No media query = visible by default = critical
    if rule.media_query.is_none() {
        // Check for common above-the-fold components
        let selector_lower = rule.selector.to_lowercase();

        // Navigation, header, hero sections are typically critical
        if selector_lower.contains("nav")
            || selector_lower.contains("header")
            || selector_lower.contains("hero")
            || selector_lower.contains("banner")
        {
            return true;
        }

        // For now, consider all non-media-query rules as critical
        // In production, you'd analyze actual component tree depth
        return true;
    }

    // Media queries are typically not critical (they're conditional)
    false
}

/// Merge split CSS back into a flat list (for rendering)
pub fn merge_split_css(split: &SplitCss) -> Vec<CssRule> {
    let mut merged = Vec::with_capacity(split.total_rules());

    // Order: critical first (above fold), then global, then components, then deferred
    merged.extend(split.critical.clone());
    merged.extend(split.global.clone());
    merged.extend(split.components.clone());
    merged.extend(split.deferred.clone());

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_identify_global_styles() {
        let rule = CssRule {
            selector: ":root".to_string(),
            properties: [("--primary-color".to_string(), "blue".to_string())].into(),
            media_query: None,
        };
        assert!(is_global_style(&rule));

        let rule = CssRule {
            selector: "body".to_string(),
            properties: [("margin".to_string(), "0".to_string())].into(),
            media_query: None,
        };
        assert!(is_global_style(&rule));
    }

    #[test]
    fn test_identify_critical_styles() {
        let rule = CssRule {
            selector: "._Navigation-nav-123".to_string(),
            properties: HashMap::new(),
            media_query: None,
        };
        assert!(is_critical_style(&rule));

        let rule = CssRule {
            selector: "._Footer-div-456".to_string(),
            properties: HashMap::new(),
            media_query: Some("@media screen".to_string()),
        };
        assert!(!is_critical_style(&rule)); // Has media query
    }

    #[test]
    fn test_split_css() {
        let rules = vec![
            CssRule {
                selector: ":root".to_string(),
                properties: [("--color".to_string(), "blue".to_string())].into(),
                media_query: None,
            },
            CssRule {
                selector: "._Header-div-123".to_string(),
                properties: [("padding".to_string(), "10px".to_string())].into(),
                media_query: None,
            },
            CssRule {
                selector: "._Footer-div-456".to_string(),
                properties: HashMap::new(),
                media_query: Some("@media screen".to_string()),
            },
        ];

        let split = split_css_rules(rules);

        assert_eq!(split.global.len(), 1);
        assert_eq!(split.critical.len(), 1);
        assert_eq!(split.components.len(), 1);
    }
}
