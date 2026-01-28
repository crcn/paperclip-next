# paperclip-evaluator

High-performance AST evaluator that transforms Paperclip components into Virtual DOM.

## Features

- ⚡ **Ultra-fast** - Evaluates components in 0.7-3 microseconds
- 🌳 **Virtual DOM output** - JSON-serializable for streaming
- 📊 **Expression evaluation** - Variables, operators, member access
- 🎨 **Style application** - Inline styles with CSS properties
- 🔑 **Semantic Identity** - Stable, refactoring-safe node IDs
- 📦 **Bundle Support** - Cross-file component resolution
- 🎯 **Stable Patches** - Semantic ID-based diffing
- ✅ **Well-tested** - 102 passing tests

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
paperclip-evaluator = { path = "../evaluator" }
paperclip-parser = { path = "../parser" }
```

## Usage

### Basic Evaluation

```rust
use paperclip_parser::parse_with_path;
use paperclip_evaluator::Evaluator;

fn main() {
    let source = r#"
        public component Button {
            render button {
                style {
                    padding: 8px 16px
                    background: #3366FF
                }
                text "Click me"
            }
        }
    "#;

    // Parse with file path for stable IDs
    let doc = parse_with_path(source, "/components/button.pc")
        .expect("Failed to parse");

    // Evaluate with document ID
    let mut evaluator = Evaluator::with_document_id("/components/button.pc");
    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

    // Use Virtual DOM
    println!("Generated {} root nodes", vdoc.nodes.len());
    println!("Generated {} CSS rules", vdoc.styles.len());
}
```

### Semantic Identity (Stable IDs)

Every VNode has a semantic ID that remains stable across refactoring:

```rust
use paperclip_parser::parse_with_path;
use paperclip_evaluator::{Evaluator, VNode};

fn main() {
    let source = r#"
        public component Card {
            render div {
                h1 { text "Title" }
                Button()
                Button()
            }
        }

        component Button {
            render button { text "Click" }
        }
    "#;

    let doc = parse_with_path(source, "/card.pc").unwrap();
    let mut evaluator = Evaluator::with_document_id("/card.pc");
    let vdom = evaluator.evaluate(&doc).unwrap();

    // Every element has a semantic ID
    if let VNode::Element { semantic_id, children, .. } = &vdom.nodes[0] {
        println!("Card semantic ID: {}", semantic_id.to_selector());
        // Output: Card{"Card-0"}::div[id]

        for child in children {
            if let VNode::Element { semantic_id, .. } = child {
                println!("Child semantic ID: {}", semantic_id.to_selector());
                // Output: Card{"Card-0"}::div[id]::h1[id]
                //         Card{"Card-0"}::div[id]::Button{"Button-0"}::button[id]
                //         Card{"Card-0"}::div[id]::Button{"Button-1"}::button[id]
            }
        }
    }
}
```

### Stable Patches (Diffing)

Generate minimal patches using semantic ID-based matching:

```rust
use paperclip_parser::parse_with_path;
use paperclip_evaluator::{Evaluator, diff_vdocument};

fn main() {
    let source_v1 = r#"
        public component Card {
            render div {
                h1 { text "Title" }
                p { text "Content" }
            }
        }
    "#;

    let source_v2 = r#"
        public component Card {
            render div {
                h1 { text "Updated Title" }
                p { text "Content" }
            }
        }
    "#;

    // Generate VDOM for both versions
    let doc_v1 = parse_with_path(source_v1, "/card.pc").unwrap();
    let mut eval_v1 = Evaluator::with_document_id("/card.pc");
    let vdom_v1 = eval_v1.evaluate(&doc_v1).unwrap();

    let doc_v2 = parse_with_path(source_v2, "/card.pc").unwrap();
    let mut eval_v2 = Evaluator::with_document_id("/card.pc");
    let vdom_v2 = eval_v2.evaluate(&doc_v2).unwrap();

    // Generate minimal patches
    let patches = diff_vdocument(&vdom_v1, &vdom_v2);

    // Nodes matched by semantic ID - minimal patches!
    println!("Patches: {}", patches.len());
    // Only the text content changed, not the structure
}
```

### Working with Virtual DOM

```rust
use paperclip_evaluator::{Evaluator, VNode};
use paperclip_parser::parse;

fn main() {
    let source = r#"
        public component Card {
            render div {
                style {
                    padding: 16px
                    background: white
                }
                div {
                    text "Title"
                }
                div {
                    text "Content"
                }
            }
        }
    "#;

    let doc = parse(source).unwrap();
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Traverse Virtual DOM
    for node in &vdoc.nodes {
        print_node(node, 0);
    }
}

fn print_node(node: &VNode, indent: usize) {
    let prefix = "  ".repeat(indent);

    match node {
        VNode::Element { tag, attributes, styles, children, .. } => {
            println!("{}Element: {}", prefix, tag);
            println!("{}  Attributes: {:?}", prefix, attributes);
            println!("{}  Styles: {:?}", prefix, styles);
            for child in children {
                print_node(child, indent + 1);
            }
        }
        VNode::Text { content } => {
            println!("{}Text: {}", prefix, content);
        }
        VNode::Comment { content } => {
            println!("{}Comment: {}", prefix, content);
        }
    }
}
```

### Expression Evaluation with Context

```rust
use paperclip_evaluator::{Evaluator, Value};
use paperclip_parser::parse;
use std::collections::HashMap;

fn main() {
    let source = r#"
        public component Greeting {
            render div {
                text "Hello, {name}!"
            }
        }
    "#;

    let doc = parse(source).unwrap();
    let mut evaluator = Evaluator::new();

    // Set variables
    evaluator.context.set_variable(
        "name".to_string(),
        Value::String("Alice".to_string())
    );

    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Result will contain "Hello, Alice!"
    println!("{:?}", vdoc);
}
```

### Serializing to JSON

```rust
use paperclip_evaluator::Evaluator;
use paperclip_parser::parse;

fn main() {
    let source = r#"
        public component Button {
            render button {
                text "Click"
            }
        }
    "#;

    let doc = parse(source).unwrap();
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).unwrap();

    // Serialize to JSON for streaming
    let json = serde_json::to_string(&vdoc).unwrap();
    println!("{}", json);

    // Output:
    // {"nodes":[{"type":"Element","tag":"button",...}],"styles":[]}
}
```

## API Reference

### `Evaluator`

Main evaluator struct.

#### Methods

**`new() -> Self`**

Create a new evaluator instance with anonymous document ID.

**`with_document_id(path: &str) -> Self`**

Create evaluator with document ID from file path. **Recommended** for stable semantic IDs.

**Example:**
```rust
let mut evaluator = Evaluator::with_document_id("/components/button.pc");
```

**`evaluate(&mut self, doc: &Document) -> EvalResult<VirtualDomDocument>`**

Evaluate a parsed document to Virtual DOM.

**`evaluate_bundle(&mut self, bundle: &Bundle, entry_path: &Path) -> EvalResult<VirtualDomDocument>`**

Evaluate a bundle with cross-file imports.

### `EvalContext`

Evaluation context storing components, tokens, and variables.

#### Methods

**`add_component(&mut self, component: Component)`**

Register a component in the context.

**`add_token(&mut self, name: String, value: String)`**

Register a design token.

**`set_variable(&mut self, name: String, value: Value)`**

Set a variable for expression evaluation.

**`get_variable(&self, name: &str) -> Option<&Value>`**

Get a variable value.

### `Value`

Runtime value type (enum):
- `String(String)` - String value
- `Number(f64)` - Number value
- `Boolean(bool)` - Boolean value
- `Array(Vec<Value>)` - Array value
- `Object(HashMap<String, Value>)` - Object value
- `Null` - Null value

#### Methods

**`to_string(&self) -> String`**

Convert value to string representation.

**`is_truthy(&self) -> bool`**

Check if value is truthy for conditionals.

### `VNode`

Virtual DOM node (enum):

**`Element`** - HTML element
- `tag: String` - Element tag name
- `attributes: HashMap<String, String>` - Element attributes
- `styles: HashMap<String, String>` - Inline styles
- `children: Vec<VNode>` - Child nodes
- `semantic_id: SemanticID` - Stable semantic identity
- `key: Option<String>` - Explicit key (from key attribute)
- `id: Option<String>` - Legacy AST-based ID (deprecated)

**`Text`** - Text node
- `content: String` - Text content

**`Comment`** - Comment node
- `content: String` - Comment content

### `VirtualDomDocument`

Virtual document containing:
- `nodes: Vec<VNode>` - Root nodes
- `styles: Vec<CssRule>` - Global CSS rules

### `SemanticID`

Hierarchical semantic identity for stable node tracking.

#### Methods

**`to_selector() -> String`**

Convert to CSS-like selector string.

**Example:**
```rust
let id: SemanticID = // ...
println!("{}", id.to_selector());
// Output: Card{"Card-0"}::div[id]::Button{"Button-1"}::button[id]
```

**`is_descendant_of(&self, other: &SemanticID) -> bool`**

Check if this ID is a descendant of another.

**`parent() -> Option<SemanticID>`**

Get parent semantic ID.

### `SemanticSegment`

Segment of a semantic path (enum):

- `Component { name: String, key: Option<String> }` - Component instance
- `Element { tag: String, role: Option<String>, ast_id: String }` - HTML element
- `Slot { name: String, variant: SlotVariant }` - Slot insertion
- `RepeatItem { repeat_id: String, key: String }` - Repeat item
- `ConditionalBranch { condition_id: String, branch: Branch }` - Conditional branch

### Diffing Functions

**`diff_vdocument(old: &VirtualDomDocument, new: &VirtualDomDocument) -> Vec<VDocPatch>`**

Generate minimal patches using semantic ID-based node matching.

**Benefits:**
- Nodes matched by semantic ID, not position
- Reordering produces zero patches (if content unchanged)
- Refactoring-safe - IDs survive structural changes

**Example:**
```rust
let patches = diff_vdocument(&old_vdom, &new_vdom);
for patch in patches {
    // Apply patches...
}
```

### `CssRule`

CSS rule:
- `selector: String` - CSS selector
- `properties: HashMap<String, String>` - CSS properties

### `Bundle`

Multi-file bundle with dependency resolution.

#### Methods

**`new() -> Self`**

Create a new empty bundle.

**`add_document(&mut self, path: PathBuf, document: Document)`**

Add a parsed document to the bundle.

**`build_dependencies(&mut self) -> Result<(), BundleError>`**

Resolve dependencies between documents.

**`get_document(&self, path: &Path) -> Option<&Document>`**

Get a document by path.

**`find_component(&self, name: &str, from_path: &Path) -> Option<(&Component, &Path)>`**

Find a component by name from a specific file.

**Example:**
```rust
use paperclip_evaluator::Bundle;
use std::path::PathBuf;

let mut bundle = Bundle::new();
bundle.add_document(PathBuf::from("/main.pc"), main_doc);
bundle.add_document(PathBuf::from("/button.pc"), button_doc);
bundle.build_dependencies()?;

// Evaluate bundle
let mut evaluator = Evaluator::new();
let vdom = evaluator.evaluate_bundle(&bundle, &PathBuf::from("/main.pc"))?;
```

### `CssEvaluator`

CSS extraction and evaluation.

#### Methods

**`new() -> Self`**

Create evaluator with anonymous document ID.

**`with_document_id(path: &str) -> Self`**

Create with document ID from file path.

**`evaluate(&mut self, doc: &Document) -> CssResult<VirtualCssDocument>`**

Evaluate document to CSS rules.

**Example:**
```rust
use paperclip_evaluator::CssEvaluator;
use paperclip_parser::parse_with_path;

let doc = parse_with_path(source, "/styles.pc")?;
let mut css_eval = CssEvaluator::with_document_id("/styles.pc");
let css_doc = css_eval.evaluate(&doc)?;

for rule in &css_doc.rules {
    println!("{} {{ {:?} }}", rule.selector, rule.properties);
}
```

## Expression Evaluation

The evaluator supports these expression types:

### Literals

```rust
// String literals
Value::String("hello".to_string())

// Number literals
Value::Number(42.0)

// Boolean literals
Value::Boolean(true)
```

### Variables

```rust
// Variable reference: {userName}
evaluator.context.set_variable(
    "userName".to_string(),
    Value::String("Alice".to_string())
);
```

### Member Access

```rust
// Object.property: {user.name}
let mut user = HashMap::new();
user.insert("name".to_string(), Value::String("Alice".to_string()));

evaluator.context.set_variable(
    "user".to_string(),
    Value::Object(user)
);
```

### Binary Operations

Supported operators:
- `Add` - Addition or string concatenation
- `Subtract` - Subtraction
- `Multiply` - Multiplication
- `Divide` - Division (with zero check)
- `Equals` - Equality comparison
- `NotEquals` - Inequality comparison

```rust
// Example: {price * quantity}
evaluator.context.set_variable("price".to_string(), Value::Number(10.0));
evaluator.context.set_variable("quantity".to_string(), Value::Number(3.0));
// Result: 30.0
```

## Performance

Benchmarks on Apple Silicon M-series:

| Operation | Time | Throughput |
|-----------|------|------------|
| Evaluate simple component | 745 ns | ~1.3M components/sec |
| Evaluate medium component | 2.9 µs | ~345K components/sec |
| Evaluate 10 components | 9.9 µs | ~101K batches/sec |
| Parse + Evaluate | 2.2 µs | ~450K components/sec |

See `../../BENCHMARKS.md` for detailed results.

## Testing

Run tests:

```bash
cargo test -p paperclip-evaluator
```

Run benchmarks:

```bash
cargo bench -p paperclip-evaluator
```

## Error Handling

```rust
use paperclip_evaluator::{Evaluator, EvalError};
use paperclip_parser::parse;

fn main() {
    let source = r#"
        public component Card {
            render Container() // Component doesn't exist
        }
    "#;

    let doc = parse(source).unwrap();
    let mut evaluator = Evaluator::new();

    match evaluator.evaluate(&doc) {
        Ok(vdoc) => println!("Success: {:?}", vdoc),
        Err(e) => match e {
            EvalError::ComponentNotFound(name) => {
                eprintln!("Component '{}' not found", name);
            }
            EvalError::VariableNotFound(name) => {
                eprintln!("Variable '{}' not found", name);
            }
            EvalError::EvaluationError(msg) => {
                eprintln!("Evaluation error: {}", msg);
            }
        }
    }
}
```

## Virtual DOM Output Example

Input `.pc`:
```javascript
public component Button {
    render button {
        style {
            padding: 8px 16px
            background: #3366FF
        }
        text "Click me"
    }
}
```

Output JSON:
```json
{
  "nodes": [
    {
      "type": "Element",
      "tag": "button",
      "attributes": {},
      "styles": {
        "padding": "8px 16px",
        "background": "#3366FF"
      },
      "children": [
        {
          "type": "Text",
          "content": "Click me"
        }
      ]
    }
  ],
  "styles": []
}
```

## Development

Built with:
- `paperclip-parser` - AST input
- `serde` - JSON serialization
- `thiserror` - Error handling

## License

MIT
