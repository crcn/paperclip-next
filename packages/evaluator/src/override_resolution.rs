//! Override Path Resolution
//!
//! Resolves override paths (e.g., "Card.Button.Icon") to actual VNode IDs
//! in the rendered virtual DOM tree.

use crate::vdom::VNode;
use paperclip_parser::ast::{Component, Document, Override};
use std::collections::HashMap;

/// Resolved override with target semantic ID
#[derive(Debug, Clone)]
pub struct ResolvedOverride {
    /// Original override definition
    pub override_def: Override,

    /// Resolved semantic ID prefix (e.g., "Card{\"card-1\"}::Button{\"btn-2\"}")
    pub target_semantic_id: String,
}

/// Override resolver
pub struct OverrideResolver<'a> {
    document: &'a Document,
    component_defs: HashMap<String, &'a Component>,
}

impl<'a> OverrideResolver<'a> {
    /// Create a new resolver for a document
    pub fn new(document: &'a Document) -> Self {
        let mut component_defs = HashMap::new();
        for component in &document.components {
            component_defs.insert(component.name.clone(), component);
        }

        Self {
            document,
            component_defs,
        }
    }

    /// Resolve all overrides in a component
    pub fn resolve_overrides(&self, component: &Component) -> Vec<ResolvedOverride> {
        let mut resolved = Vec::new();

        for override_def in &component.overrides {
            if let Some(target_id) = self.resolve_path(&override_def.path, component) {
                resolved.push(ResolvedOverride {
                    override_def: override_def.clone(),
                    target_semantic_id: target_id,
                });
            }
        }

        resolved
    }

    /// Resolve a path to a semantic ID
    fn resolve_path(&self, path: &[String], current_component: &Component) -> Option<String> {
        if path.is_empty() {
            return None;
        }

        // For now, simple implementation:
        // - First element is the component instance name
        // - Build semantic ID from path segments

        // In a full implementation, this would:
        // 1. Walk the component tree
        // 2. Find instances matching each path segment
        // 3. Handle indices (Button.0, Button.1)
        // 4. Resolve through component definitions

        // Simplified version: just build a semantic ID from the path
        // Use proper SemanticID format: tag[ast_id]
        let semantic_parts: Vec<String> = path
            .iter()
            .map(|segment| format!("{}[{}]", segment, segment.to_lowercase()))
            .collect();

        Some(semantic_parts.join("::"))
    }

    /// Apply resolved overrides to a virtual DOM tree
    pub fn apply_overrides(&self, vnode: &mut VNode, resolved: &[ResolvedOverride]) {
        for resolved_override in resolved {
            self.apply_override_recursive(vnode, resolved_override);
        }
    }

    /// Recursively search and apply override
    fn apply_override_recursive(&self, vnode: &mut VNode, resolved: &ResolvedOverride) {
        match vnode {
            VNode::Element {
                semantic_id,
                styles,
                attributes,
                children,
                ..
            } => {
                // Check if this node's semantic ID matches the target
                if semantic_id
                    .to_string()
                    .starts_with(&resolved.target_semantic_id)
                {
                    // Apply styles
                    for style_block in &resolved.override_def.styles {
                        for (prop, value) in &style_block.properties {
                            styles.insert(prop.clone(), value.clone());
                        }
                    }

                    // Apply attributes
                    for (name, expr) in &resolved.override_def.attributes {
                        // Convert Expression to string value
                        if let paperclip_parser::ast::Expression::Literal { value, .. } = expr {
                            attributes.insert(name.clone(), value.clone());
                        }
                    }
                }

                // Recurse into children
                for child in children {
                    self.apply_override_recursive(child, resolved);
                }
            }
            _ => {
                // Text, Comment, Error nodes don't have overrides
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse;

    #[test]
    fn test_override_resolver_creation() {
        let source = r#"
            component Card {
                render div {}

                override div {
                    style { color: red }
                }
            }
        "#;

        let doc = parse(source).unwrap();
        let resolver = OverrideResolver::new(&doc);

        assert_eq!(resolver.component_defs.len(), 1);
    }

    #[test]
    fn test_resolve_simple_override() {
        let source = r#"
            component Card {
                render div {
                    Button {}
                }

                override Button {
                    style { color: red }
                }
            }

            component Button {
                render button {}
            }
        "#;

        let doc = parse(source).unwrap();
        let resolver = OverrideResolver::new(&doc);

        let resolved = resolver.resolve_overrides(&doc.components[0]);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].override_def.path, vec!["Button"]);
    }

    #[test]
    fn test_resolve_deep_path() {
        let source = r#"
            component Page {
                render Card {}

                override Card.Button.Icon {
                    style { fill: blue }
                }
            }

            component Card {
                render div { Button {} }
            }

            component Button {
                render button { Icon {} }
            }

            component Icon {
                render svg {}
            }
        "#;

        let doc = parse(source).unwrap();
        let resolver = OverrideResolver::new(&doc);

        let resolved = resolver.resolve_overrides(&doc.components[0]);
        assert_eq!(resolved.len(), 1);
        assert_eq!(
            resolved[0].override_def.path,
            vec!["Card", "Button", "Icon"]
        );

        // Semantic ID should be hierarchical
        assert!(resolved[0].target_semantic_id.contains("Card"));
        assert!(resolved[0].target_semantic_id.contains("Button"));
        assert!(resolved[0].target_semantic_id.contains("Icon"));
    }

    #[test]
    fn test_apply_override_to_vnode() {
        use crate::vdom::VNode;
        use paperclip_semantics::{SemanticID, SemanticSegment};
        use std::collections::HashMap;

        let source = r#"
            component Card {
                render div {}

                override div(id="custom") {
                    style { color: red }
                }
            }
        "#;

        let doc = parse(source).unwrap();
        let resolver = OverrideResolver::new(&doc);
        let resolved = resolver.resolve_overrides(&doc.components[0]);

        // Create a test VNode with matching semantic ID
        let semantic_id = SemanticID::new(vec![SemanticSegment::Element {
            tag: "div".to_string(),
            role: None,
            ast_id: "div".to_string(),
        }]);
        let mut vnode = VNode::Element {
            tag: "div".to_string(),
            semantic_id,
            attributes: HashMap::new(),
            styles: HashMap::new(),
            children: vec![],
            key: None,
        };

        // Apply overrides
        resolver.apply_overrides(&mut vnode, &resolved);

        // Check that styles and attributes were applied
        if let VNode::Element {
            attributes, styles, ..
        } = &vnode
        {
            assert_eq!(attributes.get("id"), Some(&"custom".to_string()));
            assert_eq!(styles.get("color"), Some(&"red".to_string()));
        } else {
            panic!("Expected Element variant");
        }
    }
}
