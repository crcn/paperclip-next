//! # Undo/Redo Stack
//!
//! Tracks mutation history and enables undo/redo operations.
//!
//! ## Design
//!
//! - Each mutation records its inverse before being applied
//! - Undo applies the inverse and moves mutation to redo stack
//! - Redo reapplies the original mutation
//! - New mutations clear the redo stack
//! - Supports batched operations (group multiple mutations as one undo step)
//!
//! ## Example
//!
//! ```rust,ignore
//! let mut stack = UndoStack::new();
//! let mut doc = Document::from_source(...)?;
//!
//! // Apply mutation with undo support
//! let mutation = Mutation::UpdateText { ... };
//! stack.apply(&mutation, &mut doc)?;
//!
//! // Undo
//! stack.undo(&mut doc)?;
//!
//! // Redo
//! stack.redo(&mut doc)?;
//! ```

use crate::{Mutation, MutationError};
use paperclip_parser::ast::Document;

/// A group of mutations that should be undone/redone together
#[derive(Debug, Clone)]
pub struct MutationBatch {
    /// The mutations in this batch (in application order)
    pub mutations: Vec<Mutation>,

    /// The inverse mutations (in reverse order for undo)
    pub inverses: Vec<Mutation>,

    /// Optional description of this batch
    pub description: Option<String>,
}

impl MutationBatch {
    /// Create a single-mutation batch
    pub fn single(mutation: Mutation, inverse: Mutation) -> Self {
        Self {
            mutations: vec![mutation],
            inverses: vec![inverse],
            description: None,
        }
    }

    /// Create a batch from multiple mutations
    pub fn from_mutations(mutations: Vec<Mutation>, inverses: Vec<Mutation>) -> Self {
        Self {
            mutations,
            inverses,
            description: None,
        }
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Undo/redo stack for document editing
#[derive(Debug)]
pub struct UndoStack {
    /// Stack of applied mutations (most recent last)
    undo_stack: Vec<MutationBatch>,

    /// Stack of undone mutations (most recent last)
    redo_stack: Vec<MutationBatch>,

    /// Maximum number of undo levels (0 = unlimited)
    max_levels: usize,

    /// Currently building a batch
    current_batch: Option<MutationBatch>,
}

impl UndoStack {
    /// Create a new undo stack with default max levels (100)
    pub fn new() -> Self {
        Self::with_max_levels(100)
    }

    /// Create an undo stack with custom max levels
    pub fn with_max_levels(max_levels: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_levels,
            current_batch: None,
        }
    }

    /// Apply a mutation and record it for undo
    pub fn apply(&mut self, mutation: &Mutation, doc: &mut Document) -> Result<(), MutationError> {
        // Generate inverse before applying
        let inverse = mutation.to_inverse(doc)?;

        // Apply mutation
        mutation.apply(doc)?;

        // Record for undo
        if let Some(batch) = &mut self.current_batch {
            // Add to current batch
            batch.mutations.push(mutation.clone());
            batch.inverses.insert(0, inverse); // Inverses go in reverse order
        } else {
            // Push as single mutation
            let batch = MutationBatch::single(mutation.clone(), inverse);
            self.push_batch(batch);
        }

        Ok(())
    }

    /// Start a batch of mutations (will be undone/redone together)
    pub fn begin_batch(&mut self) {
        self.current_batch = Some(MutationBatch {
            mutations: Vec::new(),
            inverses: Vec::new(),
            description: None,
        });
    }

    /// End the current batch and push to undo stack
    pub fn end_batch(&mut self) {
        if let Some(batch) = self.current_batch.take() {
            if !batch.mutations.is_empty() {
                self.push_batch(batch);
            }
        }
    }

    /// Set description for current batch (if batching)
    pub fn set_batch_description(&mut self, description: impl Into<String>) {
        if let Some(batch) = &mut self.current_batch {
            batch.description = Some(description.into());
        }
    }

    /// Push a batch to the undo stack
    fn push_batch(&mut self, batch: MutationBatch) {
        self.undo_stack.push(batch);

        // Trim if exceeded max levels
        if self.max_levels > 0 && self.undo_stack.len() > self.max_levels {
            self.undo_stack.remove(0);
        }

        // Clear redo stack (new action invalidates future)
        self.redo_stack.clear();
    }

    /// Undo the most recent mutation/batch
    pub fn undo(&mut self, doc: &mut Document) -> Result<bool, MutationError> {
        if let Some(batch) = self.undo_stack.pop() {
            // Apply inverses in order
            for inverse in &batch.inverses {
                inverse.apply(doc)?;
            }

            // Move to redo stack
            self.redo_stack.push(batch);

            Ok(true)
        } else {
            Ok(false) // Nothing to undo
        }
    }

    /// Redo the most recently undone mutation/batch
    pub fn redo(&mut self, doc: &mut Document) -> Result<bool, MutationError> {
        if let Some(batch) = self.redo_stack.pop() {
            // Reapply original mutations
            for mutation in &batch.mutations {
                mutation.apply(doc)?;
            }

            // Move back to undo stack
            self.undo_stack.push(batch);

            Ok(true)
        } else {
            Ok(false) // Nothing to redo
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of undo levels available
    pub fn undo_levels(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo levels available
    pub fn redo_levels(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all undo/redo history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_batch = None;
    }

    /// Get description of the next undo operation
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack
            .last()
            .and_then(|batch| batch.description.as_deref())
    }

    /// Get description of the next redo operation
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack
            .last()
            .and_then(|batch| batch.description.as_deref())
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperclip_parser::parse;

    #[test]
    fn test_undo_stack_creation() {
        let stack = UndoStack::new();
        assert_eq!(stack.undo_levels(), 0);
        assert_eq!(stack.redo_levels(), 0);
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_apply_and_undo_text_mutation() {
        let source = r#"
            component Test {
                render div {
                    text "Hello"
                }
            }
        "#;
        let mut doc = parse(source).unwrap();
        let mut stack = UndoStack::new();

        // Get text node ID
        let text_id = doc.components[0].body.as_ref().unwrap().children().unwrap()[0]
            .span()
            .id
            .clone();

        // Apply mutation
        let mutation = Mutation::UpdateText {
            node_id: text_id.clone(),
            content: "World".to_string(),
        };

        stack.apply(&mutation, &mut doc).unwrap();

        assert_eq!(stack.undo_levels(), 1);
        assert!(stack.can_undo());

        // Undo
        let undone = stack.undo(&mut doc).unwrap();
        assert!(undone);
        assert_eq!(stack.undo_levels(), 0);
        assert_eq!(stack.redo_levels(), 1);
        assert!(stack.can_redo());

        // Redo
        let redone = stack.redo(&mut doc).unwrap();
        assert!(redone);
        assert_eq!(stack.undo_levels(), 1);
        assert_eq!(stack.redo_levels(), 0);
    }

    #[test]
    fn test_batched_mutations() {
        let source = r#"
            component Test {
                render div {
                    text "Hello"
                }
            }
        "#;
        let mut doc = parse(source).unwrap();
        let mut stack = UndoStack::new();

        let text_id = doc.components[0].body.as_ref().unwrap().children().unwrap()[0]
            .span()
            .id
            .clone();

        // Start batch
        stack.begin_batch();
        stack.set_batch_description("Update greeting");

        // Apply multiple mutations
        let mut1 = Mutation::UpdateText {
            node_id: text_id.clone(),
            content: "World".to_string(),
        };
        stack.apply(&mut1, &mut doc).unwrap();

        let mut2 = Mutation::UpdateText {
            node_id: text_id.clone(),
            content: "Everyone!".to_string(),
        };
        stack.apply(&mut2, &mut doc).unwrap();

        // End batch
        stack.end_batch();

        // Should be one batch with 2 mutations
        assert_eq!(stack.undo_levels(), 1);
        assert_eq!(stack.undo_description(), Some("Update greeting"));

        // Undo should revert both
        stack.undo(&mut doc).unwrap();
        assert_eq!(stack.undo_levels(), 0);
    }

    #[test]
    fn test_new_mutation_clears_redo() {
        let source = "component Test { render div { text \"Hello\" } }";
        let mut doc = parse(source).unwrap();
        let mut stack = UndoStack::new();

        let text_id = doc.components[0].body.as_ref().unwrap().children().unwrap()[0]
            .span()
            .id
            .clone();

        // Apply and undo
        let mut1 = Mutation::UpdateText {
            node_id: text_id.clone(),
            content: "World".to_string(),
        };
        stack.apply(&mut1, &mut doc).unwrap();
        stack.undo(&mut doc).unwrap();

        assert_eq!(stack.redo_levels(), 1);

        // New mutation clears redo
        let mut2 = Mutation::UpdateText {
            node_id: text_id.clone(),
            content: "Everyone".to_string(),
        };
        stack.apply(&mut2, &mut doc).unwrap();

        assert_eq!(stack.redo_levels(), 0);
    }

    #[test]
    fn test_max_levels_enforced() {
        let source = "component Test { render div { text \"Hello\" } }";
        let mut doc = parse(source).unwrap();
        let mut stack = UndoStack::with_max_levels(2);

        let text_id = doc.components[0].body.as_ref().unwrap().children().unwrap()[0]
            .span()
            .id
            .clone();

        // Apply 3 mutations
        for i in 0..3 {
            let mutation = Mutation::UpdateText {
                node_id: text_id.clone(),
                content: format!("Text {}", i),
            };
            stack.apply(&mutation, &mut doc).unwrap();
        }

        // Should only keep 2 (max levels)
        assert_eq!(stack.undo_levels(), 2);
    }
}
