# Architecture Documentation

## Source Maps Implementation

Complete architectural documentation for implementing source maps in Paperclip compilers.

### Quick Links

| Document | Purpose | Audience |
|----------|---------|----------|
| **[Quick Reference](./source-maps-quick-reference.md)** | Visual org chart, checklists, TL;DR | Start here! |
| **[Starter Scaffold](./source-maps-starter-scaffold.md)** | Copy-paste code to begin | Phase 1 implementation |
| **[Organization](./source-maps-organization.md)** | File structure, phases, dependencies | Project planning |
| **[Implementation](./source-maps-implementation.md)** | Detailed technical specs | Full implementation |

### Reading Order

1. **Start:** [Quick Reference](./source-maps-quick-reference.md) - Get the big picture
2. **Plan:** [Organization](./source-maps-organization.md) - Understand file structure
3. **Code:** [Starter Scaffold](./source-maps-starter-scaffold.md) - Begin Phase 1
4. **Detail:** [Implementation](./source-maps-implementation.md) - Reference during coding

### What You'll Build

```
Input:  button.pc (your source)
        ↓
Output: button.pc.tsx (generated code)
        button.pc.tsx.map (source map)
        ↓
Result: Chrome DevTools shows button.pc:3
        VSCode "Go to definition" works
        Breakpoints map correctly
```

### Project Structure

```
packages/
├── sourcemap/           ← NEW: Phase 1 (Week 1)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── builder.rs
│   │   └── utils.rs
│   └── tests/
│
├── compiler-react/      ← ENHANCED: Phase 2 (Week 2)
├── compiler-css/        ← ENHANCED: Phase 4 (Week 3)
├── compiler-html/       ← ENHANCED: Phase 4 (Week 3)
├── wasm/               ← ENHANCED: Phase 3 (Week 2-3)
├── plugin-vite/        ← ENHANCED: Phase 3 (Week 2-3)
├── loader-webpack/     ← ENHANCED: Phase 5 (Week 4)
└── plugin-rollup/      ← ENHANCED: Phase 5 (Week 4)
```

### Timeline

- **Week 1:** Core utilities (packages/sourcemap)
- **Week 2:** React compiler + WASM integration
- **Week 3:** CSS/HTML compilers + Vite testing
- **Week 4:** All bundlers + documentation

**Total:** ~3-4 weeks, ~2500 LOC

### Key Features

✅ **Stack traces** show `.pc` files instead of `.tsx`
✅ **Breakpoints** work in Chrome DevTools
✅ **Go to definition** jumps to `.pc` source in VSCode
✅ **Component names** preserved in dev tools
✅ **Zero overhead** in production (optional generation)

### Success Criteria

| Test | How to Verify | Status |
|------|---------------|--------|
| Valid source maps | `source-map-validator button.pc.tsx` | ⬜ |
| Stack traces | Throw error, check DevTools console | ⬜ |
| Breakpoints | Set breakpoint in Sources panel | ⬜ |
| VSCode integration | Cmd+Click on component name | ⬜ |
| HMR preservation | Edit .pc file, mappings still work | ⬜ |
| Performance | <10% overhead with source maps | ⬜ |

### Getting Started

```bash
# 1. Read the quick reference
open docs/architecture/source-maps-quick-reference.md

# 2. Create the sourcemap package
mkdir -p packages/sourcemap/src
cd packages/sourcemap

# 3. Copy scaffold files
# See source-maps-starter-scaffold.md

# 4. Build and test
cargo build -p paperclip-sourcemap
cargo test -p paperclip-sourcemap

# 5. Move to Phase 2
# See source-maps-implementation.md
```

### Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Package structure** | Separate `sourcemap/` | Reusable, clean deps |
| **Library** | `sourcemap` crate | Industry standard |
| **Generation** | Optional via flag | Zero prod overhead |
| **Granularity** | Every AST node | Best debugging UX |
| **Format** | External `.map` files | Standard practice |

### Questions?

- **"Where do I start?"** → [Quick Reference](./source-maps-quick-reference.md)
- **"What files do I create?"** → [Starter Scaffold](./source-maps-starter-scaffold.md)
- **"How is this organized?"** → [Organization](./source-maps-organization.md)
- **"How does X work?"** → [Implementation](./source-maps-implementation.md)

---

## Other Architecture Docs

(Add other architecture documents here as you create them)
