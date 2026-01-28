# paperclip-evaluator

High-performance AST evaluator that transforms Paperclip components into Virtual DOM.

## Features

- ⚡ **Ultra-fast** - Evaluates components in 0.7-3 microseconds
- 🌳 **Virtual DOM output** - JSON-serializable for streaming
- 📊 **Expression evaluation** - Variables, operators, member access
- 🎨 **Style application** - Inline styles with CSS properties
- ✅ **Well-tested** - 2 passing tests

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
use paperclip_parser::parse;
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

    // Parse
    let doc = parse(source).expect("Failed to parse");

    // Evaluate
    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).expect("Failed to evaluate");

    // Use Virtual DOM
    println!("Generated {} root nodes", vdoc.nodes.len());
    println!("Generated {} CSS rules", vdoc.styles.len());
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

Create a new evaluator instance.

**`evaluate(&mut self, doc: &Document) -> EvalResult<VDocument>`**

Evaluate a parsed document to Virtual DOM.

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
- `id: Option<String>` - Optional ID for tracking

**`Text`** - Text node
- `content: String` - Text content

**`Comment`** - Comment node
- `content: String` - Comment content

### `VDocument`

Virtual document containing:
- `nodes: Vec<VNode>` - Root nodes
- `styles: Vec<CssRule>` - Global CSS rules

### `CssRule`

CSS rule:
- `selector: String` - CSS selector
- `properties: HashMap<String, String>` - CSS properties

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
