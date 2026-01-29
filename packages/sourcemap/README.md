# Paperclip Source Maps

Shared utilities for generating source maps during compilation from `.pc` files.

## Purpose

This package provides core source map functionality used by all Paperclip compilers (React, CSS, HTML) to generate mappings from compiled output back to original `.pc` source files.

## Features

- **Source map generation** using industry-standard `sourcemap` crate
- **Position tracking** as code is emitted during compilation
- **Byte offset ↔ line/column conversion** for working with parser spans
- **Zero dependencies** on other Paperclip packages (shared utility)

## Usage

```rust
use paperclip_sourcemap::{SourceMapBuilder, byte_offset_to_line_col};

// Create a builder
let source = "component Button {}";
let mut builder = SourceMapBuilder::new("button.pc", source);

// Track generated code position
builder.advance("const Button = () => {\n");

// Add a mapping
let (gen_line, gen_col) = builder.current_position();
let (src_line, src_col) = byte_offset_to_line_col(source, 10);
builder.add_mapping(gen_line, gen_col, src_line, src_col, Some("Button"));

// Build the source map
let map = builder.build()?;
let json = map.to_json()?;
```

## API

### `SourceMapBuilder`

Main builder for generating source maps during compilation.

#### Methods

- `new(source_file: &str, source_content: &str) -> Self`
  Create a new builder for the given source file

- `add_mapping(gen_line: u32, gen_col: u32, src_line: u32, src_col: u32, name: Option<&str>)`
  Add a mapping from generated position to source position

- `advance(text: &str)`
  Track position after emitting text to the output

- `current_position() -> (u32, u32)`
  Get current (line, col) in generated output

- `build(self) -> Result<SourceMap, sourcemap::Error>`
  Build the final SourceMap

- `to_json(self) -> Result<String, sourcemap::Error>`
  Build and convert to JSON string

### Helper Functions

- `byte_offset_to_line_col(source: &str, offset: usize) -> (u32, u32)`
  Convert byte offset (from parser spans) to line/column

- `line_col_to_byte_offset(source: &str, line: u32, col: u32) -> usize`
  Convert line/column to byte offset

## Example: Complete Compiler Integration

```rust
use paperclip_sourcemap::{SourceMapBuilder, byte_offset_to_line_col};
use paperclip_parser::ast::{Document, Span};

fn compile_with_sourcemap(doc: &Document, source: &str) -> (String, String) {
    let mut output = String::new();
    let mut builder = SourceMapBuilder::new("input.pc", source);

    // Emit code
    output.push_str("const Button = () => {");
    builder.advance("const Button = () => {");

    // Add mapping for component name
    let (gen_line, gen_col) = builder.current_position();
    let (src_line, src_col) = byte_offset_to_line_col(source, component.span.start);
    builder.add_mapping(gen_line, gen_col, src_line, src_col, Some("Button"));

    // Continue compilation...

    let sourcemap_json = builder.to_json().unwrap();
    (output, sourcemap_json)
}
```

## Testing

```bash
cargo test -p paperclip-sourcemap
```

All tests should pass:
- Builder creation and position tracking
- Mapping generation
- Byte offset ↔ line/column conversion
- Unicode handling
- Edge cases (empty files, out of bounds)

## Implementation Notes

### Position Tracking

The builder automatically tracks the current position in generated output as you call `advance()`. This eliminates manual line/column counting.

### Byte Offsets vs Line/Column

Parser spans use byte offsets, but source maps need line/column. Use `byte_offset_to_line_col()` to convert.

### Unicode Support

All functions correctly handle multi-byte Unicode characters.

### Performance

- Position tracking: O(n) where n is generated code length
- Byte offset conversion: O(m) where m is source length
- Both are called once per mapping, not a bottleneck

## Dependencies

- `sourcemap` (8.0) - Industry-standard source map library
- `thiserror` - Error handling (from workspace)

## Used By

- `paperclip-compiler-react` - React/JSX compilation
- `paperclip-compiler-css` - CSS compilation
- `paperclip-compiler-html` - HTML compilation

## License

MIT
