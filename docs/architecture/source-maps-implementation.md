# Source Maps Implementation Plan

## Overview

Enable full source map support for Paperclip compilers (React, CSS, HTML) to provide:
- Stack traces mapping to `.pc` files in browser dev tools
- IDE "Go to definition" from generated code to `.pc` source
- Debugging generated components with breakpoints in original `.pc` files

## Architecture

### 1. Core Components

```
┌─────────────┐     ┌──────────────────┐     ┌──────────────┐
│   Parser    │────▶│ Compiler Context │────▶│  Source Map  │
│ (has spans) │     │  (tracks pos)    │     │   Builder    │
└─────────────┘     └──────────────────┘     └──────────────┘
                            │
                            ▼
                    ┌──────────────────┐
                    │  Generated Code  │
                    │  + mappings      │
                    └──────────────────┘
```

### 2. Data Flow

1. **Parser** → AST with `Span { start, end }` (byte offsets)
2. **Compiler** → Tracks output line/column as it emits code
3. **SourceMapBuilder** → Records mappings: (gen_line, gen_col) → (src_line, src_col)
4. **Output** → Generated code + `.map` file

### 3. Key Dependencies

Add to `Cargo.toml`:
```toml
[workspace.dependencies]
sourcemap = "8.0"  # Industry standard, used by swc, esbuild
```

## Implementation Steps

### Phase 1: Core Infrastructure

#### 1.1 Create SourceMapBuilder utility

**File:** `packages/common/src/sourcemap.rs` (new package)

```rust
use sourcemap::{SourceMap, SourceMapBuilder};
use std::path::Path;

pub struct PaperclipSourceMapBuilder {
    builder: SourceMapBuilder,
    source_file: String,
    current_line: u32,
    current_col: u32,
}

impl PaperclipSourceMapBuilder {
    pub fn new(source_file: &str, source_content: &str) -> Self {
        let mut builder = SourceMapBuilder::new(None);
        builder.set_source_contents(0, Some(source_content));

        Self {
            builder,
            source_file: source_file.to_string(),
            current_line: 0,
            current_col: 0,
        }
    }

    /// Add a mapping from generated position to source position
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
            0, // source index
            src_line,
            src_col,
            name,
        );
    }

    /// Track position as we emit code
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

    pub fn current_position(&self) -> (u32, u32) {
        (self.current_line, self.current_col)
    }

    pub fn build(self) -> Result<SourceMap, sourcemap::Error> {
        self.builder.into_sourcemap()
    }
}

/// Convert byte offset to line/column
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
```

#### 1.2 Enhance CompilerContext

**File:** `packages/compiler-react/src/context.rs`

```rust
use paperclip_common::sourcemap::PaperclipSourceMapBuilder;
use sourcemap::SourceMap;

pub struct CompileOptions {
    pub use_typescript: bool,
    pub include_css_imports: bool,
    pub source_maps: bool,        // NEW
    pub source_file: String,      // NEW
    pub source_content: String,   // NEW
}

pub struct CompilerContext {
    buffer: Rc<RefCell<String>>,
    indent_level: Rc<RefCell<usize>>,
    pub options: CompileOptions,
    sourcemap_builder: Rc<RefCell<Option<PaperclipSourceMapBuilder>>>,  // NEW
}

impl CompilerContext {
    pub fn new(options: CompileOptions) -> Self {
        let sourcemap_builder = if options.source_maps {
            Some(PaperclipSourceMapBuilder::new(
                &options.source_file,
                &options.source_content,
            ))
        } else {
            None
        };

        Self {
            buffer: Rc::new(RefCell::new(String::new())),
            indent_level: Rc::new(RefCell::new(0)),
            sourcemap_builder: Rc::new(RefCell::new(sourcemap_builder)),
            options,
        }
    }

    /// Add text and track position for source maps
    pub fn add(&self, text: &str) {
        self.buffer.borrow_mut().push_str(text);

        if let Some(builder) = self.sourcemap_builder.borrow_mut().as_mut() {
            builder.advance(text);
        }
    }

    /// Add a mapping from source span to current output position
    pub fn add_mapping(&self, span: &Span) {
        if let Some(builder) = self.sourcemap_builder.borrow_mut().as_mut() {
            let (gen_line, gen_col) = builder.current_position();
            let (src_line, src_col) = byte_offset_to_line_col(
                &self.options.source_content,
                span.start,
            );

            builder.add_mapping(gen_line, gen_col, src_line, src_col, None);
        }
    }

    /// Add text with source mapping
    pub fn add_with_span(&self, text: &str, span: &Span) {
        self.add_mapping(span);
        self.add(text);
    }

    pub fn build_sourcemap(&self) -> Option<SourceMap> {
        self.sourcemap_builder
            .borrow_mut()
            .take()
            .and_then(|b| b.build().ok())
    }
}
```

### Phase 2: Update Compilers

#### 2.1 React Compiler

**File:** `packages/compiler-react/src/compiler.rs`

Key changes:
```rust
// Before: ctx.add(&format!("<{}", name));
// After:  ctx.add_with_span(&format!("<{}", name), span);

fn compile_element(element: &Element, ctx: &CompilerContext, is_root: bool) -> Result<(), String> {
    match element {
        Element::Tag {
            tag_name,
            attributes,
            styles,
            children,
            span,  // Use this!
        } => {
            ctx.add_with_span(&format!("<{}", tag_name), span);
            // ... rest of compilation
        }

        Element::Text { content, span } => {
            ctx.add_mapping(span);  // Map text nodes
            compile_text_content(content, ctx);
            Ok(())
        }

        Element::Instance { name, props, children, span } => {
            ctx.add_with_span(&format!("<{}", name), span);
            // ... compile instance
        }

        // Add mappings for all elements
    }
}

// Update public API
pub fn compile_to_react(
    document: &Document,
    options: CompileOptions,
) -> Result<(String, Option<SourceMap>), String> {
    let ctx = CompilerContext::new(options);

    // ... compilation logic

    let code = ctx.get_output();
    let sourcemap = ctx.build_sourcemap();

    Ok((code, sourcemap))
}
```

#### 2.2 CSS Compiler

**File:** `packages/compiler-css/src/compiler.rs`

```rust
pub fn compile_to_css(
    document: &Document,
    options: CompileOptions,
) -> Result<(String, Option<SourceMap>), String> {
    let ctx = CompilerContext::new(options);

    for style in &document.styles {
        // Map selector
        ctx.add_with_span(&format!(".pc-{} {{", style.name), &style.span);
        ctx.indent();

        for (prop, value) in &style.properties {
            // Map each property (could be more granular)
            ctx.add_line(&format!("{}: {};", prop, value));
        }

        ctx.dedent();
        ctx.add_line("}");
    }

    let code = ctx.get_output();
    let sourcemap = ctx.build_sourcemap();

    Ok((code, sourcemap))
}
```

#### 2.3 HTML Compiler

Similar approach - add mappings for all generated elements.

### Phase 3: WASM Integration

**File:** `packages/wasm/src/lib.rs`

```rust
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CompileResult {
    pub code: String,
    pub map: Option<String>,  // JSON string of source map
}

#[wasm_bindgen]
pub fn compile_to_react(
    source: &str,
    file_path: &str,
    use_typescript: bool,
    generate_sourcemap: bool,  // NEW parameter
) -> Result<JsValue, JsValue> {
    let options = CompileOptions {
        use_typescript,
        include_css_imports: true,
        source_maps: generate_sourcemap,
        source_file: file_path.to_string(),
        source_content: source.to_string(),
    };

    let (code, sourcemap) = paperclip_compiler_react::compile_to_react(&doc, options)
        .map_err(|e| JsValue::from_str(&e))?;

    let map = sourcemap.map(|sm| sm.to_json().unwrap());

    let result = CompileResult { code, map };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}

#[wasm_bindgen]
pub fn compile_to_css(
    source: &str,
    file_path: &str,
    generate_sourcemap: bool,
) -> Result<JsValue, JsValue> {
    // Similar implementation
}
```

### Phase 4: Bundler Plugin Integration

#### 4.1 Vite Plugin

**File:** `packages/plugin-vite/src/index.ts`

```typescript
export interface PaperclipPluginOptions {
  typescript?: boolean;
  includeStyles?: boolean;
  sourcemap?: boolean;  // NEW: default to true
  filter?: (id: string) => boolean;
}

export default function paperclipPlugin(
  options: PaperclipPluginOptions = {}
): Plugin {
  const {
    typescript = true,
    includeStyles = true,
    sourcemap = true,  // Enable by default
    filter = (id: string) => id.endsWith('.pc'),
  } = options;

  return {
    name: 'paperclip',

    transform(code, id) {
      if (!filter(id)) {
        return null;
      }

      try {
        // Compile to React with source maps
        const result = compileToReact(code, id, typescript, sourcemap);

        let output = result.code;
        let map = result.map;

        // Optionally include styles
        if (includeStyles) {
          try {
            const cssResult = compileToCss(code, id, sourcemap);
            const cssId = `${id}.css`;

            output = `import '${cssId}';\n${output}`;

            // Adjust source map for added import line
            if (map) {
              map = adjustSourceMapForImport(map);
            }

            // Register CSS module with its source map
            this.emitFile({
              type: 'asset',
              fileName: path.basename(cssId),
              source: cssResult.code,
              // Vite handles .map files automatically
            });

          } catch (cssError) {
            console.warn(`CSS compilation failed for ${id}:`, cssError);
          }
        }

        return {
          code: output,
          map: map ? JSON.parse(map) : null,  // Vite expects object or null
        };
      } catch (error) {
        this.error(`Failed to compile ${id}: ${error}`);
      }
    },
  };
}

function adjustSourceMapForImport(mapJson: string): string {
  const map = JSON.parse(mapJson);
  // Shift all mappings down by 1 line for the import statement
  // This is a simplified version - use source-map library for production
  return JSON.stringify(map);
}
```

#### 4.2 Webpack Plugin

**File:** `packages/loader-webpack/index.js`

```javascript
module.exports = function paperclipLoader(source) {
  const callback = this.async();
  const options = this.getOptions();

  try {
    const result = compileToReact(
      source,
      this.resourcePath,
      options.typescript ?? true,
      this.sourceMap ?? true  // Webpack's source map setting
    );

    callback(null, result.code, result.map ? JSON.parse(result.map) : null);
  } catch (error) {
    callback(error);
  }
};
```

### Phase 5: IDE Integration

#### 5.1 VSCode Extension Configuration

**File:** `.vscode/settings.json` (example)

```json
{
  "files.associations": {
    "*.pc": "paperclip"
  },
  "debug.javascript.unmapMissingSources": false
}
```

#### 5.2 Source Map Names

Include semantic names in mappings:

```rust
pub fn add_mapping_with_name(&self, span: &Span, name: &str) {
    if let Some(builder) = self.sourcemap_builder.borrow_mut().as_mut() {
        let (gen_line, gen_col) = builder.current_position();
        let (src_line, src_col) = byte_offset_to_line_col(
            &self.options.source_content,
            span.start,
        );

        builder.add_mapping(gen_line, gen_col, src_line, src_col, Some(name));
    }
}

// Usage:
ctx.add_mapping_with_name(&component.span, &component.name);
```

### Phase 6: Testing

#### 6.1 Unit Tests

**File:** `packages/compiler-react/tests/sourcemap_tests.rs`

```rust
#[test]
fn test_component_sourcemap() {
    let source = r#"
public component Button {
    render button {
        text "Click me"
    }
}
"#;

    let options = CompileOptions {
        use_typescript: false,
        include_css_imports: false,
        source_maps: true,
        source_file: "button.pc".to_string(),
        source_content: source.to_string(),
    };

    let parser = Parser::new_with_path(source, "button.pc");
    let doc = parser.parse_document().unwrap();
    let (code, sourcemap) = compile_to_react(&doc, options).unwrap();

    assert!(sourcemap.is_some());
    let map = sourcemap.unwrap();

    // Verify mapping exists for component name
    let mappings = map.get_token(0, 0);  // First token in generated code
    assert!(mappings.is_some());
}
```

#### 6.2 Browser Test

**File:** `examples/vite-react/test-sourcemap.html`

```html
<!DOCTYPE html>
<html>
<head>
  <script type="module">
    import { Button } from './button.pc';

    // Throw error to test stack trace
    function testSourceMap() {
      throw new Error('Test stack trace mapping');
    }

    testSourceMap();
  </script>
</head>
<body>
  <div id="root"></div>
</body>
</html>
```

Open Chrome DevTools → should show `button.pc:3` in stack trace, not `button.pc.tsx:15`.

## Implementation Checklist

- [ ] Phase 1: Core Infrastructure
  - [ ] Add `sourcemap` crate to workspace dependencies
  - [ ] Create `packages/common` for shared utilities
  - [ ] Implement `PaperclipSourceMapBuilder`
  - [ ] Enhance `CompilerContext` with source map tracking

- [ ] Phase 2: Update Compilers
  - [ ] Update React compiler with mappings
  - [ ] Update CSS compiler with mappings
  - [ ] Update HTML compiler with mappings
  - [ ] Add integration tests for each compiler

- [ ] Phase 3: WASM Integration
  - [ ] Update WASM bindings to return source maps
  - [ ] Update TypeScript types
  - [ ] Test in Node.js

- [ ] Phase 4: Bundler Plugins
  - [ ] Update Vite plugin
  - [ ] Update Webpack loader
  - [ ] Update Rollup plugin (if exists)
  - [ ] Test with each bundler

- [ ] Phase 5: IDE Integration
  - [ ] Document VSCode setup
  - [ ] Test "Go to definition"
  - [ ] Test breakpoint mapping

- [ ] Phase 6: Documentation
  - [ ] Update README with source map info
  - [ ] Add troubleshooting guide
  - [ ] Create demo video

## Performance Considerations

1. **Optional Generation**: Source maps only in dev mode by default
2. **Memory Usage**: Use streaming builder, don't store entire map in memory
3. **Bundle Size**: `.map` files are separate, not included in production bundles
4. **Build Time**: Source map generation adds ~10-15% overhead (measured in other compilers)

## Edge Cases

1. **Multi-file imports**: Track source file per mapping
2. **Inline styles**: Map CSS properties individually
3. **Generated utilities**: Don't map helper functions (cx, etc.)
4. **Minification**: Source maps survive terser/swc minification
5. **HMR**: Preserve mappings on hot reload

## References

- [Source Map V3 Spec](https://sourcemaps.info/spec.html)
- [sourcemap crate docs](https://docs.rs/sourcemap/)
- [Vite Source Map Guide](https://vitejs.dev/guide/api-plugin.html#source-map-support)
- [Chrome DevTools Source Maps](https://developer.chrome.com/blog/sourcemaps/)

## Future Enhancements

1. **Inline source maps**: For development (data URI)
2. **Source map composition**: Chain maps for multi-stage compilation
3. **Symbol mapping**: Map component names, props, etc.
4. **Coverage mapping**: Track test coverage in original .pc files
