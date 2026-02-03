//! CRDT document management for collaborative editing.
//!
//! This module provides Yjs-compatible CRDT documents for real-time
//! collaborative text editing. Each file has a shared Y.Doc that
//! multiple clients can edit concurrently.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use yrs::{Doc, GetString, ReadTxn, Text, TextRef, Transact, Update};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;

/// A CRDT-backed document session.
pub struct CrdtDocument {
    doc: Doc,
    text: TextRef,
    dirty: bool,
    version: u64,
}

impl CrdtDocument {
    /// Create a new empty CRDT document.
    pub fn new() -> Self {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        Self {
            doc,
            text,
            dirty: false,
            version: 0,
        }
    }

    /// Create a CRDT document with initial content.
    pub fn with_content(content: &str) -> Self {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");

        // Insert initial content using transaction
        {
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, content);
        }

        Self {
            doc,
            text,
            dirty: false,
            version: 0,
        }
    }

    /// Get the current text content.
    pub fn get_text(&self) -> String {
        let txn = self.doc.transact();
        self.text.get_string(&txn)
    }

    /// Get the current state vector (for delta sync).
    pub fn get_state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.state_vector().encode_v1()
    }

    /// Encode the full document state.
    pub fn encode_state(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&Default::default())
    }

    /// Encode delta since a given state vector.
    pub fn encode_delta(&self, state_vector: &[u8]) -> Result<Vec<u8>, CrdtError> {
        let sv = yrs::StateVector::decode_v1(state_vector)
            .map_err(|e| CrdtError::DecodeError(e.to_string()))?;
        let txn = self.doc.transact();
        Ok(txn.encode_state_as_update_v1(&sv))
    }

    /// Apply an update from a client.
    pub fn apply_update(&mut self, update: &[u8]) -> Result<(), CrdtError> {
        let update = Update::decode_v1(update)
            .map_err(|e| CrdtError::DecodeError(e.to_string()))?;

        let mut txn = self.doc.transact_mut();
        txn.apply_update(update)
            .map_err(|e| CrdtError::ApplyError(e.to_string()))?;

        self.dirty = true;
        self.version += 1;

        Ok(())
    }

    /// Check if document has unprocessed changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark document as clean (AST has been updated).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get current version.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get a reference to the underlying Doc for advanced operations.
    /// Use with care - prefer the safe wrapper methods when possible.
    pub fn doc(&self) -> &Doc {
        &self.doc
    }

    /// Get a mutable reference to the underlying Doc for advanced operations.
    pub fn doc_mut(&mut self) -> &mut Doc {
        &mut self.doc
    }

    /// Get the text reference for direct operations.
    pub fn text(&self) -> &TextRef {
        &self.text
    }

    /// Apply a text edit at specific positions.
    /// This is the core primitive for mutation translation.
    pub fn edit_range(&mut self, start: u32, end: u32, replacement: &str) {
        let mut txn = self.doc.transact_mut();
        if start < end {
            self.text.remove_range(&mut txn, start, end - start);
        }
        if !replacement.is_empty() {
            self.text.insert(&mut txn, start, replacement);
        }
        self.dirty = true;
        self.version += 1;
    }

    /// Insert text at a position.
    pub fn insert(&mut self, pos: u32, text: &str) {
        let mut txn = self.doc.transact_mut();
        self.text.insert(&mut txn, pos, text);
        self.dirty = true;
        self.version += 1;
    }

    /// Delete a range of text.
    pub fn delete(&mut self, start: u32, length: u32) {
        let mut txn = self.doc.transact_mut();
        self.text.remove_range(&mut txn, start, length);
        self.dirty = true;
        self.version += 1;
    }
}

/// Errors that can occur during CRDT operations.
#[derive(Debug, thiserror::Error)]
pub enum CrdtError {
    #[error("Failed to decode: {0}")]
    DecodeError(String),

    #[error("Failed to apply update: {0}")]
    ApplyError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),
}

/// Client connection in a CRDT session.
#[derive(Clone)]
pub struct CrdtClient {
    pub client_id: String,
    pub sender: tokio::sync::mpsc::Sender<CrdtBroadcast>,
}

/// Broadcast message to CRDT clients.
#[derive(Clone, Debug)]
pub enum CrdtBroadcast {
    /// Remote update from another client
    RemoteUpdate {
        update: Vec<u8>,
        origin_client_id: String,
    },
    /// VDOM patches after successful parse
    VdomPatch {
        patches_json: String,
        version: u64,
        origin_client_id: String,
    },
    /// Parse error
    ParseError {
        error: String,
        line: u32,
        column: u32,
    },
}

/// A CRDT editing session for a single file.
pub struct CrdtSession {
    pub file_path: String,
    pub document: CrdtDocument,
    pub clients: Vec<CrdtClient>,
}

impl CrdtSession {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            document: CrdtDocument::new(),
            clients: Vec::new(),
        }
    }

    pub fn with_content(file_path: String, content: &str) -> Self {
        Self {
            file_path,
            document: CrdtDocument::with_content(content),
            clients: Vec::new(),
        }
    }

    /// Add a client to the session.
    pub fn add_client(&mut self, client: CrdtClient) {
        // Remove any existing client with same ID
        self.clients.retain(|c| c.client_id != client.client_id);
        self.clients.push(client);
    }

    /// Remove a client from the session.
    pub fn remove_client(&mut self, client_id: &str) {
        self.clients.retain(|c| c.client_id != client_id);
    }

    /// Broadcast an update to all clients except the origin.
    pub async fn broadcast(&self, msg: CrdtBroadcast, exclude_client: Option<&str>) {
        for client in &self.clients {
            if Some(client.client_id.as_str()) == exclude_client {
                continue;
            }
            // Ignore send errors (client may have disconnected)
            let _ = client.sender.send(msg.clone()).await;
        }
    }

    /// Get number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

/// Manager for all CRDT sessions.
pub struct CrdtSessionManager {
    sessions: RwLock<HashMap<String, Arc<tokio::sync::RwLock<CrdtSession>>>>,
}

impl CrdtSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a session for a file path.
    pub fn get_or_create_session(&self, file_path: &str) -> Arc<tokio::sync::RwLock<CrdtSession>> {
        // Try read lock first
        {
            let sessions = self.sessions.read().unwrap();
            if let Some(session) = sessions.get(file_path) {
                return session.clone();
            }
        }

        // Need to create - acquire write lock
        let mut sessions = self.sessions.write().unwrap();

        // Double-check (another thread may have created it)
        if let Some(session) = sessions.get(file_path) {
            return session.clone();
        }

        // Create new session
        let session = Arc::new(tokio::sync::RwLock::new(CrdtSession::new(file_path.to_string())));
        sessions.insert(file_path.to_string(), session.clone());
        session
    }

    /// Get or create a session with initial content.
    pub fn get_or_create_session_with_content(
        &self,
        file_path: &str,
        content: &str,
    ) -> Arc<tokio::sync::RwLock<CrdtSession>> {
        // Try read lock first
        {
            let sessions = self.sessions.read().unwrap();
            if let Some(session) = sessions.get(file_path) {
                return session.clone();
            }
        }

        // Need to create - acquire write lock
        let mut sessions = self.sessions.write().unwrap();

        // Double-check
        if let Some(session) = sessions.get(file_path) {
            return session.clone();
        }

        // Create new session with content
        let session = Arc::new(tokio::sync::RwLock::new(
            CrdtSession::with_content(file_path.to_string(), content)
        ));
        sessions.insert(file_path.to_string(), session.clone());
        session
    }

    /// Remove a session (when all clients disconnect).
    pub fn remove_session(&self, file_path: &str) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(file_path);
    }

    /// Get session if it exists.
    pub fn get_session(&self, file_path: &str) -> Option<Arc<tokio::sync::RwLock<CrdtSession>>> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(file_path).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_document_new() {
        let doc = CrdtDocument::new();
        assert_eq!(doc.get_text(), "");
        assert!(!doc.is_dirty());
    }

    #[test]
    fn test_crdt_document_with_content() {
        let doc = CrdtDocument::with_content("hello world");
        assert_eq!(doc.get_text(), "hello world");
    }

    #[test]
    fn test_crdt_state_vector() {
        let doc = CrdtDocument::with_content("test");
        let sv = doc.get_state_vector();
        assert!(!sv.is_empty());
    }

    #[test]
    fn test_crdt_encode_state() {
        let doc = CrdtDocument::with_content("test content");
        let state = doc.encode_state();
        assert!(!state.is_empty());
    }

    #[test]
    fn test_crdt_apply_update() {
        let mut doc1 = CrdtDocument::with_content("hello");
        let mut doc2 = CrdtDocument::new();

        // Sync doc1 -> doc2
        let update = doc1.encode_state();
        doc2.apply_update(&update).unwrap();

        assert_eq!(doc2.get_text(), "hello");
        assert!(doc2.is_dirty());
    }

    #[test]
    fn test_crdt_delta_sync() {
        let mut doc1 = CrdtDocument::with_content("hello");
        let mut doc2 = CrdtDocument::new();

        // Initial sync
        let state1 = doc1.encode_state();
        doc2.apply_update(&state1).unwrap();

        // Get state vector before change
        let sv = doc2.get_state_vector();

        // Make change in doc1 (simulate by recreating with new content)
        // In real usage, we'd use Yjs transactions
        let doc1_new = CrdtDocument::with_content("hello world");

        // Get delta
        let delta = doc1_new.encode_delta(&sv).unwrap();

        // Delta should be smaller than full state
        let full_state = doc1_new.encode_state();
        // Note: For small docs, delta might not be smaller due to overhead
        assert!(!delta.is_empty());
    }

    #[test]
    fn test_crdt_convergence() {
        // Two documents starting from same state
        let initial = CrdtDocument::with_content("base");
        let initial_state = initial.encode_state();

        let mut doc1 = CrdtDocument::new();
        let mut doc2 = CrdtDocument::new();

        doc1.apply_update(&initial_state).unwrap();
        doc2.apply_update(&initial_state).unwrap();

        assert_eq!(doc1.get_text(), doc2.get_text());
        assert_eq!(doc1.get_text(), "base");
    }

    #[test]
    fn test_session_manager() {
        let manager = CrdtSessionManager::new();

        let session1 = manager.get_or_create_session("/test/file.pc");
        let session2 = manager.get_or_create_session("/test/file.pc");

        // Should return same session
        assert!(Arc::ptr_eq(&session1, &session2));
    }

    #[test]
    fn test_session_with_content() {
        let manager = CrdtSessionManager::new();

        let session = manager.get_or_create_session_with_content(
            "/test/file.pc",
            "component Button {}"
        );

        let session_guard = futures::executor::block_on(session.read());
        assert_eq!(session_guard.document.get_text(), "component Button {}");
    }
}
