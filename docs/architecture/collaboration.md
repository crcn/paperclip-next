# Paperclip Collaboration Architecture

## Overview

Paperclip uses a **code-first, AST-level collaboration model** where the source code (`.pc` files) is the source of truth, and VDOM/patches are derived views optimized for streaming updates to clients.

## Core Principles

1. **AST is source of truth**: VDOM and patches are derived views, never canonical
2. **CRDT for convergence**: Guarantees eventual consistency, but we define operation semantics
3. **Structural collaboration**: Node-level operations (move, update, delete), not text-level
4. **Optimistic clients**: Local state is a speculative projection that can be rebuilt
5. **Server authority**: Client state always defers to server

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│ parser: .pc text → AST                                  │ (syntax)
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ editor: Document lifecycle + mutations + CRDT           │ (semantics)
│  - Load/save documents                                  │
│  - Apply mutations with validation                      │
│  - CRDT-backed convergence (optional)                   │
│  - Coordinate parse → evaluate pipeline                 │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ evaluator: AST → VDOM                                   │ (rendering)
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ workspace: Multi-client server + networking             │ (distribution)
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ client: Visual UI + optimistic updates                  │ (presentation)
└─────────────────────────────────────────────────────────┘
```

## Data Flow

### Single-User Editing

```
User Edit → Mutation → Apply to AST → Evaluate → VDOM → Render
```

### Multi-User Collaborative Editing

```
Client A: Visual Edit → Mutation → Optimistic Apply → Render
                           ↓
Server: Validate → Apply to CRDT → Update AST → Evaluate → VDOM
                           ↓
Client B: Receive Patches → Apply → Render
Client A: Receive Confirmation → Reconcile
```

## Key Components

### 1. Document (packages/editor/src/document.rs)

Core abstraction for a `.pc` file. Can be:
- **Memory-backed**: Temporary, for testing
- **File-backed**: Single-user editing
- **CRDT-backed**: Multi-user collaboration

```rust
let mut doc = Document::load("button.pc")?;
doc.apply(mutation)?;
let vdom = doc.evaluate()?;
doc.save()?;
```

### 2. Mutation (packages/editor/src/mutations.rs)

High-level semantic operations. **Not** low-level tree edits.

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

Each mutation:
- Validates structural constraints
- Has well-defined semantics
- Is serializable (JSON/protobuf)
- Is intention-preserving

### 3. EditSession (packages/editor/src/session.rs)

Per-client editing state:
- Local document copy
- Pending mutations (optimistic)
- Selection state
- Rebase capability

```rust
let mut session = EditSession::new("client-1", doc);
let mutation_id = session.apply_optimistic(mutation)?;
// ... send to server ...
session.confirm_mutation(&mutation_id);
```

### 4. Pipeline (packages/editor/src/pipeline.rs)

Coordinates: Parse → Mutate → Evaluate → Diff

```rust
let mut pipeline = Pipeline::new(doc);
let result = pipeline.apply_mutation(mutation)?;
// result.patches contains incremental updates
```

### 5. CRDT (packages/editor/src/crdt.rs)

Optional CRDT backend for collaboration:

```rust
#[cfg(feature = "collaboration")]
let doc = Document::collaborative(path, source)?;
```

**Key insight**: CRDT operates on **AST structure**, not character-level text. This keeps the system simple and predictable.

## Collaboration Model

### Server State

```rust
pub struct CollaborationServer {
    // One authoritative document per file (CRDT-backed)
    documents: HashMap<PathBuf, Document>,

    // Per-client sessions
    sessions: HashMap<ClientId, EditSession>,

    // Per-client last VDOM (for efficient diffing)
    vdom_cache: HashMap<ClientId, VirtualDomDocument>,
}
```

### Client State

```rust
pub struct EditorClient {
    // Speculative local copy
    local_document: Document,
    local_version: u64,

    // Pending mutations
    pending: Vec<PendingMutation>,

    // Current rendered VDOM
    current_vdom: VirtualDomDocument,
}
```

### Mutation Flow

1. **Client applies optimistically**:
   ```rust
   let mutation_id = session.apply_optimistic(mutation)?;
   // Immediate UI update
   ```

2. **Server validates and applies**:
   ```rust
   doc.apply(mutation)?;  // CRDT handles convergence
   let new_vdom = doc.evaluate()?;
   ```

3. **Server broadcasts to all clients**:
   ```rust
   for client in clients {
       let patches = diff(client.last_vdom, new_vdom);
       send_patches(client, patches);
   }
   ```

4. **Clients reconcile**:
   ```rust
   if update.mutation_id == pending_mutation_id {
       // Our mutation confirmed
       session.confirm_mutation(update.mutation_id);
   } else {
       // Other client's mutation
       session.rebase(update.document);
   }
   ```

## Mutation Semantics

### Move

- **Intent**: Relocate node to new parent
- **Validation**: Parent exists, no cycles, valid parent-child relationship
- **Conflict resolution**: Last move wins (timestamp-based)
- **Failed if**: Parent deleted, would create cycle

### UpdateText

- **Intent**: Replace text content
- **Validation**: Node is text type
- **Conflict resolution**: Last write wins
- **No merging**: Atomic replacement

### Delete

- **Intent**: Remove node and all descendants
- **Validation**: Node exists, not required structural element
- **Conflict resolution**: Delete wins over edits
- **Side effects**: Concurrent moves to deleted nodes fail

### Insert

- **Intent**: Add new element
- **Validation**: Parent exists, index valid
- **Conflict resolution**: Both inserts succeed (CRDT handles ordering)

## Identity System

Every VNode has a `source_id` that maps to an AST node:

```rust
pub struct VNode {
    pub source_id: String,  // From AST parsing (e.g., "80f4925f-5")
    // ... other fields
}
```

**Critical rules**:
1. Every VNode MUST correspond to exactly one AST node
2. No runtime-generated IDs without AST lineage
3. Repeat instances share template source_id
4. Source IDs are stable across evaluation

This allows:
- Visual operations to target specific AST nodes
- Refactoring without breaking references
- Stable diffing across structure changes

## Repeat Block Handling

**Decision**: Repeat instances share template identity (Option A)

```paperclip
repeat item in items {
  div { text {item.name} }  // All divs share same source_id
}
```

**Implications**:
- ✅ Simple, unambiguous AST mapping
- ✅ Template edits affect all instances
- ⚠️ Cannot edit individual instances visually
- ⚠️ Cannot move/delete single instance

**Rationale**: Preserves AST integrity. Per-instance customization requires breaking out into separate elements.

## Forbidden Operations

To preserve AST integrity, we **do not support**:

1. **Editing individual repeat instances**: All instances share template identity
2. **Conditional branch manipulation**: Cannot delete just then/else branch
3. **Slot content restructuring**: Defined at call site, not visually movable
4. **Subtree replacement**: Too generic, loses intent
5. **innerHTML-style edits**: Bypasses structural validation

**Philosophy**: Paperclip is code-first. When visual editing becomes ambiguous, drop to code.

## Conflict Resolution Strategies

| Scenario | Resolution |
|----------|------------|
| Concurrent moves of same node | Last move wins (by timestamp) |
| Concurrent text edits | Last write wins |
| Move + Delete | Delete wins (move fails) |
| Edit + Delete | Delete wins (edit ignored) |
| Concurrent inserts at same position | Both succeed (CRDT orders) |
| Move to deleted parent | Move fails gracefully |

## Performance Optimizations

1. **Per-client VDOM caching**: Server tracks last VDOM per client for efficient diffing
2. **Incremental evaluation**: Only re-evaluate affected components (future)
3. **Batched updates**: Collect mutations over 16ms window before broadcasting
4. **Compressed patches**: Use binary format (protobuf) instead of JSON

## Error Handling

1. **Validation errors**: Reject mutation before applying
2. **Concurrent conflicts**: Apply conflict resolution rules
3. **Invalid mutations**: Client receives error, can retry
4. **Network partitions**: Client rebases on reconnect
5. **Diverged state**: Full VDOM re-sync as fallback

## Future Enhancements

1. **Undo/Redo**: Based on mutation history
2. **Conflict warnings**: UI hints when clients edit same region
3. **Soft locking**: Visual indication of what others are editing
4. **Merge strategies**: User-configurable conflict resolution
5. **Time travel**: Replay mutation history

## What We Explicitly Do NOT Do

1. ❌ **Character-level text CRDT**: Too complex, unnecessary
2. ❌ **Patch-level OT**: Wrong abstraction layer
3. ❌ **Automatic merge of conflicting content**: Be explicit
4. ❌ **Generate synthetic nodes**: All nodes from AST
5. ❌ **Per-instance repeat customization**: Breaks template model

## References

- **Yjs**: CRDT library we use for convergence guarantees
- **Operational Transformation**: Inspiration for mutation model
- **Figma multiplayer**: Similar architecture (visual → structured ops → CRDT)
- **Google Docs**: Character-level CRDT (we do AST-level instead)

## Summary

**Paperclip collaboration is AST-first, intent-driven, CRDT-backed, and render-derived — never patch-merged.**

This architecture:
- Respects source code as truth
- Scales to real-time collaboration
- Avoids visual editor pitfalls
- Keeps Paperclip's code-first integrity
