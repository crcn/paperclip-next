# Source Maps Starter Scaffold

Copy-paste these files to get started quickly.

## 1. Create `packages/sourcemap/Cargo.toml`

```toml
[package]
name = "paperclip-sourcemap"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
sourcemap = "8.0"
thiserror.workspace = true

[dev-dependencies]
# Add test dependencies if needed
```

## 2. Create `packages/sourcemap/src/lib.rs`

```rust
//! Source map utilities for Paperclip compilers
//!
//! This crate provides shared utilities for generating source maps
//! during compilation from .pc files to various targets (React, CSS, HTML).

pub mod builder;
pub mod utils;

pub use builder::SourceMapBuilder;
pub use utils::{byte_offset_to_line_col, line_col_to_byte_offset};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let source = "line 1\nline 2\nline 3";
        let (line, col) = byte_offset_to_line_col(source, 7);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }
}
```

## 3. Create `packages/sourcemap/src/builder.rs`

```rust
use sourcemap::{SourceMap as ExternalSourceMap, SourceMapBuilder as ExternalBuilder};
use std::path::Path;

/// Builder for generating source maps during compilation
pub struct SourceMapBuilder {
    builder: ExternalBuilder,
    source_file: String,
    current_line: u32,
    current_col: u32,
}

impl SourceMapBuilder {
    /// Create a new source map builder
    ///
    /// # Arguments
    /// * `source_file` - The original .pc file path
    /// * `source_content` - The original .pc file content
    pub fn new(source_file: &str, source_content: &str) -> Self {
        let mut builder = ExternalBuilder::new(None);

        // Add the source file and its content
        let source_id = builder.add_source(source_file);
        builder.set_source_contents(source_id, Some(source_content));

        Self {
            builder,
            source_file: source_file.to_string(),
            current_line: 0,
            current_col: 0,
        }
    }

    /// Add a mapping from generated position to source position
    ///
    /// # Arguments
    /// * `gen_line` - Line in generated file (0-indexed)
    /// * `gen_col` - Column in generated file (0-indexed)
    /// * `src_line` - Line in original .pc file (0-indexed)
    /// * `src_col` - Column in original .pc file (0-indexed)
    /// * `name` - Optional symbol name (e.g., component name)
    pub fn add_mapping(
        &mut self,
        gen_line: u32,
        gen_col: u32,
        src_line: u32,
        src_col: u32,
        name: Option<&str>,
    ) {
        self.builder.add_raw(
            gen_line,
            gen_col,
            0, // source index (always 0 since we have one source file)
            src_line,
            src_col,
            name,
        );
    }

    /// Track position advancement as we emit generated code
    ///
    /// Call this after appending text to the output buffer to keep
    /// track of the current position in the generated file.
    pub fn advance(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                self.current_line += 1;
                self.current_col = 0;
            } else {
                self.current_col += 1;
            }
        }
    }

    /// Get the current position in the generated output
    pub fn current_position(&self) -> (u32, u32) {
        (self.current_line, self.current_col)
    }

    /// Build the final source map
    pub fn build(self) -> Result<ExternalSourceMap, sourcemap::Error> {
        self.builder.into_sourcemap()
    }

    /// Convert to JSON string
    pub fn to_json(self) -> Result<String, sourcemap::Error> {
        let map = self.build()?;
        Ok(map.to_json()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let source = "component Button {}";
        let builder = SourceMapBuilder::new("button.pc", source);
        assert_eq!(builder.current_position(), (0, 0));
    }

    #[test]
    fn test_advance_tracking() {
        let source = "component Button {}";
        let mut builder = SourceMapBuilder::new("button.pc", source);

        builder.advance("const Button = () => {");
        let (line, col) = builder.current_position();
        assert_eq!(line, 0);
        assert_eq!(col, 23);

        builder.advance("\n");
        let (line, col) = builder.current_position();
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_mapping_generation() {
        let source = "component Button {}";
        let mut builder = SourceMapBuilder::new("button.pc", source);

        // Add a mapping: generated position (0,6) -> source position (0,10)
        builder.add_mapping(0, 6, 0, 10, Some("Button"));

        let map = builder.build().unwrap();

        // Verify the source map has our source file
        assert_eq!(map.get_source(0), Some("button.pc"));
    }

    #[test]
    fn test_json_output() {
        let source = "component Button {}";
        let mut builder = SourceMapBuilder::new("button.pc", source);
        builder.add_mapping(0, 0, 0, 0, None);

        let json = builder.to_json().unwrap();

        // Basic validation - should be valid JSON with required fields
        assert!(json.contains("\"version\":3"));
        assert!(json.contains("\"sources\""));
        assert!(json.contains("button.pc"));
    }
}
```

## 4. Create `packages/sourcemap/src/utils.rs`

```rust
/// Convert byte offset to line and column number
///
/// # Arguments
/// * `source` - The source text
/// * `offset` - Byte offset in the source
///
/// # Returns
/// Tuple of (line, column) both 0-indexed
pub fn byte_offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0;
    let mut col = 0;

    for (i, ch) in source.chars().enumerate() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

/// Convert line and column to byte offset
///
/// # Arguments
/// * `source` - The source text
/// * `line` - Line number (0-indexed)
/// * `col` - Column number (0-indexed)
///
/// # Returns
/// Byte offset in the source, or source.len() if out of bounds
pub fn line_col_to_byte_offset(source: &str, target_line: u32, target_col: u32) -> usize {
    let mut line = 0;
    let mut col = 0;

    for (i, ch) in source.chars().enumerate() {
        if line == target_line && col == target_col {
            return i;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_offset_to_line_col() {
        let source = "line 1\nline 2\nline 3";

        // Start of file
        assert_eq!(byte_offset_to_line_col(source, 0), (0, 0));

        // Start of second line
        assert_eq!(byte_offset_to_line_col(source, 7), (1, 0));

        // Middle of second line
        assert_eq!(byte_offset_to_line_col(source, 10), (1, 3));

        // Start of third line
        assert_eq!(byte_offset_to_line_col(source, 14), (2, 0));
    }

    #[test]
    fn test_line_col_to_byte_offset() {
        let source = "line 1\nline 2\nline 3";

        // Start of file
        assert_eq!(line_col_to_byte_offset(source, 0, 0), 0);

        // Start of second line
        assert_eq!(line_col_to_byte_offset(source, 1, 0), 7);

        // Middle of second line
        assert_eq!(line_col_to_byte_offset(source, 1, 3), 10);

        // Start of third line
        assert_eq!(line_col_to_byte_offset(source, 2, 0), 14);
    }

    #[test]
    fn test_roundtrip() {
        let source = "component Button {\n  render button\n}";
        let offset = 15;

        let (line, col) = byte_offset_to_line_col(source, offset);
        let back_to_offset = line_col_to_byte_offset(source, line, col);

        assert_eq!(offset, back_to_offset);
    }

    #[test]
    fn test_unicode_handling() {
        let source = "日本語\ntext";

        // Unicode characters should be handled correctly
        let (line, col) = byte_offset_to_line_col(source, 10); // After "日本語\n"
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_empty_source() {
        let source = "";
        assert_eq!(byte_offset_to_line_col(source, 0), (0, 0));
        assert_eq!(line_col_to_byte_offset(source, 0, 0), 0);
    }

    #[test]
    fn test_out_of_bounds() {
        let source = "short";

        // Out of bounds offset should return last position
        let (line, col) = byte_offset_to_line_col(source, 1000);
        assert_eq!(line, 0);
        assert_eq!(col, 5);

        // Out of bounds line/col should return source.len()
        assert_eq!(line_col_to_byte_offset(source, 10, 0), source.len());
    }
}
```

## 5. Update `Cargo.toml` (workspace root)

Add to `workspace.members`:

```toml
[workspace]
members = [
    "packages/parser",
    "packages/editor",
    "packages/evaluator",
    "packages/workspace",
    "packages/inference",
    "packages/compiler-react",
    "packages/compiler-css",
    "packages/compiler-html",
    "packages/cli",
    "packages/wasm",
    "packages/vision",
    "packages/sourcemap",  # ADD THIS LINE
]
```

Add to `workspace.dependencies`:

```toml
[workspace.dependencies]
# ... existing dependencies
sourcemap = "8.0"  # ADD THIS LINE
```

## 6. Test the Foundation

Run these commands to verify everything works:

```bash
# Build the sourcemap package
cargo build -p paperclip-sourcemap

# Run tests
cargo test -p paperclip-sourcemap

# Check for errors
cargo check --workspace

# Expected output:
# running 13 tests
# test result: ok. 13 passed; 0 failed; 0 ignored
```

## 7. Create README

Create `packages/sourcemap/README.md`:

```markdown
# Paperclip Source Maps

Shared utilities for generating source maps during compilation.

## Usage

```rust
use paperclip_sourcemap::{SourceMapBuilder, byte_offset_to_line_col};

// Create a builder
let source = "component Button {}";
let mut builder = SourceMapBuilder::new("button.pc", source);

// Track generated code position
builder.advance("const Button = () => {\\n");

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

- `new(source_file, source_content)` - Create a new builder
- `add_mapping(gen_line, gen_col, src_line, src_col, name)` - Add a mapping
- `advance(text)` - Track position after emitting text
- `current_position()` - Get current (line, col) in generated output
- `build()` - Build the final SourceMap
- `to_json()` - Build and convert to JSON string

### Helper Functions

- `byte_offset_to_line_col(source, offset)` - Convert byte offset to (line, col)
- `line_col_to_byte_offset(source, line, col)` - Convert (line, col) to byte offset

## Testing

```bash
cargo test -p paperclip-sourcemap
```
```

---

## Next Steps After Foundation

Once you've created these files and tests pass, move to Phase 2:

1. Add sourcemap dependency to `packages/compiler-react/Cargo.toml`:
   ```toml
   paperclip-sourcemap = { path = "../sourcemap" }
   ```

2. Enhance `CompilerContext` in `packages/compiler-react/src/context.rs`

3. Update compiler to use `ctx.add_with_span()`

See [source-maps-implementation.md](./source-maps-implementation.md) for detailed Phase 2 code.
