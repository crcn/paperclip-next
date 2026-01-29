# Workspace Server - Editor Integration Example

## Overview

This document shows how the `workspace` server uses the `editor` crate for collaborative document editing.

## Architecture

```rust
use paperclip_editor::{Document, Mutation, Pipeline};
use std::collections::HashMap;
use std::path::PathBuf;

type ClientId = String;

/// Collaborative workspace server
pub struct WorkspaceServer {
    /// Documents being edited (one per file)
    documents: HashMap<PathBuf, Document>,

    /// Pipelines for each document (evaluation + diffing)
    pipelines: HashMap<PathBuf, Pipeline>,

    /// Client sessions
    sessions: HashMap<ClientId, ClientSession>,
}

pub struct ClientSession {
    client_id: ClientId,
    document_path: PathBuf,
    selected_nodes: Vec<String>,
}
```

## Key Operations

### 1. Client Connects

```rust
impl WorkspaceServer {
    pub fn client_connect(
        &mut self,
        client_id: ClientId,
        path: PathBuf,
        source: String
    ) -> Result<(), EditorError> {
        // Create or get document
        if !self.documents.contains_key(&path) {
            let doc = Document::from_source(path.clone(), source)?;
            let mut pipeline = Pipeline::new(doc.clone());

            // Initial evaluation
            pipeline.full_evaluate()?;

            self.documents.insert(path.clone(), doc);
            self.pipelines.insert(path.clone(), pipeline);
        }

        // Create session
        self.sessions.insert(client_id.clone(), ClientSession {
            client_id,
            document_path: path,
            selected_nodes: vec![],
        });

        Ok(())
    }
}
```

### 2. Client Sends Mutation

```rust
impl WorkspaceServer {
    pub fn apply_mutation(
        &mut self,
        client_id: &ClientId,
        mutation: Mutation,
    ) -> Result<BroadcastUpdate, EditorError> {
        // Get client session
        let session = self.sessions.get(client_id)
            .ok_or(EditorError::NotFound)?;

        let path = &session.document_path;

        // Get pipeline
        let pipeline = self.pipelines.get_mut(path)
            .ok_or(EditorError::NotFound)?;

        // Apply mutation through pipeline
        let result = pipeline.apply_mutation(mutation.clone())?;

        // Broadcast to all clients editing this document
        let mut client_patches = HashMap::new();
        for (cid, sess) in &self.sessions {
            if &sess.document_path == path {
                client_patches.insert(cid.clone(), result.patches.clone());
            }
        }

        Ok(BroadcastUpdate {
            version: result.version,
            source_client: client_id.clone(),
            patches: client_patches,
        })
    }
}

pub struct BroadcastUpdate {
    pub version: u64,
    pub source_client: ClientId,
    pub patches: HashMap<ClientId, Vec<u8>>,
}
```

### 3. Example Usage

```rust
fn main() {
    let mut server = WorkspaceServer::new();

    let source = r#"
        component Button {
            render div {
                text "Click me"
            }
        }
    "#;

    // Client A connects
    server.client_connect(
        "client-a".to_string(),
        PathBuf::from("button.pc"),
        source.to_string(),
    ).unwrap();

    // Client B connects
    server.client_connect(
        "client-b".to_string(),
        PathBuf::from("button.pc"),
        source.to_string(),
    ).unwrap();

    // Client A sends mutation
    let mutation = Mutation::UpdateText {
        node_id: "text-node-id".to_string(),
        content: "Hello World!".to_string(),
    };

    let update = server.apply_mutation(&"client-a".to_string(), mutation).unwrap();

    println!("✓ Mutation applied - version {}", update.version);
    println!("✓ Broadcasting to {} clients", update.patches.len());
}
```

## Key Benefits

1. **Single Document per File**: One authoritative document
2. **Pipeline Coordination**: Evaluation + diffing handled automatically
3. **Per-Client Patches**: Efficient updates
4. **Session Management**: Track client state
5. **Clean Separation**: Editor crate handles document logic, workspace handles networking

## gRPC Integration

```protobuf
service WorkspaceService {
    rpc ApplyMutation(MutationRequest) returns (MutationResponse);
    rpc StreamUpdates(stream ClientUpdate) returns (stream ServerUpdate);
}

message MutationRequest {
    string client_id = 1;
    bytes mutation = 2;  // Serialized Mutation
}

message ServerUpdate {
    uint64 version = 1;
    bytes patches = 2;  // Serialized patches
}
```

## Collaboration with CRDT

When using the `collaboration` feature:

```rust
// Create CRDT-backed document
let doc = Document::collaborative(path, source)?;

// Mutations automatically sync via CRDT
pipeline.apply_mutation(mutation)?;

// CRDT handles convergence
// All clients eventually reach same state
```

## Summary

The `workspace` server:
- Uses `editor::Document` for document lifecycle
- Uses `editor::Pipeline` for evaluation + diffing
- Uses `editor::Mutation` for all edits
- Broadcasts patches to clients
- Handles session management and networking

The `editor` crate:
- Manages document state
- Validates mutations
- Coordinates parse → evaluate → diff
- (Optional) CRDT for collaboration

This clean separation allows:
- Editor to be reused in CLI, standalone apps
- Workspace to focus on networking/gRPC
- Testing each layer independently
