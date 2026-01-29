# Paperclip Architecture Constitution

**Version**: 1.0 (2026-01-29)
**Status**: Foundation Rules
**Purpose**: Non-negotiable architectural invariants that prevent correctness issues

---

## Core Principles

1. **Determinism First**: Same inputs → identical outputs (no environment/time/random dependencies)
2. **Identity is Semantic**: Semantic IDs survive refactoring, AST IDs deleted
3. **Fail Fast at Boundaries**: Error recovery only at leaf nodes, structural failures are fatal
4. **Explicit Over Implicit**: No magic, no hidden coupling, no leaked abstractions
5. **Delete Over Deprecate**: No backwards compatibility for broken designs

---

## Critical Invariants

### 1. Evaluation Determinism

```rust
/// INVARIANT: evaluate(doc, ctx) produces identical output on every invocation
/// - No HashMap iteration order dependencies (use .sorted() or BTreeMap)
/// - No time/random/environment access (std::time, rand forbidden)
/// - All ID generation deterministic (CRC32, sequential with reset)
/// - No floating-point non-associativity issues
```

**Why**: Enables caching, diffing, collaboration, reproducibility.

**Enforcement**: Lint rule + determinism tests (byte-identical outputs).

---

### 2. Identity System

```rust
/// PRIMARY: SemanticID - hierarchical path (Component → Slot → Element)
pub struct VNode {
    pub semantic_id: String,           // REQUIRED: stable across refactoring
    pub key: Option<String>,           // REQUIRED for repeat items
    // id field DELETED - use Span for source location
}

/// INVARIANT: SemanticID unique within document, NOT globally
/// - Bundle-level: (DocumentID, SemanticID) tuple
/// - Prevents accidental cross-file coupling
```

**Identity Hierarchy**:
- **Patching**: Use `semantic_id` (survives structural changes)
- **Repeat items**: Use `key` attribute (explicit or auto-generated)
- **Source location**: Use `Span` (AST position tracking)
- **Bundle lookup**: Use `DocumentID` (CRC32 of path)

**Enforcement**: Make `semantic_id` non-optional, require keys for repeat blocks.

---

### 3. Error Recovery Boundaries

```rust
/// ALLOWED: Expression/leaf-node failures
text { user.invalid.property }     // → VNode::Error ✅
style { color: bad.value }         // → Style error or VNode::Error ✅

/// FORBIDDEN: Structural boundary failures
SomeComponent()                    // Component not found → FATAL ❌
slot unknownSlot                   // Slot not found → FATAL ❌
```

**Why**: Half-constructed trees break semantic guarantees and identity.

**Enforcement**: Document in evaluator, add boundary tests.

---

### 4. Component Recursion Protection

```rust
/// FORBIDDEN: Structural recursion (no data boundary)
component AB {
    render div { AB() }  // → EvalError::RecursiveComponent
}

/// ALLOWED: Data-driven recursion (bounded by data)
component TreeNode {
    render div {
        repeat child in node.children {
            TreeNode(node=child)  // ✅ Different data each iteration
        }
    }
}
```

**Implementation**: Track `component_stack` in `EvalContext`, detect cycles.

**Future**: Reset stack across `repeat` boundaries for data-driven recursion.

---

### 5. Bundle Document Ownership

```rust
/// RULE: Bundle is the ONLY owner of Document lifetimes

// ❌ AVOID: Long-lived references
let doc: &Document = bundle.get_document(path)?;
cache.store(doc);  // Dangerous across rebuilds

// ✅ PREFER: IDs or copies
let doc_id: DocumentID = bundle.get_document_id(path)?;
let component: Component = bundle.get_component(path, "Button")?.clone();
```

**Why**: Enables incremental rebuilds, prevents stale reference bugs.

**Enforcement**: Make Bundle fields private, return views/IDs not refs.

---

### 6. Module Encapsulation

```rust
/// RULE: No public mutable state in core packages

// ❌ BEFORE
pub struct Bundle {
    pub documents: HashMap<PathBuf, Document>,  // Direct access
    pub graph: GraphManager,                     // Bypassable
}

// ✅ AFTER
pub struct Bundle {
    documents: HashMap<PathBuf, Document>,  // Private
    graph: GraphManager,                     // Private
}

impl Bundle {
    pub fn get_document(&self, path: &Path) -> Option<&Document>;
    pub fn add_document(&mut self, path: PathBuf, doc: Document);
}
```

**Why**: Validates invariants, enables optimization, prevents corruption.

**Applies to**: Bundle, EvalContext, GraphManager, Resolver.

---

## Architectural Layers

```
┌─────────────────────────────────────────────┐
│ parser: .pc text → AST (deterministic)      │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ semantics: AST → SemanticID (stable)        │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ bundle: Multi-file coordination             │
│  - GraphManager: Dependency graph           │
│  - Resolver: Name resolution                │
│  - AssetManager: Deduplication              │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ evaluator: AST → VDOM (deterministic)       │
│  - Recursion protection                     │
│  - Error recovery (leaf nodes only)         │
│  - Semantic ID generation                   │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ editor: Document lifecycle + mutations      │
│  - Optional CRDT backend                    │
│  - AST is source of truth                   │
└─────────────────────────────────────────────┘
```

**Cross-Cutting**: All layers must maintain determinism and respect identity rules.

---

## What We Are NOT

### Not OT-Compatible
**Reality**: Deterministic, serializable patch protocol (single-writer).
**Future**: CRDT layer (Yjs/Automerge) if collaboration needed.
**Language**: Don't claim OT support, it's misleading.

### Not Backwards Compatible
**Philosophy**: Delete legacy code cleanly, no deprecated fields.
**Example**: `VNode.id` field deleted entirely, not marked deprecated.
**Why**: Clean foundations > migration ease.

### Not Incrementally Parsed
**Trade-off**: Full file re-parse on change (simple, fast enough for <1000 LOC files).
**Revisit**: If files grow beyond 10KB or parse time exceeds 10ms.

---

## Enforcement Checklist

Before merging any PR that touches core packages:

- [ ] **Determinism**: Does it preserve deterministic evaluation?
- [ ] **Identity**: Does it respect semantic ID rules and uniqueness scope?
- [ ] **Boundaries**: Are error recovery boundaries respected?
- [ ] **Encapsulation**: Are fields private with validated accessors?
- [ ] **Legacy**: Did we delete cleanly (no deprecated code)?
- [ ] **Tests**: Do tests validate invariants, not just happy paths?
- [ ] **Performance**: Do benchmarks still pass (<10ms parse, <20ms eval)?

---

## Violations = Architectural Debt

If you find yourself:
- Accessing `bundle.documents` directly → Use accessor API
- Returning `&Document` with long lifetime → Return ID or copy
- Catching errors at component boundaries → Let them propagate (fatal)
- Adding `time` or `rand` dependencies → Breaks determinism
- Marking fields `deprecated` → Delete them instead
- Iterating HashMap without sorting → Non-deterministic order

**Stop. These violate the constitution.**

---

## Amendment Process

This is version 1.0. To amend:

1. Identify the invariant that needs changing
2. Document why it's blocking progress
3. Propose new invariant that solves the problem
4. Validate with test suite (no regressions)
5. Update this document with version bump
6. Communicate breaking changes clearly

**Last Updated**: 2026-01-29 (Foundation Solid effort)
