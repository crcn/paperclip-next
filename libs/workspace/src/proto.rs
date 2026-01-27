//! Proto definitions for the gRPC service
//!
//! These would normally be generated from .proto files,
//! but for the spike we define them directly in Rust.

use serde::{Deserialize, Serialize};
use paperclip_proto::virt::EvaluatedModule;

/// Request to open a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFileRequest {
    pub path: String,
}

/// Response with file content and evaluated result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFileResponse {
    pub path: String,
    pub source: String,
    pub evaluated: Option<EvaluatedModule>,
}

/// Request to apply mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyMutationsRequest {
    pub path: String,
    pub mutations: Vec<Mutation>,
}

/// A mutation to apply to the AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mutation {
    /// Insert a new node
    InsertNode {
        parent_path: Vec<usize>,
        index: usize,
        node_type: String,
        props: serde_json::Value,
    },
    /// Remove a node
    RemoveNode {
        path: Vec<usize>,
    },
    /// Update node properties
    UpdateNode {
        path: Vec<usize>,
        props: serde_json::Value,
    },
    /// Move a node to a new location
    MoveNode {
        from_path: Vec<usize>,
        to_parent: Vec<usize>,
        to_index: usize,
    },
    /// Update a style declaration
    UpdateStyle {
        element_path: Vec<usize>,
        property: String,
        value: String,
    },
}

/// Result of mutation application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub success: bool,
    pub new_source: Option<String>,
    pub evaluated: Option<EvaluatedModule>,
    pub error: Option<String>,
}

/// Event streamed to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerEvent {
    /// File was changed (externally or via mutation)
    FileChanged {
        path: String,
        source: String,
        evaluated: EvaluatedModule,
    },
    /// A file was deleted
    FileDeleted {
        path: String,
    },
    /// An error occurred
    Error {
        message: String,
    },
}

/// Undo/Redo request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResponse {
    pub success: bool,
    pub source: Option<String>,
    pub evaluated: Option<EvaluatedModule>,
}
