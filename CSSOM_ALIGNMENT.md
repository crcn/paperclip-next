# CSSOM/DOM Alignment Strategy

## Analysis of Existing Paperclip Implementation

After reviewing the existing Paperclip CSS and HTML evaluators, I've identified the key synchronization mechanism.

### Key Finding: Shared Class Name Generation

Both evaluators use the **same utility function** to generate class names:

```rust
// From: libs/evaluator/src/core/utils.rs
pub fn get_style_namespace(
    name: &Option<String>,
    id: &str,  // ← AST node ID (unique, stable)
    current_component: Option<&ast::Component>,
) -> String {
    if let Some(name) = name {
        let ns = if let Some(component) = &current_component {
            format!("{}-{}", component.name, name)
        } else {
            name.to_string()
        };
        format!("_{}-{}", ns, id)  // e.g., "_Button-div-abc123"
    } else {
        if let Some(component) = &current_component {
            format!("_{}-{}", component.name, id)  // e.g., "_Button-abc123"
        } else {
            format!("_{}", id)  // e.g., "_abc123"
        }
    }
}
```

### How It Works

**CSS Evaluator** (`libs/evaluator/src/css/evaluator.rs`):
```rust
// Line 142
selector_text: format!(".{}", get_style_namespace(&style.name, &style.id, None))
```

Generates CSS rules like:
```css
._Button-button-abc123 {
  padding: 8px;
}
```

**HTML Evaluator** (`libs/evaluator/src/html/evaluator.rs`):
```rust
// Lines 598-619
let mut class_name =
    get_style_namespace(&element.name, &element.id, context.current_component);

attributes.insert(
    "class".to_string(),
    virt::ObjectProperty {
        value: Some(class_name.into()),
        ...
    },
);
```

Generates HTML elements with matching classes:
```html
<button class="_Button-button-abc123">Click me</button>
```

### Critical Properties

1. **AST Node IDs**: Uses stable IDs from AST (not sequential counters)
2. **Deterministic**: Same AST → same class names
3. **Readable**: Includes element name + component name for debugging
4. **Scoped**: Component name prefix prevents collisions

## Current Implementation Gap

Our current implementation:
- ✅ Separates DOM and CSS evaluation
- ✅ Generates CSS rules
- ❌ Uses sequential counters (`Button-1`, `button-2`)
- ❌ Doesn't apply class names to VNode elements
- ❌ No synchronization between evaluators

## Proposed Changes

### 1. Add AST Node IDs to Span

Currently our AST doesn't have stable IDs. We need to add them:

```rust
// In paperclip_parser::ast
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub id: String,  // ← ADD: Unique stable ID (uuid or hash-based)
}
```

### 2. Create Shared Class Name Generator

```rust
// In paperclip_evaluator/src/css_evaluator.rs
pub fn get_style_namespace(
    element_name: &Option<String>,
    id: &str,
    component_name: Option<&str>,
) -> String {
    if let Some(name) = element_name {
        let ns = if let Some(comp) = component_name {
            format!("{}-{}", comp, name)
        } else {
            name.to_string()
        };
        format!("_{}-{}", ns, id)
    } else {
        if let Some(comp) = component_name {
            format!("_{}-{}", comp, id)
        } else {
            format!("_{}", id)
        }
    }
}
```

### 3. Update CSS Evaluator

```rust
// Use AST IDs instead of counters
fn extract_element_styles(
    &mut self,
    element: &Element,
    component_name: Option<&str>,
    rules: &mut Vec<CssRule>,
) -> CssResult<()> {
    match element {
        Element::Tag { name, styles, span, .. } => {
            // Generate class name from AST ID
            let class_name = get_style_namespace(
                &Some(name.clone()),
                &span.id,  // ← Use AST ID
                component_name,
            );

            let mut properties = HashMap::new();
            for style_block in styles {
                for (key, value) in &style_block.properties {
                    properties.insert(key.clone(), self.resolve_value(value)?);
                }
            }

            if !properties.is_empty() {
                rules.push(CssRule {
                    selector: format!(".{}", class_name),
                    properties,
                });
            }
        }
        _ => {}
    }
    Ok(())
}
```

### 4. Update DOM Evaluator

Add class names to VNode elements:

```rust
// In paperclip_evaluator/src/evaluator.rs
fn evaluate_element(&self, element: &Element) -> EvalResult<VNode> {
    match element {
        Element::Tag { name, attributes, styles, children, span } => {
            let mut vnode = VNode::element(name);

            // Generate and apply class name
            let component_name = self.context.current_component.as_ref().map(|c| c.name.as_str());
            let class_name = get_style_namespace(
                &Some(name.clone()),
                &span.id,  // ← Same AST ID as CSS evaluator
                component_name,
            );

            // Merge with existing class attribute
            let class_value = if let Some(existing_class) = attributes.get("class") {
                format!("{} {}", class_name, self.evaluate_expression(existing_class)?.to_string())
            } else {
                class_name
            };

            vnode = vnode.with_attr("class", class_value);

            // ... rest of evaluation
        }
    }
}
```

### 5. Track Current Component

```rust
// Add to EvalContext
pub struct EvalContext {
    components: HashMap<String, Component>,
    tokens: HashMap<String, String>,
    variables: HashMap<String, Value>,
    current_component: Option<Component>,  // ← ADD
}
```

## Migration Path

### Phase 1: Add AST IDs (Parser)
1. Modify parser to generate unique IDs for each AST node
2. Use `uuid::Uuid::new_v4()` or content-based hashing
3. Store ID in Span struct

### Phase 2: Shared Namespace Function (Evaluator)
1. Create `get_style_namespace()` utility
2. Export from evaluator crate
3. Add comprehensive tests

### Phase 3: Update CSS Evaluator
1. Replace sequential counters with AST IDs
2. Use `get_style_namespace()` for all selectors
3. Update tests to check stable class names

### Phase 4: Update DOM Evaluator
1. Track current component in context
2. Apply class names to all stylable elements
3. Merge with user-provided classes
4. Update tests to verify class attributes

### Phase 5: Integration Testing
1. Test that CSS rules match DOM classes
2. Verify style application in preview
3. Check edge cases (nested components, no names, etc.)

## Example: Before & After

### Before (Current Implementation)

CSS:
```css
.Button-1 .button-2 {
  padding: 8px;
}
```

HTML:
```html
<button>Click me</button>
```

❌ **Problem**: No class on element, styles don't apply!

### After (Proposed Implementation)

CSS:
```css
._Button-button-abc123 {
  padding: 8px;
}
```

HTML:
```html
<button class="_Button-button-abc123">Click me</button>
```

✅ **Solution**: Matching class names, styles apply correctly!

## Benefits of This Approach

1. **Perfect Synchronization**: CSS rules match DOM elements exactly
2. **Stable IDs**: Same source → same class names (cache-friendly)
3. **Debuggable**: Class names include element/component names
4. **Collision-Free**: Scoped to component + ID
5. **Battle-Tested**: Proven in production Paperclip

## Next Steps

1. Review this proposal with user
2. Implement AST ID generation in parser
3. Create shared namespace utility
4. Update both evaluators
5. Add integration tests
6. Update documentation

## Questions for User

1. Should we use UUIDs or content-based hashing for AST IDs?
2. Should we maintain the `_` prefix convention?
3. Should we support custom class name prefixes?
4. Do we need to handle render scopes like the existing implementation?
