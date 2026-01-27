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
- **Recursive descent parser** supporting:
  - Components, styles, tokens
  - HTML elements (div, button, etc.)
  - Text nodes and expressions
  - Conditionals and iteration
- **12 passing tests**

#### ⚙️ Evaluator (Rust)
- **AST → Virtual DOM transformation**
- **Expression evaluation** (literals, variables, binary operators)
- **Component rendering** with inline styles
- **2 passing tests**

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
- npm or yarn

**Run the Tests:**
```bash
# Test Rust code
cargo test --workspace

# Results: 15 tests passing
# - 12 parser tests
# - 2 evaluator tests
# - 1 watcher test
```

**Run the TypeScript Demo:**
```bash
# Install dependencies
cd client
npm install

# Start dev server
npm run dev

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

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     .pc Source Files                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                 Parser (libs/parser)                     │
│  • Tokenizer (logos)                                     │
│  • Recursive descent parser                              │
│  • AST generation                                        │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│               Evaluator (libs/evaluator)                 │
│  • AST → Virtual DOM                                     │
│  • Expression evaluation                                 │
│  • Style application                                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│            Workspace Server (libs/workspace)             │
│  • File watching (notify)                                │
│  • gRPC streaming (Tonic)                                │
│  • JSON serialization                                    │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              TypeScript Client (client/)                 │
│  • Virtual DOM differ                                    │
│  • Efficient DOM patcher                                 │
│  • Preview rendering                                     │
└─────────────────────────────────────────────────────────┘
```

### What's Next

See the full implementation plan in `docs/plans/2026-01-27-feat-paperclip-next-full-rewrite-plan.md`.

**Phase 0 (Architecture Spikes)** - In Progress:
- ✅ Spike 0.1: Parser Performance
- ✅ Spike 0.2: Evaluator + Virtual DOM
- ✅ Spike 0.3: gRPC Streaming Preview Loop (partial)
- 🔲 Spike 0.4: Roundtrip Serialization
- 🔲 Spike 0.5: Live Component Preview Loading
- ... and 16 more spikes to validate architecture

**Phase 1 (Core Engine):**
- Incremental parsing with tree-sitter
- GraphManager for dependency resolution
- React/Yew compilers
- Performance benchmarks (<10ms parse, <20ms evaluate)

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

## Contributing

This is a rewrite from scratch. The old codebase serves as reference, but we're building fresh with modern tooling.

**Tech Stack:**
- Parser: Rust + logos + recursive descent
- Evaluator: Rust (zero-copy, arena allocation)
- Server: Rust + Tonic (gRPC)
- Client: TypeScript + Virtual DOM
- (Future) Designer: React
- (Future) Compilers: React, Yew, HTML/CSS

## License

MIT
