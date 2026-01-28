/// Development mode validators for detecting unstable patterns
use crate::vdom::{VNode, VirtualDomDocument};
use crate::semantic_identity::SemanticID;
use std::collections::HashSet;

/// Validation warning level
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    /// Warning that should be addressed
    Warning,
    /// Error that will cause issues
    Error,
}

/// Validation warning
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationWarning {
    pub level: ValidationLevel,
    pub message: String,
    pub semantic_id: Option<SemanticID>,
}

impl ValidationWarning {
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: ValidationLevel::Warning,
            message: message.into(),
            semantic_id: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: ValidationLevel::Error,
            message: message.into(),
            semantic_id: None,
        }
    }

    pub fn with_semantic_id(mut self, semantic_id: SemanticID) -> Self {
        self.semantic_id = Some(semantic_id);
        self
    }
}

/// Validator for Virtual DOM documents
pub struct Validator {
    /// Whether dev mode is enabled
    dev_mode: bool,
    /// Collected warnings
    warnings: Vec<ValidationWarning>,
}

impl Validator {
    /// Create a new validator
    pub fn new(dev_mode: bool) -> Self {
        Self {
            dev_mode,
            warnings: Vec::new(),
        }
    }

    /// Validate a Virtual DOM document
    pub fn validate(&mut self, vdoc: &VirtualDomDocument) -> Vec<ValidationWarning> {
        self.warnings.clear();

        if !self.dev_mode {
            return vec![];
        }

        // Check semantic ID uniqueness
        self.check_semantic_id_uniqueness(&vdoc.nodes);

        // Check for duplicate keys in repeat blocks
        self.check_duplicate_repeat_keys(&vdoc.nodes);

        // Validate each node
        for node in &vdoc.nodes {
            self.validate_node(node);
        }

        self.warnings.clone()
    }

    /// Validate a single node
    fn validate_node(&mut self, node: &VNode) {
        match node {
            VNode::Element {
                semantic_id,
                children,
                ..
            } => {
                // Validate this element
                self.validate_semantic_id(semantic_id);

                // Recursively validate children
                for child in children {
                    self.validate_node(child);
                }
            }
            VNode::Text { .. } | VNode::Comment { .. } => {
                // No validation needed for text/comments
            }
            VNode::Error { semantic_id, .. } => {
                // Validate error node's semantic ID
                self.validate_semantic_id(semantic_id);
            }
        }
    }

    /// Validate a semantic ID for common issues
    fn validate_semantic_id(&mut self, semantic_id: &SemanticID) {
        use crate::semantic_identity::SemanticSegment;

        for segment in &semantic_id.segments {
            match segment {
                SemanticSegment::RepeatItem { key, repeat_id: _ } => {
                    // Warn if repeat key looks auto-generated
                    if key.starts_with("item-") {
                        self.warnings.push(
                            ValidationWarning::warning(
                                format!(
                                    "Repeat item has auto-generated key '{}'. \
                                     Consider providing explicit keys for stable identity.",
                                    key
                                )
                            )
                            .with_semantic_id(semantic_id.clone()),
                        );
                    }
                }
                SemanticSegment::ConditionalBranch { .. } => {
                    // Note: Conditional branches are fine, just informational
                    // Could add warning here if needed for specific patterns
                }
                SemanticSegment::Component { name, key } => {
                    // Warn if component key is missing for multiple instances
                    if key.is_none() {
                        self.warnings.push(
                            ValidationWarning::warning(
                                format!(
                                    "Component '{}' instance has no explicit key. \
                                     Auto-generated keys may not be stable.",
                                    name
                                )
                            )
                            .with_semantic_id(semantic_id.clone()),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// Check for duplicate semantic IDs
    fn check_semantic_id_uniqueness(&mut self, nodes: &[VNode]) {
        let mut seen = HashSet::new();
        self.collect_semantic_ids(nodes, &mut seen);
    }

    fn collect_semantic_ids(&mut self, nodes: &[VNode], seen: &mut HashSet<String>) {
        for node in nodes {
            if let VNode::Element {
                semantic_id,
                children,
                ..
            } = node
            {
                let selector = semantic_id.to_selector();

                if seen.contains(&selector) {
                    self.warnings.push(
                        ValidationWarning::error(format!(
                            "Duplicate semantic ID detected: {}",
                            selector
                        ))
                        .with_semantic_id(semantic_id.clone()),
                    );
                } else {
                    seen.insert(selector);
                }

                // Recurse
                self.collect_semantic_ids(children, seen);
            }
        }
    }

    /// Check for duplicate keys within repeat blocks
    fn check_duplicate_repeat_keys(&mut self, nodes: &[VNode]) {
        use std::collections::HashMap;

        // Track keys by repeat_id: repeat_id -> (key -> [semantic_ids])
        let mut repeat_keys: HashMap<String, HashMap<String, Vec<SemanticID>>> = HashMap::new();

        // Collect all repeat items
        self.collect_repeat_keys(nodes, &mut repeat_keys);

        // Check for duplicates within each repeat block
        for (repeat_id, key_map) in repeat_keys {
            for (key, semantic_ids) in key_map {
                if semantic_ids.len() > 1 {
                    // Duplicate key found!
                    for semantic_id in &semantic_ids {
                        self.warnings.push(
                            ValidationWarning::error(format!(
                                "Duplicate key '{}' in repeat block '{}'. Each item must have a unique key.",
                                key, repeat_id
                            ))
                            .with_semantic_id(semantic_id.clone()),
                        );
                    }
                }
            }
        }
    }

    fn collect_repeat_keys(
        &self,
        nodes: &[VNode],
        repeat_keys: &mut std::collections::HashMap<String, std::collections::HashMap<String, Vec<SemanticID>>>,
    ) {
        use crate::semantic_identity::SemanticSegment;

        for node in nodes {
            if let VNode::Element {
                semantic_id,
                children,
                ..
            } = node
            {
                // Check if this node has a RepeatItem segment
                for segment in &semantic_id.segments {
                    if let SemanticSegment::RepeatItem { key, repeat_id } = segment {
                        repeat_keys
                            .entry(repeat_id.clone())
                            .or_insert_with(std::collections::HashMap::new)
                            .entry(key.clone())
                            .or_insert_with(Vec::new)
                            .push(semantic_id.clone());
                    }
                }

                // Recurse into children
                self.collect_repeat_keys(children, repeat_keys);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_identity::{SemanticID, SemanticSegment};
    use std::collections::HashMap;

    #[test]
    fn test_validator_detects_auto_generated_repeat_keys() {
        let semantic_id = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "List".to_string(),
                key: Some("List-0".to_string()),
            },
            SemanticSegment::RepeatItem {
                repeat_id: "repeat-1".to_string(),
                key: "item-0".to_string(), // Auto-generated!
            },
        ]);

        let vdom = VirtualDomDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: HashMap::new(),
                styles: HashMap::new(),
                children: vec![],
                semantic_id,
                key: None,
                id: None,
            }],
            styles: vec![],
        };

        let mut validator = Validator::new(true);
        let warnings = validator.validate(&vdom);

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0]
            .message
            .contains("auto-generated key 'item-0'"));
    }

    #[test]
    fn test_validator_detects_duplicate_semantic_ids() {
        let semantic_id = SemanticID::new(vec![SemanticSegment::Element {
            tag: "div".to_string(),
            role: None,
            ast_id: "same-id".to_string(),
        }]);

        let vdom = VirtualDomDocument {
            nodes: vec![
                VNode::Element {
                    tag: "div".to_string(),
                    attributes: HashMap::new(),
                    styles: HashMap::new(),
                    children: vec![],
                    semantic_id: semantic_id.clone(),
                    key: None,
                    id: None,
                },
                VNode::Element {
                    tag: "div".to_string(),
                    attributes: HashMap::new(),
                    styles: HashMap::new(),
                    children: vec![],
                    semantic_id: semantic_id.clone(),
                    key: None,
                    id: None,
                },
            ],
            styles: vec![],
        };

        let mut validator = Validator::new(true);
        let warnings = validator.validate(&vdom);

        // Should detect duplicate
        assert!(warnings.iter().any(|w| w.message.contains("Duplicate")));
    }

    #[test]
    fn test_validator_disabled_in_production_mode() {
        let semantic_id = SemanticID::new(vec![SemanticSegment::RepeatItem {
            repeat_id: "repeat-1".to_string(),
            key: "item-0".to_string(),
        }]);

        let vdom = VirtualDomDocument {
            nodes: vec![VNode::Element {
                tag: "div".to_string(),
                attributes: HashMap::new(),
                styles: HashMap::new(),
                children: vec![],
                semantic_id,
                key: None,
                id: None,
            }],
            styles: vec![],
        };

        // Production mode (dev_mode = false)
        let mut validator = Validator::new(false);
        let warnings = validator.validate(&vdom);

        // No warnings in production mode
        assert_eq!(warnings.len(), 0);
    }

    #[test]
    fn test_validator_detects_duplicate_repeat_keys() {
        // Create two items in the same repeat block with duplicate keys
        let semantic_id_1 = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "List".to_string(),
                key: Some("List-0".to_string()),
            },
            SemanticSegment::RepeatItem {
                repeat_id: "repeat-1".to_string(),
                key: "user-123".to_string(), // Explicit key
            },
        ]);

        let semantic_id_2 = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "List".to_string(),
                key: Some("List-0".to_string()),
            },
            SemanticSegment::RepeatItem {
                repeat_id: "repeat-1".to_string(),
                key: "user-123".to_string(), // DUPLICATE!
            },
        ]);

        let vdom = VirtualDomDocument {
            nodes: vec![VNode::Element {
                tag: "ul".to_string(),
                attributes: HashMap::new(),
                styles: HashMap::new(),
                children: vec![
                    VNode::Element {
                        tag: "li".to_string(),
                        attributes: HashMap::new(),
                        styles: HashMap::new(),
                        children: vec![],
                        semantic_id: semantic_id_1,
                        key: Some("user-123".to_string()),
                        id: None,
                    },
                    VNode::Element {
                        tag: "li".to_string(),
                        attributes: HashMap::new(),
                        styles: HashMap::new(),
                        children: vec![],
                        semantic_id: semantic_id_2,
                        key: Some("user-123".to_string()),
                        id: None,
                    },
                ],
                semantic_id: SemanticID::new(vec![SemanticSegment::Element {
                    tag: "ul".to_string(),
                    role: None,
                    ast_id: "ul-1".to_string(),
                }]),
                key: None,
                id: None,
            }],
            styles: vec![],
        };

        let mut validator = Validator::new(true);
        let warnings = validator.validate(&vdom);

        // Should detect duplicate key in repeat block
        let duplicate_errors: Vec<_> = warnings
            .iter()
            .filter(|w| {
                w.level == ValidationLevel::Error && w.message.contains("Duplicate key 'user-123'")
            })
            .collect();

        assert_eq!(duplicate_errors.len(), 2); // One error for each duplicate
        assert!(duplicate_errors[0]
            .message
            .contains("in repeat block 'repeat-1'"));
    }

    #[test]
    fn test_validator_allows_same_key_in_different_repeats() {
        // Create two items with same key but in DIFFERENT repeat blocks (should be OK)
        let semantic_id_1 = SemanticID::new(vec![SemanticSegment::RepeatItem {
            repeat_id: "repeat-1".to_string(),
            key: "item-0".to_string(),
        }]);

        let semantic_id_2 = SemanticID::new(vec![SemanticSegment::RepeatItem {
            repeat_id: "repeat-2".to_string(), // Different repeat!
            key: "item-0".to_string(),         // Same key is OK
        }]);

        let vdom = VirtualDomDocument {
            nodes: vec![
                VNode::Element {
                    tag: "li".to_string(),
                    attributes: HashMap::new(),
                    styles: HashMap::new(),
                    children: vec![],
                    semantic_id: semantic_id_1,
                    key: None,
                    id: None,
                },
                VNode::Element {
                    tag: "li".to_string(),
                    attributes: HashMap::new(),
                    styles: HashMap::new(),
                    children: vec![],
                    semantic_id: semantic_id_2,
                    key: None,
                    id: None,
                },
            ],
            styles: vec![],
        };

        let mut validator = Validator::new(true);
        let warnings = validator.validate(&vdom);

        // Should NOT have duplicate key errors (different repeat blocks)
        let duplicate_errors: Vec<_> = warnings
            .iter()
            .filter(|w| w.level == ValidationLevel::Error && w.message.contains("Duplicate key"))
            .collect();

        assert_eq!(duplicate_errors.len(), 0);
    }
}
