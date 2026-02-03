//! AST Index - Tracks node positions using StickyIndex
//!
//! This module indexes AST nodes and stores their positions as Yjs StickyIndex,
//! allowing mutations to target nodes even after concurrent edits shift byte offsets.

use paperclip_parser::ast::{Component, Document, Element, Span};
use std::collections::HashMap;
use tracing;
use yrs::updates::encoder::Encode;
use yrs::{Assoc, Doc, GetString, IndexedSequence, StickyIndex, TextRef, Transact};

/// A node's position in the source, stored as sticky positions that survive concurrent edits.
#[derive(Debug, Clone)]
pub struct NodePosition {
    /// The node's unique ID (from AST Span.id)
    pub node_id: String,
    /// Start position encoded as StickyIndex bytes
    pub rel_start: Vec<u8>,
    /// End position encoded as StickyIndex bytes
    pub rel_end: Vec<u8>,
    /// Expected content at parse time (for conflict detection)
    pub expected_content: String,
    /// Node type for validation
    pub node_type: NodeType,
}

/// Types of nodes we track in the index
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Frame,
    Component,
    Element,
    Text,
    Style,
    Attribute,
}

/// Index of all AST nodes with their relative positions.
/// Built after parsing, used during mutation handling.
pub struct AstIndex {
    /// Map from node_id to its position info
    nodes: HashMap<String, NodePosition>,
    /// Map from node_id to parent node_id
    parents: HashMap<String, String>,
    /// Map from node_id to child node_ids (in order)
    children: HashMap<String, Vec<String>>,
}

impl AstIndex {
    /// Create an empty index
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }

    /// Build an index from a parsed AST and CRDT document.
    /// The CRDT document is used to create StickyIndex positions.
    pub fn build_from_ast(doc: &Document, crdt_doc: &Doc, source: &str) -> Self {
        let mut index = Self::new();
        let text = crdt_doc.get_or_insert_text("content");

        // Create a single transaction for all indexing operations
        let mut txn = crdt_doc.transact_mut();

        // Index @frame annotations, linking them to their components
        index.index_frames_with_components(source, doc, &text, &mut txn);

        // Index @frame annotations for top-level renders
        index.index_frames_with_renders(source, doc, &text, &mut txn);

        // Index components
        for component in &doc.components {
            index.index_component(component, &text, source, &mut txn, None);
        }

        // Index top-level renders
        for render in &doc.renders {
            index.index_element(render, &text, source, &mut txn, None);
        }

        // Debug: Log all indexed frames
        let frames: Vec<_> = index.nodes.iter()
            .filter(|(_, n)| n.node_type == NodeType::Frame)
            .map(|(id, _)| id.as_str())
            .collect();
        tracing::debug!("AstIndex built with {} frames: {:?}", frames.len(), frames);

        index
    }

    /// Create a StickyIndex for a position in the text (requires mutable transaction)
    fn create_sticky_index(
        text: &TextRef,
        txn: &mut yrs::TransactionMut,
        pos: u32,
    ) -> Option<Vec<u8>> {
        // Use Assoc::After so the position stays attached to the character after it
        text.sticky_index(txn, pos, Assoc::After)
            .map(|si: StickyIndex| si.encode_v1())
    }

    /// Index @frame annotations, linking each to its component body's span.id.
    /// This allows the designer to target frames using the VDOM element's source_id.
    fn index_frames_with_components(
        &mut self,
        source: &str,
        doc: &Document,
        text: &TextRef,
        txn: &mut yrs::TransactionMut,
    ) {
        // Iterate over components and find @frame annotations from their doc_comments
        for component in &doc.components {
            // Skip components without doc comments
            let doc_comment = match &component.doc_comment {
                Some(dc) => dc,
                None => continue,
            };

            // Check if there's a @frame annotation
            let has_frame = doc_comment.annotations.iter().any(|a| a.name == "frame");
            if !has_frame {
                continue;
            }

            // Find the actual @frame(...) position within the doc comment span
            // The annotation.span currently points to the entire doc comment, so we need to
            // search for the actual @frame text within the doc comment region
            let doc_start = doc_comment.span.start;
            let doc_end = doc_comment.span.end;
            let doc_source = source.get(doc_start..doc_end).unwrap_or("");

            // Find @frame( in the doc comment
            let frame_offset = match doc_source.find("@frame(") {
                Some(offset) => offset,
                None => continue,
            };

            let frame_start = doc_start + frame_offset;

            // Find the matching closing paren
            let after_frame = &doc_source[frame_offset..];
            let frame_end = if let Some(paren_start) = after_frame.find('(') {
                // Find matching closing paren
                let mut depth = 0;
                let mut end_pos = paren_start;
                for (i, c) in after_frame[paren_start..].chars().enumerate() {
                    match c {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                end_pos = paren_start + i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                doc_start + frame_offset + end_pos
            } else {
                // No parens, just @frame
                doc_start + frame_offset + "@frame".len()
            };

            let content = source.get(frame_start..frame_end).unwrap_or("").to_string();

            // Use the component body's span.id if available, otherwise fall back to component span.id
            let frame_id = if let Some(body) = &component.body {
                // Get body span.id - this matches the VDOM element's source_id
                match body {
                    Element::Tag { span, .. } => span.id.clone(),
                    Element::Instance { span, .. } => span.id.clone(),
                    Element::Text { span, .. } => span.id.clone(),
                    Element::Repeat { span, .. } => span.id.clone(),
                    Element::Conditional { span, .. } => span.id.clone(),
                    Element::Insert { span, .. } => span.id.clone(),
                    Element::SlotInsert { span, .. } => span.id.clone(),
                }
            } else {
                // Component without body - use component span.id
                component.span.id.clone()
            };

            // Create sticky positions for the @frame annotation itself
            if let (Some(rel_start), Some(rel_end)) = (
                Self::create_sticky_index(text, txn, frame_start as u32),
                Self::create_sticky_index(text, txn, frame_end as u32),
            ) {
                self.nodes.insert(
                    frame_id.clone(),
                    NodePosition {
                        node_id: frame_id,
                        rel_start,
                        rel_end,
                        expected_content: content,
                        node_type: NodeType::Frame,
                    },
                );
            }
        }
    }

    /// Index @frame annotations for top-level render elements.
    /// For renders, the frame ID is the element's span.id.
    fn index_frames_with_renders(
        &mut self,
        source: &str,
        doc: &Document,
        text: &TextRef,
        txn: &mut yrs::TransactionMut,
    ) {
        // Iterate over renders and their doc comments
        for (index, render) in doc.renders.iter().enumerate() {
            // Get the doc comment for this render (if any)
            let doc_comment = match doc.render_doc_comments.get(index) {
                Some(Some(dc)) => dc,
                _ => continue,
            };

            // Check if there's a @frame annotation
            let has_frame = doc_comment.annotations.iter().any(|a| a.name == "frame");
            if !has_frame {
                continue;
            }

            // Find the actual @frame(...) position within the doc comment span
            let doc_start = doc_comment.span.start;
            let doc_end = doc_comment.span.end;
            let doc_source = source.get(doc_start..doc_end).unwrap_or("");

            // Find @frame( in the doc comment
            let frame_offset = match doc_source.find("@frame(") {
                Some(offset) => offset,
                None => continue,
            };

            let frame_start = doc_start + frame_offset;

            // Find the matching closing paren
            let after_frame = &doc_source[frame_offset..];
            let frame_end = if let Some(paren_start) = after_frame.find('(') {
                let mut depth = 0;
                let mut end_pos = paren_start;
                for (i, c) in after_frame[paren_start..].chars().enumerate() {
                    match c {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                end_pos = paren_start + i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                doc_start + frame_offset + end_pos
            } else {
                doc_start + frame_offset + "@frame".len()
            };

            let content = source.get(frame_start..frame_end).unwrap_or("").to_string();

            // Use the render element's span.id as the frame ID
            let frame_id = match render {
                Element::Tag { span, .. } => span.id.clone(),
                Element::Instance { span, .. } => span.id.clone(),
                Element::Text { span, .. } => span.id.clone(),
                Element::Repeat { span, .. } => span.id.clone(),
                Element::Conditional { span, .. } => span.id.clone(),
                Element::Insert { span, .. } => span.id.clone(),
                Element::SlotInsert { span, .. } => span.id.clone(),
            };

            // Create sticky positions for the @frame annotation
            if let (Some(rel_start), Some(rel_end)) = (
                Self::create_sticky_index(text, txn, frame_start as u32),
                Self::create_sticky_index(text, txn, frame_end as u32),
            ) {
                self.nodes.insert(
                    frame_id.clone(),
                    NodePosition {
                        node_id: frame_id,
                        rel_start,
                        rel_end,
                        expected_content: content,
                        node_type: NodeType::Frame,
                    },
                );
            }
        }
    }

    /// Index a component and its children
    fn index_component(
        &mut self,
        component: &Component,
        text: &TextRef,
        source: &str,
        txn: &mut yrs::TransactionMut,
        parent_id: Option<&str>,
    ) {
        let node_id = &component.span.id;
        let start = component.span.start as u32;
        let end = component.span.end as u32;

        if let (Some(rel_start), Some(rel_end)) = (
            Self::create_sticky_index(text, txn, start),
            Self::create_sticky_index(text, txn, end),
        ) {
            let expected = source
                .get(start as usize..end as usize)
                .unwrap_or("")
                .to_string();

            self.nodes.insert(
                node_id.clone(),
                NodePosition {
                    node_id: node_id.clone(),
                    rel_start,
                    rel_end,
                    expected_content: expected,
                    node_type: NodeType::Component,
                },
            );

            if let Some(parent) = parent_id {
                self.parents.insert(node_id.clone(), parent.to_string());
                self.children
                    .entry(parent.to_string())
                    .or_default()
                    .push(node_id.clone());
            }
        }

        // Index component body
        if let Some(body) = &component.body {
            self.index_element(body, text, source, txn, Some(node_id));
        }
    }

    /// Index an element and its children
    fn index_element(
        &mut self,
        element: &Element,
        text: &TextRef,
        source: &str,
        txn: &mut yrs::TransactionMut,
        parent_id: Option<&str>,
    ) {
        match element {
            Element::Tag {
                span,
                children,
                styles,
                attributes,
                ..
            } => {
                let node_id = &span.id;
                self.index_span(span, text, source, txn, NodeType::Element, parent_id);

                // Index styles (placeholder - in full impl, we'd parse style positions)
                for (i, _style) in styles.iter().enumerate() {
                    let _style_id = format!("{}-style-{}", node_id, i);
                }

                // Index attributes (placeholder - in full impl, we'd track attribute positions)
                for (attr_name, _attr_value) in attributes {
                    let _attr_id = format!("{}-attr-{}", node_id, attr_name);
                }

                // Index children
                for child in children {
                    self.index_element(child, text, source, txn, Some(node_id));
                }
            }
            Element::Text { span, .. } => {
                self.index_span(span, text, source, txn, NodeType::Text, parent_id);
            }
            Element::Instance {
                span, children, ..
            } => {
                self.index_span(span, text, source, txn, NodeType::Element, parent_id);
                let node_id = &span.id;
                for child in children {
                    self.index_element(child, text, source, txn, Some(node_id));
                }
            }
            Element::Repeat { span, body, .. } => {
                self.index_span(span, text, source, txn, NodeType::Element, parent_id);
                let node_id = &span.id;
                for child in body {
                    self.index_element(child, text, source, txn, Some(node_id));
                }
            }
            Element::Conditional {
                span,
                then_branch,
                else_branch,
                ..
            } => {
                self.index_span(span, text, source, txn, NodeType::Element, parent_id);
                let node_id = &span.id;
                for child in then_branch {
                    self.index_element(child, text, source, txn, Some(node_id));
                }
                if let Some(else_children) = else_branch {
                    for child in else_children {
                        self.index_element(child, text, source, txn, Some(node_id));
                    }
                }
            }
            Element::Insert { span, content, .. } => {
                self.index_span(span, text, source, txn, NodeType::Element, parent_id);
                let node_id = &span.id;
                for child in content {
                    self.index_element(child, text, source, txn, Some(node_id));
                }
            }
            Element::SlotInsert { span, .. } => {
                self.index_span(span, text, source, txn, NodeType::Element, parent_id);
            }
        }
    }

    /// Index a single span
    fn index_span(
        &mut self,
        span: &Span,
        text: &TextRef,
        source: &str,
        txn: &mut yrs::TransactionMut,
        node_type: NodeType,
        parent_id: Option<&str>,
    ) {
        let node_id = &span.id;

        tracing::debug!("[AstIndex] index_span called: node_id={}, type={:?}", node_id, node_type);

        // If already indexed as a Frame, we still want to index the Element separately
        // because Frame points to @frame annotation but Element points to actual element.
        // Use a suffix to differentiate element entries for nodes that are also frames.
        let existing = self.nodes.get(node_id);
        if let Some(existing_node) = existing {
            tracing::debug!("[AstIndex] Node {} already exists as {:?}, incoming type: {:?}",
                node_id, existing_node.node_type, node_type);

            if existing_node.node_type == NodeType::Frame && node_type == NodeType::Element {
                // This element is also a frame - index it with a different key
                // The frame entry points to @frame, element entry points to actual element
                let element_node_id = format!("{}-element", node_id);
                tracing::info!("[AstIndex] Creating element variant: {}", element_node_id);

                let start = span.start as u32;
                let end = span.end as u32;

                if let (Some(rel_start), Some(rel_end)) = (
                    Self::create_sticky_index(text, txn, start),
                    Self::create_sticky_index(text, txn, end),
                ) {
                    let expected = source
                        .get(start as usize..end as usize)
                        .unwrap_or("")
                        .to_string();

                    self.nodes.insert(
                        element_node_id.clone(),
                        NodePosition {
                            node_id: element_node_id.clone(),
                            rel_start,
                            rel_end,
                            expected_content: expected,
                            node_type: NodeType::Element,
                        },
                    );
                }

                // Update parent/child with original node_id (for tree traversal)
                if let Some(parent) = parent_id {
                    self.parents.insert(node_id.clone(), parent.to_string());
                    self.children
                        .entry(parent.to_string())
                        .or_default()
                        .push(node_id.clone());
                }
                return;
            }

            // Already indexed with same type - just update relationships
            if let Some(parent) = parent_id {
                self.parents.insert(node_id.clone(), parent.to_string());
                self.children
                    .entry(parent.to_string())
                    .or_default()
                    .push(node_id.clone());
            }
            return;
        }

        let start = span.start as u32;
        let end = span.end as u32;

        if let (Some(rel_start), Some(rel_end)) = (
            Self::create_sticky_index(text, txn, start),
            Self::create_sticky_index(text, txn, end),
        ) {
            let expected = source
                .get(start as usize..end as usize)
                .unwrap_or("")
                .to_string();

            self.nodes.insert(
                node_id.clone(),
                NodePosition {
                    node_id: node_id.clone(),
                    rel_start,
                    rel_end,
                    expected_content: expected,
                    node_type,
                },
            );

            if let Some(parent) = parent_id {
                self.parents.insert(node_id.clone(), parent.to_string());
                self.children
                    .entry(parent.to_string())
                    .or_default()
                    .push(node_id.clone());
            }
        }
    }

    /// Look up a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&NodePosition> {
        self.nodes.get(node_id)
    }

    /// Get parent of a node
    pub fn get_parent(&self, node_id: &str) -> Option<&String> {
        self.parents.get(node_id)
    }

    /// Get children of a node
    pub fn get_children(&self, node_id: &str) -> Option<&Vec<String>> {
        self.children.get(node_id)
    }

    /// Resolve a StickyIndex to an absolute index in the current document state
    pub fn resolve_position(rel_pos: &[u8], crdt_doc: &Doc, _text: &TextRef) -> Option<u32> {
        use yrs::updates::decoder::Decode;
        let sticky = StickyIndex::decode_v1(rel_pos).ok()?;
        let txn = crdt_doc.transact();
        sticky.get_offset(&txn).map(|o| o.index)
    }

    /// Resolve a node's current position in the document
    pub fn resolve_node_position(
        &self,
        node_id: &str,
        crdt_doc: &Doc,
        text: &TextRef,
    ) -> Option<(u32, u32)> {
        let node = self.get_node(node_id)?;
        let start = Self::resolve_position(&node.rel_start, crdt_doc, text)?;
        let end = Self::resolve_position(&node.rel_end, crdt_doc, text)?;
        Some((start, end))
    }

    /// Check if node content matches expected (conflict detection)
    pub fn check_conflict(
        &self,
        node_id: &str,
        crdt_doc: &Doc,
        text: &TextRef,
    ) -> Result<(), ConflictError> {
        let node = self
            .get_node(node_id)
            .ok_or_else(|| ConflictError::NodeNotFound(node_id.to_string()))?;

        let (start, end) = self
            .resolve_node_position(node_id, crdt_doc, text)
            .ok_or_else(|| ConflictError::PositionLost(node_id.to_string()))?;

        let txn = crdt_doc.transact();
        let current_content = text.get_string(&txn);
        let actual = current_content
            .get(start as usize..end as usize)
            .unwrap_or("");

        if actual != node.expected_content {
            return Err(ConflictError::ContentMismatch {
                node_id: node_id.to_string(),
                expected: node.expected_content.clone(),
                actual: actual.to_string(),
            });
        }

        Ok(())
    }

    /// Get all node IDs
    pub fn all_node_ids(&self) -> impl Iterator<Item = &String> {
        self.nodes.keys()
    }

    /// Get number of indexed nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for AstIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during conflict detection
#[derive(Debug, thiserror::Error)]
pub enum ConflictError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Node position lost due to deletion: {0}")]
    PositionLost(String),

    #[error("Content mismatch for node {node_id}: expected '{expected}', found '{actual}'")]
    ContentMismatch {
        node_id: String,
        expected: String,
        actual: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use yrs::Text;

    fn create_test_doc(content: &str) -> Doc {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        {
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, content);
        }
        doc
    }

    #[test]
    fn test_sticky_position_survives_insert_before() {
        let source = "@frame(x: 0, y: 0, width: 100, height: 100)";
        let doc = create_test_doc(source);
        let text = doc.get_or_insert_text("content");

        // Create sticky positions for @frame (requires mutable transaction)
        let (rel_start, rel_end) = {
            let mut txn = doc.transact_mut();
            let start = text.sticky_index(&mut txn, 0, Assoc::After).unwrap();
            let end = text
                .sticky_index(&mut txn, source.len() as u32, Assoc::Before)
                .unwrap();
            (start, end)
        };

        // Insert content before @frame
        {
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, "// comment\n");
        }

        // Resolve positions - should point to same content
        let txn = doc.transact();
        let new_start = rel_start.get_offset(&txn).unwrap().index;
        let new_end = rel_end.get_offset(&txn).unwrap().index;

        let current_text = text.get_string(&txn);
        let actual_content = &current_text[new_start as usize..new_end as usize];

        assert_eq!(actual_content, source);
    }

    #[test]
    fn test_sticky_position_detects_modification() {
        let source = "@frame(x: 0, y: 0, width: 100, height: 100)";
        let doc = create_test_doc(source);
        let text = doc.get_or_insert_text("content");

        let (rel_start, rel_end) = {
            let mut txn = doc.transact_mut();
            let start = text.sticky_index(&mut txn, 0, Assoc::After).unwrap();
            let end = text
                .sticky_index(&mut txn, source.len() as u32, Assoc::Before)
                .unwrap();
            (start, end)
        };

        let expected = source.to_string();

        // Modify the frame content (conflict!)
        {
            let mut txn = doc.transact_mut();
            text.remove_range(&mut txn, 7, 1); // Delete 'x'
            text.insert(&mut txn, 7, "X"); // Insert 'X' (capital)
        }

        // Check content
        let txn = doc.transact();
        let new_start = rel_start.get_offset(&txn).unwrap().index;
        let new_end = rel_end.get_offset(&txn).unwrap().index;
        let current_text = text.get_string(&txn);
        let actual = &current_text[new_start as usize..new_end as usize];

        // Content should NOT match expected (it was modified)
        assert_ne!(actual, expected);
        assert!(actual.contains("X:")); // Modified version
    }

    #[test]
    fn test_index_frames() {
        let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Button {
    render div {
        text "Click me"
    }
}"#;
        let doc = create_test_doc(source);

        let ast = paperclip_parser::parse(source).unwrap();
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Should have indexed the frame
        let frame_nodes: Vec<_> = index
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Frame)
            .collect();

        assert_eq!(frame_nodes.len(), 1);
        assert!(frame_nodes[0].expected_content.contains("@frame"));
    }

    #[test]
    fn test_index_component() {
        let source = r#"component Button {
    render div {
        text "Click me"
    }
}"#;
        let doc = create_test_doc(source);
        let ast = paperclip_parser::parse(source).unwrap();
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Should have component indexed
        let component_nodes: Vec<_> = index
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Component)
            .collect();

        assert!(!component_nodes.is_empty());
    }

    #[test]
    fn test_resolve_after_concurrent_edit() {
        let source = r#"component A {
    render div { text "hello" }
}"#;
        let doc = create_test_doc(source);
        let text = doc.get_or_insert_text("content");

        let ast = paperclip_parser::parse(source).unwrap();
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Simulate concurrent edit: insert at beginning
        {
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, "// prefix\n");
        }

        // All nodes should still resolve correctly
        for node_id in index.all_node_ids() {
            let resolved = index.resolve_node_position(node_id, &doc, &text);
            assert!(resolved.is_some(), "Failed to resolve node: {}", node_id);
        }
    }

    #[test]
    fn test_frame_id_matches_component_body_span_id() {
        // TDD: Frame ID should be the component body's span.id (the render element).
        // The VDOM element's source_id comes from the render element's span.id.
        // Designer uses source_id to target mutations, so frame ID must match.
        use paperclip_parser::parse_with_path;

        let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Button {
    render div {}
}"#;
        let doc = create_test_doc(source);
        let ast = parse_with_path(source, "/test.pc").unwrap();
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Get the component body's span.id (the render element's span.id)
        // This is what the VDOM element's source_id will be
        let body = ast.components[0].body.as_ref().expect("Component should have body");
        let body_span_id = match body {
            paperclip_parser::ast::Element::Tag { span, .. } => &span.id,
            _ => panic!("Expected Tag element"),
        };

        // Frame should be findable by the body's span.id (which matches VDOM source_id)
        let frame_node = index.get_node(body_span_id);
        assert!(
            frame_node.is_some(),
            "Frame should be indexed by body span.id '{}' (VDOM source_id). Found IDs: {:?}",
            body_span_id,
            index.all_node_ids().collect::<Vec<_>>()
        );

        // The frame node type should be Frame
        let frame = frame_node.unwrap();
        assert_eq!(frame.node_type, NodeType::Frame, "Node should be a Frame type");
    }

    #[test]
    fn test_simple_pc_exact_content() {
        // Test with the exact content from simple.pc file
        use paperclip_parser::parse_with_path;

        let source = r#"/**
 * @frame(x: -37, y: -184, width: 1061, height: 952)
 */
component Card {
    render div {
        style {
            padding: 32px
            color: orange
            font-size: 32px
            font-weight: bold
            text-decoration: underline
        }
        text "hello world " {
          style {
            color: red
          }
        }
    }
}"#;
        // Use same path as the real file to get consistent span.id
        let ast = parse_with_path(source, "/Users/craig/Developer/crcn/paperclip-next/examples/simple.pc").unwrap();
        let doc = create_test_doc(source);
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Get all frame IDs
        let frame_ids: Vec<_> = index.nodes.iter()
            .filter(|(_, n)| n.node_type == NodeType::Frame)
            .map(|(id, _)| id.clone())
            .collect();

        // Get the component body's span.id
        let body = ast.components[0].body.as_ref().expect("Component should have body");
        let body_span_id = match body {
            paperclip_parser::ast::Element::Tag { span, .. } => span.id.clone(),
            _ => panic!("Expected Tag element"),
        };

        println!("Component body span.id: {}", body_span_id);
        println!("Indexed frame IDs: {:?}", frame_ids);
        println!("All node IDs: {:?}", index.all_node_ids().collect::<Vec<_>>());

        // Verify frame is indexed correctly
        assert!(
            frame_ids.contains(&body_span_id),
            "Frame should be indexed by body span.id '{}'. Indexed frames: {:?}",
            body_span_id, frame_ids
        );
    }

    // ==================== Render Frame Tests ====================

    #[test]
    fn test_index_render_frame_basic() {
        // Test that top-level renders with @frame are indexed correctly
        use paperclip_parser::parse_with_path;

        let source = r#"/**
 * @frame(x: 100, y: 200, width: 300, height: 400)
 */
div {
    text "Hello"
}"#;
        let doc = create_test_doc(source);
        let ast = parse_with_path(source, "/test.pc").unwrap();

        // Verify parser captured the frame data
        assert_eq!(ast.renders.len(), 1);
        assert_eq!(ast.render_frames.len(), 1);
        assert!(ast.render_frames[0].is_some());

        let frame = ast.render_frames[0].as_ref().unwrap();
        assert_eq!(frame.x, 100.0);
        assert_eq!(frame.y, 200.0);
        assert_eq!(frame.width, Some(300.0));
        assert_eq!(frame.height, Some(400.0));

        // Build index
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Should have indexed the frame
        let frame_nodes: Vec<_> = index
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Frame)
            .collect();

        assert_eq!(frame_nodes.len(), 1, "Should have exactly one frame indexed");
        assert!(frame_nodes[0].expected_content.contains("@frame"));
    }

    #[test]
    fn test_index_render_frame_id_matches_element_span() {
        // The frame ID should match the render element's span.id
        use paperclip_parser::parse_with_path;

        let source = r#"/**
 * @frame(x: 0, y: 0)
 */
div {
    text "Test"
}"#;
        let doc = create_test_doc(source);
        let ast = parse_with_path(source, "/test.pc").unwrap();
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Get the render element's span.id
        let render_span_id = match &ast.renders[0] {
            paperclip_parser::ast::Element::Tag { span, .. } => span.id.clone(),
            _ => panic!("Expected Tag element"),
        };

        // Frame should be indexed by the render's span.id
        let frame_node = index.get_node(&render_span_id);
        assert!(
            frame_node.is_some(),
            "Frame should be indexed by render span.id '{}'. All node IDs: {:?}",
            render_span_id,
            index.all_node_ids().collect::<Vec<_>>()
        );
        assert_eq!(frame_node.unwrap().node_type, NodeType::Frame);
    }

    #[test]
    fn test_index_mixed_components_and_renders_with_frames() {
        // Test document with both component frames and render frames
        use paperclip_parser::parse_with_path;

        let source = r#"/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Card {
    render div {
        text "Card"
    }
}

/**
 * @frame(x: 200, y: 0, width: 100, height: 100)
 */
div {
    text "Standalone"
}

/**
 * @frame(x: 400, y: 0)
 */
text "Just text"
"#;
        let doc = create_test_doc(source);
        let ast = parse_with_path(source, "/test.pc").unwrap();
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Should have 3 frames indexed (1 component + 2 renders)
        let frame_nodes: Vec<_> = index
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Frame)
            .collect();

        assert_eq!(
            frame_nodes.len(),
            3,
            "Should have 3 frames (1 component + 2 renders). Found: {:?}",
            frame_nodes.iter().map(|n| &n.node_id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_index_render_without_frame() {
        // Test that renders without @frame are not indexed as frames
        use paperclip_parser::parse_with_path;

        let source = r#"div {
    text "No frame"
}

/**
 * @frame(x: 100, y: 100)
 */
div {
    text "Has frame"
}"#;
        let doc = create_test_doc(source);
        let ast = parse_with_path(source, "/test.pc").unwrap();
        let index = AstIndex::build_from_ast(&ast, &doc, source);

        // Should have only 1 frame indexed
        let frame_nodes: Vec<_> = index
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Frame)
            .collect();

        assert_eq!(frame_nodes.len(), 1, "Only one render has @frame");
    }

    #[test]
    fn test_render_frame_with_description() {
        // Test that renders with description + frame annotation work
        use paperclip_parser::parse_with_path;

        let source = r#"/**
 * This is a hero section for the landing page.
 * @frame(x: 0, y: 0, width: 1200, height: 600)
 * More notes about usage.
 */
div {
    text "Hero"
}"#;
        let doc = create_test_doc(source);
        let ast = parse_with_path(source, "/test.pc").unwrap();

        // Verify doc comment preserved
        assert!(ast.render_doc_comments[0].is_some());
        let doc_comment = ast.render_doc_comments[0].as_ref().unwrap();
        assert!(doc_comment.description.contains("hero section"));

        // Verify frame extracted
        assert!(ast.render_frames[0].is_some());
        let frame = ast.render_frames[0].as_ref().unwrap();
        assert_eq!(frame.x, 0.0);
        assert_eq!(frame.y, 0.0);
        assert_eq!(frame.width, Some(1200.0));
        assert_eq!(frame.height, Some(600.0));

        // Verify indexed
        let index = AstIndex::build_from_ast(&ast, &doc, source);
        let frame_nodes: Vec<_> = index
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Frame)
            .collect();
        assert_eq!(frame_nodes.len(), 1);
    }

    #[test]
    fn test_render_frame_negative_coordinates() {
        use paperclip_parser::parse_with_path;

        let source = r#"/**
 * @frame(x: -100, y: -50, width: 200, height: 150)
 */
div {}"#;
        let doc = create_test_doc(source);
        let ast = parse_with_path(source, "/test.pc").unwrap();

        let frame = ast.render_frames[0].as_ref().unwrap();
        assert_eq!(frame.x, -100.0);
        assert_eq!(frame.y, -50.0);

        let index = AstIndex::build_from_ast(&ast, &doc, source);
        let frame_nodes: Vec<_> = index
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Frame)
            .collect();
        assert_eq!(frame_nodes.len(), 1);
    }
}
