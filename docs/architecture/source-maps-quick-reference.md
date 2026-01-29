# Source Maps Quick Reference

## TL;DR

```
New Package:     packages/sourcemap/        (shared utilities)
Enhanced:        compiler-{react,css,html}/ (use sourcemap)
Updated:         wasm/                      (return {code, map})
Integrated:      plugin-{vite,webpack}/     (pass map to bundler)
```

## Visual Organization

```
┌─────────────────────────────────────────────────────────────┐
│                    LAYER 1: CORE UTILITIES                   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         packages/sourcemap/ (NEW)                    │   │
│  │  • PaperclipSourceMapBuilder                         │   │
│  │  • byte_offset_to_line_col()                         │   │
│  │  • Uses: sourcemap crate                             │   │
│  │  • ~400 LOC, no Paperclip deps                       │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │ imports
                            │
┌─────────────────────────────────────────────────────────────┐
│                   LAYER 2: COMPILERS                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │compiler-react│  │compiler-css  │  │compiler-html │      │
│  │              │  │              │  │              │      │
│  │ ENHANCED:    │  │ ENHANCED:    │  │ ENHANCED:    │      │
│  │ • context.rs │  │ • context.rs │  │ • context.rs │      │
│  │   + SourceMap│  │   + SourceMap│  │   + SourceMap│      │
│  │   Builder    │  │   Builder    │  │   Builder    │      │
│  │              │  │              │  │              │      │
│  │ • compiler.rs│  │ • compiler.rs│  │ • compiler.rs│      │
│  │   use        │  │   use        │  │   use        │      │
│  │   add_with_  │  │   add_with_  │  │   add_with_  │      │
│  │   span()     │  │   span()     │  │   span()     │      │
│  │              │  │              │  │              │      │
│  │ NEW:         │  │ NEW:         │  │ NEW:         │      │
│  │ • sourcemap  │  │ • sourcemap  │  │ • sourcemap  │      │
│  │   .rs        │  │   .rs        │  │   .rs        │      │
│  │ • tests/     │  │ • tests/     │  │ • tests/     │      │
│  │   sourcemap_ │  │   sourcemap_ │  │   sourcemap_ │      │
│  │   tests.rs   │  │   tests.rs   │  │   tests.rs   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │ links to
                            │
┌─────────────────────────────────────────────────────────────┐
│                    LAYER 3: WASM BINDINGS                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │            packages/wasm/                            │   │
│  │                                                      │   │
│  │  ENHANCED:                                           │   │
│  │  • lib.rs                                            │   │
│  │    compile_to_react(..., sourcemap: bool)           │   │
│  │    → Result<JsValue>                                 │   │
│  │                                                      │   │
│  │  NEW:                                                │   │
│  │  • types.rs                                          │   │
│  │    struct CompileResult {                            │   │
│  │      code: String,                                   │   │
│  │      map: Option<String>  // JSON source map        │   │
│  │    }                                                 │   │
│  │                                                      │   │
│  │  • pkg/types.d.ts                                    │   │
│  │    interface CompileResult {                         │   │
│  │      code: string;                                   │   │
│  │      map: string | null;                             │   │
│  │    }                                                 │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │ imports
                            │
┌─────────────────────────────────────────────────────────────┐
│                   LAYER 4: BUNDLER PLUGINS                   │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐    │
│  │ plugin-vite   │  │loader-webpack │  │plugin-rollup │    │
│  │               │  │               │  │              │    │
│  │ ENHANCED:     │  │ ENHANCED:     │  │ ENHANCED:    │    │
│  │ transform() { │  │ module.exports│  │ transform()  │    │
│  │   result =    │  │  = function() │  │  {           │    │
│  │     compile   │  │  {            │  │   result =   │    │
│  │     ToReact(  │  │   result =    │  │     compile  │    │
│  │       ...,    │  │     compile   │  │     ToReact( │    │
│  │       true    │  │     ToReact(  │  │       ...,   │    │
│  │     );        │  │       ...,    │  │       true   │    │
│  │               │  │       this.   │  │     );       │    │
│  │   return {    │  │       source  │  │              │    │
│  │     code,     │  │       Map     │  │   return {   │    │
│  │     map: JSON │  │     );        │  │     code,    │    │
│  │       .parse  │  │               │  │     map      │    │
│  │       (result │  │   callback(   │  │   };         │    │
│  │       .map)   │  │     null,     │  │ }            │    │
│  │   };          │  │     code,     │  │              │    │
│  │ }             │  │     map       │  │              │    │
│  │               │  │   );          │  │              │    │
│  │               │  │ }             │  │              │    │
│  └───────────────┘  └───────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      LAYER 5: OUTPUT                         │
│                                                              │
│  button.pc.tsx  ────┐                                        │
│  (generated code)   │                                        │
│                     ├──→  Browser DevTools                   │
│  button.pc.tsx.map  │     • Stack traces → button.pc        │
│  (source map)  ─────┘     • Breakpoints in button.pc        │
│                           • "Go to definition" works         │
└─────────────────────────────────────────────────────────────┘
```

## File Organization

### Phase 1 Files (Week 1)
```
CREATE  packages/sourcemap/Cargo.toml
CREATE  packages/sourcemap/src/lib.rs
CREATE  packages/sourcemap/src/builder.rs
CREATE  packages/sourcemap/src/utils.rs
CREATE  packages/sourcemap/tests/builder_test.rs
MODIFY  Cargo.toml (add to workspace.members)
```

### Phase 2 Files (Week 2)
```
MODIFY  packages/compiler-react/Cargo.toml (add sourcemap dep)
MODIFY  packages/compiler-react/src/context.rs
MODIFY  packages/compiler-react/src/compiler.rs
CREATE  packages/compiler-react/src/sourcemap.rs
CREATE  packages/compiler-react/tests/sourcemap_tests.rs
```

### Phase 3 Files (Week 2-3)
```
MODIFY  packages/wasm/src/lib.rs
CREATE  packages/wasm/src/types.rs
MODIFY  packages/wasm/pkg/types.d.ts
MODIFY  packages/plugin-vite/src/index.ts
CREATE  packages/plugin-vite/test/sourcemap.test.ts
CREATE  examples/sourcemap-demo/
```

### Phase 4 Files (Week 3)
```
(Same pattern for compiler-css and compiler-html)
```

### Phase 5 Files (Week 4)
```
MODIFY  packages/loader-webpack/index.js
MODIFY  packages/plugin-rollup/src/index.ts
CREATE  tests for each
```

## Implementation Checklist

### Phase 1: Foundation ✓
- [ ] Create `packages/sourcemap/` directory
- [ ] Add Cargo.toml with sourcemap dependency
- [ ] Implement `PaperclipSourceMapBuilder`
- [ ] Implement `byte_offset_to_line_col()`
- [ ] Write unit tests
- [ ] Add to workspace Cargo.toml

**Estimated time:** 2-3 days
**LOC:** ~400 new

### Phase 2: React Compiler ✓
- [ ] Add sourcemap dependency to compiler-react
- [ ] Enhance `CompilerContext` with SourceMapBuilder
- [ ] Add `add_with_span()` method
- [ ] Update `compile_element()` to track positions
- [ ] Update `compile_component()` to track positions
- [ ] Update `compile_expression()` to track positions
- [ ] Write integration tests
- [ ] Verify with manual test

**Estimated time:** 3-4 days
**LOC:** ~600 modified/new

### Phase 3: WASM + Vite ✓
- [ ] Create `CompileResult` type in wasm
- [ ] Update WASM bindings to return {code, map}
- [ ] Update TypeScript type definitions
- [ ] Modify Vite plugin to pass map to Vite
- [ ] Create browser test example
- [ ] Test in Chrome DevTools

**Estimated time:** 2-3 days
**LOC:** ~300 modified/new

### Phase 4: CSS + HTML ✓
- [ ] Repeat Phase 2 for compiler-css
- [ ] Repeat Phase 2 for compiler-html
- [ ] Write integration tests for both
- [ ] Verify CSS source maps in DevTools

**Estimated time:** 4-5 days
**LOC:** ~800 modified/new

### Phase 5: All Bundlers ✓
- [ ] Update Webpack loader
- [ ] Update Rollup plugin
- [ ] Test with each bundler
- [ ] Write integration tests

**Estimated time:** 2-3 days
**LOC:** ~200 modified/new

### Phase 6: Documentation ✓
- [ ] Write debugging guide
- [ ] Update all package READMEs
- [ ] Create demo project
- [ ] Record demo video

**Estimated time:** 2 days
**LOC:** ~1000 words docs

## Quick Commands

### Setup
```bash
# Create sourcemap package
mkdir -p packages/sourcemap/src packages/sourcemap/tests
cd packages/sourcemap

# Initialize with Cargo
cargo init --lib

# Add dependency
cargo add sourcemap
```

### Build
```bash
# Build just sourcemap package
cargo build -p paperclip-sourcemap

# Build all compilers with sourcemap
cargo build --workspace

# Run tests
cargo test -p paperclip-sourcemap
cargo test -p paperclip-compiler-react
```

### Test
```bash
# Test in browser (after Phase 3)
cd examples/sourcemap-demo
npm install
npm run dev
# Open DevTools, trigger error, see .pc file in stack trace
```

## Key Design Decisions

### 1. **Separate Package vs Inline**
   - ✅ **Decision:** Separate `packages/sourcemap` package
   - **Rationale:** Reusable across all compilers, clean dependency graph
   - **Trade-off:** One more package to maintain, but worth it

### 2. **Source Map Library**
   - ✅ **Decision:** Use `sourcemap` crate
   - **Rationale:** Industry standard, used by swc/esbuild
   - **Alternative:** Custom implementation (too much work)

### 3. **When to Generate**
   - ✅ **Decision:** Optional via `CompileOptions.source_maps`
   - **Rationale:** Zero overhead in production
   - **Default:** true in dev, false in prod

### 4. **Granularity**
   - ✅ **Decision:** Map every AST node with a span
   - **Rationale:** Best debugging experience
   - **Alternative:** Only map components (insufficient)

### 5. **Inline vs External**
   - ✅ **Decision:** External `.map` files
   - **Rationale:** Standard practice, smaller bundles
   - **Alternative:** Inline data URI (future enhancement)

## Common Pitfalls to Avoid

### ❌ DON'T: Map generated helper code
```rust
// DON'T add mappings for utility functions
ctx.add("const cx = (...classes) => classes.join(' ');");
// This is generated code, not from .pc source
```

### ✅ DO: Map original source elements
```rust
// DO add mappings for user's components
ctx.add_with_span(&format!("<{}", tag_name), &element.span);
```

### ❌ DON'T: Forget to advance position
```rust
// DON'T
ctx.buffer.push_str(text);  // Bypasses position tracking
```

### ✅ DO: Use context methods
```rust
// DO
ctx.add(text);  // Automatically tracks position
```

### ❌ DON'T: Map to byte offsets directly
```rust
// DON'T - source maps need line/col
builder.add_mapping(gen_line, gen_col, span.start, 0, None);
```

### ✅ DO: Convert byte offsets to line/col
```rust
// DO
let (src_line, src_col) = byte_offset_to_line_col(source, span.start);
builder.add_mapping(gen_line, gen_col, src_line, src_col, None);
```

## Performance Tips

1. **Lazy generation:** Only build source map when needed
2. **Streaming:** Don't buffer entire map in memory
3. **Caching:** Reuse line/col conversions
4. **Benchmarking:** Track overhead in CI

## Debug Commands

### Validate Source Map
```bash
npm install -g source-map-validator
source-map-validator button.pc.tsx
# Should output: "✓ button.pc.tsx.map is valid"
```

### Inspect Mappings
```bash
npm install -g source-map-cli
source-map resolve button.pc.tsx.map 10 5
# Should output: button.pc:3:2
```

### Browser Console
```javascript
// In DevTools console
console.trace();
// Stack trace should show button.pc:3 not button.pc.tsx:15
```

## Success Criteria

| Criterion | How to Verify |
|-----------|---------------|
| ✅ Stack traces show .pc files | Throw error in component, check DevTools |
| ✅ Breakpoints work | Set breakpoint in DevTools Sources panel |
| ✅ "Go to definition" works | Cmd+Click in VSCode on component name |
| ✅ Source map validates | Run source-map-validator |
| ✅ HMR preserves mappings | Edit .pc file, check DevTools still works |
| ✅ Performance overhead <10% | Run benchmarks with/without source maps |

## Next Steps

1. Read [source-maps-implementation.md](./source-maps-implementation.md) for detailed code
2. Start with Phase 1: Create `packages/sourcemap/`
3. Follow checklist above
4. Test at each phase before moving on
5. Celebrate when stack traces show .pc files! 🎉
