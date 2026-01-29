//! CSS optimizer - deduplicates and merges CSS rules for better performance

use crate::vdom::CssRule;
use std::collections::HashMap;

/// Optimize a list of CSS rules by:
/// 1. Deduplicating identical rules
/// 2. Merging rules with same selector + media query
/// 3. Removing empty rules
pub fn optimize_css_rules(rules: Vec<CssRule>) -> Vec<CssRule> {
    if rules.is_empty() {
        return rules;
    }

    // Group rules by (selector, media_query)
    let mut grouped: HashMap<(String, Option<String>), HashMap<String, String>> = HashMap::new();

    for rule in rules {
        let key = (rule.selector.clone(), rule.media_query.clone());
        let props = grouped.entry(key).or_insert_with(HashMap::new);

        // Merge properties (later properties override earlier ones)
        for (prop_name, prop_value) in rule.properties {
            props.insert(prop_name, prop_value);
        }
    }

    // Convert back to rules
    let mut optimized = Vec::new();
    for ((selector, media_query), properties) in grouped {
        if !properties.is_empty() {
            optimized.push(CssRule {
                selector,
                properties,
                media_query,
            });
        }
    }

    // Sort for deterministic output (helps with testing and caching)
    optimized.sort_by(|a, b| {
        // Sort by media query presence first (no media query first)
        match (&a.media_query, &b.media_query) {
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            _ => a.selector.cmp(&b.selector),
        }
    });

    optimized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_identical_rules() {
        let rules = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            },
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            },
        ];

        let optimized = optimize_css_rules(rules);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].selector, ".foo");
        assert_eq!(optimized[0].properties.get("color"), Some(&"red".to_string()));
    }

    #[test]
    fn test_merge_same_selector() {
        let rules = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            },
            CssRule {
                selector: ".foo".to_string(),
                properties: [("background".to_string(), "blue".to_string())].into(),
                media_query: None,
            },
        ];

        let optimized = optimize_css_rules(rules);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].selector, ".foo");
        assert_eq!(optimized[0].properties.len(), 2);
        assert_eq!(optimized[0].properties.get("color"), Some(&"red".to_string()));
        assert_eq!(optimized[0].properties.get("background"), Some(&"blue".to_string()));
    }

    #[test]
    fn test_property_override() {
        let rules = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            },
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "blue".to_string())].into(),
                media_query: None,
            },
        ];

        let optimized = optimize_css_rules(rules);
        assert_eq!(optimized.len(), 1);
        // Later rule should win
        assert_eq!(optimized[0].properties.get("color"), Some(&"blue".to_string()));
    }

    #[test]
    fn test_separate_media_queries() {
        let rules = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            },
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "blue".to_string())].into(),
                media_query: Some("@media screen".to_string()),
            },
        ];

        let optimized = optimize_css_rules(rules);
        // Should NOT merge - different media queries
        assert_eq!(optimized.len(), 2);
    }

    #[test]
    fn test_remove_empty_rules() {
        let rules = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: HashMap::new(),
                media_query: None,
            },
            CssRule {
                selector: ".bar".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            },
        ];

        let optimized = optimize_css_rules(rules);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].selector, ".bar");
    }

    #[test]
    fn test_sorting() {
        let rules = vec![
            CssRule {
                selector: ".zebra".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: Some("@media screen".to_string()),
            },
            CssRule {
                selector: ".apple".to_string(),
                properties: [("color".to_string(), "blue".to_string())].into(),
                media_query: None,
            },
        ];

        let optimized = optimize_css_rules(rules);
        // Non-media query rules should come first
        assert_eq!(optimized[0].selector, ".apple");
        assert_eq!(optimized[1].selector, ".zebra");
    }
}
