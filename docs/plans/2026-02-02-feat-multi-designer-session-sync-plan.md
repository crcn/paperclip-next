---
title: "feat: Multi-Designer Session Synchronization"
type: feat
date: 2026-02-02
revised: 2026-02-02
status: final
---

# Multi-Designer Session Synchronization

## Architecture Decision

**Y.Text is the single source of truth, owned by the server.**

This is a clean separation based on the fundamental insight that there are **two different edit types** with different requirements:

| Edit Type | Client | Needs Local Y.Doc? | Why |
|-----------|--------|-------------------|-----|
| **Text edits** | VS Code | **Yes** | Instant keystroke feedback, character-level merge |
| **Visual edits** | Designer | **No** | Semantic mutations translated server-side |

---

## Architecture

```
                    ┌─────────────────────────────────────────────────────┐
                    │                      SERVER                          │
                    │                                                      │
                    │   ┌────────────────────────────────────────────┐    │
                    │   │              Y.Text (SINGLE)                │    │
                    │   │           Source of Truth                   │    │
                    │   │                                             │    │
                    │   │  • Receives Yjs updates from VS Code        │    │
                    │   │  • Receives semantic mutations from Designer│    │
                    │   │  • Translates mutations → text edits        │    │
                    │   └────────────────────────────────────────────┘    │
                    │                        │                             │
                    │                        ▼                             │
                    │   ┌────────────────────────────────────────────┐    │
                    │   │     Parse → AST → Evaluate → VDOM          │    │
                    │   │                                             │    │
                    │   │  • AstIndex: maps node IDs to StickyIndex   │    │
                    │   │  • Rebuilt after every Y.Text change        │    │
                    │   └────────────────────────────────────────────┘    │
                    │                        │                             │
                    └────────────────────────┼─────────────────────────────┘
                                             │
                    ┌────────────────────────┼────────────────────────┐
                    │                        │                        │
                    ▼                        ▼                        ▼
           ┌──────────────┐         ┌──────────────┐         ┌──────────────┐
           │   VS Code    │         │   Designer   │         │   Designer   │
           │              │         │              │         │              │
           │ Local Y.Doc  │         │  NO Y.Doc    │         │  NO Y.Doc    │
           │ (Yjs sync)   │         │              │         │              │
           │              │         │ Sends:       │         │ Sends:       │
           │ Sends:       │         │  Mutations   │         │  Mutations   │
           │  Yjs updates │         │              │         │              │
           │              │         │ Receives:    │         │ Receives:    │
           │ Receives:    │         │  VDOM patches│         │  VDOM patches│
           │  Yjs updates │         │  (via SSE)   │         │  (via SSE)   │
           └──────────────┘         └──────────────┘         └──────────────┘
```

---

## Why This Architecture

### VS Code: Local Y.Doc Required

VS Code users expect **instant feedback** when typing. A network roundtrip per keystroke is unacceptable.

```typescript
// User types 'a' - must appear immediately
ytext.insert(cursor, 'a');  // Local Y.Doc - instant

// Sync happens in background
ydoc.on('update', (update) => sendToServer(update));
```

The standard Yjs sync protocol handles:
- Character-level merge for concurrent edits
- State vector exchange for efficient delta sync
- Automatic conflict resolution

### Designer: NO Y.Doc Required

Designers don't type text. They perform **semantic operations**:
- Resize a frame → `SetFrameBounds(frameId, {x, y, width, height})`
- Move an element → `MoveElement(nodeId, newParentId, index)`
- Change a style → `SetStyleProperty(nodeId, property, value)`

The **server** translates these to text edits:

```rust
// Server receives: SetFrameBounds { frame_id: "abc", bounds: {x: 100, ...} }
fn handle_mutation(&mut self, mutation: Mutation) -> Result<()> {
    // 1. Find node in AST by ID
    let (start, end) = self.ast_index.resolve(&mutation.frame_id)?;

    // 2. Generate new text
    let new_text = format!("@frame(x: {}, y: {}, width: {}, height: {})",
        mutation.bounds.x, mutation.bounds.y,
        mutation.bounds.width, mutation.bounds.height);

    // 3. Apply to Y.Text (single source of truth)
    let mut txn = self.doc.transact_mut();
    self.text.remove_range(&mut txn, start, end - start);
    self.text.insert(&mut txn, start, &new_text);

    // 4. Re-parse, re-evaluate, broadcast VDOM patches
    self.process_and_broadcast()
}
```

This is **clean** because:
- Designer never manipulates text
- Server owns all text operations
- No client-side CRDT complexity for designers
- Clear separation of concerns

---

## Data Flows

### Flow 1: Designer Resizes Frame

```
Designer                          Server                           VS Code
   │                                │                                 │
   │ 1. User drags resize handle    │                                 │
   │    (local optimistic update)   │                                 │
   │                                │                                 │
   │ 2. POST SetFrameBounds ───────>│                                 │
   │    {frameId, bounds}           │                                 │
   │                                │ 3. Resolve frameId → position   │
   │                                │    via AstIndex + StickyIndex   │
   │                                │                                 │
   │                                │ 4. Apply text edit to Y.Text    │
   │                                │                                 │
   │                                │ 5. Re-parse → AST               │
   │                                │    Re-evaluate → VDOM           │
   │                                │    Diff → patches               │
   │                                │                                 │
   │<────── 6. SSE: VDOM patches ───│                                 │
   │                                │───── 7. Yjs update ────────────>│
   │                                │                                 │
   │ 8. Apply patches to renderer   │                 8. Apply to doc │
   │    (confirms optimistic)       │                                 │
```

### Flow 2: VS Code Types Character

```
VS Code                           Server                           Designer
   │                                │                                 │
   │ 1. User types 'a'              │                                 │
   │    Local Y.Doc updated         │                                 │
   │    (instant feedback)          │                                 │
   │                                │                                 │
   │ 2. Yjs update ────────────────>│                                 │
   │                                │                                 │
   │                                │ 3. Apply to server Y.Doc        │
   │                                │                                 │
   │                                │ 4. Re-parse → AST               │
   │                                │    Re-evaluate → VDOM           │
   │                                │    Diff → patches               │
   │                                │                                 │
   │                                │───── 5. SSE: VDOM patches ─────>│
   │                                │                                 │
   │<──── 6. Yjs updates to other ──│                 6. Apply patches│
   │      VS Code instances         │                    to renderer  │
```

### Flow 3: Concurrent Edits (Designer + VS Code)

```
Designer                          Server                           VS Code
   │                                │                                 │
   │ SetFrameBounds ───────────────>│                                 │
   │                                │<───────────── Types 'hello' ────│
   │                                │                                 │
   │                                │ Both edits applied to Y.Text    │
   │                                │ (StickyIndex ensures correct    │
   │                                │  positions even with concurrent │
   │                                │  modifications)                 │
   │                                │                                 │
   │<────── VDOM patches ───────────│───────── Yjs update ───────────>│
   │                                │───────── VDOM patches ─────────>│
   │                                │                                 │
```

---

## Server Implementation

### Core Components

```rust
// packages/workspace/src/session.rs

/// A collaborative editing session for a single file
pub struct EditSession {
    /// The authoritative Y.Doc for this file
    doc: Doc,

    /// Y.Text containing the source code
    text: TextRef,

    /// Maps AST node IDs to StickyIndex positions
    /// Rebuilt after every parse
    ast_index: AstIndex,

    /// Current parsed AST (derived from Y.Text)
    ast: Option<Document>,

    /// Current evaluated VDOM (derived from AST)
    vdom: Option<VDocument>,
}

impl EditSession {
    /// Handle a semantic mutation from a designer
    pub fn apply_mutation(&mut self, mutation: Mutation) -> Result<Vec<VDocPatch>> {
        // 1. Resolve node position using StickyIndex
        let span = self.ast_index.resolve(&mutation.node_id())?;

        // 2. Generate replacement text
        let new_text = mutation.generate_text(&self.ast)?;

        // 3. Apply to Y.Text within a transaction
        {
            let mut txn = self.doc.transact_mut();
            self.text.remove_range(&mut txn, span.start, span.len());
            self.text.insert(&mut txn, span.start, &new_text);
        }

        // 4. Re-parse, re-evaluate, diff
        self.process_change()
    }

    /// Handle a Yjs update from VS Code
    pub fn apply_yjs_update(&mut self, update: &[u8]) -> Result<Vec<VDocPatch>> {
        // 1. Apply update to Y.Doc
        {
            let mut txn = self.doc.transact_mut();
            let update = Update::decode_v1(update)?;
            txn.apply_update(update)?;
        }

        // 2. Re-parse, re-evaluate, diff
        self.process_change()
    }

    /// Common processing after any change to Y.Text
    fn process_change(&mut self) -> Result<Vec<VDocPatch>> {
        // 1. Get current text
        let source = {
            let txn = self.doc.transact();
            self.text.get_string(&txn)
        };

        // 2. Parse to AST
        let new_ast = parse(&source)?;

        // 3. Rebuild AstIndex with fresh StickyIndex positions
        self.ast_index = AstIndex::build(&new_ast, &self.doc, &self.text);

        // 4. Evaluate to VDOM
        let new_vdom = evaluate(&new_ast)?;

        // 5. Diff against previous VDOM
        let patches = match &self.vdom {
            Some(old) => diff_vdocument(old, &new_vdom),
            None => vec![VDocPatch::Initialize(new_vdom.clone())],
        };

        // 6. Store new state
        self.ast = Some(new_ast);
        self.vdom = Some(new_vdom);

        Ok(patches)
    }
}
```

### AstIndex with StickyIndex

```rust
// packages/workspace/src/ast_index.rs

/// Maps AST node IDs to StickyIndex positions
///
/// StickyIndex (from yrs) tracks positions that survive concurrent edits.
/// When another user inserts text before our target, StickyIndex adjusts.
pub struct AstIndex {
    /// Map from node ID to (start, end) StickyIndex pair
    positions: HashMap<String, (StickyIndex, StickyIndex)>,
}

impl AstIndex {
    /// Build index from freshly parsed AST
    pub fn build(ast: &Document, doc: &Doc, text: &TextRef) -> Self {
        let mut positions = HashMap::new();
        let mut txn = doc.transact_mut();

        // Index all nodes that can be targeted by mutations
        for component in &ast.components {
            Self::index_component(component, text, &mut txn, &mut positions);
        }

        Self { positions }
    }

    fn index_component(
        component: &Component,
        text: &TextRef,
        txn: &mut TransactionMut,
        positions: &mut HashMap<String, (StickyIndex, StickyIndex)>,
    ) {
        // Index the component itself
        if let Some(id) = &component.id {
            let start = text.sticky_index(txn, component.span.start, Assoc::After);
            let end = text.sticky_index(txn, component.span.end, Assoc::Before);
            if let (Some(s), Some(e)) = (start, end) {
                positions.insert(id.clone(), (s, e));
            }
        }

        // Index @frame annotations
        for comment in &component.comments {
            if let Some(frame) = parse_frame_annotation(comment) {
                let start = text.sticky_index(txn, frame.span.start, Assoc::After);
                let end = text.sticky_index(txn, frame.span.end, Assoc::Before);
                if let (Some(s), Some(e)) = (start, end) {
                    positions.insert(frame.id.clone(), (s, e));
                }
            }
        }

        // Recursively index children
        for child in &component.body {
            Self::index_node(child, text, txn, positions);
        }
    }

    /// Resolve a node ID to current absolute positions
    pub fn resolve(&self, node_id: &str, txn: &Transaction) -> Option<Span> {
        let (start_sticky, end_sticky) = self.positions.get(node_id)?;

        let start = start_sticky.get_offset(txn)?;
        let end = end_sticky.get_offset(txn)?;

        Some(Span {
            start: start.index,
            end: end.index,
        })
    }
}
```

---

## VS Code Implementation

VS Code needs a local Y.Doc for instant feedback, synced with the server via Yjs protocol.

```typescript
// packages/vscode-extension/src/crdt-sync.ts

import * as Y from 'yjs';
import * as vscode from 'vscode';

export class DocumentSync {
    private readonly ydoc: Y.Doc;
    private readonly ytext: Y.Text;
    private applyingRemoteCount = 0;
    private readonly disposables: vscode.Disposable[] = [];

    constructor(
        private readonly document: vscode.TextDocument,
        private readonly transport: CrdtTransport,
    ) {
        this.ydoc = new Y.Doc();
        this.ytext = this.ydoc.getText('source');

        this.setupSync();
    }

    private setupSync(): void {
        // 1. Initialize with current document content
        this.ydoc.transact(() => {
            this.ytext.insert(0, this.document.getText());
        }, 'init');

        // 2. Send local changes to server
        this.ydoc.on('update', (update: Uint8Array, origin: unknown) => {
            if (origin === 'remote') return;  // Don't echo remote changes
            this.transport.sendUpdate(update);
        });

        // 3. Apply VS Code edits to local Y.Doc
        this.disposables.push(
            vscode.workspace.onDidChangeTextDocument(e => {
                if (e.document !== this.document) return;
                if (this.applyingRemoteCount > 0) return;  // Skip remote-caused events

                this.ydoc.transact(() => {
                    for (const change of e.contentChanges) {
                        const offset = change.rangeOffset;
                        if (change.rangeLength > 0) {
                            this.ytext.delete(offset, change.rangeLength);
                        }
                        if (change.text) {
                            this.ytext.insert(offset, change.text);
                        }
                    }
                }, 'local');
            })
        );

        // 4. Apply remote changes to VS Code document
        this.transport.onUpdate((update: Uint8Array) => {
            // Apply to Y.Doc
            Y.applyUpdate(this.ydoc, update, 'remote');

            // Sync to VS Code
            const newContent = this.ytext.toString();
            const currentContent = this.document.getText();

            if (newContent !== currentContent) {
                this.applyingRemoteCount++;

                const edit = new vscode.WorkspaceEdit();
                const fullRange = new vscode.Range(
                    this.document.positionAt(0),
                    this.document.positionAt(currentContent.length)
                );
                edit.replace(this.document.uri, fullRange, newContent);

                vscode.workspace.applyEdit(edit).then(
                    () => this.applyingRemoteCount--,
                    () => this.applyingRemoteCount--
                );
            }
        });
    }

    async initialSync(): Promise<void> {
        // Request current state from server
        const stateVector = Y.encodeStateVector(this.ydoc);
        const serverState = await this.transport.sync(stateVector);

        if (serverState.length > 0) {
            Y.applyUpdate(this.ydoc, serverState, 'remote');
        }
    }

    dispose(): void {
        this.disposables.forEach(d => d.dispose());
        this.ydoc.destroy();
    }
}
```

---

## Designer Implementation

Designers are simple - no CRDT, just HTTP mutations and SSE for VDOM patches.

```typescript
// packages/designer/src/api.ts

export interface Mutation {
    type: string;
    [key: string]: unknown;
}

export interface SetFrameBoundsMutation extends Mutation {
    type: 'SetFrameBounds';
    frameId: string;
    bounds: { x: number; y: number; width: number; height: number };
}

export async function sendMutation(
    serverUrl: string,
    filePath: string,
    mutation: Mutation
): Promise<MutationResult> {
    const response = await fetch(`${serverUrl}/api/mutation`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ file_path: filePath, mutation }),
    });

    if (!response.ok) {
        const error = await response.text();
        return { success: false, error };
    }

    return { success: true };
}
```

```typescript
// packages/designer/src/machine/engines.ts

// On resize end, send mutation to server
if (event.type === "tool/resizeEnd") {
    const drag = prevState.tool.drag;
    if (!drag) return;

    const frame = state.frames[drag.frameIndex];
    const newBounds = calculateResizedBounds(drag.startBounds, drag.handle, delta);

    const result = await sendMutation(serverUrl, filePath, {
        type: 'SetFrameBounds',
        frameId: frame.id,
        bounds: newBounds,
    });

    if (!result.success) {
        // Rollback optimistic update
        machine.dispatch({ type: 'mutation/failed', payload: { error: result.error } });
    }
}
```

---

## Optimistic Update & Race Prevention

### The Problem

Without protection, concurrent edits cause visual snapback:

```
T1: Designer applies optimistic resize (frame at 100,100)
T2: Designer sends mutation
T3: VS Code edit arrives at server first
T4: Server broadcasts VDOM (frame still at 0,0)
T5: Designer receives VDOM → SNAPBACK to 0,0  ← BAD
T6: Server processes mutation
T7: Server broadcasts VDOM (frame at 100,100)
T8: Designer receives VDOM → SNAP FORWARD     ← BAD
```

### The Fix: Pending Mutation Guard

**Rule: While a mutation is in-flight, preserve optimistic state for that frame.**

```typescript
// packages/designer/src/machine/state.ts

interface PendingMutation {
  frameId: string;
  optimisticBounds: FrameBounds;
}

interface DesignerState {
  // ... existing fields
  pendingMutations: Map<string, PendingMutation>;
}
```

```typescript
// packages/designer/src/machine/reducers.ts

case "document/loaded": {
  let frames = event.payload.frames;

  // Preserve optimistic state for frames with pending mutations
  for (const [frameId, pending] of state.pendingMutations) {
    const idx = frames.findIndex(f => f.id === frameId);
    if (idx >= 0) {
      frames = [
        ...frames.slice(0, idx),
        { ...frames[idx], bounds: pending.optimisticBounds },
        ...frames.slice(idx + 1),
      ];
    }
  }

  return { ...state, document: event.payload.document, frames };
}

case "mutation/pending": {
  const newPending = new Map(state.pendingMutations);
  newPending.set(event.payload.frameId, {
    frameId: event.payload.frameId,
    optimisticBounds: event.payload.optimisticBounds,
  });
  return { ...state, pendingMutations: newPending };
}

case "mutation/acked":
case "mutation/failed": {
  const newPending = new Map(state.pendingMutations);
  newPending.delete(event.payload.frameId);
  return { ...state, pendingMutations: newPending };
}
```

```typescript
// packages/designer/src/machine/engines.ts

if (event.type === "tool/resizeEnd") {
  const frame = state.frames[drag.frameIndex];
  const newBounds = calculateResizedBounds(...);

  // 1. Mark mutation as pending (preserves optimistic state)
  machine.dispatch({
    type: "mutation/pending",
    payload: { frameId: frame.id, optimisticBounds: newBounds }
  });

  // 2. Send to server
  sendMutation(serverUrl, filePath, {
    type: 'SetFrameBounds',
    frameId: frame.id,
    bounds: newBounds,
  })
    .then(() => {
      // Success: clear pending, accept server state
      machine.dispatch({ type: "mutation/acked", payload: { frameId: frame.id } });
    })
    .catch((error) => {
      // Failure: clear pending, let server state win (rollback)
      machine.dispatch({ type: "mutation/failed", payload: { frameId: frame.id, error } });
    });
}
```

### The Fixed Timeline

```
T1: Designer applies optimistic resize (frame at 100,100)
    pendingMutations.set("frame-a", {bounds: 100,100})
T2: Designer sends mutation
T3: VS Code edit arrives at server first
T4: Server broadcasts VDOM (frame at 0,0)
T5: Designer receives VDOM
    → "frame-a" has pending mutation
    → Preserve optimistic bounds (100,100)
    → No visual change ✓
T6: Server processes mutation
T7: Mutation ack arrives
    pendingMutations.delete("frame-a")
T8: Server broadcasts VDOM (frame at 100,100)
T9: VDOM applied fully → No visual change ✓
```

No snapback. No flicker.

---

## Communication Channels

| Client | To Server | From Server |
|--------|-----------|-------------|
| **Designer** | HTTP POST `/api/mutation` | SSE `/api/preview` (VDOM patches) |
| **VS Code** | gRPC `CrdtSync` stream (Yjs updates) | gRPC `CrdtSync` stream (Yjs updates) + notification of VDOM change |

This is clean:
- Designers: Simple request/response for mutations, push for VDOM
- VS Code: Bidirectional stream for Yjs sync

---

## Implementation Phases

### Phase 1: Server-Side Mutation Handler

Implement the mutation → text edit translation on the server.

**Files:**
- `packages/workspace/src/mutation_handler.rs` ✓ (exists)
- `packages/workspace/src/ast_index.rs` ✓ (exists)

**Tasks:**
- [x] `AstIndex` maps node IDs to `StickyIndex` positions
- [x] `MutationHandler` translates semantic mutations to text edits
- [x] Integration tests for mutation handling

### Phase 2: Designer → Server → VDOM Flow

Wire up the designer to send mutations and receive VDOM patches.

**Files:**
- `packages/workspace/src/server.rs` - HTTP mutation endpoint
- `packages/designer/src/api.ts` - Mutation sending

**Tasks:**
- [ ] HTTP POST `/api/mutation` endpoint processes mutations
- [ ] After mutation: re-parse, re-evaluate, broadcast VDOM patches via SSE
- [ ] Designer handles mutation success/failure responses

### Phase 3: VS Code Yjs Sync

Implement bidirectional Yjs sync between VS Code and server.

**Files:**
- `packages/vscode-extension/src/crdt-sync.ts` (new)
- `packages/workspace/src/server.rs` - gRPC CrdtSync stream

**Tasks:**
- [ ] VS Code creates local Y.Doc per file
- [ ] Local edits → Yjs updates → server
- [ ] Server Yjs updates → VS Code document
- [ ] Infinite loop prevention (counter-based)

### Phase 4: Multi-Client Testing

Verify concurrent editing works correctly.

**Test Scenarios:**
- [ ] Two designers resize different frames simultaneously
- [ ] Two designers resize same frame (CRDT merge)
- [ ] Designer + VS Code edit same file
- [ ] VS Code in two windows edit same file
- [ ] Client reconnection after disconnect

---

## Acceptance Criteria

### Functional

- [ ] Designer A resizes frame, Designer B sees update via SSE
- [ ] VS Code A types, VS Code B sees update via Yjs sync
- [ ] VS Code types while Designer resizes - both succeed, merged correctly
- [ ] Mutation to deleted node returns error (not silent failure)
- [ ] Client reconnects and syncs to current state

### Performance

- [ ] Designer mutation roundtrip < 100ms
- [ ] VS Code keystroke appears locally < 16ms (60fps)
- [ ] VDOM patch broadcast < 50ms after mutation

### Reliability

- [ ] No data loss in any concurrent edit scenario
- [ ] Parse error in Y.Text doesn't corrupt state
- [ ] Server restart: clients reconnect and sync

---

## Security (Deferred to Separate Plan)

The following security concerns are documented but implementation is deferred:

| Issue | Severity | Notes |
|-------|----------|-------|
| No authentication | CRITICAL | Add workspace tokens before multi-user |
| Rate limiting broken | CRITICAL | Uses server PID, not client ID |
| CORS allows any origin | HIGH | Restrict to known origins |
| No CRDT update validation | HIGH | Add size limits |

---

## What This Plan Does NOT Include

To keep this architecture clean and focused:

1. **No client-side Y.Doc for designers** - They send semantic mutations
2. **No transaction batching class** - Yjs handles this internally
3. **No memory management GC** - Yjs default GC is sufficient
4. **No complex sync protocol** - Standard Yjs state vector sync

These can be added later if profiling shows they're needed.
