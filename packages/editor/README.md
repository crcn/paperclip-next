# Paperclip Editor

Core document editing engine for Paperclip.

## Overview

The `editor` crate provides foundational abstractions for editing Paperclip documents:

- **Document**: Core handle for loading, editing, and saving .pc files
- **Mutation**: High-level semantic operations (move, update, delete)
- **EditSession**: Manages editing state for single or multiple users
- **Pipeline**: Coordinates parse → mutate → evaluate → diff workflow
- **CRDT** (optional): Convergent document editing for collaboration

## Architecture

```
parser (AST types + parsing)
    ↓
editor (document lifecycle, CRDT, mutations)
    ↓
evaluator (AST → VDOM)
    ↓
workspace (server, multi-client)
    ↓
client (UI, visual ops)
```

## Usage

### Single-user editing

```rust
use paperclip_editor::{Document, Mutation};

// Load document
let mut doc = Document::load("button.pc")?;

// Apply mutation
let mutation = Mutation::UpdateText {
    node_id: "text-123".to_string(),
    content: "Click me!".to_string(),
};
doc.apply(mutation)?;

// Evaluate to VDOM
let vdom = doc.evaluate()?;

// Save
doc.save()?;
```

### Collaborative editing

```rust
use paperclip_editor::{Document, EditSession};

// Create session
let doc = Document::collaborative("button.pc", source)?;
let mut session = EditSession::new("client-1", doc);

// Apply optimistically
let mutation_id = session.apply_optimistic(mutation)?;

// Send to server, await confirmation
session.confirm_mutation(&mutation_id);
```

### Pipeline (for servers)

```rust
use paperclip_editor::{Document, Pipeline};

// Create pipeline
let doc = Document::load("button.pc")?;
let mut pipeline = Pipeline::new(doc);

// Initial render
let vdom = pipeline.full_evaluate()?;

// Apply mutations and get incremental patches
let result = pipeline.apply_mutation(mutation)?;
send_patches_to_clients(result.patches);
```

## Features

- `collaboration`: Enable CRDT-backed collaborative editing (requires `yrs`)
- `history`: Enable undo/redo (TODO)

## Design Principles

1. **AST is source of truth**: VDOM and patches are derived views
2. **CRDT for convergence**: Not for semantics - we define operation meaning
3. **Structural collaboration**: Node-level operations, not text-level
4. **Optimistic clients**: Local projection can be discarded and rebuilt
5. **Server authority**: Client state always defers to server

## Mutation Semantics

### Move
- Atomic relocation of node to new parent
- Fails if parent deleted (does not create orphan)
- Fails if would create cycle
- Last move wins if concurrent moves of same node

### UpdateText
- Atomic replacement (not character diff)
- Last write wins if concurrent edits
- No merge attempts

### Delete
- Removes node and all descendants
- Concurrent moves to deleted nodes fail
- Concurrent edits of deleted nodes are no-ops

## TODO

- [ ] Implement AST mutation helpers in parser
- [ ] Complete CRDT ↔ AST serialization
- [ ] Add undo/redo support
- [ ] Add text-level editing (for code editor sync)
- [ ] Performance benchmarks
- [ ] Conflict resolution strategies
