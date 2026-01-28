# Class Name Synchronization

## Overview

This document describes how CSS selectors and DOM element class names are synchronized in the Paperclip evaluator.

## Architecture

Both the CSS evaluator and DOM evaluator use the **same deterministic class name generation** to ensure perfect synchronization:

```
CSS Rule:     ._Button-button-abc123 { padding: 8px; }
DOM Element:  <button class="_Button-button-abc123">Click</button>
```

## Key Components

### 1. Deterministic AST IDs (`packages/parser/src/ast.rs`)

Each `Span` in the AST has a unique `id` field generated from its source position:

```rust
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub id: String,  // Hash of start + end positions
}

impl Span {
    fn generate_id(start: usize, end: usize) -> String {
        let mut hasher = DefaultHasher::new();
        start.hash(&mut hasher);
        end.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
```

**Benefits**:
- Same source code always generates same IDs
- IDs are unique within a document
- No sequential counters needed
- Works with incremental updates

### 2. Shared Utility Function (`packages/evaluator/src/utils.rs`)

Both evaluators use `get_style_namespace()` to generate class names:

```rust
pub fn get_style_namespace(
    element_name: Option<&str>,
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

**Class Name Format**:
- Inside component: `_ComponentName-elementName-id`
- Top-level element: `_elementName-id`
- Anonymous element: `_id`

**Examples**:
- `_Button-button-a3f2c9d1` - Button element in Button component
- `_Card-div-5e8b3a2c` - Div element in Card component
- `_div-9c7f1b4e` - Top-level div

### 3. CSS Evaluator (`packages/evaluator/src/css_evaluator.rs`)

Generates CSS rules with class selectors:

```rust
fn extract_element_styles(...) -> CssResult<()> {
    match element {
        Element::Tag { name, styles, children, span } => {
            // Generate class name using AST ID
            let class_name = get_style_namespace(
                Some(name.as_str()),
                &span.id,
                component_name,
            );

            // Create CSS rule
            rules.push(CssRule {
                selector: format!(".{}", class_name),
                properties: /* ... */,
            });
        }
        // ...
    }
}
```

**Output Example**:
```css
._Button-button-a3f2c9d1 {
  padding: 8px 16px;
  background: #3366FF;
  border-radius: 4px;
}
```

### 4. DOM Evaluator (`packages/evaluator/src/evaluator.rs`)

Applies matching class names to VNode elements:

```rust
fn evaluate_element(&mut self, element: &Element) -> EvalResult<VNode> {
    match element {
        Element::Tag { name, styles, span, .. } => {
            let mut vnode = VNode::element(name);

            // Generate SAME class name as CSS evaluator
            let class_name = get_style_namespace(
                Some(name.as_str()),
                &span.id,
                self.context.current_component.as_deref(),
            );

            // Check if user provided class attribute
            let mut has_class = false;
            for (key, expr) in attributes {
                let value = self.evaluate_expression(expr)?;
                if key == "class" {
                    // Merge with generated class
                    let merged = format!("{} {}", class_name, value);
                    vnode = vnode.with_attr(key, merged);
                    has_class = true;
                } else {
                    vnode = vnode.with_attr(key, value.to_string());
                }
            }

            // If no class attribute, add generated class
            if !has_class {
                vnode = vnode.with_attr("class", class_name);
            }

            vnode
        }
        // ...
    }
}
```

**Output Example**:
```json
{
  "type": "Element",
  "tag": "button",
  "attributes": {
    "class": "_Button-button-a3f2c9d1"
  },
  "children": [...]
}
```

## Component Scoping

The DOM evaluator tracks the current component to properly scope class names:

```rust
pub struct EvalContext {
    pub variables: HashMap<String, Value>,
    pub current_component: Option<String>,  // NEW
}

fn evaluate_component_with_props(...) {
    // Set current component for scoping
    self.context.current_component = Some(component.name.clone());

    // Evaluate body (all child elements get scoped)
    let result = if let Some(body) = &component.body {
        self.evaluate_element(body)?
    };

    // Clear component scope
    self.context.current_component = None;

    result
}
```

## Verification

The synchronization is verified by `test_class_name_synchronization`:

```rust
#[test]
fn test_class_name_synchronization() {
    let source = r#"
        public component Button {
            render button {
                style {
                    padding: 8px
                    background: blue
                }
                text "Click"
            }
        }
    "#;

    let doc = parse(source).unwrap();

    // Evaluate both
    let vdom = Evaluator::new().evaluate(&doc).unwrap();
    let css = CssEvaluator::new().evaluate(&doc).unwrap();

    // Extract class name from DOM
    let VNode::Element { attributes, .. } = &vdom.nodes[0];
    let class_name = attributes.get("class").unwrap();

    // Find matching CSS rule
    let css_rule = css.rules.iter()
        .find(|r| r.selector == format!(".{}", class_name))
        .expect("Should find matching CSS rule");

    // Verify properties match
    assert_eq!(css_rule.properties.get("padding"), Some(&"8px".to_string()));
    assert_eq!(css_rule.properties.get("background"), Some(&"blue".to_string()));
}
```

## Benefits

1. **Perfect Synchronization**: CSS selectors always match DOM class names
2. **Deterministic**: Same source code always produces same class names
3. **Collision-Free**: Component name + element name + AST ID ensures uniqueness
4. **No Side Effects**: No sequential counters or global state
5. **Incremental**: Works with patch-based updates
6. **Readable**: Class names include component and element names for debugging

## Workflow

```
Source Code:
  component Button { render button { style { padding: 8px } } }
         ↓
    Parser
         ↓
    AST (with deterministic Span IDs)
         ↓
    ┌────────────────┬────────────────┐
    ↓                ↓                ↓
CSS Evaluator    DOM Evaluator    (other evaluators)
    ↓                ↓
get_style_namespace(...)  get_style_namespace(...)
    ↓                ↓
._Button-button-abc123   class="_Button-button-abc123"
    ↓                ↓
CSS Document         VDocument
    ↓                ↓
  Production CSS    Live Preview
```

## Future Enhancements

- [ ] Support variant-specific class names (`.Button-hover-abc123`)
- [ ] Support pseudo-classes/elements (`:hover`, `::before`)
- [ ] Minify class names in production (`._a1`, `._b2`)
- [ ] Source maps for debugging minified class names
- [ ] CSS module exports for TypeScript types

## Related Files

- `packages/parser/src/ast.rs` - Span with deterministic IDs
- `packages/evaluator/src/utils.rs` - `get_style_namespace()` utility
- `packages/evaluator/src/css_evaluator.rs` - CSS rule generation
- `packages/evaluator/src/evaluator.rs` - DOM class application
- `packages/evaluator/src/tests_cssom.rs` - Synchronization tests
