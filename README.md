# Paperclip-next

**Status:** ✅ Minimal Vertical Slice Complete

A visual component builder for the AI age. Build production-quality UI components with a visual canvas, code editor, and AI assistant working in harmony.

## Minimal Vertical Slice

This repository contains a working proof-of-concept demonstrating the core pipeline:

```
.pc file → Parse → Evaluate → Virtual DOM → Stream → Diff → Patch → DOM
```

### What's Working

#### 🔤 Parser (Rust)
- **Tokenizer** using `logos` with zero-copy string slices
- **Deterministic ID generation** using CRC32 + sequential counters
- **Recursive descent parser** supporting:
  - Components, styles, tokens
  - HTML elements (div, button, etc.)
  - Text nodes and expressions
  - Conditionals and iteration
- **39 passing tests**

#### ⚙️ Evaluator (Rust)
- **AST → Virtual DOM transformation**
- **Semantic Identity** - Stable, hierarchical node IDs
- **Expression evaluation** (literals, variables, binary operators)
- **Component rendering** with inline styles
- **Stable patches** - Semantic ID-based diffing
- **Bundle support** - Cross-file component resolution
- **CSS extraction** - Scoped stylesheets
- **Slot implementation** - Default and inserted content with semantic tracking
- **Dev mode validation** - Zero-overhead warnings for unstable patterns
- **112 passing tests**

#### 🌐 Workspace Server (Rust)
- **gRPC service** with Tonic for streaming
- **File watcher** using notify crate
- **Parse → Evaluate → Stream pipeline**
- **1 passing test**

#### 💻 TypeScript Client
- **Virtual DOM types** matching Rust output
- **Efficient diff algorithm** for minimal updates
- **Patch function** applying DOM changes
- **Interactive demo** showing diff/patch in action

### Quick Start

**Prerequisites:**
- Rust (latest stable)
- Node.js 18+
- Yarn 4.x

**Run the Tests:**
```bash
# Test Rust code
cargo test --workspace

# Results: 150+ tests passing across all packages
# - 39 parser tests
# - 112 evaluator tests (includes slots + validator)
# - 1 workspace test
# - Additional tests in compiler, linter, editor, inference, vision, sourcemap
```

**Run Benchmarks:**
```bash
# Run all benchmarks
cargo bench --workspace

# Results: All performance targets EXCEEDED by 1000x-10000x!
# - Parser: 0.84 µs (simple) to 25 µs (1000 lines)
# - Evaluator: 0.75 µs to 10 µs
# - Full pipeline: ~2.2 µs (parse + evaluate)
# See BENCHMARKS.md for detailed results
```

**Use the CLI:**
```bash
# Initialize a new project
cargo run --package paperclip-cli -- init

# Compile to React + TypeScript
cargo run --package paperclip-cli -- compile --typescript

# Compile to CSS
cargo run --package paperclip-cli -- compile --target css

# Output to stdout
cargo run --package paperclip-cli -- compile --target css --stdout
```

**Or compile programmatically:**
```bash
# Run the React compiler example
cargo run --package paperclip-compiler-react --example simple
```

**Use with Bundlers:**
```bash
# Build WASM loaders for webpack, vite, rollup, esbuild
./build-loaders.sh

# See LOADERS.md for detailed usage
```

**Try the Examples:**
```bash
# Vite + React example (recommended)
cd examples/vite-react
yarn install
yarn dev

# Webpack + React example
cd examples/webpack-react
yarn install
yarn dev

# See examples/README.md for more
```

**Run the TypeScript Demo:**
```bash
# Install dependencies
cd packages/client
yarn install

# Start dev server
yarn dev

# Open browser to http://localhost:3000
```

**Try the gRPC Server:**
```bash
# Build and run
cargo run --bin paperclip-server examples

# Server listens on 127.0.0.1:50051
```

### Example .pc File

```javascript
// examples/button.pc
public component Button {
    render button {
        style {
            padding: 8px 16px
            background: #3366FF
            color: white
            border: none
            border-radius: 4px
        }
        text "Click me"
    }
}
```

This parses into an AST, evaluates into a Virtual DOM, and can be streamed to clients for real-time preview.

### Semantic Identity - Stable Node Tracking

Every VNode in the Virtual DOM has a **semantic ID** that remains stable across refactoring:

```
Card{"Card-0"}::div[id]::h1[id]
Card{"Card-0"}::div[id]::Button{"Button-0"}::button[id]
Card{"Card-0"}::div[id]::Button{"Button-1"}::button[id]
```

**Benefits:**
- 🔄 **Stable patches** - Nodes matched by ID, not position
- 📝 **Refactoring-safe** - IDs survive structural changes
- 🚀 **Zero patches on reorder** - Same content = no updates
- 🎯 **Hierarchical** - Full path from root to node

See `PHASE_3_4_COMPLETE.md` for implementation details.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     .pc Source Files                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              Parser (packages/parser)                    │
│  • Tokenizer (logos)                                     │
│  • Deterministic ID generation (CRC32)                   │
│  • Recursive descent parser                              │
│  • AST with sequential IDs                               │
└──┬───────┬─────────┬──────────┬────────────────────────┘
   │       │         │          │
   │       │         │          ▼
   │       │         │   ┌──────────────────────────────┐
   │       │         │   │ Linter (packages/linter)     │
   │       │         │   │  • Configurable rules        │
   │       │         │   │  • Diagnostics               │
   │       │         │   └──────────────────────────────┘
   │       │         │
   │       │         ▼
   │       │   ┌──────────────────────────────────────────┐
   │       │   │  Inference (packages/inference)          │
   │       │   │  • Multi-pass type inference             │
   │       │   │  • TypeScript/Rust codegen               │
   │       │   └──────────────────────────────────────────┘
   │       │
   │       ▼
   │   ┌─────────────────────────────────────────────────┐
   │   │  Compilers                                       │
   │   │  ┌───────────────────────────────────────────┐  │
   │   │  │ React (packages/compiler-react)           │  │
   │   │  │  • AST → React/JSX + TypeScript           │  │
   │   │  └───────────────────────────────────────────┘  │
   │   │  ┌───────────────────────────────────────────┐  │
   │   │  │ CSS (packages/compiler-css)               │  │
   │   │  │  • AST → Scoped CSS                       │  │
   │   │  └───────────────────────────────────────────┘  │
   │   │  • Source Maps (packages/sourcemap)             │
   │   └─────────────────────────────────────────────────┘
   │
   ▼
┌─────────────────────────────────────────────────────────┐
│            Evaluator (packages/evaluator)                │
│  • AST → Virtual DOM                                     │
│  • Semantic ID generation (hierarchical)                 │
│  • Expression evaluation                                 │
│  • Style application                                     │
│  • Bundle/cross-file resolution                          │
└────┬─────────────┬──────────────────────────────────────┘
     │             │
     │             ▼
     │     ┌──────────────────────────────────────────────┐
     │     │  Editor (packages/editor)                    │
     │     │  • Document lifecycle                        │
     │     │  • Mutation system                           │
     │     │  • Collaboration-ready (CRDT)                │
     │     └──────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│         Workspace Server (packages/workspace)            │
│  • File watching (notify)                                │
│  • gRPC streaming (Tonic)                                │
│  • JSON serialization                                    │
│  • Semantic ID-based patch generation                    │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│         TypeScript Client (packages/client)              │
│  • Virtual DOM differ                                    │
│  • Efficient DOM patcher                                 │
│  • Preview rendering                                     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│         Vision (packages/vision)                         │
│  • Screenshot capture                                    │
│  • Component documentation                               │
│  • @view annotations                                     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│         WASM (packages/wasm)                             │
│  • Browser/Node.js bindings                              │
│  • Bundler integration (Vite, Webpack)                   │
└─────────────────────────────────────────────────────────┘
```

### What's Next

See the full implementation plan in `docs/plans/2026-01-27-feat-paperclip-next-full-rewrite-plan.md`.

**Phase 0 (Architecture Spikes)** - In Progress:
- ✅ Spike 0.1: Parser Performance
- ✅ Spike 0.2: Evaluator + Virtual DOM
- ✅ Spike 0.3: gRPC Streaming Preview Loop
- ✅ Spike 0.6: Semantic Identity Implementation
- ✅ Spike 0.4: Roundtrip Serialization
- ✅ Spike 0.5: Live Component Preview Loading
- ✅ Spike 0.10: Override Path Resolution
- ✅ Spike 0.12: Mutation System + Post-Effects
- ... and more spikes to validate architecture

**Recent Completions:**
- ✅ **Phase 6: Slot Implementation** (January 2026)
  - Component slots with default and inserted content
  - Semantic ID tracking with SlotVariant (Default/Inserted)
  - Multiple named slots support
  - Empty slot handling
  - Flexible syntax: `Card { }` and `Card() { }` both work
  - 112 evaluator tests passing
  - See `PHASE_6_COMPLETE.md` for details

- ✅ **Phase 5: Dev Mode Warnings** (January 2026)
  - Zero-overhead validation framework
  - Auto-generated key detection
  - Duplicate semantic ID detection
  - Production mode bypass (no performance cost)
  - 105 evaluator tests passing
  - See `PHASE_5_COMPLETE.md` for details

- ✅ **Phase 3 & 4: Semantic Identity & Stable Patches** (January 2026)
  - Deterministic ID generation (CRC32 + sequential)
  - Hierarchical semantic IDs for all VNodes
  - Auto-generated component keys
  - Semantic ID-based diffing algorithm
  - 102 evaluator tests passing
  - See `PHASE_3_4_COMPLETE.md` for details

**Phase 1 (Core Engine):**
- Incremental parsing with tree-sitter
- GraphManager for dependency resolution
- ✅ React compiler (packages/compiler-react) with TypeScript definitions
- ✅ CSS compiler (packages/compiler-css)
- ✅ Type inference engine (packages/inference)
- ✅ Linter with configurable rules (packages/linter)
- ✅ Source map generation (packages/sourcemap)
- ✅ Editor with mutation system (packages/editor)
- ✅ Vision screenshot capture (packages/vision)
- ✅ WASM bindings (packages/wasm)
- Yew compiler
- Performance benchmarks (<10ms parse, <20ms evaluate) ✅ **EXCEEDED** by 1000x

**Phase 2 (Designer):**
- Canvas UI (React)
- Component library
- Properties panel
- Visual editing tools

**Phase 3 (MCP Integration):**
- MCP server for Claude
- AI-assisted component generation
- Context-aware editing

## Design Philosophy

> **"Nothing happens by accident."**

Every pixel on the canvas must trace to editable source. This is the same invariant that makes spreadsheets usable, shader editors intelligible, and Figma tolerable despite complexity.

**Key Principles:**
- **.pc files are the source of truth** - not React, not Figma
- **Designers author visually** - canvas generates .pc
- **Engineers register live components** - for interactive behavior
- **AI assists both** - via MCP tools with canvas context

## Package Overview

### Core Packages
- **[packages/parser](packages/parser/README.md)** - Fast, zero-copy parser with 39 tests
- **[packages/evaluator](packages/evaluator/README.md)** - AST → Virtual DOM with 112 tests
- **[packages/cli](packages/cli/README.md)** - Command-line interface

### Compiler Packages
- **[packages/compiler-react](packages/compiler-react/README.md)** - React/JSX + TypeScript output
- **[packages/compiler-css](packages/compiler-css/README.md)** - Scoped CSS generation
- **[packages/sourcemap](packages/sourcemap/README.md)** - Source map utilities

### Tooling Packages
- **[packages/linter](packages/linter/README.md)** - Configurable linting rules
- **[packages/inference](packages/inference/README.md)** - Multi-pass type inference
- **[packages/editor](packages/editor/README.md)** - Document editing with mutation system
- **[packages/vision](packages/vision/README.md)** - Screenshot capture for documentation

### Integration Packages
- **[packages/workspace](packages/workspace/README.md)** - gRPC server with file watching
- **[packages/client](packages/client/README.md)** - TypeScript Virtual DOM client
- **[packages/wasm](packages/wasm/README.md)** - WebAssembly bindings

### Build Tool Integrations
- **[packages/plugin-vite](packages/plugin-vite/README.md)** - Vite plugin
- **[packages/loader-webpack](packages/loader-webpack/README.md)** - Webpack loader

## Contributing

This is a rewrite from scratch. The old codebase serves as reference, but we're building fresh with modern tooling.

**Tech Stack:**
- **CLI:** Rust + clap (command-line interface)
- **Parser:** Rust + logos + recursive descent
- **Evaluator:** Rust (zero-copy, arena allocation)
- **Compilers:**
  - ✅ **React** (AST → JSX/React components + TypeScript definitions)
  - ✅ **CSS** (AST → Scoped stylesheets)
  - 🔲 Yew (coming soon)
  - 🔲 HTML (coming soon)
- **Linter:** Rust + configurable rules
- **Inference:** Multi-pass type inference engine
- **Editor:** Document editing with mutation system (collaboration-ready)
- **Vision:** Screenshot capture for visual documentation
- **Server:** Rust + Tonic (gRPC)
- **Client:** TypeScript + Virtual DOM
- **Source Maps:** Industry-standard source map generation
- **(Future) Designer:** React

## License

MIT
