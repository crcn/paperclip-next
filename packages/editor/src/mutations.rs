//! # AST Mutations
//!
//! High-level semantic operations on Paperclip documents.
//!
//! ## Design Principles
//!
//! 1. **Intent-preserving**: Each mutation represents a semantic operation
//! 2. **Validated**: All mutations validate structural constraints
//! 3. **Minimal**: No redundant or overly generic operations
//! 4. **Commutative where possible**: Order-independent when semantics allow
//!
//! ## Mutation Semantics
//!
//! ### Move
//! - Atomic relocation of node to new parent
//! - Fails if parent deleted (does not create orphan)
//! - Fails if would create cycle
//! - Last move wins if concurrent moves of same node
//!
//! ### UpdateText
//! - Atomic replacement (not character diff)
//! - Last write wins if concurrent edits
//! - No merge attempts
//!
//! ### Delete
//! - Removes node and all descendants
//! - Concurrent moves to deleted nodes fail
//! - Concurrent edits of deleted nodes are no-ops

use paperclip_parser::ast::{Document, Element};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Semantic mutations (intent-preserving operations)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Mutation {
    /// Move an element to a new parent at index
    MoveElement {
        node_id: String,
        new_parent_id: String,
        index: usize,
    },

    /// Update text content of a text node (atomic replacement)
    UpdateText {
        node_id: String,
        content: String,
    },

    /// Set an inline style property
    SetInlineStyle {
        node_id: String,
        property: String,
        value: String,
    },

    /// Set an attribute value
    SetAttribute {
        node_id: String,
        name: String,
        value: String,
    },

    /// Remove a node from the tree
    RemoveNode {
        node_id: String,
    },

    /// Insert a new element (rare - most creation via templates)
    InsertElement {
        parent_id: String,
        index: usize,
        element: Element,
    },
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum MutationError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Parent not found: {0}")]
    ParentNotFound(String),

    #[error("Would create cycle")]
    CycleDetected,

    #[error("Invalid structure: {0}")]
    InvalidStructure(String),

    #[error("Cannot edit repeat instance directly")]
    CannotEditRepeatInstance,

    #[error("Node is not an element")]
    NotAnElement,

    #[error("Node is not text")]
    NotText,
}

impl Mutation {
    /// Apply mutation to AST with validation
    pub fn apply(&self, doc: &mut Document) -> Result<(), MutationError> {
        // Validate first
        self.validate(doc)?;

        // Apply based on mutation type
        match self {
            Mutation::MoveElement { node_id, new_parent_id, index } => {
                Self::apply_move(doc, node_id, new_parent_id, *index)
            }

            Mutation::UpdateText { node_id, content } => {
                Self::apply_update_text(doc, node_id, content)
            }

            Mutation::SetInlineStyle { node_id, property, value } => {
                Self::apply_set_style(doc, node_id, property, value)
            }

            Mutation::SetAttribute { node_id, name, value } => {
                Self::apply_set_attribute(doc, node_id, name, value)
            }

            Mutation::RemoveNode { node_id } => {
                Self::apply_remove(doc, node_id)
            }

            Mutation::InsertElement { parent_id, index, element } => {
                Self::apply_insert(doc, parent_id, *index, element)
            }
        }
    }

    fn apply_move(doc: &mut Document, node_id: &str, new_parent_id: &str, index: usize) -> Result<(), MutationError> {
        // Find and remove the node from its current parent
        let node_to_move = Self::remove_element_from_parent(doc, node_id)?;

        // Find new parent and insert at index
        let parent = doc.find_element_mut(new_parent_id)
            .ok_or_else(|| MutationError::ParentNotFound(new_parent_id.to_string()))?;

        if let Some(children) = parent.children_mut() {
            let insert_index = index.min(children.len());
            children.insert(insert_index, node_to_move);
            Ok(())
        } else {
            Err(MutationError::InvalidStructure(
                "Parent element cannot have children".to_string()
            ))
        }
    }

    fn apply_update_text(doc: &mut Document, node_id: &str, content: &str) -> Result<(), MutationError> {
        use paperclip_parser::ast::{Element, Expression};

        let elem = doc.find_element_mut(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        match elem {
            Element::Text { content: ref mut expr, span } => {
                *expr = Expression::Literal {
                    value: content.to_string(),
                    span: span.clone(),
                };
                Ok(())
            }
            _ => Err(MutationError::NotText),
        }
    }

    fn apply_set_style(doc: &mut Document, node_id: &str, property: &str, value: &str) -> Result<(), MutationError> {
        use paperclip_parser::ast::Element;

        let elem = doc.find_element_mut(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        match elem {
            Element::Tag { styles, span, .. } => {
                // Add or update inline style
                if styles.is_empty() {
                    // Create new style block
                    styles.push(paperclip_parser::ast::StyleBlock {
                        variants: vec![],
                        extends: vec![],
                        properties: std::collections::HashMap::new(),
                        span: span.clone(),
                    });
                }

                // Update the first style block (inline styles)
                styles[0].properties.insert(property.to_string(), value.to_string());
                Ok(())
            }
            _ => Err(MutationError::NotAnElement),
        }
    }

    fn apply_set_attribute(doc: &mut Document, node_id: &str, name: &str, value: &str) -> Result<(), MutationError> {
        use paperclip_parser::ast::{Element, Expression};

        let elem = doc.find_element_mut(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        match elem {
            Element::Tag { attributes, span, .. } => {
                attributes.insert(
                    name.to_string(),
                    Expression::Literal {
                        value: value.to_string(),
                        span: span.clone(),
                    }
                );
                Ok(())
            }
            _ => Err(MutationError::NotAnElement),
        }
    }

    fn apply_remove(doc: &mut Document, node_id: &str) -> Result<(), MutationError> {
        Self::remove_element_from_parent(doc, node_id)?;
        Ok(())
    }

    fn apply_insert(doc: &mut Document, parent_id: &str, index: usize, element: &Element) -> Result<(), MutationError> {
        let parent = doc.find_element_mut(parent_id)
            .ok_or_else(|| MutationError::ParentNotFound(parent_id.to_string()))?;

        if let Some(children) = parent.children_mut() {
            let insert_index = index.min(children.len());
            children.insert(insert_index, element.clone());
            Ok(())
        } else {
            Err(MutationError::InvalidStructure(
                "Parent element cannot have children".to_string()
            ))
        }
    }

    /// Remove an element from its parent and return it
    fn remove_element_from_parent(doc: &mut Document, node_id: &str) -> Result<Element, MutationError> {
        // Search through all components to find and remove the element
        for component in &mut doc.components {
            if let Some(body) = &mut component.body {
                if let Some(elem) = Self::remove_from_element(body, node_id) {
                    return Ok(elem);
                }
            }
        }

        Err(MutationError::NodeNotFound(node_id.to_string()))
    }

    fn remove_from_element(elem: &mut Element, target_id: &str) -> Option<Element> {
        match elem {
            Element::Tag { children, .. } | Element::Instance { children, .. } => {
                if let Some(pos) = children.iter().position(|c| c.span().id == target_id) {
                    return Some(children.remove(pos));
                }

                for child in children {
                    if let Some(removed) = Self::remove_from_element(child, target_id) {
                        return Some(removed);
                    }
                }
            }
            Element::Conditional { then_branch, else_branch, .. } => {
                if let Some(pos) = then_branch.iter().position(|c| c.span().id == target_id) {
                    return Some(then_branch.remove(pos));
                }

                for child in then_branch {
                    if let Some(removed) = Self::remove_from_element(child, target_id) {
                        return Some(removed);
                    }
                }

                if let Some(else_elems) = else_branch {
                    if let Some(pos) = else_elems.iter().position(|c| c.span().id == target_id) {
                        return Some(else_elems.remove(pos));
                    }

                    for child in else_elems {
                        if let Some(removed) = Self::remove_from_element(child, target_id) {
                            return Some(removed);
                        }
                    }
                }
            }
            Element::Repeat { body, .. } => {
                if let Some(pos) = body.iter().position(|c| c.span().id == target_id) {
                    return Some(body.remove(pos));
                }

                for child in body {
                    if let Some(removed) = Self::remove_from_element(child, target_id) {
                        return Some(removed);
                    }
                }
            }
            Element::Insert { content, .. } => {
                if let Some(pos) = content.iter().position(|c| c.span().id == target_id) {
                    return Some(content.remove(pos));
                }

                for child in content {
                    if let Some(removed) = Self::remove_from_element(child, target_id) {
                        return Some(removed);
                    }
                }
            }
            Element::Text { .. } | Element::SlotInsert { .. } => {}
        }

        None
    }

    /// Validate without applying
    pub fn validate(&self, doc: &Document) -> Result<(), MutationError> {
        match self {
            Mutation::MoveElement { node_id, new_parent_id, .. } => {
                // Check node exists
                let _node = doc.find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                // Check parent exists
                let _parent = doc.find_element(new_parent_id)
                    .ok_or_else(|| MutationError::ParentNotFound(new_parent_id.clone()))?;

                // Check not in repeat template
                if doc.is_in_repeat_template(node_id) {
                    return Err(MutationError::CannotEditRepeatInstance);
                }

                // Check wouldn't create cycle
                if doc.would_create_cycle(node_id, new_parent_id) {
                    return Err(MutationError::CycleDetected);
                }

                Ok(())
            }

            Mutation::UpdateText { node_id, .. } => {
                let elem = doc.find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                match elem {
                    Element::Text { .. } => Ok(()),
                    _ => Err(MutationError::NotText),
                }
            }

            Mutation::SetInlineStyle { node_id, .. } => {
                let elem = doc.find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                match elem {
                    Element::Tag { .. } => Ok(()),
                    _ => Err(MutationError::NotAnElement),
                }
            }

            Mutation::SetAttribute { node_id, .. } => {
                let elem = doc.find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                match elem {
                    Element::Tag { .. } => Ok(()),
                    _ => Err(MutationError::NotAnElement),
                }
            }

            Mutation::RemoveNode { node_id } => {
                doc.find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;
                Ok(())
            }

            Mutation::InsertElement { parent_id, .. } => {
                let parent = doc.find_element(parent_id)
                    .ok_or_else(|| MutationError::ParentNotFound(parent_id.clone()))?;

                match parent {
                    Element::Tag { .. } | Element::Instance { .. } => Ok(()),
                    _ => Err(MutationError::InvalidStructure(
                        "Parent cannot have children".to_string()
                    )),
                }
            }
        }
    }
}

/// Result of applying a mutation
#[derive(Debug, Clone)]
pub struct MutationResult {
    /// New version number
    pub version: u64,

    /// Optional VDOM patches (if pipeline computed them)
    pub vdom_patches: Option<Vec<u8>>,  // Serialized patches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_serialization() {
        let mutation = Mutation::UpdateText {
            node_id: "text-123".to_string(),
            content: "Hello World".to_string(),
        };

        let json = serde_json::to_string(&mutation).unwrap();
        let deserialized: Mutation = serde_json::from_str(&json).unwrap();

        assert_eq!(mutation, deserialized);
    }

    #[test]
    fn test_validation_rejects_empty_ids() {
        let source = "component Test { render div {} }";
        let doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::UpdateText {
            node_id: "".to_string(),
            content: "test".to_string(),
        };

        assert!(mutation.validate(&doc).is_err());
    }
}
