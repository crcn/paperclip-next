//! Mutation Handler - Translates semantic mutations to CRDT edits
//!
//! This module takes semantic visual mutations (SetFrameBounds, MoveNode, etc.)
//! and translates them into Y.Text CRDT edits using StickyIndex for safety.

use crate::ast_index::{AstIndex, ConflictError, NodeType};
use crate::crdt::CrdtDocument;
use yrs::{Doc, GetString, Transact};

/// Result of applying a mutation
#[derive(Debug)]
pub enum MutationResult {
    /// Mutation was applied successfully
    Applied { mutation_id: String, new_version: u64 },
    /// Mutation was transformed due to concurrent changes
    Rebased {
        original_mutation_id: String,
        reason: String,
        new_version: u64,
    },
    /// Mutation had no effect (node deleted, already at target, etc.)
    Noop { mutation_id: String, reason: String },
    /// Mutation was rejected due to conflict
    Rejected { mutation_id: String, reason: String },
}

/// Semantic mutations that can be applied
#[derive(Debug, Clone)]
pub enum Mutation {
    SetFrameBounds {
        mutation_id: String,
        frame_id: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    SetStyleProperty {
        mutation_id: String,
        node_id: String,
        property: String,
        value: String,
    },
    DeleteStyleProperty {
        mutation_id: String,
        node_id: String,
        property: String,
    },
    SetTextContent {
        mutation_id: String,
        node_id: String,
        content: String,
    },
    DeleteNode {
        mutation_id: String,
        node_id: String,
    },
    MoveNode {
        mutation_id: String,
        node_id: String,
        new_parent_id: String,
        index: u32,
    },
    InsertNode {
        mutation_id: String,
        parent_id: String,
        index: u32,
        source: String,
    },
    SetAttribute {
        mutation_id: String,
        node_id: String,
        name: String,
        value: String,
    },
    /// Set or update a component annotation (e.g., @frame, @meta, @custom)
    SetComponentAnnotation {
        mutation_id: String,
        component_name: String,
        annotation_name: String,
        /// Key-value pairs serialized as "key: value, key2: value2"
        params_str: String,
    },
    /// Remove a component annotation
    RemoveComponentAnnotation {
        mutation_id: String,
        component_name: String,
        annotation_name: String,
    },
}

impl Mutation {
    pub fn mutation_id(&self) -> &str {
        match self {
            Mutation::SetFrameBounds { mutation_id, .. } => mutation_id,
            Mutation::SetStyleProperty { mutation_id, .. } => mutation_id,
            Mutation::DeleteStyleProperty { mutation_id, .. } => mutation_id,
            Mutation::SetTextContent { mutation_id, .. } => mutation_id,
            Mutation::DeleteNode { mutation_id, .. } => mutation_id,
            Mutation::MoveNode { mutation_id, .. } => mutation_id,
            Mutation::InsertNode { mutation_id, .. } => mutation_id,
            Mutation::SetAttribute { mutation_id, .. } => mutation_id,
            Mutation::SetComponentAnnotation { mutation_id, .. } => mutation_id,
            Mutation::RemoveComponentAnnotation { mutation_id, .. } => mutation_id,
        }
    }
}

/// Information about a style block's location within element source
#[derive(Debug)]
#[allow(dead_code)]
struct StyleBlockInfo {
    /// Start of "style {"
    start: usize,
    /// End of "}"
    end: usize,
    /// Start of content (after opening brace)
    content_start: usize,
    /// End of content (before closing brace)
    content_end: usize,
}

/// Errors that can occur during mutation handling
#[derive(Debug, thiserror::Error)]
pub enum MutationError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Conflict detected: {0}")]
    Conflict(#[from] ConflictError),

    #[error("Invalid mutation: {0}")]
    InvalidMutation(String),

    #[error("Position resolution failed for node: {0}")]
    PositionResolutionFailed(String),

    #[error("Parse error after mutation: {0}")]
    ParseError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Handles mutation translation and application
pub struct MutationHandler {
    /// Current AST index (rebuilt after each successful mutation)
    index: AstIndex,
    /// File path for consistent span.id generation (CRC32 of path)
    file_path: String,
}

impl MutationHandler {
    /// Create a new mutation handler (legacy, uses empty path - span.ids may not match)
    pub fn new() -> Self {
        Self {
            index: AstIndex::new(),
            file_path: String::new(),
        }
    }

    /// Create a new mutation handler with file path for correct span.id generation
    pub fn new_with_path(file_path: &str) -> Self {
        Self {
            index: AstIndex::new(),
            file_path: file_path.to_string(),
        }
    }

    /// Rebuild the index from current document state
    /// Uses stored file_path for consistent span.id generation
    pub fn rebuild_index(&mut self, doc: &Doc, source: &str) -> Result<(), MutationError> {
        let ast = paperclip_parser::parse_with_path(source, &self.file_path)
            .map_err(|e| MutationError::ParseError(e.to_string()))?;
        self.index = AstIndex::build_from_ast(&ast, doc, source);
        Ok(())
    }

    /// Apply a mutation to the CRDT document
    pub fn apply_mutation(
        &mut self,
        mutation: &Mutation,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        match mutation {
            Mutation::SetFrameBounds {
                mutation_id,
                frame_id,
                x,
                y,
                width,
                height,
            } => self.apply_set_frame_bounds(mutation_id, frame_id, *x, *y, *width, *height, crdt_doc),
            Mutation::SetTextContent {
                mutation_id,
                node_id,
                content,
            } => self.apply_set_text_content(mutation_id, node_id, content, crdt_doc),
            Mutation::DeleteNode {
                mutation_id,
                node_id,
            } => self.apply_delete_node(mutation_id, node_id, crdt_doc),
            Mutation::SetStyleProperty {
                mutation_id,
                node_id,
                property,
                value,
            } => self.apply_set_style_property(mutation_id, node_id, property, value, crdt_doc),
            Mutation::DeleteStyleProperty {
                mutation_id,
                node_id,
                property,
            } => self.apply_delete_style_property(mutation_id, node_id, property, crdt_doc),
            Mutation::MoveNode {
                mutation_id,
                node_id,
                new_parent_id,
                index,
            } => self.apply_move_node(mutation_id, node_id, new_parent_id, *index, crdt_doc),
            Mutation::InsertNode {
                mutation_id,
                parent_id,
                index,
                source,
            } => self.apply_insert_node(mutation_id, parent_id, *index, source, crdt_doc),
            Mutation::SetAttribute {
                mutation_id,
                node_id: _,
                name: _,
                value: _,
            } => Ok(MutationResult::Noop {
                mutation_id: mutation_id.clone(),
                reason: "Attribute editing not yet implemented".to_string(),
            }),

            Mutation::SetComponentAnnotation {
                mutation_id,
                component_name,
                annotation_name,
                params_str,
            } => self.apply_set_component_annotation(
                mutation_id,
                component_name,
                annotation_name,
                params_str,
                crdt_doc,
            ),

            Mutation::RemoveComponentAnnotation {
                mutation_id,
                component_name,
                annotation_name,
            } => self.apply_remove_component_annotation(
                mutation_id,
                component_name,
                annotation_name,
                crdt_doc,
            ),
        }
    }

    /// Apply SetFrameBounds mutation
    fn apply_set_frame_bounds(
        &mut self,
        mutation_id: &str,
        frame_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        // Debug: Log what frame_id is being requested
        tracing::debug!("SetFrameBounds: Looking for frame_id={}", frame_id);

        // Find the frame node
        let node = self
            .index
            .get_node(frame_id)
            .ok_or_else(|| {
                // Debug: Log all available node IDs on failure
                let all_ids: Vec<_> = self.index.all_node_ids().collect();
                let frame_ids: Vec<_> = all_ids.iter()
                    .filter(|id| self.index.get_node(id).map(|n| n.node_type == NodeType::Frame).unwrap_or(false))
                    .collect();
                tracing::error!(
                    "SetFrameBounds failed: frame_id={} not found. Available frames: {:?}. All nodes: {:?}",
                    frame_id, frame_ids, all_ids
                );
                MutationError::NodeNotFound(frame_id.to_string())
            })?;

        if node.node_type != NodeType::Frame {
            return Err(MutationError::InvalidMutation(format!(
                "Node {} is not a frame",
                frame_id
            )));
        }

        // Get doc reference for read operations
        let doc = crdt_doc.doc();
        let text = doc.get_or_insert_text("content");

        // Check for conflicts
        self.index.check_conflict(frame_id, doc, &text)?;

        // Resolve current positions
        let (start, end) = self
            .index
            .resolve_node_position(frame_id, doc, &text)
            .ok_or_else(|| MutationError::PositionResolutionFailed(frame_id.to_string()))?;

        // Generate new frame text
        let new_frame = format!(
            "@frame(x: {}, y: {}, width: {}, height: {})",
            x as i32, y as i32, width as i32, height as i32
        );

        // Apply the edit atomically
        crdt_doc.edit_range(start, end, &new_frame);

        // Rebuild index after mutation
        let current_text = crdt_doc.get_text();
        self.rebuild_index(crdt_doc.doc(), &current_text)?;

        Ok(MutationResult::Applied {
            mutation_id: mutation_id.to_string(),
            new_version: crdt_doc.version(),
        })
    }

    /// Apply SetTextContent mutation
    fn apply_set_text_content(
        &mut self,
        mutation_id: &str,
        node_id: &str,
        content: &str,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        // Find the text node
        let node = self
            .index
            .get_node(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        if node.node_type != NodeType::Text {
            return Err(MutationError::InvalidMutation(format!(
                "Node {} is not a text node",
                node_id
            )));
        }

        let doc = crdt_doc.doc();
        let text = doc.get_or_insert_text("content");

        // Check for conflicts
        self.index.check_conflict(node_id, doc, &text)?;

        // Resolve current positions
        let (start, end) = self
            .index
            .resolve_node_position(node_id, doc, &text)
            .ok_or_else(|| MutationError::PositionResolutionFailed(node_id.to_string()))?;

        // Parse the current text node to find the content part
        let current_source = {
            let txn = doc.transact();
            let full_text = text.get_string(&txn);
            full_text[start as usize..end as usize].to_string()
        };

        // Find the quoted content and replace it
        if let Some(quote_start) = current_source.find('"') {
            if let Some(quote_end) = current_source.rfind('"') {
                if quote_start < quote_end {
                    let abs_quote_start = start + quote_start as u32 + 1;
                    let abs_quote_end = start + quote_end as u32;

                    // Apply the edit
                    crdt_doc.edit_range(abs_quote_start, abs_quote_end, content);

                    // Rebuild index
                    let current_text = crdt_doc.get_text();
                    self.rebuild_index(crdt_doc.doc(), &current_text)?;

                    return Ok(MutationResult::Applied {
                        mutation_id: mutation_id.to_string(),
                        new_version: crdt_doc.version(),
                    });
                }
            }
        }

        Ok(MutationResult::Noop {
            mutation_id: mutation_id.to_string(),
            reason: "Could not find text content to replace".to_string(),
        })
    }

    /// Apply DeleteNode mutation
    fn apply_delete_node(
        &mut self,
        mutation_id: &str,
        node_id: &str,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        // Find the node
        let _node = self
            .index
            .get_node(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        let doc = crdt_doc.doc();
        let text = doc.get_or_insert_text("content");

        // Resolve current positions
        let (start, end) = self
            .index
            .resolve_node_position(node_id, doc, &text)
            .ok_or_else(|| MutationError::PositionResolutionFailed(node_id.to_string()))?;

        // Delete the node
        crdt_doc.delete(start, end - start);

        // Rebuild index
        let current_text = crdt_doc.get_text();
        self.rebuild_index(crdt_doc.doc(), &current_text)?;

        Ok(MutationResult::Applied {
            mutation_id: mutation_id.to_string(),
            new_version: crdt_doc.version(),
        })
    }

    /// Apply MoveNode mutation
    fn apply_move_node(
        &mut self,
        mutation_id: &str,
        node_id: &str,
        new_parent_id: &str,
        _index: u32,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        // Find both nodes
        let _node = self
            .index
            .get_node(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;
        let _parent = self
            .index
            .get_node(new_parent_id)
            .ok_or_else(|| MutationError::NodeNotFound(new_parent_id.to_string()))?;

        let doc = crdt_doc.doc();
        let text = doc.get_or_insert_text("content");

        // Resolve node position
        let (node_start, node_end) = self
            .index
            .resolve_node_position(node_id, doc, &text)
            .ok_or_else(|| MutationError::PositionResolutionFailed(node_id.to_string()))?;

        // Get the node's source text
        let node_source = {
            let txn = doc.transact();
            let full_text = text.get_string(&txn);
            full_text[node_start as usize..node_end as usize].to_string()
        };

        // Find insertion point in new parent
        let (parent_start, _parent_end) = self
            .index
            .resolve_node_position(new_parent_id, doc, &text)
            .ok_or_else(|| MutationError::PositionResolutionFailed(new_parent_id.to_string()))?;

        // Find the closing brace of the parent
        let parent_source = {
            let txn = doc.transact();
            let full_text = text.get_string(&txn);
            let (_, pe) = self
                .index
                .resolve_node_position(new_parent_id, doc, &text)
                .unwrap();
            full_text[parent_start as usize..pe as usize].to_string()
        };

        // Find last closing brace
        if let Some(close_brace) = parent_source.rfind('}') {
            let insert_pos = parent_start + close_brace as u32;

            // Delete first (so positions don't shift)
            crdt_doc.delete(node_start, node_end - node_start);

            // Recalculate insert position
            let adjusted_insert = if node_start < parent_start {
                insert_pos - (node_end - node_start)
            } else {
                insert_pos
            };

            // Insert at new location
            crdt_doc.insert(adjusted_insert, &format!("\n    {}", node_source));

            // Rebuild index
            let current_text = crdt_doc.get_text();
            self.rebuild_index(crdt_doc.doc(), &current_text)?;

            return Ok(MutationResult::Applied {
                mutation_id: mutation_id.to_string(),
                new_version: crdt_doc.version(),
            });
        }

        Ok(MutationResult::Noop {
            mutation_id: mutation_id.to_string(),
            reason: "Could not find insertion point in parent".to_string(),
        })
    }

    /// Apply InsertNode mutation
    fn apply_insert_node(
        &mut self,
        mutation_id: &str,
        parent_id: &str,
        _index: u32,
        source: &str,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        // Find the parent node
        let _parent = self
            .index
            .get_node(parent_id)
            .ok_or_else(|| MutationError::NodeNotFound(parent_id.to_string()))?;

        let doc = crdt_doc.doc();
        let text = doc.get_or_insert_text("content");

        // Resolve parent position
        let (parent_start, parent_end) = self
            .index
            .resolve_node_position(parent_id, doc, &text)
            .ok_or_else(|| MutationError::PositionResolutionFailed(parent_id.to_string()))?;

        // Find the closing brace of the parent
        let parent_source = {
            let txn = doc.transact();
            let full_text = text.get_string(&txn);
            full_text[parent_start as usize..parent_end as usize].to_string()
        };

        if let Some(close_brace) = parent_source.rfind('}') {
            let insert_pos = parent_start + close_brace as u32;

            // Insert the new node
            crdt_doc.insert(insert_pos, &format!("\n    {}", source));

            // Rebuild index
            let current_text = crdt_doc.get_text();
            self.rebuild_index(crdt_doc.doc(), &current_text)?;

            return Ok(MutationResult::Applied {
                mutation_id: mutation_id.to_string(),
                new_version: crdt_doc.version(),
            });
        }

        Ok(MutationResult::Noop {
            mutation_id: mutation_id.to_string(),
            reason: "Could not find insertion point in parent".to_string(),
        })
    }

    /// Apply SetComponentAnnotation mutation
    fn apply_set_component_annotation(
        &mut self,
        mutation_id: &str,
        component_name: &str,
        annotation_name: &str,
        params_str: &str,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        let current_source = crdt_doc.get_text();

        // Parse the document to find the component
        let ast = paperclip_parser::parse_with_path(&current_source, &self.file_path)
            .map_err(|e| MutationError::ParseError(e.to_string()))?;

        let component = ast
            .components
            .iter()
            .find(|c| c.name == component_name)
            .ok_or_else(|| {
                MutationError::InvalidMutation(format!("Component '{}' not found", component_name))
            })?;

        // Generate the new annotation text
        let new_annotation = if params_str.is_empty() {
            format!("@{}", annotation_name)
        } else {
            format!("@{}({})", annotation_name, params_str)
        };

        // Check if component has a doc comment
        if let Some(doc_comment) = &component.doc_comment {
            // Find existing annotation of same name
            if let Some(existing_ann) = doc_comment
                .annotations
                .iter()
                .find(|a| a.name == annotation_name)
            {
                // Replace existing annotation
                // Find the @annotation_name(...) pattern in source
                let doc_start = doc_comment.span.start;
                let doc_end = doc_comment.span.end;
                let doc_source = &current_source[doc_start..doc_end];

                // Find the annotation in the doc comment
                if let Some(ann_start) = doc_source.find(&format!("@{}", annotation_name)) {
                    let abs_ann_start = doc_start + ann_start;

                    // Find the end of the annotation (either closing paren or next whitespace/newline)
                    let after_name = abs_ann_start + annotation_name.len() + 1; // +1 for @
                    let rest = &current_source[after_name..doc_end];

                    let ann_end = if rest.starts_with('(') {
                        // Find matching paren
                        if let Some(close_pos) = Self::find_matching_paren(rest) {
                            after_name + close_pos + 1 // +1 to include the )
                        } else {
                            after_name
                        }
                    } else {
                        after_name
                    };

                    // Replace the annotation
                    crdt_doc.edit_range(abs_ann_start as u32, ann_end as u32, &new_annotation);

                    // Rebuild index
                    let current_text = crdt_doc.get_text();
                    self.rebuild_index(crdt_doc.doc(), &current_text)?;

                    return Ok(MutationResult::Applied {
                        mutation_id: mutation_id.to_string(),
                        new_version: crdt_doc.version(),
                    });
                }
            }

            // No existing annotation, add new one before the closing */
            let doc_end = doc_comment.span.end;
            let doc_source = &current_source[doc_comment.span.start..doc_end];

            if let Some(close_pos) = doc_source.rfind("*/") {
                let insert_pos = doc_comment.span.start + close_pos;
                let insert_text = format!(" * {}\n ", new_annotation);

                crdt_doc.insert(insert_pos as u32, &insert_text);

                // Rebuild index
                let current_text = crdt_doc.get_text();
                self.rebuild_index(crdt_doc.doc(), &current_text)?;

                return Ok(MutationResult::Applied {
                    mutation_id: mutation_id.to_string(),
                    new_version: crdt_doc.version(),
                });
            }
        } else {
            // No doc comment exists, create one before the component
            // Note: component.span.start may incorrectly include preceding content,
            // so we need to find the actual "component" or "public" keyword
            let span_start = component.span.start;
            let span_slice = &current_source[span_start..];

            // Find where the actual component keyword starts
            // Try "public component" first, then just "component"
            let keyword_offset = span_slice.find("public component")
                .or_else(|| span_slice.find("public\ncomponent"))
                .or_else(|| span_slice.find("component"))
                .unwrap_or(0);

            let actual_comp_start = span_start + keyword_offset;

            // Find the start of the line containing the component keyword
            let before_comp = &current_source[..actual_comp_start];
            let line_start = before_comp.rfind('\n').map(|p| p + 1).unwrap_or(0);

            let doc_comment = format!("/**\n * {}\n */\n", new_annotation);

            crdt_doc.insert(line_start as u32, &doc_comment);

            // Rebuild index
            let current_text = crdt_doc.get_text();
            self.rebuild_index(crdt_doc.doc(), &current_text)?;

            return Ok(MutationResult::Applied {
                mutation_id: mutation_id.to_string(),
                new_version: crdt_doc.version(),
            });
        }

        Ok(MutationResult::Noop {
            mutation_id: mutation_id.to_string(),
            reason: "Could not find position to insert annotation".to_string(),
        })
    }

    /// Apply RemoveComponentAnnotation mutation
    fn apply_remove_component_annotation(
        &mut self,
        mutation_id: &str,
        component_name: &str,
        annotation_name: &str,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        let current_source = crdt_doc.get_text();

        // Parse the document to find the component
        let ast = paperclip_parser::parse_with_path(&current_source, &self.file_path)
            .map_err(|e| MutationError::ParseError(e.to_string()))?;

        let component = ast
            .components
            .iter()
            .find(|c| c.name == component_name)
            .ok_or_else(|| {
                MutationError::InvalidMutation(format!("Component '{}' not found", component_name))
            })?;

        // Must have doc comment with the annotation
        let doc_comment = component.doc_comment.as_ref().ok_or_else(|| {
            MutationError::InvalidMutation(format!(
                "Component '{}' has no doc comment",
                component_name
            ))
        })?;

        let _ann = doc_comment
            .annotations
            .iter()
            .find(|a| a.name == annotation_name)
            .ok_or_else(|| {
                MutationError::InvalidMutation(format!(
                    "Annotation '@{}' not found on component '{}'",
                    annotation_name, component_name
                ))
            })?;

        // Find and remove the annotation from source
        let doc_start = doc_comment.span.start;
        let doc_end = doc_comment.span.end;
        let doc_source = &current_source[doc_start..doc_end];

        // Find the annotation pattern (including the * prefix if on its own line)
        let search_patterns = [
            format!(" * @{}", annotation_name),
            format!("@{}", annotation_name),
        ];

        for pattern in &search_patterns {
            if let Some(ann_start_in_doc) = doc_source.find(pattern.as_str()) {
                let abs_ann_start = doc_start + ann_start_in_doc;

                // Find the end of the annotation
                let after_pattern = abs_ann_start + pattern.len();
                let rest = &current_source[after_pattern..doc_end];

                let mut ann_end = if rest.starts_with('(') {
                    if let Some(close_pos) = Self::find_matching_paren(rest) {
                        after_pattern + close_pos + 1
                    } else {
                        after_pattern
                    }
                } else {
                    after_pattern
                };

                // Also remove trailing newline if present
                if ann_end < current_source.len() && current_source.as_bytes()[ann_end] == b'\n' {
                    ann_end += 1;
                }

                // Delete the annotation
                crdt_doc.delete(abs_ann_start as u32, (ann_end - abs_ann_start) as u32);

                // Rebuild index
                let current_text = crdt_doc.get_text();
                self.rebuild_index(crdt_doc.doc(), &current_text)?;

                return Ok(MutationResult::Applied {
                    mutation_id: mutation_id.to_string(),
                    new_version: crdt_doc.version(),
                });
            }
        }

        Ok(MutationResult::Noop {
            mutation_id: mutation_id.to_string(),
            reason: format!("Could not locate annotation '@{}' in source", annotation_name),
        })
    }

    /// Apply SetStyleProperty mutation
    fn apply_set_style_property(
        &mut self,
        mutation_id: &str,
        node_id: &str,
        property: &str,
        value: &str,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        tracing::info!("[SetStyleProperty] node_id={}, property={}, value={}", node_id, property, value);

        // Validate CSS property name (alphanumeric and hyphens only)
        if !property.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(MutationError::InvalidMutation(format!(
                "Invalid CSS property name: {}",
                property
            )));
        }

        // Validate CSS value (no injection attacks)
        if value.contains('{') || value.contains('}') || value.contains(';') {
            return Err(MutationError::InvalidMutation(format!(
                "Invalid CSS value (contains forbidden characters): {}",
                value
            )));
        }

        // Find the node - for frames, we need to look up the element variant
        let all_ids: Vec<_> = self.index.all_node_ids().collect();
        tracing::info!("[SetStyleProperty] Available nodes: {:?}", all_ids);

        // First try direct lookup
        let node = self.index.get_node(node_id);

        // If node is a Frame, look for the -element variant which has the actual element positions
        let (actual_node_id, node) = match node {
            Some(n) if n.node_type == NodeType::Frame => {
                let element_id = format!("{}-element", node_id);
                tracing::info!("[SetStyleProperty] Node is Frame, looking for element variant: {}", element_id);
                match self.index.get_node(&element_id) {
                    Some(elem_node) => (element_id, elem_node),
                    None => {
                        // Frame without separate element entry - this shouldn't happen
                        // but fall back to using the frame (will likely fail)
                        tracing::warn!("[SetStyleProperty] No element variant found, using frame");
                        (node_id.to_string(), n)
                    }
                }
            }
            Some(n) => (node_id.to_string(), n),
            None => {
                // Try the -element variant directly (in case node_id is the frame ID)
                let element_id = format!("{}-element", node_id);
                match self.index.get_node(&element_id) {
                    Some(elem_node) => {
                        tracing::info!("[SetStyleProperty] Found via element variant: {}", element_id);
                        (element_id, elem_node)
                    }
                    None => {
                        tracing::error!("[SetStyleProperty] Node {} not found!", node_id);
                        return Err(MutationError::NodeNotFound(node_id.to_string()));
                    }
                }
            }
        };

        tracing::info!("[SetStyleProperty] Using node: {} ({:?})", actual_node_id, node.node_type);

        // Allow both Element and Frame types (though Frame should have been redirected above)
        if node.node_type != NodeType::Element && node.node_type != NodeType::Frame {
            return Err(MutationError::InvalidMutation(format!(
                "Node {} is not an element (got {:?})",
                node_id, node.node_type
            )));
        }

        let doc = crdt_doc.doc();
        let text = doc.get_or_insert_text("content");

        // Check for conflicts using the actual node ID (element variant if applicable)
        self.index.check_conflict(&actual_node_id, doc, &text)?;

        // Resolve current positions using the actual node ID
        let (start, end) = self
            .index
            .resolve_node_position(&actual_node_id, doc, &text)
            .ok_or_else(|| MutationError::PositionResolutionFailed(actual_node_id.to_string()))?;

        // Get the element source
        let element_source = {
            let txn = doc.transact();
            let full_text = text.get_string(&txn);
            full_text[start as usize..end as usize].to_string()
        };

        tracing::info!("[SetStyleProperty] Element source ({} bytes): {:?}", element_source.len(), &element_source[..element_source.len().min(200)]);

        // Find the style block within the element
        // Look for "style {" pattern (without variants for Phase 1)
        if let Some(style_info) = Self::find_style_block(&element_source) {
            tracing::info!("[SetStyleProperty] Found style block at {:?}", style_info);
            // Check if property already exists
            if let Some((prop_start, prop_end)) =
                Self::find_property_in_style(&element_source, &style_info, property)
            {
                // Replace existing property value
                let abs_prop_start = start + prop_start as u32;
                let abs_prop_end = start + prop_end as u32;

                let new_prop_line = format!("{}: {}", property, value);
                crdt_doc.edit_range(abs_prop_start, abs_prop_end, &new_prop_line);
            } else {
                // Add new property at the end of style block (before closing brace)
                let insert_pos = start + style_info.content_end as u32;

                // Determine indentation (use same as existing properties or 12 spaces)
                let indent = Self::detect_style_indent(&element_source, &style_info);
                let new_prop = format!("\n{}{}: {}", indent, property, value);
                crdt_doc.insert(insert_pos, &new_prop);
            }

            // Rebuild index
            let current_text = crdt_doc.get_text();
            self.rebuild_index(crdt_doc.doc(), &current_text)?;

            return Ok(MutationResult::Applied {
                mutation_id: mutation_id.to_string(),
                new_version: crdt_doc.version(),
            });
        }

        // No style block found - need to create one
        tracing::info!("[SetStyleProperty] No style block found, creating new one");

        // Find insertion point (after tag name and attributes, before children or closing)
        if let Some(insert_pos) = Self::find_style_insertion_point(&element_source) {
            tracing::info!("[SetStyleProperty] Found insertion point at offset {}", insert_pos);
            let abs_insert_pos = start + insert_pos as u32;

            // Create new style block
            let new_style = format!(
                "\n        style {{\n            {}: {}\n        }}",
                property, value
            );
            tracing::info!("[SetStyleProperty] Inserting new style block: {:?}", new_style);
            crdt_doc.insert(abs_insert_pos, &new_style);

            // Rebuild index
            let current_text = crdt_doc.get_text();
            self.rebuild_index(crdt_doc.doc(), &current_text)?;

            return Ok(MutationResult::Applied {
                mutation_id: mutation_id.to_string(),
                new_version: crdt_doc.version(),
            });
        }

        tracing::error!("[SetStyleProperty] Could not find insertion point! Element source: {:?}", &element_source[..element_source.len().min(500)]);
        Ok(MutationResult::Noop {
            mutation_id: mutation_id.to_string(),
            reason: "Could not find position to insert style".to_string(),
        })
    }

    /// Apply DeleteStyleProperty mutation
    fn apply_delete_style_property(
        &mut self,
        mutation_id: &str,
        node_id: &str,
        property: &str,
        crdt_doc: &mut CrdtDocument,
    ) -> Result<MutationResult, MutationError> {
        // Find the node
        let node = self
            .index
            .get_node(node_id)
            .ok_or_else(|| MutationError::NodeNotFound(node_id.to_string()))?;

        // Allow both Element and Frame types (frames are elements with @frame metadata)
        if node.node_type != NodeType::Element && node.node_type != NodeType::Frame {
            return Err(MutationError::InvalidMutation(format!(
                "Node {} is not an element (got {:?})",
                node_id, node.node_type
            )));
        }

        let doc = crdt_doc.doc();
        let text = doc.get_or_insert_text("content");

        // Resolve current positions
        let (start, end) = self
            .index
            .resolve_node_position(node_id, doc, &text)
            .ok_or_else(|| MutationError::PositionResolutionFailed(node_id.to_string()))?;

        // Get the element source
        let element_source = {
            let txn = doc.transact();
            let full_text = text.get_string(&txn);
            full_text[start as usize..end as usize].to_string()
        };

        // Find the style block
        if let Some(style_info) = Self::find_style_block(&element_source) {
            if let Some((prop_start, prop_end)) =
                Self::find_property_in_style(&element_source, &style_info, property)
            {
                // Delete the property line (including newline if present)
                let abs_prop_start = start + prop_start as u32;
                let mut abs_prop_end = start + prop_end as u32;

                // Also delete the preceding newline if there is one
                let source_before = &element_source[..prop_start];
                if source_before.ends_with('\n') {
                    // Adjust to delete from newline
                    let delete_start = start + (prop_start - 1) as u32;
                    crdt_doc.delete(delete_start, abs_prop_end - delete_start);
                } else {
                    crdt_doc.delete(abs_prop_start, abs_prop_end - abs_prop_start);
                }

                // Rebuild index
                let current_text = crdt_doc.get_text();
                self.rebuild_index(crdt_doc.doc(), &current_text)?;

                return Ok(MutationResult::Applied {
                    mutation_id: mutation_id.to_string(),
                    new_version: crdt_doc.version(),
                });
            }
        }

        Ok(MutationResult::Noop {
            mutation_id: mutation_id.to_string(),
            reason: format!("Property '{}' not found in element", property),
        })
    }

    /// Find a style block within element source
    fn find_style_block(source: &str) -> Option<StyleBlockInfo> {
        // Look for "style {" pattern (simplest case for Phase 1)
        // This doesn't handle "style variant x {" or "style extends y {" - Phase 2
        let style_keyword = "style {";
        let style_start = source.find(style_keyword)?;

        // Find the matching closing brace
        let after_open = style_start + style_keyword.len();
        let mut depth = 1;
        let mut style_end = after_open;

        for (i, c) in source[after_open..].chars().enumerate() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        style_end = after_open + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth != 0 {
            return None; // Unbalanced braces
        }

        Some(StyleBlockInfo {
            start: style_start,
            end: style_end,
            content_start: after_open,
            content_end: style_end - 1, // Before the closing }
        })
    }

    /// Find a property within a style block, returns (start, end) of the property line
    fn find_property_in_style(
        source: &str,
        style_info: &StyleBlockInfo,
        property: &str,
    ) -> Option<(usize, usize)> {
        let content = &source[style_info.content_start..style_info.content_end];

        // Look for "property:" pattern
        for (line_start, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with(property) {
                // Check that it's followed by ":"
                let after_prop = &trimmed[property.len()..].trim_start();
                if after_prop.starts_with(':') {
                    // Found it! Calculate absolute positions
                    let line_offset = content
                        .lines()
                        .take(line_start)
                        .map(|l| l.len() + 1) // +1 for newline
                        .sum::<usize>();

                    // Find the actual start (skip leading whitespace)
                    let actual_line_start = style_info.content_start + line_offset;
                    let ws_len = line.len() - line.trim_start().len();
                    let prop_start = actual_line_start + ws_len;
                    let prop_end = actual_line_start + line.len();

                    return Some((prop_start, prop_end));
                }
            }
        }

        None
    }

    /// Detect indentation used in style block
    fn detect_style_indent(source: &str, style_info: &StyleBlockInfo) -> String {
        let content = &source[style_info.content_start..style_info.content_end];

        // Find first non-empty line and get its indentation
        for line in content.lines() {
            if !line.trim().is_empty() {
                let ws_len = line.len() - line.trim_start().len();
                return line[..ws_len].to_string();
            }
        }

        // Default indentation
        "            ".to_string()
    }

    /// Find insertion point for a new style block in an element
    fn find_style_insertion_point(source: &str) -> Option<usize> {
        // Find first '{' (opening brace of element)
        let open_brace = source.find('{')?;
        Some(open_brace + 1)
    }

    /// Find matching closing paren, returns position of closing paren
    fn find_matching_paren(s: &str) -> Option<usize> {
        if !s.starts_with('(') {
            return None;
        }

        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = '"';

        for (i, c) in s.chars().enumerate() {
            if !in_string && (c == '"' || c == '\'') {
                in_string = true;
                string_char = c;
            } else if in_string && c == string_char {
                in_string = false;
            } else if !in_string {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(i);
                        }
                    }
                    _ => {}
                }
            }
        }

        None
    }

    /// Get the current index for inspection
    pub fn index(&self) -> &AstIndex {
        &self.index
    }
}

impl Default for MutationHandler {
    fn default() -> Self {
        Self::new()
    }
}
