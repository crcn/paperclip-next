# Paperclip Identity System Guide

## Overview

Paperclip uses multiple ID types for different purposes. This guide explains when to use each type and how they work together.

## ID Types

### 1. SemanticID (Primary Identity)

**Purpose**: Stable node identity across refactoring

**Scope**: Per-VDOM-tree (evaluation output)

**Format**: Hierarchical path with segments
```
Component{"Button-0"}::Element{"button"}[ast-id-123]
```

**When to use**:
- ✅ **Patching**: Primary identifier for diff/patch operations
- ✅ **Hot reload**: Matching nodes across evaluations
- ✅ **Developer tools**: Identifying elements in preview

**Properties**:
- **Stable**: Survives component renames, element moves
- **Unique**: Within a VDOM tree (not per-file!)
- **Hierarchical**: Reflects component composition

**Example**:
```rust
use paperclip_semantics::SemanticID;

// Every VNode has a semantic_id
let node = VNode::element("div", semantic_id);

// Use for diffing
if old_semantic_id == new_semantic_id {
    // Same element, update attributes
}
```

### 2. Key (Repeat Block Identity)

**Purpose**: Stable identity for items in repeat blocks

**Scope**: Within parent element, per repeat block

**Format**: String (explicit or auto-generated)

**When to use**:
- ✅ **Repeat blocks**: ALWAYS for list items
- ✅ **Reorderable lists**: Essential for correct diffing
- ✅ **Dynamic content**: Items that can be added/removed

**Anti-patterns**:
- ❌ Using array indices as keys (breaks on reorder)
- ❌ No keys (causes full re-render on insert)

**Example**:
```paperclip
repeat users {
  div key={user.id} {  // ✅ Explicit, stable key
    text user.name
  }
}

// Auto-generated keys get warnings:
repeat items {
  div {  // ⚠️ Will get key="item-0", "item-1", etc.
    text item
  }
}
```

### 3. DocumentID

**Purpose**: Identify source files in the bundle

**Scope**: Bundle-wide (unique per file)

**Format**: CRC32 hash of file path
```
"a4b3c2d1"
```

**When to use**:
- ✅ **Bundle queries**: Looking up documents
- ✅ **Dependency tracking**: Source of imports
- ✅ **Asset management**: Which file owns which asset

**Not for**:
- ❌ Element identity (use SemanticID)
- ❌ Patch routing (use SemanticID)

**Example**:
```rust
let doc_id = bundle.get_document_id(&path)?;
let doc = bundle.get_document(&path)?;
```

### 4. AST IDs (Deprecated)

**Status**: ⚠️ **DEPRECATED** - Being removed

**Previous use**: AST node identification

**Migration**:
- For **identity**: Use SemanticID
- For **source location**: Use Span

**Why deprecated**:
- Redundant with SemanticID
- Confused identity vs location
- Dead code in VNode (id field deleted)

## Decision Tree

```
Need to identify a node?
│
├─ For patching/diffing?
│  └─ Use SemanticID
│
├─ Item in a repeat block?
│  └─ Use key attribute (with SemanticID)
│
├─ Looking up a document?
│  └─ Use DocumentID or Path
│
└─ Source code location?
   └─ Use Span (not ID)
```

## Common Patterns

### Pattern 1: Diffing VDOM Trees

```rust
use paperclip_evaluator::diff_vdocument;

let patches = diff_vdocument(&old_vdom, &new_vdom);
// Patches use SemanticID for routing
```

**Key points**:
- SemanticID is primary matching criterion
- Keys used within repeat blocks
- DocumentID not needed (patches are per-tree)

### Pattern 2: Hot Reload Across Files

```paperclip
// file_a.pc
component Button { ... }

// file_b.pc
import { Button } from "./file_a.pc"
component Card {
  render Button()  // ← SemanticID scoped to Card's VDOM
}
```

When Button changes:
1. Re-evaluate Card's VDOM
2. Diff using SemanticIDs
3. Patch Button instances in Card's tree

**Critical**: SemanticID uniqueness is per-VDOM-tree (Card's output), not per-source-file (file_a.pc).

### Pattern 3: Repeat Block with Keys

```paperclip
component UserList {
  render div {
    repeat user in users {
      div key={user.id} {  // ✅ Explicit key
        text user.name
      }
    }
  }
}
```

Generated VNode structure:
```rust
Element {
  tag: "div",
  semantic_id: SemanticID::new([
    Component { name: "UserList", key: None },
    Element { tag: "div", ast_id: "..." },
    RepeatItem { repeat_id: "...", key: "user-123" }  // ← Key here
  ]),
  key: Some("user-123"),  // ← And here
}
```

**Both SemanticID and key include the key value** for stable diffing.

## Uniqueness Scopes

### SemanticID Uniqueness

**Scope**: Per-VDOM-tree (evaluation output)

```rust
// ✅ OK - Different VDOM trees
let vdom_a = evaluate(component_a);  // Has SemanticID "Button::div[1]"
let vdom_b = evaluate(component_b);  // Can also have "Button::div[1]"

// ❌ ERROR - Same VDOM tree
component Card {
  render div {
    Button()  // SemanticID: "Button[btn-1]::..."
    Button()  // SemanticID: "Button[btn-2]::..." (different!)
  }
}
```

**Validator checks**: Duplicate SemanticIDs within a tree are ERROR-level warnings.

### Key Uniqueness

**Scope**: Within parent element, per repeat block

```rust
// ✅ OK - Different repeat blocks
repeat users {
  div key="1" { }  // OK in users repeat
}
repeat posts {
  div key="1" { }  // OK in posts repeat (different block)
}

// ❌ ERROR - Same repeat block
repeat users {
  div key="123" { }  // First user
  div key="123" { }  // ❌ DUPLICATE in same repeat
}
```

**Validator checks**: Duplicate keys within a repeat block are ERROR-level warnings.

### DocumentID Uniqueness

**Scope**: Bundle-wide

One DocumentID per file path. Deterministic (CRC32 of path).

## Best Practices

### DO

✅ **Use SemanticID for all patching**
```rust
if old_node.semantic_id == new_node.semantic_id {
    // Update node
}
```

✅ **Always use explicit keys in repeat blocks**
```paperclip
repeat items {
  div key={item.id} { ... }
}
```

✅ **Keep SemanticIDs stable across refactoring**
- Rename component → SemanticID unchanged (structure preserved)
- Move element → SemanticID may change (different path)

✅ **Use DocumentID for bundle queries only**
```rust
let doc_id = bundle.get_document_id(&path)?;
```

### DON'T

❌ **Don't use DocumentID for element identity**
```rust
// WRONG - DocumentID is for files, not elements
let element_id = doc_id + "/" + semantic_id;
```

❌ **Don't skip keys in repeat blocks**
```paperclip
// WRONG - Will get auto-generated keys with warnings
repeat users {
  div { text user.name }
}
```

❌ **Don't hold long-lived &Document refs**
```rust
// WRONG - Ties lifetime to Bundle
let doc: &Document = bundle.get_document(path)?;
cache.store(doc);

// RIGHT - Copy what you need
let component = bundle.find_component("Button", path)?.clone();
```

❌ **Don't use AST IDs** (deprecated)
```rust
// WRONG - AST IDs are deprecated
let node = VNode { id: Some(ast_id), ... };

// RIGHT - Use SemanticID
let node = VNode { semantic_id, ... };
```

## Testing

### Semantic ID Stability

```rust
#[test]
fn test_semantic_id_stable_across_evaluations() {
    let doc = parse(source)?;

    let vdom1 = evaluate(&doc)?;
    let vdom2 = evaluate(&doc)?;

    // Same IDs across evaluations
    assert_eq!(
        collect_semantic_ids(&vdom1),
        collect_semantic_ids(&vdom2)
    );
}
```

### Key Uniqueness

```rust
#[test]
fn test_duplicate_keys_detected() {
    let vdom = evaluate_with_duplicate_keys()?;

    let warnings = validator.validate(&vdom);

    assert!(warnings.iter().any(|w|
        w.message.contains("Duplicate key")
    ));
}
```

## Migration Guide

### From AST IDs to SemanticIDs

**Before** (deprecated):
```rust
let node = VNode::Element {
    tag: "div".into(),
    id: Some(ast_id),  // DEPRECATED
    ...
};
```

**After**:
```rust
let node = VNode::Element {
    tag: "div".into(),
    semantic_id,  // Use SemanticID
    ...
};
```

For source locations, use `Span`:
```rust
let error = VNode::Error {
    message: "Bad value".into(),
    span: Some(expr.span),  // Source location
    semantic_id,            // Identity
};
```

## FAQ

**Q: When should I use DocumentID vs SemanticID?**

A: DocumentID identifies **source files** in the bundle. SemanticID identifies **elements** in the VDOM tree. Use DocumentID for file-level queries, SemanticID for element operations.

**Q: Do I need both key and SemanticID for repeat items?**

A: Yes! The key is included in the SemanticID for stable matching. Both are used during diffing.

**Q: Why are SemanticIDs scoped per-VDOM-tree, not per-file?**

A: Because components can be imported and used in multiple files. The VDOM tree (evaluation output) is the unit that gets diffed and patched, not individual source files.

**Q: What happens if I don't provide keys in repeat blocks?**

A: The system auto-generates keys like "item-0", "item-1", etc. You'll get dev-mode warnings, and list reordering won't work correctly.

**Q: Can I cache &Document references?**

A: No! Bundle owns Document lifetimes. Cache DocumentIDs or clone specific data instead.

## See Also

- `packages/semantics/src/identity.rs` - SemanticID implementation
- `packages/evaluator/src/vdom.rs` - VNode structure
- `packages/evaluator/src/validator.rs` - Uniqueness checks
- `packages/evaluator/src/vdom_differ.rs` - Diffing algorithm
