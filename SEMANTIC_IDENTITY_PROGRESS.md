# Semantic Identity Implementation Progress

## Status: Phase 2 Complete ✅

---

## Completed Phases

### ✅ Phase 1: Core Types (Complete)

**File**: `packages/evaluator/src/semantic_identity.rs`

**Implemented**:
- `SemanticID` struct with `Vec<SemanticSegment>`
- `SemanticSegment` enum with 5 variants:
  - `Component` (with optional key)
  - `Slot` (with variant: Default | Inserted)
  - `Element` (with tag, role, ast_id)
  - `RepeatItem` (with repeat_id, key)
  - `ConditionalBranch` (with condition_id, branch)
- `Branch` enum (Then | Else)
- `SlotVariant` enum (Default | Inserted)
- Selector syntax: `Card{"key"}::footer[inserted]::button[abc-5]`
- Helper methods: `parent()`, `append()`, `is_descendant_of()`
- **10 tests passing**

**Examples**:
```rust
// Simple component
SemanticID("Button::button[80f4925f-2]")

// With slot
SemanticID("Card::footer[inserted]::button[xyz-9]")

// With repeat
SemanticID("UserList::repeat[abc-3]{\"user-123\"}::div[xyz-5]")

// With conditional
SemanticID("Dashboard::if[aaa-7].then::div[aaa-9]")
```

---

### ✅ Phase 2: VNode Structure Update (Complete)

**File**: `packages/evaluator/src/vdom.rs`

**Changes**:
```rust
pub enum VNode {
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        styles: HashMap<String, String>,
        children: Vec<VNode>,

        // Legacy (backwards compat)
        id: Option<String>,

        // NEW: Semantic identity
        semantic_id: Option<SemanticID>,

        // NEW: Explicit key for repeat items
        key: Option<String>,
    },
    Text { content: String },
    Comment { content: String },
}
```

**New helper methods**:
- `with_semantic_id(semantic_id: SemanticID)`
- `with_key(key: impl Into<String>)`

**Migration**:
- All fields optional (backwards compatible)
- Existing code continues to work
- New code can add semantic_id + key incrementally

**Tests**: All 94 evaluator tests passing ✅

---

## Rules Locked Down

### Core Principles

**P1**: Identity never depends on runtime state
- Position-based, structure-based, deterministic
- Runtime state affects **projection**, not **addressability**

**P2**: SemanticTree vs RenderTree
- **SemanticTree**: Complete identity space (includes inactive branches)
- **RenderTree**: Projection given runtime state (only active nodes)
- Identity lives in SemanticTree

### Segment-Specific Rules

#### Component (R1.1-R1.6)
- Static names only (no dynamic components)
- Keys auto-generated from position if not explicit
- Warn if auto-generated keys change
- Multiple instances require distinct keys

#### Slot (R2.1-R2.5)
- Identity exists even when empty
- Default vs Inserted variants
- Both exist in SemanticTree
- Only one projected to RenderTree

#### Element (R3.1-R3.4)
- Position in AST, not DOM
- Role from data-role or first class
- AST ID always present as fallback
- Identity exists even when not rendered

#### RepeatItem (R4.1-R4.7)
- Keys **required** for dynamic sources (dev error if missing)
- Index fallback only for literal/stable sources
- Keys must be unique within repeat
- Migration: warn → dev error → prod error

#### ConditionalBranch (R5.1-R5.8)
- **Both branches exist in SemanticTree always**
- Branch identity never depends on condition value
- Activation is rendering concern, not identity
- Patches to inactive branches allowed (structural ops only)

### Patch Rules

**Allowed to inactive branches**:
- InsertNode, RemoveNode, MoveNode (structural)
- UpdateAttributes, UpdateText, UpdateStyles (node-local)
- Validation is structural (schema), not rendered

**Disallowed**:
- Layout-dependent operations
- Render introspection
- Operations requiring "seeing" inactive nodes

---

## Next Phases

### ⏭️ Phase 3: Build SemanticID During Evaluation

**Goal**: Modify evaluator to construct SemanticID as it traverses AST

**Changes needed**:
```rust
struct EvaluationContext {
    semantic_path: Vec<SemanticSegment>,  // Current position
    component_key_counters: HashMap<String, usize>,  // For auto-keys
}

impl Evaluator {
    fn evaluate_element(&mut self, element: &Element) -> VNode {
        // Build semantic ID from context path
        let semantic_id = SemanticID::new(self.context.semantic_path.clone());

        // Push segment for this element
        self.context.semantic_path.push(SemanticSegment::Element {
            tag: element.tag.clone(),
            role: element.attributes.get("data-role").cloned(),
            ast_id: element.span.id.clone(),
        });

        // Evaluate children
        let children = ...;

        // Pop segment
        self.context.semantic_path.pop();

        VNode::Element {
            tag: element.tag,
            semantic_id: Some(semantic_id),
            key: element.attributes.get("key").cloned(),
            ...
        }
    }
}
```

**Files to modify**:
- `packages/evaluator/src/evaluator.rs`
- Add context tracking
- Add component key counter
- Add slot variant tracking
- Add conditional branch tracking
- Add repeat item tracking

---

### ⏭️ Phase 4: Stable Patches

**Goal**: Implement semantic ID-based patching

**New file**: `packages/evaluator/src/stable_patches.rs`

```rust
pub enum StablePatch {
    UpdateNode {
        target: SemanticID,
        attributes: HashMap<String, String>,
    },
    InsertNode {
        parent: SemanticID,
        before_sibling: Option<SemanticID>,
        node: VNode,
    },
    RemoveNode {
        target: SemanticID,
    },
    MoveNode {
        target: SemanticID,
        new_parent: SemanticID,
        before_sibling: Option<SemanticID>,
    },
    ToggleBranch {
        condition_id: String,
        active_branch: Branch,
    },
}

pub fn diff_with_semantic_id(
    old: &VirtualDomDocument,
    new: &VirtualDomDocument,
) -> Vec<StablePatch> {
    // Build maps: SemanticID -> VNode
    let old_map = build_semantic_map(&old.nodes);
    let new_map = build_semantic_map(&new.nodes);

    // Generate patches based on semantic identity
}
```

---

### ⏭️ Phase 5: Dev Mode Warnings

**Goal**: Add warnings for unstable patterns

**Warnings to implement**:
1. **Missing repeat keys** (dynamic sources)
   ```
   WARNING: repeat block at line 42 has no key attribute
   Source is dynamic (users from data)
   Add key={user.id} for stable identity
   ```

2. **Auto-generated component keys changed**
   ```
   WARNING: Component Button at line 15 has unstable auto-key
   Position changed from Button-0 to Button-1
   Add explicit key="..." for stability
   ```

3. **Patches to inactive branches**
   ```
   INFO: Patch targets inactive conditional branch
   Target: Dashboard::if[abc-7].else
   Patch will apply when branch becomes active
   ```

4. **Dynamic component names attempted**
   ```
   ERROR: Component name must be static identifier
   Found: $componentType
   Dynamic components not supported
   ```

---

## Test Coverage

### Current Tests

**semantic_identity.rs**: 10 tests
- ✅ Creation and depth
- ✅ Parent/child traversal
- ✅ Descendant checking
- ✅ Selector generation
- ✅ All segment types
- ✅ Complex nested structures

**vdom.rs**: Updated constructors
- ✅ All VNode helper methods
- ✅ with_semantic_id()
- ✅ with_key()

**vdom_differ.rs**: 4 tests updated
- ✅ Create node
- ✅ Remove node
- ✅ Update attributes
- ✅ Update text

**Total evaluator tests**: 94 passing ✅

### Tests Needed (Phase 3+)

- [ ] Semantic ID generation during evaluation
- [ ] Component key auto-generation
- [ ] Repeat key extraction
- [ ] Conditional branch tracking
- [ ] Slot variant detection
- [ ] Semantic map building
- [ ] Stable patch generation
- [ ] Dev mode warnings

---

## Documentation

### Created
- ✅ `SEMANTIC_IDENTITY_DESIGN.md` - Complete design with examples
- ✅ `SEMANTIC_IDENTITY_RULES.md` - Locked-down rules (the constitution)
- ✅ `SEMANTIC_IDENTITY_PROGRESS.md` - This file

### Needed
- [ ] `STABLE_PATCHES_SPEC.md` - Patch format specification
- [ ] `MIGRATION_GUIDE.md` - Upgrading to semantic identity
- [ ] `DEV_MODE_WARNINGS.md` - Warning messages and fixes

---

## Timeline

### Completed
- ✅ Phase 1: Core types (2 hours)
- ✅ Phase 2: VNode structure (1 hour)

### Estimated Remaining
- ⏭️ Phase 3: Build semantic IDs (4-6 hours)
  - Context tracking
  - Component key generation
  - Repeat/conditional/slot handling
  - Tests

- ⏭️ Phase 4: Stable patches (4-6 hours)
  - Semantic map building
  - Patch generation
  - Diffing algorithm
  - Tests

- ⏭️ Phase 5: Warnings (2-3 hours)
  - Detection logic
  - Message formatting
  - Tests

**Total remaining**: ~10-15 hours

---

## Breaking Changes

### None Yet!

All changes so far are **backwards compatible**:
- `semantic_id` field is `Option<SemanticID>` (defaults to None)
- `key` field is `Option<String>` (defaults to None)
- Existing code continues to work
- Legacy `id` field still present

### Future Breaking Changes (Optional)

**Phase 4+** may introduce:
- New stable patch format (alongside old path-based)
- Deprecation of path-based patches
- Eventually: removal of path-based patches

**Migration path**: Dual support period where both work.

---

## Success Metrics

### Phase 1-2 (Complete)
- [x] SemanticID type implemented
- [x] All segment types defined
- [x] VNode structure updated
- [x] All tests passing
- [x] Rules locked down

### Phase 3 (Next)
- [ ] Semantic IDs generated during evaluation
- [ ] All segment types populated correctly
- [ ] Component keys auto-generated
- [ ] Repeat keys extracted from attributes
- [ ] Tests verify correct IDs

### Phase 4
- [ ] Stable patches replace path-based
- [ ] List reordering works correctly
- [ ] Conditional toggling works correctly
- [ ] Refactoring doesn't break patches

### Phase 5
- [ ] Warnings fire for unstable patterns
- [ ] Dev mode catches missing keys
- [ ] Production builds stable

---

## Next Action

**Ready to proceed to Phase 3**: Building semantic IDs during evaluation.

This requires modifying the evaluator to track context and generate SemanticID as it traverses the AST.

Should I proceed with Phase 3?
