# Source Maps Organization Structure

## Overview

This document outlines how source map functionality will be organized across the codebase.

## Directory Structure

```
paperclip-next/
├── packages/
│   ├── sourcemap/                    ← NEW: Shared source map utilities
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs               # Public API
│   │   │   ├── builder.rs           # SourceMapBuilder
│   │   │   ├── utils.rs             # Helper functions
│   │   │   └── tests/
│   │   │       └── integration.rs
│   │   └── README.md
│   │
│   ├── compiler-react/               ← ENHANCED: Add source map support
│   │   ├── src/
│   │   │   ├── compiler.rs          # Update compile_element()
│   │   │   ├── context.rs           # Add SourceMapBuilder integration
│   │   │   └── sourcemap.rs         # NEW: React-specific mapping logic
│   │   └── tests/
│   │       └── sourcemap_tests.rs   # NEW: Source map tests
│   │
│   ├── compiler-css/                 ← ENHANCED: Add source map support
│   │   ├── src/
│   │   │   ├── compiler.rs
│   │   │   ├── context.rs
│   │   │   └── sourcemap.rs         # NEW
│   │   └── tests/
│   │       └── sourcemap_tests.rs
│   │
│   ├── compiler-html/                ← ENHANCED: Add source map support
│   │   ├── src/
│   │   │   ├── compiler.rs
│   │   │   ├── context.rs
│   │   │   └── sourcemap.rs         # NEW
│   │   └── tests/
│   │       └── sourcemap_tests.rs
│   │
│   ├── wasm/                         ← ENHANCED: Expose source maps to JS
│   │   ├── src/
│   │   │   ├── lib.rs               # Update bindings
│   │   │   └── types.rs             # NEW: CompileResult type
│   │   └── pkg/
│   │       └── types.d.ts           # TypeScript definitions
│   │
│   ├── plugin-vite/                  ← ENHANCED: Return source maps
│   │   ├── src/
│   │   │   └── index.ts             # Update transform()
│   │   └── test/
│   │       └── sourcemap.test.ts    # NEW
│   │
│   ├── loader-webpack/               ← ENHANCED: Return source maps
│   │   ├── index.js
│   │   └── test/
│   │       └── sourcemap.test.js    # NEW
│   │
│   └── plugin-rollup/                ← ENHANCED: Return source maps
│       ├── src/
│       │   └── index.ts
│       └── test/
│           └── sourcemap.test.ts
│
├── docs/
│   ├── architecture/
│   │   ├── source-maps-implementation.md  # Implementation details
│   │   └── source-maps-organization.md    # This file
│   └── guides/
│       └── debugging-with-sourcemaps.md   # User guide
│
├── examples/
│   ├── vite-react/
│   │   └── test-sourcemap.html       # Browser test
│   └── sourcemap-demo/               # NEW: Dedicated demo
│       ├── src/
│       │   └── button.pc
│       ├── index.html
│       └── README.md
│
└── Cargo.toml                        # Update workspace members
```

## Package Responsibilities

### 1. `packages/sourcemap` (NEW)

**Purpose:** Shared source map utilities used by all compilers

**Public API:**
```rust
// lib.rs
pub struct SourceMapBuilder { /* ... */ }
pub fn byte_offset_to_line_col(source: &str, offset: usize) -> (u32, u32);
pub fn line_col_to_byte_offset(source: &str, line: u32, col: u32) -> usize;
```

**Dependencies:**
- `sourcemap` crate (external)
- No other Paperclip packages

**Size:** ~500 lines of code

**Files:**
```
src/
├── lib.rs           # Public API, re-exports
├── builder.rs       # PaperclipSourceMapBuilder impl
├── utils.rs         # Helper functions
└── tests/
    └── integration.rs  # Builder tests
```

### 2. `packages/compiler-react` (ENHANCED)

**New functionality:**
- Track source positions during code generation
- Generate mappings for components, elements, expressions
- Return `(String, Option<SourceMap>)` instead of `String`

**New files:**
- `src/sourcemap.rs` - React-specific mapping helpers
- `tests/sourcemap_tests.rs` - Integration tests

**Changes to existing files:**
- `src/context.rs` - Add `SourceMapBuilder` field
- `src/compiler.rs` - Use `ctx.add_with_span()` everywhere

**Dependencies added:**
- `paperclip-sourcemap = { path = "../sourcemap" }`

### 3. `packages/compiler-css` (ENHANCED)

**New functionality:**
- Track source positions for selectors and properties
- Generate CSS source maps
- Support inline styles and external stylesheets

**Structure:** Same as compiler-react

### 4. `packages/compiler-html` (ENHANCED)

**New functionality:**
- Track source positions for HTML elements
- Generate HTML source maps

**Structure:** Same as compiler-react

### 5. `packages/wasm` (ENHANCED)

**New functionality:**
- Return `{ code: string, map: string | null }` to JS
- Expose source map generation flag

**Changes:**
```rust
// lib.rs
#[wasm_bindgen]
pub fn compile_to_react(
    source: &str,
    file_path: &str,
    use_typescript: bool,
    generate_sourcemap: bool,  // NEW
) -> Result<JsValue, JsValue>

// types.rs (NEW)
#[derive(Serialize, Deserialize)]
pub struct CompileResult {
    pub code: String,
    pub map: Option<String>,
}
```

### 6. `packages/plugin-vite` (ENHANCED)

**Changes:**
- Call WASM with `sourcemap: true`
- Return `{ code, map }` from transform()
- Handle CSS source maps

**New files:**
- `test/sourcemap.test.ts` - Vite plugin tests

### 7. `packages/loader-webpack` (ENHANCED)

**Changes:**
- Call WASM with source map flag
- Pass map to webpack via callback

**New files:**
- `test/sourcemap.test.js` - Webpack loader tests

## Dependency Graph

```
┌──────────────┐
│   sourcemap  │ ← Core utilities (no deps)
└──────┬───────┘
       │
       ├──────────────┬──────────────┬──────────────┐
       ▼              ▼              ▼              ▼
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
│  compiler  │ │  compiler  │ │  compiler  │ │    cli     │
│   -react   │ │   -css     │ │   -html    │ │            │
└──────┬─────┘ └──────┬─────┘ └──────┬─────┘ └──────┬─────┘
       │              │              │              │
       └──────────────┴──────────────┴──────────────┘
                      │
                      ▼
               ┌────────────┐
               │    wasm    │
               └──────┬─────┘
                      │
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
┌────────────┐ ┌────────────┐ ┌────────────┐
│plugin-vite │ │   loader   │ │   plugin   │
│            │ │  -webpack  │ │  -rollup   │
└────────────┘ └────────────┘ └────────────┘
```

## Implementation Phases

### Phase 1: Foundation (Week 1)

**Goal:** Create core infrastructure

**Files to create:**
```
packages/sourcemap/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── builder.rs
│   └── utils.rs
└── tests/
    └── builder_test.rs
```

**Files to modify:**
```
Cargo.toml (add sourcemap to workspace.members)
```

**Estimated LOC:** ~400 new lines

### Phase 2: React Compiler (Week 2)

**Goal:** Full source map support in React compiler

**Files to create:**
```
packages/compiler-react/src/sourcemap.rs
packages/compiler-react/tests/sourcemap_tests.rs
```

**Files to modify:**
```
packages/compiler-react/Cargo.toml (add sourcemap dep)
packages/compiler-react/src/context.rs
packages/compiler-react/src/compiler.rs
```

**Estimated LOC:** ~600 modified/new lines

**Verification:**
- Unit tests pass
- Generated source maps validate with source-map-validator

### Phase 3: WASM + Vite (Week 2-3)

**Goal:** End-to-end integration with dev tooling

**Files to create:**
```
packages/wasm/src/types.rs
packages/plugin-vite/test/sourcemap.test.ts
examples/sourcemap-demo/
```

**Files to modify:**
```
packages/wasm/src/lib.rs
packages/wasm/pkg/types.d.ts
packages/plugin-vite/src/index.ts
```

**Estimated LOC:** ~300 modified/new lines

**Verification:**
- Browser test shows correct source in DevTools
- Stack traces map to .pc files
- Vite HMR preserves source maps

### Phase 4: CSS & HTML Compilers (Week 3)

**Goal:** Complete source map coverage

**Files to create:**
```
packages/compiler-css/src/sourcemap.rs
packages/compiler-css/tests/sourcemap_tests.rs
packages/compiler-html/src/sourcemap.rs
packages/compiler-html/tests/sourcemap_tests.rs
```

**Files to modify:**
- CSS compiler context and main compiler
- HTML compiler context and main compiler

**Estimated LOC:** ~400 modified/new lines each

### Phase 5: Bundler Coverage (Week 4)

**Goal:** Support all bundlers

**Files to modify:**
```
packages/loader-webpack/index.js
packages/plugin-rollup/src/index.ts
```

**Files to create:**
```
packages/loader-webpack/test/sourcemap.test.js
packages/plugin-rollup/test/sourcemap.test.ts
```

**Estimated LOC:** ~200 modified/new lines

### Phase 6: Documentation & Polish (Week 4)

**Files to create:**
```
docs/guides/debugging-with-sourcemaps.md
examples/sourcemap-demo/README.md
```

**Files to update:**
```
README.md (add source map section)
packages/*/README.md (document sourcemap options)
```

## Testing Strategy

### Unit Tests (Rust)

**Location:** `packages/*/tests/`

**Coverage:**
- Source map builder utilities
- Byte offset ↔ line/column conversion
- Mapping generation for each AST node type
- Edge cases (empty files, long lines, Unicode)

**Example:**
```rust
// packages/compiler-react/tests/sourcemap_tests.rs
#[test]
fn test_simple_component_mapping() {
    let source = "component Button { render button }";
    let (code, map) = compile_with_sourcemap(source);

    assert!(map.is_some());
    let mappings = map.unwrap().lookup_token(0, 0).unwrap();
    assert_eq!(mappings.get_source(), Some("button.pc"));
}
```

### Integration Tests (TypeScript)

**Location:** `packages/plugin-*/test/`

**Coverage:**
- Bundler plugins return valid source maps
- Source maps validate with tools
- HMR preserves mappings

**Example:**
```typescript
// packages/plugin-vite/test/sourcemap.test.ts
import { transform } from '../src/index';

test('returns valid source map', () => {
  const result = transform('component Foo {}', 'foo.pc');
  expect(result.map).toBeDefined();
  expect(result.map.sources).toContain('foo.pc');
});
```

### Browser Tests (Manual)

**Location:** `examples/sourcemap-demo/`

**Tests:**
- Chrome DevTools shows .pc files in Sources
- Stack traces map correctly
- Breakpoints work in .pc files
- "Go to definition" works in VSCode

## Configuration

### Workspace Cargo.toml

```toml
[workspace]
members = [
    # ... existing members
    "packages/sourcemap",  # ADD
]

[workspace.dependencies]
# ... existing dependencies
sourcemap = "8.0"  # ADD
```

### Compiler Options

```rust
// All compilers expose this option
pub struct CompileOptions {
    // ... existing options
    pub source_maps: bool,          // Enable/disable
    pub source_file: String,        // Original filename
    pub source_content: String,     // Original source
}
```

### Bundler Plugin Options

```typescript
// Vite, Webpack, Rollup
interface PaperclipPluginOptions {
  // ... existing options
  sourcemap?: boolean;  // Default: true in dev, false in prod
}
```

## File Size Impact

### Development Build
- Source map utility: ~30KB (compiled)
- Per-compiler overhead: ~5KB each
- Generated .map files: ~2-5x source size
- **Total overhead:** ~50KB in binaries, .map files separate

### Production Build
- Source maps not included by default
- Zero runtime overhead
- Optional separate .map file deployment

## Migration Path

### Existing Code

No breaking changes! Existing code continues to work:

```rust
// Old API (still works)
let code = compile_to_react(&doc, options)?;

// New API (returns tuple)
let (code, map) = compile_to_react(&doc, options)?;
```

### Bundler Plugins

Automatic upgrade:
- Source maps enabled by default in dev mode
- Disabled in production
- Override with `sourcemap: false` option

## Maintenance

### Code Ownership
- **packages/sourcemap:** Core team
- **Compiler integration:** Compiler maintainers
- **Bundler plugins:** Plugin maintainers

### Version Compatibility
- Source maps are additive - no breaking changes
- Follow sourcemap crate semver
- Test with major browser updates

## Performance Budget

### Build Time Impact
- **Target:** <10% overhead with source maps enabled
- **Measurement:** Benchmark suite in each compiler
- **Monitoring:** CI fails if overhead >15%

### Memory Impact
- **Target:** <5MB additional heap during compilation
- **Implementation:** Streaming builder, no buffering
- **Monitoring:** Memory profiling in benchmarks

## Success Metrics

### Technical Metrics
- ✅ 100% of generated code has mappings
- ✅ Source map validates with source-map-validator
- ✅ All bundlers return source maps correctly

### User Experience Metrics
- ✅ Stack traces show .pc file:line
- ✅ Breakpoints work in Chrome DevTools
- ✅ "Go to definition" works in VSCode
- ✅ Zero complaints about incorrect mappings

## Related Documents

- [Implementation Details](./source-maps-implementation.md) - Technical implementation guide
- [Debugging Guide](../guides/debugging-with-sourcemaps.md) - User-facing documentation
- [API Documentation](../../packages/sourcemap/README.md) - Source map builder API
