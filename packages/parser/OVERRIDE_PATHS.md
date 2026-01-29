# Override Path Design

## Overview

Override paths allow targeting deeply nested component instances for customization in the designer. This enables drilling into component composition (e.g., `Card → Button → Icon → Path`) while maintaining stable references.

## Path Format

### Basic Format

```
ComponentName.index/ComponentName.index/...
```

Examples:
- `Button` - First Button instance in current component
- `Button.0` - First Button (explicit index)
- `Button.1` - Second Button instance
- `Card.Button` - Button inside Card instance
- `Card.0.Button.0` - First Button inside first Card

### Relationship to Semantic IDs

Semantic IDs (from Spike 0.6):
```
Card{"Card-0"}::div[div-id]::Button{"Button-0"}::button[btn-id]
```

Override path:
```
Card.Button
```

The override path is the **user-facing stable path** while semantic IDs are the **runtime identity**. Override paths resolve to semantic ID prefixes.

## AST Representation

### Override Node (New)

```rust
pub struct Override {
    /// Dot-separated path to target (e.g., "Button.Icon")
    pub path: Vec<String>,

    /// Styles to apply
    pub styles: Vec<StyleBlock>,

    /// Attributes to override
    pub attributes: HashMap<String, Expression>,

    /// Source location
    pub span: Span,
}
```

Add to Component:
```rust
pub struct Component {
    // ... existing fields
    pub overrides: Vec<Override>,
}
```

## Parser Syntax

### Basic Override

```javascript
component Card {
    render Button {}

    // Override Button instance
    override Button {
        style {
            color: red
        }
    }
}
```

### Deep Override

```javascript
component Page {
    render Card {
        // Card renders Button, Button renders Icon
    }

    // Drill into Card → Button → Icon
    override Card.Button.Icon {
        style {
            fill: blue
        }
    }
}
```

### Multiple Instances

```javascript
component Gallery {
    render Card {}  // First Card
    render Card {}  // Second Card

    // Override second Card
    override Card.1 {
        style {
            background: gray
        }
    }

    // Override Button inside first Card
    override Card.0.Button {
        style {
            color: white
        }
    }
}
```

## Resolution Algorithm

### Phase 1: Parse Override Paths

```rust
// Input: "Card.Button.Icon"
// Output: vec!["Card", "Button", "Icon"]

fn parse_override_path(path_str: &str) -> Vec<OverrideSegment> {
    path_str.split('.')
        .map(|segment| {
            if let Some((name, index)) = segment.split_once('.') {
                OverrideSegment { component: name, index: Some(index.parse().unwrap()) }
            } else {
                OverrideSegment { component: segment, index: None }
            }
        })
        .collect()
}
```

### Phase 2: Resolve Path to Semantic ID

```rust
fn resolve_override_path(
    doc: &Document,
    current_component: &Component,
    path: &[OverrideSegment],
) -> Result<String, OverrideError> {
    let mut current_node = current_component.body.as_ref()?;
    let mut semantic_id_parts = vec![];

    for segment in path {
        // Find instance of this component
        let instances = find_instances(current_node, &segment.component);

        let instance_index = segment.index.unwrap_or(0);
        let instance = instances.get(instance_index)?;

        // Build semantic ID part
        semantic_id_parts.push(format!(
            "{}{{\"{}\"}}",
            segment.component,
            instance.span.id
        ));

        // Drill into component definition
        current_node = find_component_def(doc, &segment.component)?.body;
    }

    Ok(semantic_id_parts.join("::"))
}
```

### Phase 3: Apply Overrides During Evaluation

```rust
fn evaluate_with_overrides(
    component: &Component,
    overrides: &[Override],
) -> VNode {
    // Normal evaluation
    let mut vnode = evaluate_component(component);

    // Apply matching overrides
    for override_def in overrides {
        if let Some(target_id) = resolve_override_path(doc, component, &override_def.path) {
            if let Some(target_node) = find_vnode_by_id(&vnode, &target_id) {
                apply_override_styles(target_node, &override_def.styles);
                apply_override_attributes(target_node, &override_def.attributes);
            }
        }
    }

    vnode
}
```

## Shadow Instances

Shadow instances are instances of instances (nested component usage):

```javascript
// Icon.pc
component Icon {
    render svg { path {} }
}

// Button.pc
component Button {
    render div {
        Icon {}  // Instance 1
    }
}

// Card.pc
component Card {
    render div {
        Button {}  // Instance 2 (contains Instance 1)
    }
}

// Page.pc
component Page {
    render Card {}

    // Deep override through shadows
    override Card.Button.Icon {
        // Resolves: Card -> Button def -> Icon instance -> Icon def -> svg/path
        style { fill: red }
    }
}
```

Resolution walks:
1. `Card` - Find Card instance in Page
2. `Button` - Find Button instance in Card definition
3. `Icon` - Find Icon instance in Button definition
4. Apply styles to Icon's rendered content

## Index Handling

### Implicit Index (First Match)

```javascript
render Button {}
render Button {}

override Button {  // Targets Button.0 (first)
    style { color: red }
}
```

### Explicit Index

```javascript
override Button.1 {  // Targets second Button
    style { color: blue }
}
```

### Future: Named Instances

```javascript
render Button name="primary" {}
render Button name="secondary" {}

override Button[name="primary"] {  // Future syntax
    style { color: red }
}
```

## Designer Integration

### Selection to Override Path

When user selects element in designer:

1. Get semantic ID: `Card{"card-1"}::Button{"btn-2"}::Icon{"icon-3"}`
2. Convert to override path: `Card.Button.Icon`
3. Determine indices by counting instances
4. Generate override: `override Card.Button.Icon { ... }`

### Highlighting

When hovering override in code:

1. Parse path: `["Card", "Button", "Icon"]`
2. Resolve to semantic ID
3. Find VNode with that ID prefix
4. Highlight in canvas

## Edge Cases

### Nonexistent Path

```javascript
override Card.Button {
    // But Card doesn't have a Button
}
```

**Behavior:** Validation warning, override ignored at runtime.

### Ambiguous Index

```javascript
render Button {}
render Button {}

override Button {  // Which Button?
}
```

**Behavior:** Targets first (index 0), warn about ambiguity.

### Slots

```javascript
component Card {
    render div {
        slot content
    }
}

component Page {
    render Card {
        insert content {
            Button {}  // Where is this in override path?
        }
    }

    override Card.Button {  // Should this work?
        style { color: red }
    }
}
```

**Behavior:** Override paths work through slots - Button is still considered part of Card's rendered tree.

## Performance Considerations

- **Path Resolution**: O(depth × instances) - cached per component
- **Override Application**: O(overrides × vnodes) - apply during evaluation
- **Index Lookup**: O(n) for finding nth instance - could optimize with indexing

## Testing Strategy

1. **Simple Override** - One level, single instance
2. **Deep Override** - Multiple levels (3+)
3. **Multiple Instances** - Explicit indices
4. **Shadow Instances** - Instance of instance
5. **Invalid Paths** - Nonexistent components
6. **Slot Traversal** - Override through slot boundaries
7. **Semantic ID Mapping** - Verify IDs match designer selection

## Implementation Phases

### Phase 1: Parser (Task #20)
- Add Override AST node
- Parse `override Path.To.Component { ... }` syntax
- Store in Component.overrides

### Phase 2: Resolution (Task #21)
- Implement path resolution algorithm
- Walk component graph
- Generate semantic ID prefixes

### Phase 3: Application (Task #21)
- Apply overrides during evaluation
- Merge styles with priority
- Test with simple cases

### Phase 4: Shadow Instances (Task #22)
- Test deep nesting
- Validate across component boundaries
- Handle edge cases

### Phase 5: Testing (Task #23)
- Comprehensive test suite
- All scenarios covered
- Performance validation
