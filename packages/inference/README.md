# Paperclip Inference Engine

A standalone, multi-pass type inference engine for Paperclip components that enables sophisticated type analysis for multiple compilation targets.

## Features

- **Multi-pass inference**: Signature collection → body analysis → prop extraction
- **Lexical scoping**: Proper handling of control flow, nested scopes, and variable shadowing
- **Type unification**: Automatically merges conflicting type information
- **Member access tracking**: Infers object shapes from property access patterns
- **Nested member access**: Supports deep property access like `user.address.city`
- **Binary operation constraints**: Infers types from arithmetic operations
- **Plugin-based code generation**: Extensible architecture with TypeScript and Rust generators
- **Unknown vs Any semantics**: Clear distinction between transient inference state and explicit dynamic types

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
paperclip-inference = { path = "../inference" }
```

## Usage

### Basic Type Inference

```rust
use paperclip_inference::{InferenceEngine, InferenceOptions};
use paperclip_parser::parse;

let source = r#"
public component Counter {
    variant primary
    render div {
        text {count}
        text {label}
    }
}
"#;

let doc = parse(source).unwrap();
let engine = InferenceEngine::new(InferenceOptions::default());
let props = engine.infer_component_props(&doc.components[0]).unwrap();

// props contains:
// - primary: Boolean (optional, from variant)
// - count: Any (unknown usage)
// - label: Any (unknown usage)
```

### TypeScript Code Generation

```rust
use paperclip_inference::codegen::typescript::TypeScriptGenerator;
use paperclip_inference::CodeGenerator;

let ts_gen = TypeScriptGenerator::new();

// Generate interface
let props_vec: Vec<_> = props.into_iter().collect();
let interface = ts_gen.generate_interface("CounterProps", &props_vec);

println!("{}", interface);
// Output:
// export interface CounterProps {
//   primary?: boolean;
//   count: any;
//   label: any;
// }
```

### Rust Code Generation (Stub)

```rust
use paperclip_inference::codegen::rust::RustGenerator;
use paperclip_inference::CodeGenerator;

let rust_gen = RustGenerator::new();

let props_vec: Vec<_> = props.into_iter().collect();
let struct_def = rust_gen.generate_interface("CounterProps", &props_vec);

println!("{}", struct_def);
// Output:
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct CounterProps {
//     pub primary: Option<bool>,
//     pub count: serde_json::Value,
//     pub label: serde_json::Value,
// }
```

## Type System

### Core Types

- `Unknown` - Transient, engine-internal (should not escape inference)
- `Any` - Explicitly dynamic/user-controlled (fallback for truly dynamic values)
- `String` - String type
- `Number` - Numeric type
- `Boolean` - Boolean type
- `Null` - Null type
- `Slot` - React.ReactNode equivalent
- `Union(Vec<Type>)` - Union of multiple types
- `Literal(LiteralType)` - Exact literal value (string, number, or boolean)
- `Array(Box<Type>)` - Array type
- `Optional(Box<Type>)` - Optional/undefined type
- `Function(FunctionType)` - Function/callback type
- `Element(ElementType)` - Element reference
- `Object(ObjectType)` - Object with known properties

### Type Unification

The engine automatically unifies types when variables are used multiple times:

```rust
// Variable used in two contexts
// First: count (Unknown)
// Second: count + 1 (Number)
// Result: count is inferred as Number
```

Unification rules:
- `Unknown` unifies with anything → takes the other type
- `Any` absorbs everything → stays `Any`
- Incompatible types → creates `Union`
- Objects merge properties
- Literals widen to base types

## Configuration Options

```rust
pub struct InferenceOptions {
    /// Use strict type checking (fail on Unknown types in final output)
    pub strict: bool,

    /// Infer object property types from member access
    pub infer_object_properties: bool,

    /// Infer function signatures from call expressions (not yet implemented)
    pub infer_functions: bool,

    /// Support nested member access (e.g., user.address.city)
    pub nested_member_access: bool,
}
```

### Presets

```rust
// Default configuration
let options = InferenceOptions::default();

// Strict mode (errors on Unknown types)
let options = InferenceOptions::strict();

// All features enabled
let options = InferenceOptions::full();

// Minimal inference
let options = InferenceOptions::minimal();
```

## How It Works

### Multi-Pass Algorithm

1. **Signature Collection**
   - Collect variants → Boolean props (always optional)
   - Collect slots → Slot props (optional if has default content)

2. **Body Analysis**
   - Traverse element tree
   - Infer types from expressions
   - Track member access patterns
   - Apply constraints from binary operations
   - Handle control flow (conditionals, loops)

3. **Prop Extraction**
   - Collect root scope bindings
   - Finalize types (Unknown → Any)
   - Determine optionality
   - Convert to PropertyType map

### Scope Management

The engine uses reference-counted scopes (`Rc<Scope>`) for efficient forking:

```rust
// Root scope (component props)
let mut root_scope = Scope::new();

// Child scope (e.g., inside conditional)
let child_scope = Scope::with_parent(Rc::new(root_scope.clone()));

// Only root scope bindings become component props
let props = root_scope.collect_root_props();
```

## Extending with Custom Generators

Implement the `CodeGenerator` trait:

```rust
use paperclip_inference::codegen::CodeGenerator;
use paperclip_inference::types::{Type, PropertyType};

pub struct MyGenerator;

impl CodeGenerator for MyGenerator {
    fn generate_type(&self, type_: &Type) -> String {
        match type_ {
            Type::String => "str".to_string(),
            Type::Number => "f64".to_string(),
            // ... handle other types
            _ => "unknown".to_string(),
        }
    }

    fn generate_property(&self, name: &str, prop: &PropertyType) -> String {
        format!("{}: {}", name, self.generate_type(&prop.type_))
    }

    fn generate_interface(&self, name: &str, props: &[(String, PropertyType)]) -> String {
        // Generate your target language's interface/struct
        unimplemented!()
    }
}
```

## Future Enhancements

- **Binary operations**: Full support when parser adds binary op parsing
- **Function signatures**: Infer callback types from usage patterns
- **Flow-sensitive narrowing**: Type refinement in conditionals
- **Generic components**: Support for type parameters
- **Constraint propagation**: More sophisticated type constraints

## Performance

The inference engine is designed for fast, incremental analysis:

- Reference-counted scopes avoid deep copies
- Type unification is O(1) for most cases
- Finalization pass is single-pass
- Suitable for IDE/LSP integration

## Testing

```bash
# Run all tests
cargo test -p paperclip-inference

# Run with output
cargo test -p paperclip-inference -- --nocapture

# Run specific test
cargo test -p paperclip-inference test_infer_member_access
```

## License

MIT
