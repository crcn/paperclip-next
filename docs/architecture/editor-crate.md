# Editor Crate Architecture

## Overview

The `editor` crate is the foundational document editing engine for Paperclip. It sits between the parser (syntax) and evaluator (rendering), providing the semantic layer for document lifecycle management, mutations, and collaboration.

## Why a Separate Crate?

The `editor` crate is **foundational** and **decoupled** from collaboration/workspace concerns:

1. **Reusability**: Can be used by CLI tools, workspace server, standalone apps
2. **Testability**: Unit testable without network/server setup
3. **Single-user support**: Works without collaboration features
4. **Clear abstraction**: Clean API for document operations
5. **Optional CRDT**: Collaboration is a feature flag, not required

## Package Structure

```
packages/editor/
├── Cargo.toml              # Dependencies, feature flags
├── README.md               # Usage documentation
├── src/
│   ├── lib.rs             # Public API, re-exports
│   ├── document.rs        # Document handle and lifecycle
│   ├── mutations.rs       # AST mutation operations
│   ├── session.rs         # Edit session management
│   ├── pipeline.rs        # Parse → Mutate → Evaluate pipeline
│   ├── crdt.rs           # CRDT wrapper (feature-gated)
│   └── errors.rs         # Error types
└── tests/
    └── integration_tests.rs
```

## Core Types

### Document

The main handle for a `.pc` file:

```rust
pub struct Document {
    pub path: PathBuf,
    pub version: u64,
    storage: DocumentStorage,
}

pub enum DocumentStorage {
    Memory { source: String, ast: ASTDocument },
    File { source: String, ast: ASTDocument, dirty: bool },
    #[cfg(feature = "collaboration")]
    CRDT { crdt: CRDTDocument, ast_cache: Option<ASTDocument> },
}
```

**Key methods**:
- `load(path)` - Load from file
- `from_source(path, source)` - Create from text
- `collaborative(path, source)` - CRDT-backed (requires feature)
- `ast()` - Get current AST (cached)
- `evaluate()` - Produce VDOM
- `apply(mutation)` - Apply semantic operation
- `save()` - Write to disk (if file-backed)

### Mutation

High-level semantic operations:

```rust
pub enum Mutation {
    MoveElement { node_id, new_parent_id, index },
    UpdateText { node_id, content },
    SetInlineStyle { node_id, property, value },
    SetAttribute { node_id, name, value },
    RemoveNode { node_id },
    InsertElement { parent_id, index, element },
}
```

**Key methods**:
- `validate(doc)` - Check if mutation is valid
- `apply(doc)` - Apply to document AST

### EditSession

Per-client editing state:

```rust
pub struct EditSession {
    pub id: String,
    pub document: Document,
    pub selected_nodes: Vec<String>,
    pub pending_mutations: Vec<PendingMutation>,
}
```

**Key methods**:
- `new(id, document)` - Create session
- `apply_optimistic(mutation)` - Apply immediately, queue for server
- `confirm_mutation(id)` - Remove from pending (server confirmed)
- `rebase(server_document)` - Sync with server state

### Pipeline

Coordinates full edit cycle:

```rust
pub struct Pipeline {
    document: Document,
    last_vdom: Option<VirtualDomDocument>,
}
```

**Key methods**:
- `new(document)` - Create pipeline
- `apply_mutation(mutation)` - Apply and get patches
- `full_evaluate()` - Re-render entire document
- `clear_cache()` - Force full re-render next time

### CRDTDocument (optional)

CRDT wrapper for collaboration:

```rust
#[cfg(feature = "collaboration")]
pub struct CRDTDocument {
    doc: yrs::Doc,
}
```

**Key methods**:
- `new()` - Create empty
- `from_ast(ast)` - Initialize from AST
- `apply(mutation)` - Apply mutation to CRDT
- `to_ast()` - Rebuild AST from CRDT
- `get_update()` - Get update bytes for network
- `apply_update(bytes)` - Apply remote update

## Feature Flags

### Default (no features)

Basic single-user editing:
- Load/save documents
- Apply mutations
- Evaluate to VDOM
- File-backed and memory-backed documents

### collaboration

Enable CRDT-backed multi-user editing:
- `CRDTDocument` type available
- `Document::collaborative()` constructor
- Yjs integration for convergence

### history (TODO)

Enable undo/redo:
- Mutation history tracking
- Undo/redo stack
- Checkpoint management

## Usage Patterns

### Single-User CLI Tool

```rust
use paperclip_editor::{Document, Mutation};

let mut doc = Document::load("button.pc")?;

let mutation = Mutation::UpdateText {
    node_id: "text-1".to_string(),
    content: "Click me!".to_string(),
};

doc.apply(mutation)?;
doc.save()?;
```

### Workspace Server (Multi-User)

```rust
use paperclip_editor::{Document, EditSession};

// Server maintains document
let doc = Document::collaborative("button.pc", source)?;

// Per-client sessions
let mut sessions: HashMap<ClientId, EditSession> = HashMap::new();

// Client sends mutation
fn handle_client_mutation(client_id: ClientId, mutation: Mutation) {
    // Apply to authoritative document
    doc.apply(mutation.clone())?;

    // Broadcast to all clients
    for (cid, session) in &mut sessions {
        if cid == client_id {
            session.confirm_mutation(&mutation_id);
        } else {
            // Send patches to other clients
        }
    }
}
```

### Visual Editor (Optimistic Updates)

```rust
use paperclip_editor::{Document, EditSession};

let doc = Document::from_source(path, source)?;
let mut session = EditSession::new("client-1".to_string(), doc);

// User drags element in visual editor
fn on_drag_drop(node_id: String, new_parent_id: String, index: usize) {
    let mutation = Mutation::MoveElement {
        node_id,
        new_parent_id,
        index,
    };

    // Apply optimistically (immediate UI update)
    let mutation_id = session.apply_optimistic(mutation.clone())?;

    // Send to server
    send_to_server(mutation_id, mutation);

    // Server will respond with confirmation or conflict
}
```

## Design Decisions

### 1. AST is Downstream of CRDT

For CRDT-backed documents, AST is a **derived view**:

```rust
CRDT (source of truth) → AST (materialized view) → VDOM (rendering)
```

This means:
- AST can be rebuilt from CRDT at any time
- No parallel truth
- CRDT handles convergence, AST handles semantics

### 2. Mutations are High-Level

Not "insert byte at position" or "replace subtree". Each mutation:
- Has clear semantic intent
- Validates structural constraints
- Is serializable
- Is independently meaningful

### 3. Optimistic Updates via EditSession

Clients apply mutations locally before server confirmation:
- Immediate UI feedback
- Pending queue for rebase
- Server confirmation removes from pending
- Conflicts trigger rebase

### 4. No Text-Level CRDT

We do **structural** CRDT on AST nodes, not character-level CRDT on source text:
- Simpler implementation
- Predictable merge behavior
- Matches visual editing model
- Fast evaluation

### 5. Repeat Instances Share Identity

All instances of a repeat block share the template's source_id:
- Simple AST mapping
- No synthetic IDs
- Template edits affect all instances
- Cannot edit individual instances (by design)

## Extension Points

### Custom Storage Backends

Add new `DocumentStorage` variants:

```rust
pub enum DocumentStorage {
    Memory { ... },
    File { ... },
    CRDT { ... },
    Database { conn: DbConnection },  // Example
    Remote { url: String },           // Example
}
```

### Custom Mutation Types

Extend `Mutation` enum:

```rust
pub enum Mutation {
    // Existing...
    MoveElement { ... },

    // Custom...
    SetDataBinding { node_id: String, binding: String },
}
```

### Custom Validation

Override `Mutation::validate()`:

```rust
impl Mutation {
    pub fn validate(&self, doc: &Document) -> Result<(), MutationError> {
        // Custom validation logic
        match self {
            Mutation::Custom { ... } => {
                // Your validation
            }
            _ => self.default_validate(doc)
        }
    }
}
```

## Dependencies

- `paperclip-parser`: AST types and parsing
- `paperclip-evaluator`: VDOM types and evaluation
- `serde`: Serialization
- `thiserror`: Error handling
- `yrs` (optional): CRDT backend

## Testing Strategy

### Unit Tests

Each module has unit tests:
- `document.rs`: Document lifecycle
- `mutations.rs`: Mutation validation
- `session.rs`: Session management
- `pipeline.rs`: Pipeline execution

### Integration Tests

Full workflows in `tests/integration_tests.rs`:
- Load → Mutate → Save
- Optimistic updates
- Serialization round-trips

### Property Tests (TODO)

- Mutation commutativity
- CRDT convergence
- AST ↔ CRDT round-trips

## Performance Characteristics

- **Document load**: O(n) where n = file size
- **Mutation apply**: O(log n) for node lookup, O(1) for update
- **Evaluation**: O(m) where m = AST size
- **VDOM diff**: O(k) where k = changed nodes
- **CRDT sync**: O(ops) where ops = number of operations

## Future Work

1. **Undo/Redo**: Mutation history + revert operations
2. **Incremental Evaluation**: Only re-evaluate affected subtrees
3. **Persistent CRDT**: Disk-backed CRDT state
4. **Alternative CRDTs**: Support Automerge alongside Yjs
5. **Performance Benchmarks**: Track mutation apply and evaluation perf
6. **Fuzzing**: Find edge cases in mutation validation

## Related Documentation

- [Collaboration Architecture](./collaboration.md)
- [Mutation Semantics](../specs/mutation-semantics.md) (TODO)
- [CRDT Implementation](../specs/crdt-implementation.md) (TODO)

## Summary

The `editor` crate is the **semantic layer** between syntax (parser) and rendering (evaluator). It provides:

✅ Clean document abstraction
✅ Intent-preserving mutations
✅ Optimistic editing support
✅ Optional CRDT collaboration
✅ Reusable across contexts

This foundation enables both single-user editing (CLI, standalone tools) and real-time collaboration (workspace server) while maintaining Paperclip's code-first philosophy.
