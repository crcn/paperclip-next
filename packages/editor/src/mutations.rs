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

use paperclip_parser::ast::{Annotation, AnnotationValue, Document, DocComment, Element, Span};
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
    UpdateText { node_id: String, content: String },

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
    RemoveNode { node_id: String },

    /// Insert a new element (rare - most creation via templates)
    InsertElement {
        parent_id: String,
        index: usize,
        element: Element,
    },

    /// Remove an inline style property (for undo)
    RemoveInlineStyle { node_id: String, property: String },

    /// Remove an attribute (for undo)
    RemoveAttribute { node_id: String, name: String },

    /// Set or update a component annotation (e.g., @frame, @meta, @custom)
    SetComponentAnnotation {
        component_name: String,
        annotation_name: String,
        params: Vec<(String, AnnotationValue)>,
    },

    /// Remove a component annotation
    RemoveComponentAnnotation {
        component_name: String,
        annotation_name: String,
    },

    /// Set the component description (text portion of doc comment)
    SetComponentDescription {
        component_name: String,
        description: String,
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

    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    #[error("Annotation not found: {0}")]
    AnnotationNotFound(String),
}

impl Mutation {
    /// Apply mutation to AST with validation
    pub fn apply(&self, doc: &mut Document) -> Result<(), MutationError> {
        // Validate first
        self.validate(doc)?;

        // Apply based on mutation type
        match self {
            Mutation::MoveElement {
                node_id,
                new_parent_id,
                index,
            } => Self::apply_move(doc, node_id, new_parent_id, *index),

            Mutation::UpdateText { node_id, content } => {
                Self::apply_update_text(doc, node_id, content)
            }

            Mutation::SetInlineStyle {
                node_id,
                property,
                value,
            } => Self::apply_set_style(doc, node_id, property, value),

            Mutation::SetAttribute {
                node_id,
                name,
                value,
            } => Self::apply_set_attribute(doc, node_id, name, value),

            Mutation::RemoveNode { node_id } => Self::apply_remove(doc, node_id),

            Mutation::InsertElement {
                parent_id,
                index,
                element,
            } => Self::apply_insert(doc, parent_id, *index, element),

            Mutation::RemoveInlineStyle { node_id, property } => {
                Self::apply_remove_style(doc, node_id, property)
            }

            Mutation::RemoveAttribute { node_id, name } => {
                Self::apply_remove_attribute(doc, node_id, name)
            }

            Mutation::SetComponentAnnotation {
                component_name,
                annotation_name,
                params,
            } => Self::apply_set_component_annotation(doc, component_name, annotation_name, params),

            Mutation::RemoveComponentAnnotation {
                component_name,
                annotation_name,
            } => Self::apply_remove_component_annotation(doc, component_name, annotation_name),

            Mutation::SetComponentDescription {
                component_name,
                description,
            } => Self::apply_set_component_description(doc, component_name, description),
        }
    }

    fn apply_move(
        doc: &mut Document,
        node_id: &str,
        new_parent_id: &str,
        index: usize,
    ) -> Result<(), MutationError> {
        // Find and remove the node from its current parent
        let node_to_move = Self::remove_element_from_parent(doc, node_id)?;

        // Find new parent and insert at index
        let parent = doc
            .find_element_mut(new_parent_id)
            .ok_or_else(|| MutationError::ParentNotFound(new_parent_id.to_string()))?;

        if let Some(children) = parent.children_mut() {
            let insert_index = index.min(children.len());
            children.insert(insert_index, node_to_move);
            Ok(())
        } else {
            Err(MutationError::InvalidStructure(
                "Parent element cannot have children".to_string(),
            ))
        }
    }

    fn apply_update_text(
        doc: &mut Document,
        node_id: &str,
        content: &str,
    ) -> Result<(), MutationError> {
        use paperclip_parser::ast::{Element, Expression};

        let elem = doc
            .find_element_mut(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        match elem {
            Element::Text {
                content: ref mut expr,
                span,
                ..
            } => {
                *expr = Expression::Literal {
                    value: content.to_string(),
                    span: span.clone(),
                };
                Ok(())
            }
            _ => Err(MutationError::NotText),
        }
    }

    fn apply_set_style(
        doc: &mut Document,
        node_id: &str,
        property: &str,
        value: &str,
    ) -> Result<(), MutationError> {
        use paperclip_parser::ast::Element;

        let elem = doc
            .find_element_mut(node_id)
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
                styles[0]
                    .properties
                    .insert(property.to_string(), value.to_string());
                Ok(())
            }
            _ => Err(MutationError::NotAnElement),
        }
    }

    fn apply_set_attribute(
        doc: &mut Document,
        node_id: &str,
        name: &str,
        value: &str,
    ) -> Result<(), MutationError> {
        use paperclip_parser::ast::{Element, Expression};

        let elem = doc
            .find_element_mut(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        match elem {
            Element::Tag {
                attributes, span, ..
            } => {
                attributes.insert(
                    name.to_string(),
                    Expression::Literal {
                        value: value.to_string(),
                        span: span.clone(),
                    },
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

    fn apply_insert(
        doc: &mut Document,
        parent_id: &str,
        index: usize,
        element: &Element,
    ) -> Result<(), MutationError> {
        let parent = doc
            .find_element_mut(parent_id)
            .ok_or_else(|| MutationError::ParentNotFound(parent_id.to_string()))?;

        if let Some(children) = parent.children_mut() {
            let insert_index = index.min(children.len());
            children.insert(insert_index, element.clone());
            Ok(())
        } else {
            Err(MutationError::InvalidStructure(
                "Parent element cannot have children".to_string(),
            ))
        }
    }

    fn apply_remove_style(
        doc: &mut Document,
        node_id: &str,
        property: &str,
    ) -> Result<(), MutationError> {
        use paperclip_parser::ast::Element;

        let elem = doc
            .find_element_mut(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        match elem {
            Element::Tag { styles, .. } => {
                // Remove property from inline styles
                if let Some(style_block) = styles.get_mut(0) {
                    style_block.properties.remove(property);
                }
                Ok(())
            }
            _ => Err(MutationError::NotAnElement),
        }
    }

    fn apply_remove_attribute(
        doc: &mut Document,
        node_id: &str,
        name: &str,
    ) -> Result<(), MutationError> {
        use paperclip_parser::ast::Element;

        let elem = doc
            .find_element_mut(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        match elem {
            Element::Tag { attributes, .. } => {
                attributes.remove(name);
                Ok(())
            }
            _ => Err(MutationError::NotAnElement),
        }
    }

    /// Set or update a component annotation
    fn apply_set_component_annotation(
        doc: &mut Document,
        component_name: &str,
        annotation_name: &str,
        params: &[(String, AnnotationValue)],
    ) -> Result<(), MutationError> {
        let component = doc
            .components
            .iter_mut()
            .find(|c| c.name == component_name)
            .ok_or_else(|| MutationError::ComponentNotFound(component_name.to_string()))?;

        // Ensure doc_comment exists
        if component.doc_comment.is_none() {
            component.doc_comment = Some(DocComment {
                description: String::new(),
                annotations: Vec::new(),
                span: component.span.clone(),
            });
        }

        let doc_comment = component.doc_comment.as_mut().unwrap();

        // Find existing annotation or create new one
        if let Some(existing) = doc_comment
            .annotations
            .iter_mut()
            .find(|a| a.name == annotation_name)
        {
            existing.params = params.to_vec();
        } else {
            doc_comment.annotations.push(Annotation {
                name: annotation_name.to_string(),
                params: params.to_vec(),
                span: component.span.clone(),
            });
        }

        // Update frame field for backward compat if this is a @frame annotation
        if annotation_name == "frame" {
            component.frame = Self::extract_frame_from_params(params, &component.span);
        }

        Ok(())
    }

    /// Remove a component annotation
    fn apply_remove_component_annotation(
        doc: &mut Document,
        component_name: &str,
        annotation_name: &str,
    ) -> Result<(), MutationError> {
        let component = doc
            .components
            .iter_mut()
            .find(|c| c.name == component_name)
            .ok_or_else(|| MutationError::ComponentNotFound(component_name.to_string()))?;

        if let Some(doc_comment) = &mut component.doc_comment {
            let original_len = doc_comment.annotations.len();
            doc_comment
                .annotations
                .retain(|a| a.name != annotation_name);

            if doc_comment.annotations.len() == original_len {
                return Err(MutationError::AnnotationNotFound(annotation_name.to_string()));
            }

            // Clear frame field for backward compat if this was @frame
            if annotation_name == "frame" {
                component.frame = None;
            }
        } else {
            return Err(MutationError::AnnotationNotFound(annotation_name.to_string()));
        }

        Ok(())
    }

    /// Set the component description
    fn apply_set_component_description(
        doc: &mut Document,
        component_name: &str,
        description: &str,
    ) -> Result<(), MutationError> {
        let component = doc
            .components
            .iter_mut()
            .find(|c| c.name == component_name)
            .ok_or_else(|| MutationError::ComponentNotFound(component_name.to_string()))?;

        // Ensure doc_comment exists
        if component.doc_comment.is_none() {
            component.doc_comment = Some(DocComment {
                description: String::new(),
                annotations: Vec::new(),
                span: component.span.clone(),
            });
        }

        component.doc_comment.as_mut().unwrap().description = description.to_string();

        Ok(())
    }

    /// Extract frame annotation from params
    fn extract_frame_from_params(
        params: &[(String, AnnotationValue)],
        span: &Span,
    ) -> Option<paperclip_parser::ast::FrameAnnotation> {
        let get_num = |key: &str| -> Option<f64> {
            params.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
                if let AnnotationValue::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            })
        };

        let x = get_num("x")?;
        let y = get_num("y")?;
        let width = get_num("width");
        let height = get_num("height");

        Some(paperclip_parser::ast::FrameAnnotation {
            x,
            y,
            width,
            height,
            span: span.clone(),
        })
    }

    /// Remove an element from its parent and return it
    fn remove_element_from_parent(
        doc: &mut Document,
        node_id: &str,
    ) -> Result<Element, MutationError> {
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
            Element::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
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

    /// Generate the inverse mutation for undo
    ///
    /// This captures the current state and returns a mutation that will reverse
    /// the effect of this mutation. Must be called BEFORE applying the mutation.
    pub fn to_inverse(&self, doc: &Document) -> Result<Mutation, MutationError> {
        match self {
            Mutation::MoveElement { node_id, .. } => {
                // Find current parent and index
                let (parent_id, index) = Self::find_parent_and_index(doc, node_id)?;

                Ok(Mutation::MoveElement {
                    node_id: node_id.clone(),
                    new_parent_id: parent_id,
                    index,
                })
            }

            Mutation::UpdateText { node_id, .. } => {
                // Capture current text content
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                if let Element::Text { content, .. } = elem {
                    use paperclip_parser::ast::Expression;
                    let old_content = match content {
                        Expression::Literal { value, .. } => value.clone(),
                        _ => String::new(),
                    };

                    Ok(Mutation::UpdateText {
                        node_id: node_id.clone(),
                        content: old_content,
                    })
                } else {
                    Err(MutationError::NotText)
                }
            }

            Mutation::SetInlineStyle {
                node_id, property, ..
            } => {
                // Check if property currently exists
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                if let Element::Tag { styles, .. } = elem {
                    // Check if property exists in inline styles
                    if let Some(style_block) = styles.get(0) {
                        if let Some(old_value) = style_block.properties.get(property) {
                            // Property exists, restore old value
                            return Ok(Mutation::SetInlineStyle {
                                node_id: node_id.clone(),
                                property: property.clone(),
                                value: old_value.clone(),
                            });
                        }
                    }

                    // Property doesn't exist, remove it
                    Ok(Mutation::RemoveInlineStyle {
                        node_id: node_id.clone(),
                        property: property.clone(),
                    })
                } else {
                    Err(MutationError::NotAnElement)
                }
            }

            Mutation::SetAttribute { node_id, name, .. } => {
                // Check if attribute currently exists
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                if let Element::Tag { attributes, .. } = elem {
                    if let Some(old_attr) = attributes.get(name) {
                        // Attribute exists, restore old value
                        use paperclip_parser::ast::Expression;
                        let old_value = match old_attr {
                            Expression::Literal { value, .. } => value.clone(),
                            _ => String::new(),
                        };

                        return Ok(Mutation::SetAttribute {
                            node_id: node_id.clone(),
                            name: name.clone(),
                            value: old_value,
                        });
                    }

                    // Attribute doesn't exist, remove it
                    Ok(Mutation::RemoveAttribute {
                        node_id: node_id.clone(),
                        name: name.clone(),
                    })
                } else {
                    Err(MutationError::NotAnElement)
                }
            }

            Mutation::RemoveNode { node_id } => {
                // Capture node and its location
                let (parent_id, index) = Self::find_parent_and_index(doc, node_id)?;
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                Ok(Mutation::InsertElement {
                    parent_id,
                    index,
                    element: elem.clone(),
                })
            }

            Mutation::InsertElement {
                parent_id,
                index,
                element,
            } => {
                // Inverse is remove the inserted node
                Ok(Mutation::RemoveNode {
                    node_id: element.span().id.clone(),
                })
            }

            Mutation::RemoveInlineStyle { node_id, property } => {
                // Capture current value
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                if let Element::Tag { styles, .. } = elem {
                    if let Some(style_block) = styles.get(0) {
                        if let Some(value) = style_block.properties.get(property) {
                            return Ok(Mutation::SetInlineStyle {
                                node_id: node_id.clone(),
                                property: property.clone(),
                                value: value.clone(),
                            });
                        }
                    }
                }

                // Property doesn't exist, inverse is also remove (no-op)
                Ok(Mutation::RemoveInlineStyle {
                    node_id: node_id.clone(),
                    property: property.clone(),
                })
            }

            Mutation::RemoveAttribute { node_id, name } => {
                // Capture current value
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                if let Element::Tag { attributes, .. } = elem {
                    if let Some(attr) = attributes.get(name) {
                        use paperclip_parser::ast::Expression;
                        let value = match attr {
                            Expression::Literal { value, .. } => value.clone(),
                            _ => String::new(),
                        };

                        return Ok(Mutation::SetAttribute {
                            node_id: node_id.clone(),
                            name: name.clone(),
                            value,
                        });
                    }
                }

                // Attribute doesn't exist, inverse is also remove (no-op)
                Ok(Mutation::RemoveAttribute {
                    node_id: node_id.clone(),
                    name: name.clone(),
                })
            }

            Mutation::SetComponentAnnotation {
                component_name,
                annotation_name,
                params,
            } => {
                // Find component and check if annotation exists
                let component = doc
                    .components
                    .iter()
                    .find(|c| c.name == *component_name)
                    .ok_or_else(|| MutationError::ComponentNotFound(component_name.clone()))?;

                if let Some(doc_comment) = &component.doc_comment {
                    if let Some(existing) = doc_comment
                        .annotations
                        .iter()
                        .find(|a| a.name == *annotation_name)
                    {
                        // Annotation exists, restore old params
                        return Ok(Mutation::SetComponentAnnotation {
                            component_name: component_name.clone(),
                            annotation_name: annotation_name.clone(),
                            params: existing.params.clone(),
                        });
                    }
                }

                // Annotation doesn't exist, inverse is remove
                Ok(Mutation::RemoveComponentAnnotation {
                    component_name: component_name.clone(),
                    annotation_name: annotation_name.clone(),
                })
            }

            Mutation::RemoveComponentAnnotation {
                component_name,
                annotation_name,
            } => {
                // Capture current annotation params
                let component = doc
                    .components
                    .iter()
                    .find(|c| c.name == *component_name)
                    .ok_or_else(|| MutationError::ComponentNotFound(component_name.clone()))?;

                if let Some(doc_comment) = &component.doc_comment {
                    if let Some(annotation) = doc_comment
                        .annotations
                        .iter()
                        .find(|a| a.name == *annotation_name)
                    {
                        return Ok(Mutation::SetComponentAnnotation {
                            component_name: component_name.clone(),
                            annotation_name: annotation_name.clone(),
                            params: annotation.params.clone(),
                        });
                    }
                }

                Err(MutationError::AnnotationNotFound(annotation_name.clone()))
            }

            Mutation::SetComponentDescription {
                component_name,
                description,
            } => {
                // Capture current description
                let component = doc
                    .components
                    .iter()
                    .find(|c| c.name == *component_name)
                    .ok_or_else(|| MutationError::ComponentNotFound(component_name.clone()))?;

                let old_description = component
                    .doc_comment
                    .as_ref()
                    .map(|dc| dc.description.clone())
                    .unwrap_or_default();

                Ok(Mutation::SetComponentDescription {
                    component_name: component_name.clone(),
                    description: old_description,
                })
            }
        }
    }

    /// Find the parent ID and index of a node
    fn find_parent_and_index(
        doc: &Document,
        node_id: &str,
    ) -> Result<(String, usize), MutationError> {
        for component in &doc.components {
            if let Some(body) = &component.body {
                // Check if it's the root element
                if body.span().id == node_id {
                    return Err(MutationError::InvalidStructure(
                        "Cannot move root element".to_string(),
                    ));
                }

                if let Some((parent_id, index)) = Self::find_parent_recursive(body, node_id) {
                    return Ok((parent_id, index));
                }
            }
        }

        Err(MutationError::NodeNotFound(node_id.to_string()))
    }

    fn find_parent_recursive(elem: &Element, target_id: &str) -> Option<(String, usize)> {
        match elem {
            Element::Tag { children, span, .. } | Element::Instance { children, span, .. } => {
                for (index, child) in children.iter().enumerate() {
                    if child.span().id == target_id {
                        return Some((span.id.clone(), index));
                    }

                    if let Some(found) = Self::find_parent_recursive(child, target_id) {
                        return Some(found);
                    }
                }
            }
            Element::Conditional {
                then_branch,
                else_branch,
                span,
                ..
            } => {
                for (index, child) in then_branch.iter().enumerate() {
                    if child.span().id == target_id {
                        return Some((span.id.clone(), index));
                    }

                    if let Some(found) = Self::find_parent_recursive(child, target_id) {
                        return Some(found);
                    }
                }

                if let Some(else_elems) = else_branch {
                    for (index, child) in else_elems.iter().enumerate() {
                        if child.span().id == target_id {
                            return Some((span.id.clone(), index));
                        }

                        if let Some(found) = Self::find_parent_recursive(child, target_id) {
                            return Some(found);
                        }
                    }
                }
            }
            Element::Repeat { body, span, .. } => {
                for (index, child) in body.iter().enumerate() {
                    if child.span().id == target_id {
                        return Some((span.id.clone(), index));
                    }

                    if let Some(found) = Self::find_parent_recursive(child, target_id) {
                        return Some(found);
                    }
                }
            }
            Element::Insert { content, span, .. } => {
                for (index, child) in content.iter().enumerate() {
                    if child.span().id == target_id {
                        return Some((span.id.clone(), index));
                    }

                    if let Some(found) = Self::find_parent_recursive(child, target_id) {
                        return Some(found);
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
            Mutation::MoveElement {
                node_id,
                new_parent_id,
                ..
            } => {
                // Check node exists
                let _node = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                // Check parent exists
                let _parent = doc
                    .find_element(new_parent_id)
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
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                match elem {
                    Element::Text { .. } => Ok(()),
                    _ => Err(MutationError::NotText),
                }
            }

            Mutation::SetInlineStyle { node_id, .. } => {
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                match elem {
                    Element::Tag { .. } => Ok(()),
                    _ => Err(MutationError::NotAnElement),
                }
            }

            Mutation::SetAttribute { node_id, .. } => {
                let elem = doc
                    .find_element(node_id)
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
                let parent = doc
                    .find_element(parent_id)
                    .ok_or_else(|| MutationError::ParentNotFound(parent_id.clone()))?;

                match parent {
                    Element::Tag { .. } | Element::Instance { .. } => Ok(()),
                    _ => Err(MutationError::InvalidStructure(
                        "Parent cannot have children".to_string(),
                    )),
                }
            }

            Mutation::RemoveInlineStyle { node_id, .. } => {
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                match elem {
                    Element::Tag { .. } => Ok(()),
                    _ => Err(MutationError::NotAnElement),
                }
            }

            Mutation::RemoveAttribute { node_id, .. } => {
                let elem = doc
                    .find_element(node_id)
                    .ok_or_else(|| MutationError::NodeNotFound(node_id.clone()))?;

                match elem {
                    Element::Tag { .. } => Ok(()),
                    _ => Err(MutationError::NotAnElement),
                }
            }

            Mutation::SetComponentAnnotation { component_name, .. } => {
                doc.components
                    .iter()
                    .find(|c| c.name == *component_name)
                    .ok_or_else(|| MutationError::ComponentNotFound(component_name.clone()))?;
                Ok(())
            }

            Mutation::RemoveComponentAnnotation {
                component_name,
                annotation_name,
            } => {
                let component = doc
                    .components
                    .iter()
                    .find(|c| c.name == *component_name)
                    .ok_or_else(|| MutationError::ComponentNotFound(component_name.clone()))?;

                // Check annotation exists
                if let Some(doc_comment) = &component.doc_comment {
                    if !doc_comment.annotations.iter().any(|a| a.name == *annotation_name) {
                        return Err(MutationError::AnnotationNotFound(annotation_name.clone()));
                    }
                } else {
                    return Err(MutationError::AnnotationNotFound(annotation_name.clone()));
                }
                Ok(())
            }

            Mutation::SetComponentDescription { component_name, .. } => {
                doc.components
                    .iter()
                    .find(|c| c.name == *component_name)
                    .ok_or_else(|| MutationError::ComponentNotFound(component_name.clone()))?;
                Ok(())
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
    pub vdom_patches: Option<Vec<u8>>, // Serialized patches
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

    // ==================== Annotation Mutation Tests ====================

    #[test]
    fn test_set_component_annotation_creates_new() {
        let source = "component Card { render div {} }";
        let mut doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::SetComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "frame".to_string(),
            params: vec![
                ("x".to_string(), AnnotationValue::Number(100.0)),
                ("y".to_string(), AnnotationValue::Number(200.0)),
            ],
        };

        assert!(mutation.validate(&doc).is_ok());
        assert!(mutation.apply(&mut doc).is_ok());

        // Verify annotation was added
        let component = doc.components.iter().find(|c| c.name == "Card").unwrap();
        assert!(component.doc_comment.is_some());
        let doc_comment = component.doc_comment.as_ref().unwrap();
        assert_eq!(doc_comment.annotations.len(), 1);
        assert_eq!(doc_comment.annotations[0].name, "frame");

        // Verify frame field was also set for backward compat
        assert!(component.frame.is_some());
        let frame = component.frame.as_ref().unwrap();
        assert_eq!(frame.x, 100.0);
        assert_eq!(frame.y, 200.0);
    }

    #[test]
    fn test_set_component_annotation_updates_existing() {
        let source = r#"/**
 * @frame(x: 100, y: 200)
 */
component Card { render div {} }"#;
        let mut doc = paperclip_parser::parse(source).unwrap();

        // Update frame annotation
        let mutation = Mutation::SetComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "frame".to_string(),
            params: vec![
                ("x".to_string(), AnnotationValue::Number(300.0)),
                ("y".to_string(), AnnotationValue::Number(400.0)),
                ("width".to_string(), AnnotationValue::Number(500.0)),
            ],
        };

        assert!(mutation.apply(&mut doc).is_ok());

        let component = doc.components.iter().find(|c| c.name == "Card").unwrap();
        let doc_comment = component.doc_comment.as_ref().unwrap();

        // Should still have 1 annotation (updated, not duplicated)
        assert_eq!(doc_comment.annotations.len(), 1);

        // Verify params were updated
        let frame_ann = &doc_comment.annotations[0];
        assert_eq!(frame_ann.params.len(), 3);

        let x = frame_ann.params.iter().find(|(k, _)| k == "x").unwrap();
        assert_eq!(x.1, AnnotationValue::Number(300.0));
    }

    #[test]
    fn test_set_custom_annotation() {
        let source = "component Button { render button {} }";
        let mut doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::SetComponentAnnotation {
            component_name: "Button".to_string(),
            annotation_name: "meta".to_string(),
            params: vec![
                ("category".to_string(), AnnotationValue::String("interactive".to_string())),
                ("deprecated".to_string(), AnnotationValue::Boolean(false)),
                ("priority".to_string(), AnnotationValue::Number(5.0)),
            ],
        };

        assert!(mutation.apply(&mut doc).is_ok());

        let component = doc.components.iter().find(|c| c.name == "Button").unwrap();
        let doc_comment = component.doc_comment.as_ref().unwrap();
        let meta_ann = doc_comment.annotations.iter().find(|a| a.name == "meta").unwrap();

        let category = meta_ann.params.iter().find(|(k, _)| k == "category").unwrap();
        assert_eq!(category.1, AnnotationValue::String("interactive".to_string()));

        let deprecated = meta_ann.params.iter().find(|(k, _)| k == "deprecated").unwrap();
        assert_eq!(deprecated.1, AnnotationValue::Boolean(false));
    }

    #[test]
    fn test_remove_component_annotation() {
        let source = r#"/**
 * A card component
 * @frame(x: 100, y: 200)
 * @meta(category: cards)
 */
component Card { render div {} }"#;
        let mut doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::RemoveComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "frame".to_string(),
        };

        assert!(mutation.validate(&doc).is_ok());
        assert!(mutation.apply(&mut doc).is_ok());

        let component = doc.components.iter().find(|c| c.name == "Card").unwrap();
        let doc_comment = component.doc_comment.as_ref().unwrap();

        // Should have 1 annotation remaining
        assert_eq!(doc_comment.annotations.len(), 1);
        assert_eq!(doc_comment.annotations[0].name, "meta");

        // frame field should be cleared
        assert!(component.frame.is_none());
    }

    #[test]
    fn test_remove_annotation_not_found() {
        let source = "component Card { render div {} }";
        let doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::RemoveComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "nonexistent".to_string(),
        };

        assert!(matches!(
            mutation.validate(&doc),
            Err(MutationError::AnnotationNotFound(_))
        ));
    }

    #[test]
    fn test_set_component_description() {
        let source = "component Card { render div {} }";
        let mut doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::SetComponentDescription {
            component_name: "Card".to_string(),
            description: "A beautiful card component for displaying content".to_string(),
        };

        assert!(mutation.apply(&mut doc).is_ok());

        let component = doc.components.iter().find(|c| c.name == "Card").unwrap();
        let doc_comment = component.doc_comment.as_ref().unwrap();
        assert_eq!(
            doc_comment.description,
            "A beautiful card component for displaying content"
        );
    }

    #[test]
    fn test_annotation_mutation_serialization() {
        let mutation = Mutation::SetComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "frame".to_string(),
            params: vec![
                ("x".to_string(), AnnotationValue::Number(100.0)),
                ("y".to_string(), AnnotationValue::Number(200.0)),
                ("tags".to_string(), AnnotationValue::Array(vec![
                    AnnotationValue::String("ui".to_string()),
                    AnnotationValue::String("card".to_string()),
                ])),
            ],
        };

        let json = serde_json::to_string(&mutation).unwrap();
        let deserialized: Mutation = serde_json::from_str(&json).unwrap();

        assert_eq!(mutation, deserialized);
    }

    #[test]
    fn test_annotation_inverse_existing() {
        let source = r#"/**
 * @frame(x: 100, y: 200)
 */
component Card { render div {} }"#;
        let doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::SetComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "frame".to_string(),
            params: vec![
                ("x".to_string(), AnnotationValue::Number(500.0)),
                ("y".to_string(), AnnotationValue::Number(600.0)),
            ],
        };

        let inverse = mutation.to_inverse(&doc).unwrap();

        // Inverse should restore old params
        if let Mutation::SetComponentAnnotation { params, .. } = inverse {
            let x = params.iter().find(|(k, _)| k == "x").unwrap();
            assert_eq!(x.1, AnnotationValue::Number(100.0));
            let y = params.iter().find(|(k, _)| k == "y").unwrap();
            assert_eq!(y.1, AnnotationValue::Number(200.0));
        } else {
            panic!("Expected SetComponentAnnotation inverse");
        }
    }

    #[test]
    fn test_annotation_inverse_new() {
        let source = "component Card { render div {} }";
        let doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::SetComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "frame".to_string(),
            params: vec![("x".to_string(), AnnotationValue::Number(100.0))],
        };

        let inverse = mutation.to_inverse(&doc).unwrap();

        // Inverse should be RemoveComponentAnnotation since annotation didn't exist
        assert!(matches!(
            inverse,
            Mutation::RemoveComponentAnnotation {
                annotation_name,
                ..
            } if annotation_name == "frame"
        ));
    }

    #[test]
    fn test_remove_annotation_inverse() {
        let source = r#"/**
 * @meta(priority: 5)
 */
component Card { render div {} }"#;
        let doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::RemoveComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "meta".to_string(),
        };

        let inverse = mutation.to_inverse(&doc).unwrap();

        // Inverse should restore the annotation
        if let Mutation::SetComponentAnnotation {
            annotation_name,
            params,
            ..
        } = inverse
        {
            assert_eq!(annotation_name, "meta");
            let priority = params.iter().find(|(k, _)| k == "priority").unwrap();
            assert_eq!(priority.1, AnnotationValue::Number(5.0));
        } else {
            panic!("Expected SetComponentAnnotation inverse");
        }
    }

    #[test]
    fn test_description_inverse() {
        let source = r#"/**
 * Original description
 */
component Card { render div {} }"#;
        let doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::SetComponentDescription {
            component_name: "Card".to_string(),
            description: "New description".to_string(),
        };

        let inverse = mutation.to_inverse(&doc).unwrap();

        if let Mutation::SetComponentDescription { description, .. } = inverse {
            assert!(description.contains("Original description"));
        } else {
            panic!("Expected SetComponentDescription inverse");
        }
    }

    #[test]
    fn test_annotation_mutation_component_not_found() {
        let source = "component Card { render div {} }";
        let doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::SetComponentAnnotation {
            component_name: "NonExistent".to_string(),
            annotation_name: "frame".to_string(),
            params: vec![],
        };

        assert!(matches!(
            mutation.validate(&doc),
            Err(MutationError::ComponentNotFound(_))
        ));
    }

    #[test]
    fn test_multiple_annotations_preserved() {
        let source = r#"/**
 * @frame(x: 100, y: 200)
 * @meta(priority: 1)
 * @custom(data: test)
 */
component Card { render div {} }"#;
        let mut doc = paperclip_parser::parse(source).unwrap();

        // Update just the frame annotation
        let mutation = Mutation::SetComponentAnnotation {
            component_name: "Card".to_string(),
            annotation_name: "frame".to_string(),
            params: vec![
                ("x".to_string(), AnnotationValue::Number(999.0)),
                ("y".to_string(), AnnotationValue::Number(888.0)),
            ],
        };

        assert!(mutation.apply(&mut doc).is_ok());

        let component = doc.components.iter().find(|c| c.name == "Card").unwrap();
        let doc_comment = component.doc_comment.as_ref().unwrap();

        // All 3 annotations should still be present
        assert_eq!(doc_comment.annotations.len(), 3);

        // meta and custom should be unchanged
        let meta = doc_comment.annotations.iter().find(|a| a.name == "meta").unwrap();
        let priority = meta.params.iter().find(|(k, _)| k == "priority").unwrap();
        assert_eq!(priority.1, AnnotationValue::Number(1.0));

        let custom = doc_comment.annotations.iter().find(|a| a.name == "custom").unwrap();
        let data = custom.params.iter().find(|(k, _)| k == "data").unwrap();
        assert_eq!(data.1, AnnotationValue::String("test".to_string()));
    }

    #[test]
    fn test_annotation_with_array_values() {
        let source = "component List { render ul {} }";
        let mut doc = paperclip_parser::parse(source).unwrap();

        let mutation = Mutation::SetComponentAnnotation {
            component_name: "List".to_string(),
            annotation_name: "config".to_string(),
            params: vec![
                ("items".to_string(), AnnotationValue::Array(vec![
                    AnnotationValue::Number(1.0),
                    AnnotationValue::Number(2.0),
                    AnnotationValue::Number(3.0),
                ])),
                ("tags".to_string(), AnnotationValue::Array(vec![
                    AnnotationValue::String("a".to_string()),
                    AnnotationValue::String("b".to_string()),
                ])),
            ],
        };

        assert!(mutation.apply(&mut doc).is_ok());

        let component = doc.components.iter().find(|c| c.name == "List").unwrap();
        let doc_comment = component.doc_comment.as_ref().unwrap();
        let config = doc_comment.annotations.iter().find(|a| a.name == "config").unwrap();

        let items = config.params.iter().find(|(k, _)| k == "items").unwrap();
        if let AnnotationValue::Array(arr) = &items.1 {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], AnnotationValue::Number(1.0));
        } else {
            panic!("Expected array");
        }
    }
}
