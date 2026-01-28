# paperclip-parser

High-performance parser for the Paperclip visual component language.

## Features

- ⚡ **Blazing fast** - Parses 1000-line files in 25 microseconds
- 🔤 **Zero-copy tokenization** - Using `logos` with string slices
- 🌳 **Complete AST** - Components, styles, tokens, expressions
- ✅ **Well-tested** - 12 passing tests
- 🎨 **CSS support** - All CSS properties including dashes (margin-bottom, line-height)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
paperclip-parser = { path = "../parser" }
```

Or from the workspace:

```toml
[dependencies]
paperclip-parser.workspace = true
```

## Usage

### Basic Parsing

```rust
use paperclip_parser::parse;

fn main() {
    let source = r#"
        public component Button {
            render button {
                style {
                    padding: 8px 16px
                    background: #3366FF
                    color: white
                }
                text "Click me"
            }
        }
    "#;

    match parse(source) {
        Ok(doc) => {
            println!("Parsed successfully!");
            println!("Components: {}", doc.components.len());
            println!("First component: {}", doc.components[0].name);
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
        }
    }
}
```

### Tokenization Only

```rust
use paperclip_parser::tokenize;

fn main() {
    let source = "component Button { }";
    let tokens = tokenize(source);

    for (token, span) in tokens {
        println!("{:?} at {:?}", token, span);
    }
}
```

### Working with the AST

```rust
use paperclip_parser::{parse, ast::*};

fn main() {
    let source = r#"
        public component Card {
            render div {
                style {
                    padding: 16px
                }
                text "Hello"
            }
        }
    "#;

    let doc = parse(source).unwrap();

    // Access components
    for component in &doc.components {
        println!("Component: {}", component.name);
        println!("Public: {}", component.public);

        // Access render body
        if let Some(Element::Tag { name, children, styles, .. }) = &component.body {
            println!("Root element: {}", name);
            println!("Children: {}", children.len());
            println!("Style blocks: {}", styles.len());
        }
    }

    // Access tokens
    for token in &doc.tokens {
        println!("Token: {} = {}", token.name, token.value);
    }
}
```

### Error Handling

```rust
use paperclip_parser::{parse, ParseError};

fn main() {
    let source = "component Invalid {"; // Missing closing brace

    match parse(source) {
        Ok(_) => println!("Parsed successfully"),
        Err(e) => match e {
            ParseError::UnexpectedToken { pos, expected, found } => {
                eprintln!("Syntax error at position {}", pos);
                eprintln!("Expected: {}", expected);
                eprintln!("Found: {}", found);
            }
            ParseError::UnexpectedEof { pos } => {
                eprintln!("Unexpected end of file at position {}", pos);
            }
            ParseError::InvalidSyntax { pos, message } => {
                eprintln!("Invalid syntax at {}: {}", pos, message);
            }
            _ => eprintln!("Parse error: {:?}", e),
        }
    }
}
```

## API Reference

### Main Functions

#### `parse(source: &str) -> ParseResult<Document>`

Parse a complete .pc file into an AST.

**Returns:** `Result<Document, ParseError>`

#### `tokenize(source: &str) -> Vec<(Token, Range<usize>)>`

Tokenize source code into tokens with source positions.

**Returns:** Vector of (token, span) tuples

### AST Types

#### `Document`

Root document containing:
- `imports: Vec<Import>` - Import statements
- `tokens: Vec<TokenDecl>` - Token declarations
- `styles: Vec<StyleDecl>` - Style declarations
- `components: Vec<Component>` - Component definitions

#### `Component`

Component definition:
- `public: bool` - Visibility
- `name: String` - Component name
- `variants: Vec<Variant>` - Variant definitions
- `slots: Vec<Slot>` - Slot definitions
- `body: Option<Element>` - Render body
- `span: Span` - Source location

#### `Element`

Render tree node (enum):
- `Tag` - HTML element (div, button, etc.)
- `Text` - Text node with expression
- `Instance` - Component instance
- `Conditional` - if/else rendering
- `Repeat` - Iteration
- `SlotInsert` - Slot insertion

#### `Expression`

Expression type (enum):
- `Literal` - String literal
- `Number` - Number literal
- `Boolean` - Boolean literal
- `Variable` - Variable reference
- `Member` - Member access (obj.prop)
- `Binary` - Binary operation (a + b)
- `Call` - Function call
- `Template` - String template

## Supported Syntax

### Components

```javascript
public component Button {
    variant hover trigger { ":hover" }

    slot children {
        text "Default content"
    }

    render button {
        style {
            padding: 8px 16px
        }
        children
    }
}
```

### Styles

```javascript
public style baseButton {
    padding: 8px
    background: #333
    color: white
}

style extends baseButton {
    border-radius: 4px
}
```

### Tokens

```javascript
public token primaryColor #3366FF
public token spacing 16px
public token fontFamily Inter, sans-serif
```

### Elements

```javascript
// HTML elements
div {
    style { padding: 16px }
    text "Content"
}

// Component instances
Button(label="Click me")

// Conditionals
if showBadge {
    Badge()
}

// Iteration
repeat item in items {
    Card()
}
```

### Expressions

```javascript
// Literals
text "Hello"
text "Hello {name}"  // Template

// Variables
text {userName}

// Member access
text {user.name}

// Binary operations
text {price * quantity}
```

## Performance

Benchmarks on Apple Silicon M-series:

| Operation | Time | Throughput |
|-----------|------|------------|
| Parse simple component | 840 ns | ~1.2M components/sec |
| Parse medium component | 2.2 µs | ~450K components/sec |
| Parse 1000-line file | 25 µs | ~40K files/sec |
| Tokenize only | 347 ns | ~2.9M files/sec |

See `../../BENCHMARKS.md` for detailed results.

## Testing

Run tests:

```bash
cargo test -p paperclip-parser
```

Run benchmarks:

```bash
cargo bench -p paperclip-parser
```

## Examples

See `../../examples/` for complete .pc files:
- `button.pc` - Simple button component
- `simple.pc` - Basic component example

## Error Types

- `ParseError::UnexpectedToken` - Unexpected token in source
- `ParseError::UnexpectedEof` - Unexpected end of file
- `ParseError::InvalidSyntax` - Invalid syntax structure
- `ParseError::LexerError` - Tokenization error

## Development

Built with:
- `logos` - Fast lexer generator
- `serde` - Serialization support
- `thiserror` - Error handling

## License

MIT
