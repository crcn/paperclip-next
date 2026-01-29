//! CSS diffing - compute incremental updates for hot reload

use crate::vdom::CssRule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A CSS patch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CssPatch {
    /// Add a new CSS rule
    Add {
        rule: CssRule,
    },

    /// Update an existing rule's properties
    Update {
        selector: String,
        media_query: Option<String>,
        properties: HashMap<String, String>,
    },

    /// Remove a CSS rule
    Remove {
        selector: String,
        media_query: Option<String>,
    },
}

/// Result of CSS diffing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssDiff {
    pub patches: Vec<CssPatch>,
}

impl CssDiff {
    pub fn new() -> Self {
        Self {
            patches: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }

    pub fn patch_count(&self) -> usize {
        self.patches.len()
    }
}

/// Compute CSS diff between old and new rules
pub fn diff_css_rules(old_rules: &[CssRule], new_rules: &[CssRule]) -> CssDiff {
    let mut diff = CssDiff::new();

    // Index old rules by (selector, media_query)
    let mut old_map: HashMap<(String, Option<String>), &CssRule> = HashMap::new();
    for rule in old_rules {
        let key = (rule.selector.clone(), rule.media_query.clone());
        old_map.insert(key, rule);
    }

    // Index new rules by (selector, media_query)
    let mut new_map: HashMap<(String, Option<String>), &CssRule> = HashMap::new();
    for rule in new_rules {
        let key = (rule.selector.clone(), rule.media_query.clone());
        new_map.insert(key, rule);
    }

    // Find added and updated rules
    for (key, new_rule) in &new_map {
        if let Some(old_rule) = old_map.get(key) {
            // Rule exists - check if properties changed
            if old_rule.properties != new_rule.properties {
                diff.patches.push(CssPatch::Update {
                    selector: new_rule.selector.clone(),
                    media_query: new_rule.media_query.clone(),
                    properties: new_rule.properties.clone(),
                });
            }
            // If properties are the same, no patch needed
        } else {
            // New rule
            diff.patches.push(CssPatch::Add {
                rule: (*new_rule).clone(),
            });
        }
    }

    // Find removed rules
    for (key, _old_rule) in &old_map {
        if !new_map.contains_key(key) {
            diff.patches.push(CssPatch::Remove {
                selector: key.0.clone(),
                media_query: key.1.clone(),
            });
        }
    }

    diff
}

/// Apply CSS patches to a rule set (for testing)
pub fn apply_css_patches(rules: &mut Vec<CssRule>, patches: &[CssPatch]) {
    for patch in patches {
        match patch {
            CssPatch::Add { rule } => {
                rules.push(rule.clone());
            }
            CssPatch::Update {
                selector,
                media_query,
                properties,
            } => {
                // Find and update the rule
                if let Some(existing) = rules.iter_mut().find(|r| {
                    &r.selector == selector && &r.media_query == media_query
                }) {
                    existing.properties = properties.clone();
                }
            }
            CssPatch::Remove {
                selector,
                media_query,
            } => {
                // Remove the rule
                rules.retain(|r| {
                    &r.selector != selector || &r.media_query != media_query
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_changes() {
        let rules = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            }
        ];

        let diff = diff_css_rules(&rules, &rules);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_add_rule() {
        let old = vec![];
        let new = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            }
        ];

        let diff = diff_css_rules(&old, &new);
        assert_eq!(diff.patch_count(), 1);
        assert!(matches!(diff.patches[0], CssPatch::Add { .. }));
    }

    #[test]
    fn test_remove_rule() {
        let old = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            }
        ];
        let new = vec![];

        let diff = diff_css_rules(&old, &new);
        assert_eq!(diff.patch_count(), 1);
        assert!(matches!(diff.patches[0], CssPatch::Remove { .. }));
    }

    #[test]
    fn test_update_rule() {
        let old = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            }
        ];
        let new = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "blue".to_string())].into(),
                media_query: None,
            }
        ];

        let diff = diff_css_rules(&old, &new);
        assert_eq!(diff.patch_count(), 1);
        assert!(matches!(diff.patches[0], CssPatch::Update { .. }));
    }

    #[test]
    fn test_apply_patches() {
        let mut rules = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            }
        ];

        let patches = vec![
            CssPatch::Update {
                selector: ".foo".to_string(),
                media_query: None,
                properties: [("color".to_string(), "blue".to_string())].into(),
            },
            CssPatch::Add {
                rule: CssRule {
                    selector: ".bar".to_string(),
                    properties: [("background".to_string(), "green".to_string())].into(),
                    media_query: None,
                }
            }
        ];

        apply_css_patches(&mut rules, &patches);

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].properties.get("color"), Some(&"blue".to_string()));
        assert_eq!(rules[1].selector, ".bar");
    }

    #[test]
    fn test_media_query_differentiation() {
        let old = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            }
        ];
        let new = vec![
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "red".to_string())].into(),
                media_query: None,
            },
            CssRule {
                selector: ".foo".to_string(),
                properties: [("color".to_string(), "blue".to_string())].into(),
                media_query: Some("@media screen".to_string()),
            }
        ];

        let diff = diff_css_rules(&old, &new);
        // Should add the media query version (different key)
        assert_eq!(diff.patch_count(), 1);
        assert!(matches!(diff.patches[0], CssPatch::Add { .. }));
    }
}
