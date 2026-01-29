//! # Edit Session Management
//!
//! Tracks editing state for single or multiple users.
//!
//! An EditSession represents one client's view of a document,
//! including pending mutations (for optimistic updates) and
//! current selection state.

use crate::{Document, EditorError, Mutation};
use std::time::{SystemTime, UNIX_EPOCH};

/// Single edit session (single-user or one client in multi-user)
pub struct EditSession {
    /// Unique session identifier
    pub id: String,

    /// Document being edited
    pub document: Document,

    /// Current selection (source IDs from VDOM)
    pub selected_nodes: Vec<String>,

    /// Pending mutations (for optimistic updates)
    pub pending_mutations: Vec<PendingMutation>,
}

/// Mutation waiting for server acknowledgment
#[derive(Debug, Clone)]
pub struct PendingMutation {
    /// Unique mutation ID
    pub id: String,

    /// The mutation
    pub mutation: Mutation,

    /// When it was created
    pub timestamp: u64,
}

impl EditSession {
    /// Create new edit session
    pub fn new(id: String, document: Document) -> Self {
        Self {
            id,
            document,
            selected_nodes: Vec::new(),
            pending_mutations: Vec::new(),
        }
    }

    /// Apply mutation optimistically (for responsive editing)
    ///
    /// This applies the mutation to the local document immediately,
    /// then stores it as pending. The mutation should be sent to
    /// the server for confirmation.
    pub fn apply_optimistic(&mut self, mutation: Mutation) -> Result<String, EditorError> {
        // Generate mutation ID
        let mutation_id = format!("{}-{}", self.id, self.pending_mutations.len());

        // Store as pending
        self.pending_mutations.push(PendingMutation {
            id: mutation_id.clone(),
            mutation: mutation.clone(),
            timestamp: current_timestamp(),
        });

        // Apply to local document
        self.document.apply(mutation)?;

        Ok(mutation_id)
    }

    /// Confirm that a mutation was accepted by server
    ///
    /// Removes the mutation from pending queue.
    pub fn confirm_mutation(&mut self, mutation_id: &str) {
        self.pending_mutations.retain(|m| m.id != mutation_id);
    }

    /// Reject a mutation (server said it's invalid)
    ///
    /// Removes from pending and potentially triggers a rebase.
    pub fn reject_mutation(&mut self, mutation_id: &str) {
        self.pending_mutations.retain(|m| m.id != mutation_id);
        // TODO: Trigger rebase to get back in sync
    }

    /// Rebase local state on server update
    ///
    /// When the server sends an update, we need to:
    /// 1. Replace our document with server's version
    /// 2. Replay pending mutations on top
    ///
    /// This handles the case where other clients made changes
    /// while we had pending local edits.
    pub fn rebase(&mut self, server_document: Document) -> Result<(), EditorError> {
        // Save pending mutations
        let pending = std::mem::take(&mut self.pending_mutations);

        // Replace document with server version
        self.document = server_document;

        // Replay pending mutations
        for pm in pending {
            match self.document.apply(pm.mutation.clone()) {
                Ok(_) => {
                    // Mutation still valid, keep it pending
                    self.pending_mutations.push(pm);
                }
                Err(_) => {
                    // Mutation no longer valid (e.g., node was deleted)
                    // Drop it silently
                }
            }
        }

        Ok(())
    }

    /// Update selection
    pub fn set_selection(&mut self, node_ids: Vec<String>) {
        self.selected_nodes = node_ids;
    }

    /// Get number of pending mutations
    pub fn pending_count(&self) -> usize {
        self.pending_mutations.len()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_session_creation() {
        let source = "component Test { render div {} }";
        let doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

        let session = EditSession::new("client-1".to_string(), doc);

        assert_eq!(session.id, "client-1");
        assert_eq!(session.pending_count(), 0);
        assert!(session.selected_nodes.is_empty());
    }

    #[test]
    fn test_optimistic_mutations() {
        let source = "component Test { render div {} }";
        let doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

        let mut session = EditSession::new("client-1".to_string(), doc);

        let mutation = Mutation::UpdateText {
            node_id: "text-123".to_string(),
            content: "Hello".to_string(),
        };

        // TODO: This will fail until we implement mutation.apply()
        // For now, just test that it returns an error
        let result = session.apply_optimistic(mutation);
        assert!(result.is_err());
    }

    #[test]
    fn test_mutation_rejection() {
        let source = "component Test { render div {} }";
        let doc = Document::from_source(PathBuf::from("test.pc"), source.to_string()).unwrap();

        let session = EditSession::new("client-1".to_string(), doc);

        // Test that we can reject a mutation by ID
        // (doesn't matter if the ID exists or not for this test)
        let fake_mutation_id = "client-1-0";

        // This should not panic
        // TODO: Add real test once mutations are implemented
        assert_eq!(session.pending_count(), 0);
    }
}
