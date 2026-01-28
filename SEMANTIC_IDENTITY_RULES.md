# Semantic Identity Rules - The Constitution

This document defines the **exact semantics** for semantic identity. These rules are the foundation of patch stability, editor selection, and AI targeting. They must be locked down before implementing VNode changes.

---

## Core Principles

### Principle 1: Identity Never Depends on Runtime State

**Semantic identity MUST NEVER depend on runtime state.**

Identity is:
- ✅ Position-based
- ✅ Structure-based
- ✅ Deterministic from source code
- ❌ NOT value-based
- ❌ NOT dependent on truthiness
- ❌ NOT dependent on data

**Corollary**: Runtime state can only affect **projection** (active/inactive, evaluated text), not **addressability**.

### Principle 2: SemanticTree vs RenderTree

**SemanticTree**: Complete identity space including inactive branches, unfilled slots, unrendered conditionals
- Contains ALL possible nodes from source code
- Includes both then AND else branches
- Includes empty slots
- This is where identity lives

**RenderTree**: Projection of SemanticTree given current runtime state
- Only contains ACTIVE nodes
- Only one branch of conditionals
- Only filled slot content
- This is what preview shows

**Critical**: Identity is over SemanticTree. Preview is over RenderTree.

Branches exist in the **semantic tree**, not necessarily as realized VNodes in the active **render output**.

---

## Verification: SemanticID is Structured-First

**Status**: ✅ CORRECT

```rust
pub struct SemanticID {
    pub segments: Vec<SemanticSegment>,  // Structured data
}

impl SemanticID {
    pub fn to_selector(&self) -> String {  // Stringification is secondary
        // Only for: debugging, logs, protocol output
    }
}
```

**Internal operations** (comparison, traversal, hashing) work on `Vec<SemanticSegment>`, not strings.

---

## Segment Type Rules

### 1. Component Segment

```rust
Component {
    name: String,
    key: Option<String>,
}
```

#### Rules

**R1.1: Name is the component definition name**
```paperclip
component Button { ... }
// name = "Button"
```

**R1.2: Key is explicit or auto-generated**

**Explicit** (user-provided):
```paperclip
Button key="primary" { ... }
// key = Some("primary")
```

**Auto-generated** (from position):
```paperclip
Card {          // First Card instance
  Button { ... }  // key = Some("Button-0")
  Button { ... }  // key = Some("Button-1")
}
```

**R1.3: Key generation is deterministic from source order**
- Order in AST determines auto key
- NOT order in data
- NOT order in render output

**R1.4: Key stability warning**
In dev mode, warn if:
- Auto-generated key
- Position changes between renders
- No explicit key provided

**R1.5: Multiple instances require distinct keys**
```paperclip
// ERROR: Duplicate keys
Button key="save" { ... }
Button key="save" { ... }  // Must be unique within parent
```

**R1.6: Component names must be static identifiers**

**Allowed** (static):
```paperclip
Button { ... }
Card { ... }
MyCustomComponent { ... }
```

**Disallowed** (dynamic):
```paperclip
// NOT SUPPORTED - computed component names
$componentType { ... }
{getComponent()} { ... }
```

**Rule**: Semantic segments are resolved from static identifiers only. No computed tags/components participate in semantic identity.

**Why**: Preserves determinism. Dynamic component selection requires union typing + variant resolution (separate feature category, deferred).

---

### 2. Slot Segment

```rust
Slot {
    name: String,
}
```

#### Rules

**R2.1: Slot name is the slot definition name**
```paperclip
component Card {
  slot header   // name = "header"
  slot footer   // name = "footer"
}
```

**R2.2: Default slot has explicit name**
```paperclip
component Button {
  slot default  // name = "default" (not empty string)
}
```

**R2.3: Slot identity exists even when empty**
```paperclip
Card {
  // header slot is EMPTY, but identity exists:
  // Card::header (present in semantic space)
  slot footer { div { ... } }
}
```

**R2.4: Slot identity is independent of fill state**
- Empty slot: identity exists
- Filled slot: identity exists
- Same identity regardless of fill

**Why**: Patches can target "add content to empty slot" using slot identity.

**R2.5: Default slot content vs inserted content**

**Default content** (defined in component):
```paperclip
component Button {
  slot default

  render button {
    insert default {
      text "Default Text"  // Default content
    }
  }
}
```

**Inserted content** (provided by user):
```paperclip
Button {
  slot default {
    text "Custom Text"  // Inserted content
  }
}
```

**Identity model**:
- Default content lives under: `Slot(name="default", variant=Default)`
- Inserted content lives under: `Slot(name="default", variant=Inserted)`
- Insertion toggles which subtree is **projected** to RenderTree, not which exists in SemanticTree
- Both subtrees exist in SemanticTree always
- Patches can target either

**Example semantic IDs**:
```
Button::default[variant=Default]::text[abc-5]    // Default text node
Button::default[variant=Inserted]::text[xyz-7]   // Inserted text node
```

**Projection**: Only one is active in RenderTree at a time.

---

### 3. Element Segment

```rust
Element {
    tag: String,
    role: Option<String>,
    ast_id: String,
}
```

#### Rules

**R3.1: Tag is the element tag name**
```paperclip
div { ... }     // tag = "div"
button { ... }  // tag = "button"
```

**R3.2: Role is optional semantic identifier**

**From data-role attribute**:
```paperclip
div data-role="user-card" { ... }
// role = Some("user-card")
```

**Fallback to first class**:
```paperclip
div class="user-card highlighted" { ... }
// role = Some("user-card") (first class)
```

**No role**:
```paperclip
div { ... }
// role = None
```

**R3.3: AST ID is always present**
- Fallback identifier when tag/role not unique
- From AST node ID (e.g., "80f4925f-5")
- MUST be deterministic (same source = same ID)

**R3.4: Element identity is position in AST, not DOM**
```paperclip
if condition {
  div { text "A" }  // ast_id = "abc-5"
}

// When condition = false:
// - Element identity "abc-5" still exists in semantic space
// - Just not rendered in DOM
// - Patches can still target it
```

---

### 4. RepeatItem Segment

```rust
RepeatItem {
    repeat_id: String,
    key: String,
}
```

#### Rules

**R4.1: repeat_id is the AST node ID of the repeat block**
```paperclip
repeat users as user {  // repeat_id = "xyz-10"
  div { ... }
}
```

**R4.2: key is REQUIRED for stable identity**

**Explicit key attribute** (required for dynamic sources):
```paperclip
repeat users as user key={user.id} {
  div { ... }
}
// key = value of user.id (e.g., "user-123")
```

**Index fallback** (only for synthetically stable sources):
```paperclip
// Literal array - provably stable
repeat [1, 2, 3] as item {
  div { ... }
}
// key = stringified index (e.g., "0", "1", "2")
// ⚠️ WARN: "Literal array uses index, consider explicit keys"

// Dynamic source - UNSTABLE
repeat users as user {  // users from data/props
  div { ... }
}
// 🚨 ERROR in dev mode: "Missing key for dynamic repeat source"
// ⚠️ WARN in prod mode: "Missing key, using unstable index"
```

**Rule**: RepeatItem must have a stable key source. Index fallback is only permitted when repeat source is synthetically stable (e.g., literal list). For data-driven lists, missing keys are a **dev error**.

**Migration path**:
- Phase 1: Warn in both dev and prod
- Phase 2: Error in dev, warn in prod
- Phase 3: Error in both (strict mode)

**R4.3: Key must be unique within repeat block**
```paperclip
repeat users as user key={user.id} {
  div { ... }
}
// If two users have same ID → ERROR
```

**R4.4: Key stability across data changes**

**Scenario**: User list reordered
```
Before: [user-123, user-456, user-789]
After:  [user-789, user-456, user-123]
```

**With keys**:
- `RepeatItem { repeat_id: "xyz-10", key: "user-123" }` → Same identity!
- Position changed (index 0 → 2), but semantic ID unchanged
- Patches targeting "user-123" still work

**Without keys** (index fallback):
- `RepeatItem { repeat_id: "xyz-10", key: "0" }` → Now points to user-789 (wrong!)
- Patches corrupt

**R4.5: Key must be stringifiable**
- Numbers: `42` → `"42"`
- Strings: `"user-123"` → `"user-123"`
- Objects: NOT ALLOWED (must extract scalar)

**R4.6: Empty array semantics**
```paperclip
repeat emptyArray as item {
  div { ... }
}
// Zero RepeatItem segments created
// Repeat block identity exists: "xyz-10"
// No children
```

**R4.7: Null/undefined source semantics**
```paperclip
repeat nullValue as item {
  div { ... }
}
// Treated as empty array
// Zero RepeatItem segments
// No error
```

---

### 5. ConditionalBranch Segment

```rust
ConditionalBranch {
    condition_id: String,
    branch: Branch,  // Then | Else
}
```

#### Rules

**CRITICAL**: This is the most important section.

**R5.1: condition_id is AST node ID of if block**
```paperclip
if showWelcome {  // condition_id = "aaa-7"
  div { ... }
}
```

**R5.2: Branch identity is POSITION-BASED, not VALUE-BASED**

```paperclip
if condition {
  div { text "A" }  // ConditionalBranch { condition_id: "aaa-7", branch: Then }
} else {
  div { text "B" }  // ConditionalBranch { condition_id: "aaa-7", branch: Else }
}
```

**Both branches have stable identity regardless of condition value.**

**R5.3: Identity exists even when branch is inactive**

**When condition = true**:
- Then branch: **ACTIVE** (rendered to DOM)
- Else branch: **INACTIVE** (exists in semantic space, not in DOM)

**When condition = false**:
- Then branch: **INACTIVE** (exists in semantic space, not in DOM)
- Else branch: **ACTIVE** (rendered to DOM)

**Key point**: Both identities exist at all times. Activation is a rendering concern, not an identity concern.

**R5.4: Patches can target inactive branches (structural only)**

Patches to inactive branches are allowed, but **validation is structural (schema) not rendered**.

**Allowed operations** (purely structural/node-local):
- `InsertNode` / `RemoveNode` / `MoveNode` within branch subtree
- `UpdateText` / `UpdateAttributes` / `UpdateStyles`
- Any operation that doesn't require evaluating the branch

**Disallowed operations** (require rendered context):
- Operations depending on computed sizes/layout
- Operations requiring introspecting rendered output
- Operations depending on runtime data fetch results
- Any operation requiring "seeing" the branch in preview

**Example** (allowed):
```rust
// Patch targeting inactive branch
UpdateNode {
    target: SemanticID("Dashboard::if[aaa-7].else::div[aaa-9]"),
    attributes: { "class": "updated" }
}

// VALID even if condition = true (else branch inactive)
// Applied to SemanticTree immediately
// Will appear in RenderTree when branch becomes active
```

**Example** (disallowed):
```rust
// Operation requiring layout
MeasureAndAdjust {
    target: SemanticID("Dashboard::if[aaa-7].else::div[aaa-9]"),
    // ERROR: Cannot measure inactive node
}
```

**Why**: Allows pre-populating content (killer feature) without implying you can "see" inactive branches in preview.

**R5.5: Branch switch is not an identity change**

```
Condition flips from true → false:
- Then branch: ACTIVE → INACTIVE
- Else branch: INACTIVE → ACTIVE

Identity unchanged:
- Then: ConditionalBranch { id: "aaa-7", branch: Then }  // Still same
- Else: ConditionalBranch { id: "aaa-7", branch: Else }  // Still same
```

**Patch operation**:
```rust
// NOT UpdateNode (identity unchanged)
// Instead: ToggleBranch
ToggleBranch {
    condition_id: "aaa-7",
    active_branch: Branch::Else  // Changed from Then to Else
}
```

**R5.6: Missing else branch**

```paperclip
if condition {
  div { text "A" }
}
// No else branch
```

- Then branch identity exists: `if[aaa-7].then`
- Else branch identity does NOT exist (no segment)
- Semantic space only contains what's defined in source

**R5.7: Conditional without braces is same semantics**

```paperclip
if condition { div { ... } }
// Same as:
if condition
  div { ... }

// Both create: ConditionalBranch { id: "aaa-7", branch: Then }
```

**R5.8: Nested conditionals compound**

```paperclip
if outer {
  if inner {
    div { ... }
  }
}
```

**Semantic ID**:
```rust
SemanticID {
    segments: vec![
        ConditionalBranch { condition_id: "aaa-5", branch: Then },
        ConditionalBranch { condition_id: "aaa-7", branch: Then },
        Element { tag: "div", ast_id: "aaa-9", ... },
    ]
}
```

**Selector**: `if[aaa-5].then::if[aaa-7].then::div[aaa-9]`

---

## Identity Stability Examples

### Example 1: List Reordering

**Source**:
```paperclip
repeat users as user key={user.id} {
  div { text user.name }
}
```

**Initial state** (users = [user-123, user-456]):
```
RepeatItem { repeat_id: "r-1", key: "user-123" }::div[d-1]  → "Alice"
RepeatItem { repeat_id: "r-1", key: "user-456" }::div[d-1]  → "Bob"
```

**After reorder** (users = [user-456, user-123]):
```
RepeatItem { repeat_id: "r-1", key: "user-456" }::div[d-1]  → "Bob"
RepeatItem { repeat_id: "r-1", key: "user-123" }::div[d-1]  → "Alice"
```

**Patch**:
```rust
MoveNode {
    target: SemanticID("repeat[r-1]{\"user-123\"}::div[d-1]"),
    new_index: 1  // Moved from index 0 to 1
}
```

**Identity unchanged! Patch targets semantic ID, not index.**

---

### Example 2: Conditional Toggle

**Source**:
```paperclip
if showWelcome {
  div { text "Welcome!" }
} else {
  div { text "Goodbye!" }
}
```

**When showWelcome = true**:
```
ACTIVE:   if[c-1].then::div[d-1]  → "Welcome!"
INACTIVE: if[c-1].else::div[d-2]  → (not rendered)
```

**When showWelcome = false**:
```
INACTIVE: if[c-1].then::div[d-1]  → (not rendered)
ACTIVE:   if[c-1].else::div[d-2]  → "Goodbye!"
```

**Patch**:
```rust
// NOT remove + insert
// Just toggle active branch
ToggleBranch {
    condition_id: "c-1",
    active_branch: Branch::Else
}
```

**Both identities exist at all times. Only rendering changes.**

---

### Example 3: Slot Fill/Empty Toggle

**Source**:
```paperclip
component Card {
  slot header

  render div {
    div class="header-container" {
      insert header
    }
  }
}

// Usage
Card {
  slot header { div { text "Title" } }
}
```

**With header filled**:
```
Card::header::div[d-5]  → "Title"
```

**With header empty** (slot removed from usage):
```
Card::header  → (empty, but identity exists)
```

**Patch to add content**:
```rust
InsertNode {
    parent: SemanticID("Card::header"),
    node: VNode { tag: "div", ... }
}
```

**Identity "Card::header" exists regardless of fill state.**

---

## Implementation Checklist

Phase 2 prerequisites:

- [x] ✅ SemanticID is structured-first (Vec<SemanticSegment>)
- [x] ✅ ConditionalBranch rules locked down (R5.1-R5.8)
- [x] ✅ RepeatItem key rules defined (R4.1-R4.7, strict for dynamic sources)
- [x] ✅ Slot default vs inserted semantics defined (R2.5)
- [x] ✅ Component instance key generation specified (R1.1-R1.6)
- [x] ✅ SemanticTree vs RenderTree distinction established
- [x] ✅ Static identifier constraint locked (R1.6)
- [x] ✅ Patches to inactive branches rules (R5.4, structural only)
- [ ] ⏭️ Tests for each rule written (next step)
- [ ] ⏭️ Dev mode warnings for unstable patterns (next step)

**Ready to proceed to Phase 2: VNode structure changes**

---

## Design Decisions (Locked)

### Q1: Patches to inactive branches?

**Decision**: ✅ **Allow, but only structural operations**

- **Allowed**: InsertNode, RemoveNode, UpdateAttributes, UpdateText (structural)
- **Disallowed**: Layout-dependent ops, render introspection
- **Validation**: Structural (schema) not rendered
- **Benefit**: Pre-populate content before condition flips

**Locked in R5.4**

---

### Q2: Component instance keys required?

**Decision**: ✅ **Two-tier approach**

**Component instances**: Optional with warnings
- Position-based fallback acceptable
- Warn if auto-generated key changes between evaluations
- For selection stability, not correctness

**Repeat items**: Strict - error in dev for dynamic sources
- Keys required for correctness under reorder/insert/delete
- Index fallback only for literal/stable sources
- Dynamic sources without keys = dev error

**Migration path**: Warn → dev error → prod error

**Locked in R1.4 and R4.2**

---

### Q3: Dynamic component names?

**Decision**: ✅ **Hard constraint - static identifiers only**

- Component/type names must be static
- No computed tags/components
- Dynamic selection is separate feature (union types + variant resolution)
- Preserves determinism

**Locked in R1.6**

---

## Summary

**Core rules locked**:
1. Identity is position-based, never value-based
2. ConditionalBranch identity exists for both branches regardless of active state
3. RepeatItem requires keys for stability (index fallback warns)
4. Slot identity exists regardless of fill state
5. SemanticID is structured data, stringification is secondary

**Next steps**:
1. Review and approve these rules
2. Implement tests for each rule
3. Add dev mode warnings
4. THEN proceed to Phase 2 (VNode changes)

These rules are the "constitution" - once locked, they can't change without breaking stability guarantees.

Ready for review?
