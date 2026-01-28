# Project Status

## ✅ Recently Completed

### 1. Asset Deduplication
- Changed Bundle.assets from Vec to HashMap
- Tracks unique assets with source file mapping
- New API: `unique_assets()`, `asset_users()`, `assets_for_file()`
- **Tests**: 146/146 passing

### 2. FileState Optimization
- Removed redundant `ast` and `assets` fields from FileState
- Added WorkspaceState methods: `get_ast()`, `get_file_assets()`
- Clear separation: Bundle = input, FileState = output

### 3. Async Support
- Added `build_dependencies_async()` behind "async" feature flag
- Uses tokio::task::spawn_blocking for compatibility
- Backwards compatible with sync API

### 4. Namespacing Architecture
- Sequential IDs with CRC32-based document IDs
- Public styles properly namespaced: `._ButtonStyle-{docId}-{seqNum}`
- CSS variables for style extends
- All identifiers globally unique
- **Tests**: All passing, proper namespacing verified

---

## 🔴 Critical Issues Identified (OpenAI Feedback)

### Issue 1: VNode Identity - No Stable Keys
**Problem**: Repeat blocks and conditionals cause patch corruption

**Impact**:
- Inserting at top of list breaks all paths
- Toggling conditions changes paths
- Changing repeat length invalidates patches

**Required**:
```paperclip
repeat users as user key={user.id} {
  div { text user.name }
}
```

**Status**: ⚠️ NOT IMPLEMENTED

---

### Issue 2: Bundle Becoming "God Object"
**Problem**: Too many responsibilities coupled together

**Current Bundle**:
- Semantic resolution
- Dependency graph
- Asset management
- Evaluation input

**Will break when adding**:
- Incremental parsing
- Partial recompilation
- Per-component compilation

**Required**: Separate into GraphManager, Resolver, EvaluatorCache

**Status**: ⚠️ NEEDS REFACTORING

---

### Issue 3: OT Compatibility Claims
**Problem**: Claims OT-compatible patches, but no transform rules

**Have**:
- ✅ Serializable patches
- ✅ Deterministic diffing

**Missing**:
- ❌ Transform rules for concurrent operations
- ❌ Conflict resolution
- ❌ Intent preservation

**Required**: Either implement full OT or remove claims

**Status**: ⚠️ FALSE CLAIM

---

### Issue 4: Path-Based Patches Not Stable
**Problem**: Patches use `path: [2, 1]` which breaks on structure changes

**Example**:
```rust
Update { path: [2, 1], ... }
// If conditional added above, [2, 1] now points to WRONG node
```

**Required**: Semantic identity

```rust
pub struct StableNode {
    identity: NodeIdentity,  // "Card::footer::button"
    current_path: Vec<usize>,  // Can change
    node: VNode,
}
```

**Status**: ⚠️ NOT IMPLEMENTED

---

### Issue 5: Repeat/If Semantics Undefined
**Problem**: Behavior unclear for edge cases

**Unclear**:
- What if items is null?
- What if items is not iterable?
- Are keys required?
- How are items identified?

**Required**: Hard spec in documentation

**Status**: ⚠️ NEEDS SPECIFICATION

---

### Issue 6: Error Handling Not Localized
**Problem**: One bad expression crashes entire preview

**Current**:
```rust
// This crashes everything:
text user.invalid.property.chain
```

**Required**:
```rust
pub enum VNode {
    Element { ... },
    Text { ... },
    Error {  // NEW
        error: EvalError,
        fallback: Option<Box<VNode>>,
        source_location: Span,
    },
}
```

**Status**: ⚠️ NOT IMPLEMENTED

---

### Issue 7: Semantic Identity Layer Missing
**Problem**: Can't target "the button in footer slot of Card X"

**Current**:
- AST IDs: "80f4925f-5" (too low-level)
- VDOM paths: [2, 1] (too fragile)

**Required**:
```rust
SemanticID {
  segments: [
    Component("Card", instance="card-1"),
    Slot("footer"),
    Element(role="button"),
  ]
}
```

**Status**: ⚠️ NOT IMPLEMENTED

---

## Priority Roadmap

### 🔥 P0 - Critical (Blocks Scaling)

1. **Explicit keys in repeat**
   - Add `key` attribute support
   - Implement keyed diffing algorithm
   - Warn when keys missing in dev mode

2. **Semantic identity for patches**
   - Design identity model
   - Implement StableNode
   - Update patch format

3. **Error nodes in VDOM**
   - Add Error variant to VNode
   - Implement partial evaluation
   - Visual error rendering

### ⚠️ P1 - Important (Prevents Technical Debt)

4. **Lock repeat/if semantics**
   - Document behavior in spec
   - Add edge case tests
   - Enforce semantics in evaluator

5. **Extract GraphManager**
   - Separate dependency graph
   - Extract resolver
   - Add EvaluatorCache

6. **Design semantic identity**
   - Full model for targeting
   - Selector syntax
   - Survives refactoring

### 📋 P2 - Nice to Have

7. **Remove OT claims OR implement CRDT**
   - Document current limitations
   - Plan for true collaboration
   - Implement transform rules if needed

8. **Complete Bundle separation**
   - AssetManager extracted
   - Clean interfaces
   - Testable in isolation

---

## Testing Status

**Current**: 146 tests passing
- ✅ Parser: 31 tests
- ✅ Evaluator: 84 tests
- ✅ Workspace: 19 tests
- ✅ Bundle: 3 tests
- ✅ Integration: 9 tests

**Coverage**:
- ✅ Namespacing verified
- ✅ Asset deduplication tested
- ✅ CSS variables working
- ⚠️ No tests for repeat keys (doesn't exist yet)
- ⚠️ No tests for error nodes (doesn't exist yet)
- ⚠️ No tests for semantic identity (doesn't exist yet)

---

## Documentation

### Created
- ✅ `IMPLEMENTATION_SUMMARY.md` - Asset dedup, FileState, Async
- ✅ `NAMESPACING_IMPLEMENTATION.md` - Full namespacing architecture
- ✅ `NAMESPACING_EXAMPLE.md` - End-to-end example
- ✅ `ARCHITECTURAL_CONCERNS.md` - Deep analysis of issues
- ✅ `BUNDLE_API_EXAMPLES.md` - API usage examples
- ✅ `BUNDLE_QUICK_REFERENCE.md` - Quick reference

### Needed
- ⚠️ `SPEC.md` - Formal specification of repeat/if semantics
- ⚠️ `SEMANTIC_IDENTITY.md` - Identity model design
- ⚠️ `STABLE_PATCHES.md` - Patch format for stability

---

## Immediate Next Steps

Choose priority:

### Option A: Foundation First (Recommended)
1. Design semantic identity model
2. Implement in VNode structure
3. Update patch format
4. **Then** add explicit keys
5. **Then** add error nodes

**Rationale**: Identity is foundation for keys and stable patches

### Option B: Quick Wins First
1. Add explicit keys to repeat
2. Add error nodes
3. Lock down repeat/if spec
4. **Then** tackle semantic identity

**Rationale**: Immediate stability improvements

### Option C: Documentation First
1. Write formal SPEC.md
2. Design semantic identity
3. Design stable patch format
4. **Then** implement in order

**Rationale**: Prevent rework by planning first

---

## Risk Assessment

### Current State
- ✅ Good for demos
- ✅ Good for simple components
- ⚠️ NOT ready for production
- ⚠️ NOT ready for large components
- ⚠️ NOT ready for concurrent edits
- ⚠️ NOT ready for live/native components

### Blockers to Production
1. No stable identity → refactors break patches
2. No repeat keys → list operations corrupt
3. No error isolation → one error crashes all
4. Bundle coupling → hard to optimize
5. No OT/CRDT → can't collaborate

### Timeline Estimate
- **P0 (Critical)**: 2-3 weeks
- **P1 (Important)**: 2-3 weeks
- **P2 (Nice to have)**: 1-2 weeks

**Total**: 5-8 weeks to production-ready

---

## Questions for Decision

1. **Identity Model**: What's the right granularity?
   - Component-level only?
   - Element-level with roles?
   - Full semantic paths?

2. **Keys Required**: Should repeat keys be mandatory?
   - Always required (strict, safe)
   - Optional with warnings (flexible, risky)
   - Required in production mode only?

3. **Error Rendering**: How aggressive should recovery be?
   - Show error boundary always
   - Try to render siblings
   - Degrade gracefully to parent

4. **Bundle Refactor**: Big bang or gradual?
   - Extract all at once (risky, clean)
   - Extract one piece at a time (safe, messy)

5. **OT/CRDT**: Which path?
   - Remove claims, defer collaboration
   - Implement OT (complex, powerful)
   - Use CRDT library (simpler, less control)

---

## Conclusion

✅ **Recent work is solid**: Asset deduplication, namespacing, async support all well-implemented and tested.

⚠️ **Architecture needs attention**: The concerns raised by OpenAI are valid and critical for production use.

🎯 **Recommended path**: Start with semantic identity design (Option A), as it's the foundation for most other fixes.

Let me know which direction you'd like to take!
