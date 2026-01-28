# Semantic Identity Design

## Problem Statement

Current node identification uses:
- **AST IDs**: `"80f4925f-5"` - Too low-level, breaks on refactoring
- **VDOM Paths**: `[2, 1, 0]` - Too fragile, breaks on structure changes

**We need**: Stable identity that represents "the button in the footer slot of Card instance X" and survives refactoring.

---

## Design Principles

1. **Survives Refactoring** - Moving code doesn't break identity
2. **Human Readable** - Can be understood without looking at code
3. **Deterministic** - Same structure = same identity
4. **Hierarchical** - Reflects component/slot nesting
5. **Optional Keys** - User can provide explicit keys for instances

---

## Core Types

```rust
/// Semantic identity that survives refactoring
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticID {
    /// Hierarchical path through component tree
    pub segments: Vec<SemanticSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticSegment {
    /// Component instance
    Component {
        name: String,              // "Button", "Card", etc.
        key: Option<String>,       // User-provided or auto-generated
    },

    /// Slot reference within component
    Slot {
        name: String,              // "header", "footer", "default"
    },

    /// Element within component body
    Element {
        tag: String,               // "div", "button", "span"
        role: Option<String>,      // Optional semantic role
        ast_id: String,            // Fallback to AST position
    },

    /// Item within repeat block
    RepeatItem {
        repeat_id: String,         // ID of the repeat block AST node
        key: String,               // From key attribute or index
    },

    /// Branch within conditional
    ConditionalBranch {
        condition_id: String,      // ID of the if block AST node
        branch: Branch,            // Then or Else
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Branch {
    Then,
    Else,
}
```

---

## Examples

### Example 1: Simple Component

**Source**:
```paperclip
component Button {
  render button {
    text "Click"
  }
}
```

**Semantic ID**:
```rust
SemanticID {
    segments: vec![
        Component { name: "Button", key: None },
        Element { tag: "button", role: None, ast_id: "80f4925f-2" },
    ]
}
```

**Selector**: `Button::button[80f4925f-2]`

---

### Example 2: Component with Slots

**Source**:
```paperclip
component Card {
  slot header
  slot footer

  render div {
    div class="header" {
      insert header
    }
    div class="footer" {
      insert footer
    }
  }
}

component App {
  render Card {
    slot header {
      div { text "Title" }
    }
    slot footer {
      button { text "Action" }
    }
  }
}
```

**Semantic IDs**:

Title div in header slot:
```rust
SemanticID {
    segments: vec![
        Component { name: "App", key: None },
        Component { name: "Card", key: None },
        Slot { name: "header" },
        Element { tag: "div", role: None, ast_id: "abc-7" },
    ]
}
```
**Selector**: `App::Card::header::div[abc-7]`

Button in footer slot:
```rust
SemanticID {
    segments: vec![
        Component { name: "App", key: None },
        Component { name: "Card", key: None },
        Slot { name: "footer" },
        Element { tag: "button", role: None, ast_id: "abc-9" },
    ]
}
```
**Selector**: `App::Card::footer::button[abc-9]`

---

### Example 3: Repeat Block

**Source**:
```paperclip
component UserList {
  render div {
    repeat users as user key={user.id} {
      div class="user-card" {
        text user.name
      }
    }
  }
}
```

**Semantic ID for user "user-123"**:
```rust
SemanticID {
    segments: vec![
        Component { name: "UserList", key: None },
        Element { tag: "div", role: None, ast_id: "xyz-2" },
        RepeatItem { repeat_id: "xyz-3", key: "user-123" },
        Element { tag: "div", role: Some("user-card"), ast_id: "xyz-4" },
    ]
}
```
**Selector**: `UserList::div[xyz-2]::repeat[xyz-3]{"user-123"}::div[xyz-4]`

**Key Point**: When user-123 moves from index 0 to index 5, the semantic ID stays the same!

---

### Example 4: Conditional

**Source**:
```paperclip
component Dashboard {
  render div {
    if showWelcome {
      div { text "Welcome!" }
    } else {
      div { text "Loading..." }
    }
  }
}
```

**Semantic ID for "Welcome!" (then branch)**:
```rust
SemanticID {
    segments: vec![
        Component { name: "Dashboard", key: None },
        Element { tag: "div", role: None, ast_id: "aaa-2" },
        ConditionalBranch { condition_id: "aaa-3", branch: Branch::Then },
        Element { tag: "div", role: None, ast_id: "aaa-4" },
    ]
}
```
**Selector**: `Dashboard::div[aaa-2]::if[aaa-3].then::div[aaa-4]`

**Semantic ID for "Loading..." (else branch)**:
```rust
SemanticID {
    segments: vec![
        Component { name: "Dashboard", key: None },
        Element { tag: "div", role: None, ast_id: "aaa-2" },
        ConditionalBranch { condition_id: "aaa-3", branch: Branch::Else },
        Element { tag: "div", role: None, ast_id: "aaa-5" },
    ]
}
```
**Selector**: `Dashboard::div[aaa-2]::if[aaa-3].else::div[aaa-5]`

---

### Example 5: Multiple Instances with Keys

**Source**:
```paperclip
component App {
  render div {
    Button key="primary" { slot default { text "Save" } }
    Button key="secondary" { slot default { text "Cancel" } }
  }
}
```

**Semantic ID for primary button's text**:
```rust
SemanticID {
    segments: vec![
        Component { name: "App", key: None },
        Element { tag: "div", role: None, ast_id: "bbb-2" },
        Component { name: "Button", key: Some("primary") },
        Slot { name: "default" },
    ]
}
```
**Selector**: `App::div[bbb-2]::Button{"primary"}::default`

**Semantic ID for secondary button's text**:
```rust
SemanticID {
    segments: vec![
        Component { name: "App", key: None },
        Element { tag: "div", role: None, ast_id: "bbb-2" },
        Component { name: "Button", key: Some("secondary") },
        Slot { name: "default" },
    ]
}
```
**Selector**: `App::div[bbb-2]::Button{"secondary"}::default`

---

## Implementation Plan

### Phase 1: Core Types

File: `packages/evaluator/src/semantic_identity.rs`

```rust
pub struct SemanticID { ... }
pub enum SemanticSegment { ... }

impl SemanticID {
    pub fn new(segments: Vec<SemanticSegment>) -> Self;
    pub fn to_selector(&self) -> String;
    pub fn from_selector(selector: &str) -> Result<Self, ParseError>;
    pub fn parent(&self) -> Option<SemanticID>;
    pub fn append(&self, segment: SemanticSegment) -> SemanticID;
}
```

### Phase 2: Update VNode

File: `packages/evaluator/src/vdom.rs`

```rust
pub struct VNode {
    pub id: String,                    // Keep for backwards compat
    pub semantic_id: SemanticID,       // NEW: Stable identity
    pub tag: String,
    pub key: Option<String>,           // NEW: Explicit key for repeat
    pub attributes: HashMap<String, String>,
    pub children: Vec<VNode>,
}
```

### Phase 3: Build SemanticID During Evaluation

File: `packages/evaluator/src/evaluator.rs`

```rust
struct EvaluationContext {
    semantic_path: Vec<SemanticSegment>,  // Current position
}

impl Evaluator {
    fn evaluate_element(&mut self, element: &Element) -> VNode {
        // Build semantic ID from current context
        let semantic_id = SemanticID::new(self.context.semantic_path.clone());

        // Push element segment
        self.context.semantic_path.push(SemanticSegment::Element {
            tag: element.tag.clone(),
            role: element.attributes.get("data-role").cloned(),
            ast_id: element.span.id.clone(),
        });

        // Evaluate children
        let children = self.evaluate_children(&element.children);

        // Pop segment
        self.context.semantic_path.pop();

        VNode {
            id: element.span.id.clone(),
            semantic_id,
            tag: element.tag.clone(),
            key: element.attributes.get("key").cloned(),
            attributes: self.evaluate_attributes(&element.attributes),
            children,
        }
    }
}
```

### Phase 4: Stable Patches

File: `packages/evaluator/src/stable_patches.rs`

```rust
pub enum StablePatch {
    UpdateNode {
        target: SemanticID,              // Stable identity
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
}

pub fn diff_with_semantic_id(
    old: &VirtualDomDocument,
    new: &VirtualDomDocument,
) -> Vec<StablePatch> {
    // Build maps: SemanticID -> VNode
    let old_map = build_semantic_map(&old.nodes);
    let new_map = build_semantic_map(&new.nodes);

    // Generate patches based on semantic identity
    // - Nodes with same semantic ID = UPDATE
    // - Nodes only in old = REMOVE
    // - Nodes only in new = INSERT
    // - Nodes that changed position = MOVE
}
```

---

## Benefits

### 1. Survives Refactoring

**Before** (path-based):
```rust
// Move button to different position
// OLD: Update { path: [2, 1] }
// NEW: Update { path: [3, 0] }  // BREAKS!
```

**After** (semantic):
```rust
// Move button to different position
// Update { target: SemanticID("Card::footer::button[abc-5]") }
// WORKS! Identity doesn't change
```

### 2. Visual Editor Targeting

```typescript
// Visual editor can target specific elements
editor.select("UserList::repeat{'user-123'}::div.user-card");

// AI can target elements semantically
ai.update("Card::footer::button", { text: "New Text" });
```

### 3. Stable Across Updates

**Scenario**: User reorders list

**Without semantic ID**:
```
Before: user-123 at path [0]
After:  user-123 at path [5]
Patches reference wrong nodes!
```

**With semantic ID**:
```
Before: user-123 at SemanticID("UserList::repeat{'user-123'}")
After:  user-123 at SemanticID("UserList::repeat{'user-123'}")
Same identity! Patches work correctly.
```

---

## Migration Path

### Phase 1: Add Fields (Non-Breaking)
- Add `semantic_id` to VNode (optional)
- Add `key` to VNode (optional)
- Build semantic IDs during evaluation
- Keep existing path-based patches working

### Phase 2: Dual Support
- Support both path-based and semantic-based patches
- Gradually migrate to semantic patches
- Tools can use whichever they prefer

### Phase 3: Deprecate Paths
- Mark path-based patches as deprecated
- Encourage migration to semantic patches
- Provide migration tools

### Phase 4: Remove Paths (Breaking)
- Remove path-based patch support
- Only semantic patches supported

---

## Open Questions

### 1. Auto-Generated Keys for Component Instances?

**Option A**: Require explicit keys
```paperclip
Button key="primary" { ... }  // REQUIRED
```

**Option B**: Auto-generate from position
```paperclip
Button { ... }  // Auto-key: "Button-0"
Button { ... }  // Auto-key: "Button-1"
```

**Recommendation**: Option B (auto-generate), but warn in dev mode if position changes frequently.

### 2. Role Attribute for Elements?

**Option A**: Use data-role
```paperclip
div data-role="user-card" { ... }
```

**Option B**: Use class
```paperclip
div class="user-card" { ... }
```

**Option C**: Infer from context
```paperclip
// Role automatically set to "user-card" based on class
div class="user-card" { ... }
```

**Recommendation**: Option A (explicit data-role), fall back to first class name.

### 3. Selector Syntax?

**Option A**: Double colon (proposed)
```
Card::footer::button[abc-5]
```

**Option B**: Slash
```
Card/footer/button[abc-5]
```

**Option C**: Dot (CSS-like)
```
Card.footer.button[abc-5]
```

**Recommendation**: Option A (double colon) to distinguish from CSS selectors.

---

## Next Steps

1. ✅ Design complete (this document)
2. ⏭️ Implement core types in `semantic_identity.rs`
3. ⏭️ Update VNode structure
4. ⏭️ Build semantic IDs during evaluation
5. ⏭️ Implement stable patch format
6. ⏭️ Add tests for semantic identity
7. ⏭️ Update differ to use semantic IDs

Ready to implement Phase 1?
