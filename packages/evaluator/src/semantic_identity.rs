/// Semantic identity system for stable node references
///
/// This module provides semantic identities that survive refactoring,
/// structure changes, and code movement. Unlike AST IDs (too low-level)
/// or VDOM paths (too fragile), semantic IDs represent the conceptual
/// location of a node: "the button in the footer slot of Card instance X"

use serde::{Deserialize, Serialize};
use std::fmt;

/// Semantic identity that survives refactoring
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticID {
    /// Hierarchical path through component tree
    pub segments: Vec<SemanticSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticSegment {
    /// Component instance
    Component {
        name: String,
        /// User-provided key or auto-generated from position
        key: Option<String>,
    },

    /// Slot reference within component
    Slot {
        name: String,
        variant: SlotVariant,
    },

    /// Element within component body
    Element {
        tag: String,
        /// Optional semantic role (from data-role or class)
        role: Option<String>,
        /// AST node ID as fallback identifier
        ast_id: String,
    },

    /// Item within repeat block
    RepeatItem {
        /// ID of the repeat block AST node
        repeat_id: String,
        /// From key attribute or stringified index
        key: String,
    },

    /// Branch within conditional
    ConditionalBranch {
        /// ID of the if block AST node
        condition_id: String,
        branch: Branch,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Branch {
    Then,
    Else,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotVariant {
    /// Default content defined in component definition
    Default,
    /// Content inserted by component user
    Inserted,
}

impl SemanticID {
    /// Create a new semantic ID from segments
    pub fn new(segments: Vec<SemanticSegment>) -> Self {
        Self { segments }
    }

    /// Create an empty semantic ID (root)
    pub fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Convert to selector string
    ///
    /// Format: `Component{"key"}::Slot::Element[ast-id]`
    ///
    /// Examples:
    /// - `Button::button[abc-5]`
    /// - `Card{"card-1"}::footer::button[xyz-9]`
    /// - `UserList::repeat[abc-3]{"user-123"}::div[xyz-5]`
    pub fn to_selector(&self) -> String {
        self.segments
            .iter()
            .map(|seg| seg.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }

    /// Get the parent semantic ID (all segments except last)
    pub fn parent(&self) -> Option<SemanticID> {
        if self.segments.is_empty() {
            None
        } else {
            Some(SemanticID {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        }
    }

    /// Append a segment to create a child semantic ID
    pub fn append(&self, segment: SemanticSegment) -> SemanticID {
        let mut segments = self.segments.clone();
        segments.push(segment);
        SemanticID { segments }
    }

    /// Check if this ID is a descendant of another
    pub fn is_descendant_of(&self, potential_ancestor: &SemanticID) -> bool {
        if self.segments.len() <= potential_ancestor.segments.len() {
            return false;
        }

        self.segments
            .iter()
            .zip(potential_ancestor.segments.iter())
            .all(|(a, b)| a == b)
    }

    /// Get the depth (number of segments)
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Check if this is a root ID (no segments)
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }
}

impl SemanticSegment {
    /// Get a short identifier for this segment (for debugging)
    pub fn short_name(&self) -> String {
        match self {
            SemanticSegment::Component { name, key } => {
                if let Some(k) = key {
                    format!("{}#{}", name, k)
                } else {
                    name.clone()
                }
            }
            SemanticSegment::Slot { name, variant } => {
                let variant_str = match variant {
                    SlotVariant::Default => "default",
                    SlotVariant::Inserted => "inserted",
                };
                format!("slot:{}[{}]", name, variant_str)
            }
            SemanticSegment::Element { tag, role, .. } => {
                if let Some(r) = role {
                    format!("{}.{}", tag, r)
                } else {
                    tag.clone()
                }
            }
            SemanticSegment::RepeatItem { key, .. } => format!("item:{}", key),
            SemanticSegment::ConditionalBranch { branch, .. } => match branch {
                Branch::Then => "if:then".to_string(),
                Branch::Else => "if:else".to_string(),
            },
        }
    }
}

impl fmt::Display for SemanticSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticSegment::Component { name, key } => {
                if let Some(k) = key {
                    write!(f, "{}{{\"{}\"}}", name, k)
                } else {
                    write!(f, "{}", name)
                }
            }
            SemanticSegment::Slot { name, variant } => {
                let variant_str = match variant {
                    SlotVariant::Default => "default",
                    SlotVariant::Inserted => "inserted",
                };
                write!(f, "{}[{}]", name, variant_str)
            }
            SemanticSegment::Element { tag, role, ast_id } => {
                if let Some(r) = role {
                    write!(f, "{}.{}[{}]", tag, r, ast_id)
                } else {
                    write!(f, "{}[{}]", tag, ast_id)
                }
            }
            SemanticSegment::RepeatItem { repeat_id, key } => {
                write!(f, "repeat[{}]{{\"{}\"}}", repeat_id, key)
            }
            SemanticSegment::ConditionalBranch {
                condition_id,
                branch,
            } => match branch {
                Branch::Then => write!(f, "if[{}].then", condition_id),
                Branch::Else => write!(f, "if[{}].else", condition_id),
            },
        }
    }
}

impl fmt::Display for SemanticID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_selector())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_id_creation() {
        let id = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "Button".to_string(),
                key: None,
            },
            SemanticSegment::Element {
                tag: "button".to_string(),
                role: None,
                ast_id: "abc-5".to_string(),
            },
        ]);

        assert_eq!(id.depth(), 2);
        assert!(!id.is_root());
        assert_eq!(id.to_selector(), "Button::button[abc-5]");
    }

    #[test]
    fn test_root_semantic_id() {
        let root = SemanticID::root();
        assert_eq!(root.depth(), 0);
        assert!(root.is_root());
        assert_eq!(root.to_selector(), "");
    }

    #[test]
    fn test_parent() {
        let id = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "Card".to_string(),
                key: Some("card-1".to_string()),
            },
            SemanticSegment::Slot {
                name: "footer".to_string(),
                variant: SlotVariant::Inserted,
            },
            SemanticSegment::Element {
                tag: "button".to_string(),
                role: None,
                ast_id: "xyz-9".to_string(),
            },
        ]);

        let parent = id.parent().unwrap();
        assert_eq!(parent.depth(), 2);
        assert_eq!(parent.to_selector(), "Card{\"card-1\"}::footer[inserted]");

        let grandparent = parent.parent().unwrap();
        assert_eq!(grandparent.depth(), 1);
        assert_eq!(grandparent.to_selector(), "Card{\"card-1\"}");

        let root = grandparent.parent().unwrap();
        assert!(root.is_root());

        assert!(root.parent().is_none());
    }

    #[test]
    fn test_append() {
        let mut id = SemanticID::root();

        id = id.append(SemanticSegment::Component {
            name: "App".to_string(),
            key: None,
        });
        assert_eq!(id.to_selector(), "App");

        id = id.append(SemanticSegment::Element {
            tag: "div".to_string(),
            role: Some("container".to_string()),
            ast_id: "abc-2".to_string(),
        });
        assert_eq!(id.to_selector(), "App::div.container[abc-2]");
    }

    #[test]
    fn test_is_descendant_of() {
        let ancestor = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "Card".to_string(),
                key: None,
            },
            SemanticSegment::Slot {
                name: "footer".to_string(),
                variant: SlotVariant::Inserted,
            },
        ]);

        let descendant = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "Card".to_string(),
                key: None,
            },
            SemanticSegment::Slot {
                name: "footer".to_string(),
                variant: SlotVariant::Inserted,
            },
            SemanticSegment::Element {
                tag: "button".to_string(),
                role: None,
                ast_id: "xyz-5".to_string(),
            },
        ]);

        assert!(descendant.is_descendant_of(&ancestor));
        assert!(!ancestor.is_descendant_of(&descendant));
        assert!(!ancestor.is_descendant_of(&ancestor)); // Not descendant of self
    }

    #[test]
    fn test_selector_with_component_key() {
        let id = SemanticID::new(vec![SemanticSegment::Component {
            name: "Button".to_string(),
            key: Some("primary".to_string()),
        }]);

        assert_eq!(id.to_selector(), "Button{\"primary\"}");
    }

    #[test]
    fn test_selector_with_repeat_item() {
        let id = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "UserList".to_string(),
                key: None,
            },
            SemanticSegment::RepeatItem {
                repeat_id: "abc-3".to_string(),
                key: "user-123".to_string(),
            },
            SemanticSegment::Element {
                tag: "div".to_string(),
                role: Some("user-card".to_string()),
                ast_id: "abc-5".to_string(),
            },
        ]);

        assert_eq!(
            id.to_selector(),
            "UserList::repeat[abc-3]{\"user-123\"}::div.user-card[abc-5]"
        );
    }

    #[test]
    fn test_selector_with_conditional() {
        let then_id = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "Dashboard".to_string(),
                key: None,
            },
            SemanticSegment::ConditionalBranch {
                condition_id: "xyz-3".to_string(),
                branch: Branch::Then,
            },
            SemanticSegment::Element {
                tag: "div".to_string(),
                role: None,
                ast_id: "xyz-4".to_string(),
            },
        ]);

        assert_eq!(
            then_id.to_selector(),
            "Dashboard::if[xyz-3].then::div[xyz-4]"
        );

        let else_id = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "Dashboard".to_string(),
                key: None,
            },
            SemanticSegment::ConditionalBranch {
                condition_id: "xyz-3".to_string(),
                branch: Branch::Else,
            },
            SemanticSegment::Element {
                tag: "div".to_string(),
                role: None,
                ast_id: "xyz-5".to_string(),
            },
        ]);

        assert_eq!(
            else_id.to_selector(),
            "Dashboard::if[xyz-3].else::div[xyz-5]"
        );
    }

    #[test]
    fn test_short_name() {
        let segments = vec![
            SemanticSegment::Component {
                name: "Button".to_string(),
                key: Some("primary".to_string()),
            },
            SemanticSegment::Slot {
                name: "footer".to_string(),
                variant: SlotVariant::Inserted,
            },
            SemanticSegment::Element {
                tag: "div".to_string(),
                role: Some("container".to_string()),
                ast_id: "abc-5".to_string(),
            },
            SemanticSegment::RepeatItem {
                repeat_id: "xyz-3".to_string(),
                key: "item-1".to_string(),
            },
            SemanticSegment::ConditionalBranch {
                condition_id: "aaa-7".to_string(),
                branch: Branch::Then,
            },
        ];

        assert_eq!(segments[0].short_name(), "Button#primary");
        assert_eq!(segments[1].short_name(), "slot:footer[inserted]");
        assert_eq!(segments[2].short_name(), "div.container");
        assert_eq!(segments[3].short_name(), "item:item-1");
        assert_eq!(segments[4].short_name(), "if:then");
    }

    #[test]
    fn test_complex_nested_structure() {
        // Represents: App::Card{"main"}::footer[inserted]::Button{"save"}::button[xyz-10]
        let id = SemanticID::new(vec![
            SemanticSegment::Component {
                name: "App".to_string(),
                key: None,
            },
            SemanticSegment::Component {
                name: "Card".to_string(),
                key: Some("main".to_string()),
            },
            SemanticSegment::Slot {
                name: "footer".to_string(),
                variant: SlotVariant::Inserted,
            },
            SemanticSegment::Component {
                name: "Button".to_string(),
                key: Some("save".to_string()),
            },
            SemanticSegment::Element {
                tag: "button".to_string(),
                role: None,
                ast_id: "xyz-10".to_string(),
            },
        ]);

        assert_eq!(
            id.to_selector(),
            "App::Card{\"main\"}::footer[inserted]::Button{\"save\"}::button[xyz-10]"
        );

        // Verify traversal
        let mut current = Some(id.clone());
        let mut depth_count = 0;

        while let Some(curr) = current {
            depth_count += 1;
            current = curr.parent();
        }

        assert_eq!(depth_count, 6); // 5 segments + root
    }
}
