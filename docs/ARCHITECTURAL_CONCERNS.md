# Architectural Concerns - OpenAI Feedback Analysis

## Overview

This document addresses critical architectural concerns raised about the current implementation. These issues must be resolved before scaling to production use with:
- Large components
- Concurrent edits
- Live/native component targets
- AI-assisted editing

---

## Issue 1: VNode Identity & Stable Keys

### Current Problem

**VNode definition does NOT guarantee**:
- Stable sibling identity
- Stable ordering under repeat
- Explicit keys

### Why It Matters

Operations that corrupt patches:
```rust
// ❌ Inserting at top of list
repeat items {
  div { text item }  // All paths shift
}

// ❌ Toggling if above repeated block
if condition {
  div { text "Header" }
}
repeat items {  // Paths change based on condition
  div { text item }
}

// ❌ Changing repeat source length
// Paths like [2, 0] become invalid when item 2 is removed
```

### Current State

```rust
pub struct VNode {
    pub id: String,           // AST node ID (e.g., "80f4925f-5")
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<VNode>,
}
```

**Missing**: No `key` field for stable identity across structural changes.

### Proposed Solution

#### Option A: Explicit Keys in Repeat (React-style)

```paperclip
repeat users as user {
  div key={user.id} {
    text user.name
  }
}
```

**VNode structure**:
```rust
pub struct VNode {
    pub id: String,
    pub tag: String,
    pub key: Option<String>,  // NEW: Explicit key for stable identity
    pub attributes: HashMap<String, String>,
    pub children: Vec<VNode>,
}
```

**Patching logic**:
```rust
fn diff_keyed_children(old: &[VNode], new: &[VNode]) -> Vec<VDocPatch> {
    // Build maps: key -> (index, node)
    let old_map: HashMap<String, (usize, &VNode)> = old.iter()
        .enumerate()
        .filter_map(|(i, n)| n.key.as_ref().map(|k| (k.clone(), (i, n))))
        .collect();

    let new_map: HashMap<String, (usize, &VNode)> = new.iter()
        .enumerate()
        .filter_map(|(i, n)| n.key.as_ref().map(|k| (k.clone(), (i, n))))
        .collect();

    // Generate patches based on key matching
    // - Nodes with same key = UPDATE
    // - Nodes without key in new = REMOVE
    // - Nodes without key in old = INSERT
}
```

#### Option B: Implicit Stable IDs from AST

```rust
// Derive stable path from AST structure, not runtime position
pub struct VNode {
    pub ast_path: Vec<ASTPathSegment>,  // e.g., [Component("Button"), Element(2), RepeatItem]
    pub runtime_index: Option<usize>,    // Position within repeat
    // ...
}

pub enum ASTPathSegment {
    Component(String),
    Element(usize),      // Static position in AST
    RepeatItem,          // Marks dynamic position
    ConditionalThen,
    ConditionalElse,
}
```

**Example**:
```paperclip
component Card {
  render div {           // [Component("Card"), Element(0)]
    if showHeader {      // [Component("Card"), Element(0), ConditionalThen]
      div { ... }
    }
    repeat items {       // [Component("Card"), Element(0), RepeatItem]
      div { ... }        // [Component("Card"), Element(0), RepeatItem, Element(0)]
    }
  }
}
```

#### Option C: Reconciliation Layer

Add a reconciliation layer that rewrites paths based on structural changes:

```rust
pub struct PathMapper {
    old_structure: StructureMap,
    new_structure: StructureMap,
}

impl PathMapper {
    pub fn rewrite_path(&self, old_path: &[usize]) -> Result<Vec<usize>, PathMappingError> {
        // Use semantic markers to map old path to new path
        // Even when indices shift
    }
}
```

### Recommendation

**Phase 1**: Implement **Option A (Explicit Keys)** immediately
- Required for repeat blocks
- React-proven approach
- Straightforward migration path

**Phase 2**: Add **Option B (AST Path)** for non-repeat elements
- Stable identity for static structure
- Enables better diffing

---

## Issue 2: Bundle as "God Object in Waiting"

### Current Coupling

```rust
pub struct Bundle {
    documents: HashMap<PathBuf, Document>,           // Parsed ASTs
    import_graph: HashMap<PathBuf, Vec<PathBuf>>,   // Dependencies
    import_aliases: HashMap<(PathBuf, String), PathBuf>,  // Alias resolution
    assets: HashMap<String, (AssetReference, HashSet<PathBuf>)>,  // Asset dedup
    cycle_detection: Vec<PathBuf>,
}
```

**Responsibilities**:
1. Semantic resolution (imports, aliases)
2. Dependency graph (imports, cycles)
3. Asset management (deduplication, tracking)
4. Evaluation input (documents storage)

### Problem

Adding these features will create conflicts:
- **Incremental parsing**: Need to update documents without rebuilding entire graph
- **Partial recompilation**: Need to eval single component without full bundle
- **Per-component compilation**: Need to resolve imports without loading all files
- **Evaluation caching**: Need to memoize by component, not by bundle

### Proposed Separation

#### GraphManager: Dependency Graph Only

```rust
pub struct GraphManager {
    edges: HashMap<PathBuf, Vec<PathBuf>>,  // file -> dependencies
    reverse_edges: HashMap<PathBuf, Vec<PathBuf>>,  // file -> dependents
}

impl GraphManager {
    pub fn add_dependency(&mut self, from: &Path, to: &Path);
    pub fn get_dependencies(&self, file: &Path) -> &[PathBuf];
    pub fn get_dependents(&self, file: &Path) -> &[PathBuf];
    pub fn has_cycle(&self, file: &Path) -> Option<Vec<PathBuf>>;
    pub fn invalidation_set(&self, changed: &Path) -> Vec<PathBuf>;
}
```

#### Resolver: Name Lookup

```rust
pub struct Resolver {
    import_aliases: HashMap<(PathBuf, String), PathBuf>,
    style_registry: HashMap<PathBuf, Vec<String>>,  // file -> style names
    component_registry: HashMap<PathBuf, Vec<String>>,  // file -> component names
}

impl Resolver {
    pub fn resolve_import(&self, from: &Path, alias: &str) -> Option<PathBuf>;
    pub fn resolve_style(&self, file: &Path, reference: &str) -> Option<(PathBuf, String)>;
    pub fn resolve_component(&self, file: &Path, reference: &str) -> Option<(PathBuf, String)>;
}
```

#### EvaluatorCache: Memoized Evaluation

```rust
pub struct EvaluatorCache {
    vdom_cache: HashMap<CacheKey, VirtualDomDocument>,
    css_cache: HashMap<CacheKey, VirtualCssDocument>,
}

#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    file: PathBuf,
    version: u64,
    dependencies_hash: u64,  // Hash of all dependency versions
}

impl EvaluatorCache {
    pub fn get_or_evaluate<F>(&mut self, key: CacheKey, eval_fn: F) -> VirtualDomDocument
    where F: FnOnce() -> VirtualDomDocument;

    pub fn invalidate(&mut self, file: &Path);
}
```

#### Refactored Bundle

```rust
pub struct Bundle {
    documents: HashMap<PathBuf, Document>,
    graph: GraphManager,     // Separated responsibility
    resolver: Resolver,      // Separated responsibility
    assets: AssetManager,    // Separated responsibility
}

pub struct AssetManager {
    assets: HashMap<String, (AssetReference, HashSet<PathBuf>)>,
}
```

### Migration Path

1. **Phase 1**: Extract GraphManager (no API changes)
2. **Phase 2**: Extract Resolver (minimal API changes)
3. **Phase 3**: Add EvaluatorCache (new feature, backwards compatible)
4. **Phase 4**: Extract AssetManager (cleanup)

---

## Issue 3: Patch Protocol Capabilities ✅ CLARIFIED

### Current State - Honest Assessment

**What we have**:
- ✅ Deterministic, serializable patch protocol (single-writer)
- ✅ Patches are reproducible and can be transmitted/stored
- ✅ Suitable for HMR (hot module reload) and single-user editing
- ✅ Protobuf format for efficient serialization

**What we DON'T have** (and don't claim to have):
- ❌ Operational Transform rules for concurrent writes
- ❌ Conflict resolution semantics
- ❌ Intent preservation across concurrent edits

### Why It Matters

```rust
// User A: Insert at beginning
patches_a = [Insert { path: [0], node: div_a }]

// User B: Insert at beginning (concurrent)
patches_b = [Insert { path: [0], node: div_b }]

// Without OT transform:
apply(patches_a, doc)  // div_a at [0]
apply(patches_b, doc)  // ERROR: div_b overwrites div_a!

// With OT transform:
patches_b_transformed = transform(patches_b, patches_a)
// patches_b_transformed = [Insert { path: [1], node: div_b }]
apply(patches_a, doc)
apply(patches_b_transformed, doc)  // Both inserted correctly
```

### OT Transform Rules Needed

```rust
pub trait OperationalTransform {
    fn transform(&self, other: &Self) -> (Self, Self);
}

impl OperationalTransform for VDocPatch {
    fn transform(&self, other: &Self) -> (Self, Self) {
        match (self, other) {
            // Insert vs Insert: Adjust paths
            (Insert { path: p1, .. }, Insert { path: p2, .. }) => {
                if p1 == p2 {
                    // Same position: later insert gets +1
                    (self.clone(), other.with_path_offset(p2, 1))
                } else {
                    (self.clone(), other.clone())
                }
            }

            // Delete vs Update: Update becomes no-op if node deleted
            (Delete { path: p1 }, Update { path: p2, .. }) => {
                if p1 == p2 {
                    (self.clone(), NoOp)  // Update target deleted
                } else {
                    (self.clone(), other.clone())
                }
            }

            // ... more rules for all combinations
        }
    }
}
```

### Required for Collaboration

```rust
pub struct CollaborationLayer {
    local_version: u64,
    server_version: u64,
    pending_patches: Vec<VDocPatch>,
}

impl CollaborationLayer {
    pub fn apply_remote_patch(&mut self, remote: VDocPatch) -> VDocPatch {
        // Transform remote patch against all pending local patches
        let mut transformed = remote;
        for local in &self.pending_patches {
            let (local_transformed, remote_transformed) = local.transform(&transformed);
            transformed = remote_transformed;
        }
        transformed
    }
}
```

### Resolution ✅ COMPLETE

**Updated documentation language** to accurately describe what exists:

> "Deterministic, serializable patch protocol (single-writer). Patches are reproducible and can be transmitted/stored, but do not include operational transform rules for concurrent writes. Suitable for HMR and single-user editing; collaboration requires CRDT layer (planned)."

**Benefits of honest framing**:
- ✅ Preserves credibility (doesn't oversell)
- ✅ Accurately describes what IS there (deterministic, serializable)
- ✅ Leaves room for semantic-OT or CRDT layer later
- ✅ Sets correct expectations for collaboration timeline

**Future Options for Collaboration**:

**Option 1**: Use CRDT layer above patches (recommended)
- Simpler to implement than full OT
- Automerge or Yjs can provide this
- Patches become CRDT operations

**Option 2**: Implement full OT
- Required for true collaborative editing
- Complex but well-understood (see Etherpad, Google Docs)

**Current Status**: Single-writer patch protocol documented honestly. Collaboration deferred to Phase 2+.

---

## Issue 4: Path-Based Patching Needs Stable Identity

### Current Problem

```rust
// Patch references node at path [2, 1]
Update {
    path: vec![2, 1],
    new_attributes: ...
}

// But if structure changes:
if condition {  // This gets added
  div { ... }
}
// [2, 1] now points to DIFFERENT node!
```

### Solution: Semantic Identity

```rust
pub enum NodeIdentity {
    ASTNode(String),                    // AST node ID: "80f4925f-5"
    Keyed(String),                      // Explicit key: "user-123"
    Semantic(Vec<SemanticSegment>),     // Semantic path
}

pub enum SemanticSegment {
    Component { name: String, instance_key: Option<String> },
    Slot { name: String },
    RepeatItem { key: String },
    ConditionalBranch { condition_id: String, branch: Branch },
    Element { tag: String, ast_id: String },
}

pub struct StableNode {
    pub identity: NodeIdentity,
    pub current_path: Vec<usize>,  // Runtime path (can change)
    pub node: VNode,
}
```

### Stable Patch Format

```rust
pub enum StablePatch {
    UpdateNode {
        identity: NodeIdentity,  // Stable identity
        attributes: HashMap<String, String>,
    },
    InsertBefore {
        sibling_identity: NodeIdentity,
        new_node: VNode,
    },
    RemoveNode {
        identity: NodeIdentity,
    },
}
```

**Benefits**:
- Patches survive structure changes
- Visual editor can target specific nodes
- AI edits can reference semantic locations

---

## Issue 5: Repeat/If Semantics Must Be Locked

### Current Ambiguity

```paperclip
repeat items as item {
  div { text item.name }
}
```

**Unclear**:
- What happens if items is null?
- What if items is not iterable?
- Are keys required or optional?
- How are items identified across updates?

### Required Specification

```rust
// In SPEC.md or similar:

## Repeat Semantics

1. **Source types**:
   - Array: iterate 0..length
   - Null/undefined: render nothing (0 iterations)
   - Non-iterable: error node

2. **Item identity**:
   - If `key` attribute present: use key value
   - Otherwise: use index (UNSTABLE, warn in dev mode)

3. **Empty state**:
   - Optional `empty` branch for 0 items

4. **Ordering**:
   - Preserves source array order
   - Reordering source triggers move patches

## Conditional Semantics

1. **Truthiness**:
   - null, false, 0, "" → false branch
   - Everything else → true branch

2. **Structure**:
   - `if cond { ... }` - optional then branch
   - `if cond { ... } else { ... }` - both branches

3. **Diffing**:
   - Branch switch = remove old branch + insert new branch
   - Within branch = normal diffing
```

---

## Issue 6: Error Locality & Partial Evaluation

### Current Problem

```rust
// One bad expression nukes entire preview
component Card {
  render div {
    text user.invalid.property.chain  // ERROR: crashes evaluation
    button { ... }  // Never rendered
  }
}
```

### Required Solution

```rust
pub enum VNode {
    Element {
        id: String,
        tag: String,
        attributes: HashMap<String, String>,
        children: Vec<VNode>,
    },
    Text {
        id: String,
        content: String,
    },
    Error {
        id: String,
        error: EvalError,
        fallback: Option<Box<VNode>>,
        source_location: Span,
    },
}
```

### Partial Evaluation

```rust
impl Evaluator {
    pub fn evaluate_with_error_recovery(&mut self, doc: &Document) -> VirtualDomDocument {
        let mut nodes = Vec::new();

        for component in &doc.components {
            match self.evaluate_component(component) {
                Ok(node) => nodes.push(node),
                Err(error) => {
                    // Create error node instead of failing
                    nodes.push(VNode::Error {
                        id: component.span.id.clone(),
                        error,
                        fallback: None,
                        source_location: component.span.clone(),
                    });
                }
            }
        }

        VirtualDomDocument { nodes }
    }
}
```

### Visual Error Rendering

```rust
// In preview/dev mode:
<div class="error-boundary" data-source-location="5:10-5:35">
  <div class="error-message">
    EvalError: Cannot read property 'chain' of undefined
  </div>
  <div class="error-source">
    <code>user.invalid.property.chain</code>
  </div>
</div>
```

---

## Issue 7: Semantic Identity Layer Missing

### Problem Statement

Current identity:
- AST node IDs: "80f4925f-5"
- VDOM paths: [2, 1, 0]

**Neither represents**: "the button in the footer slot of Card instance X"

### Semantic Identity Proposal

```rust
pub struct SemanticID {
    segments: Vec<SemanticSegment>,
}

pub enum SemanticSegment {
    // Component instance
    Component {
        name: String,              // "Card"
        instance_key: Option<String>,  // User-provided or auto
    },

    // Slot reference
    Slot {
        name: String,              // "footer"
    },

    // Element within component
    Element {
        role: Option<String>,      // "button", "icon", etc.
        ast_id: String,            // Fallback to AST position
    },

    // Repeat item
    RepeatItem {
        key: String,               // From key attribute or index
    },
}

impl SemanticID {
    pub fn to_selector(&self) -> String {
        // Card[X]::footer::button
        self.segments.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
}
```

### Usage in Patches

```rust
pub struct StablePatch {
    target: SemanticID,       // Survives refactoring
    operation: PatchOperation,
}

// Example:
StablePatch {
    target: SemanticID {
        segments: vec![
            Component { name: "Card", instance_key: Some("card-1") },
            Slot { name: "footer" },
            Element { role: Some("button"), ast_id: "abc-5" },
        ]
    },
    operation: UpdateAttributes { ... },
}
```

---

## Summary: Must-Fix Before Scaling

### Priority 1 (Critical)

1. **[Issue 1]** Add explicit keys to repeat blocks
2. **[Issue 4]** Implement semantic identity for stable patches
3. **[Issue 6]** Add error nodes and partial evaluation

### Priority 2 (Important)

4. **[Issue 5]** Lock down repeat/if semantics in spec
5. **[Issue 2]** Extract GraphManager from Bundle
6. **[Issue 7]** Design semantic identity model

### Priority 3 (Plan For)

7. **[Issue 3]** Remove OT claims OR implement CRDT layer
8. **[Issue 2]** Complete Bundle separation

---

## Next Steps

Choose one of:

1. **Design semantic identity model** - Foundation for many fixes
2. **Hard-spec repeat/if semantics** - Prevents future breaking changes
3. **Implement explicit keys** - Immediate stability improvement

Which should we tackle first?
