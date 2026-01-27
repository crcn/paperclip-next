---
title: "feat: Paperclip-next Full Project Rewrite"
type: feat
date: 2026-01-27
status: draft
timeline: weeks (focused sprint)
approach: rewrite from scratch using old codebase as reference
deepened: 2026-01-27
---

## Enhancement Summary

**Deepened on:** 2026-01-27
**Research agents used:** 14 (Architecture Strategist, Performance Oracle, Security Sentinel, TypeScript Reviewer, Agent-Native Reviewer, Frontend Races Reviewer, Pattern Recognition, Simplicity Reviewer, MCP Best Practices, Live Component Research, Rust Parser Research, React Canvas Research, Agent-Native Skill, Frontend Design)

### Key Improvements

1. **Parser Architecture:** Use `logos` (lexer) + `chumsky` (parser combinator) with `bumpalo` arena allocation for 10x performance improvement
2. **Real-time Preview:** Virtual DOM diffing/patching via gRPC streaming - NO recompilation on each change (<40ms total)
3. **State Management:** Machine + Engines pattern (same as Shay app and legacy Paperclip) - event-driven, domain engines for side effects
4. **MCP Tools Expanded:** Added `delete_component`, `update_token`, `create_token`, `delete_token`, `undo`, `redo` for complete CRUD
5. **Security Hardening:** Path traversal validation, sandboxed preview execution, input sanitization
6. **Race Condition Prevention:** Optimistic UI with rollback, operation queuing, interaction locking during mutations
7. **Designer-First Architecture:** Inverted model where .pc is the entry point; engineers register live components for interactive behavior
8. **Live Component System:** Hybrid rendering with Virtual DOM for static content + real JS components for interactive elements (maps, 3D, etc.)
9. **Dual-View Editing:** Canvas shows rendered output, Props pane shows source expressions (Excel formula bar model)
10. **Expression Language:** Formula-like expressions only - no control flow, no side effects, no async
11. **Integrated AI Agent:** Cursor-style chat assistant embedded in designer, context-aware of canvas/selection, MCP-powered

### New Considerations Discovered

- **Simplicity Focus:** Consider 50-60% scope reduction for true MVP - core loop is parse→evaluate→compile
- **Agent-Native Design:** Every visual action must have MCP tool equivalent for AI parity
- **GraphManager Component:** Centralized dependency graph management for incremental updates
- **Canvas Architecture:** DOM-based rendering preferred over HTML Canvas for accessibility
- **The Core Invariant:** Every pixel on canvas must trace to an editable source in the props pane
- **Design Philosophy:** "Nothing happens by accident" - power must show its source
- **Dogfooding Principle:** The designer itself must be built with Paperclip - this is the litmus test for language expressiveness

---

# Paperclip-next: Visual Component Builder for the AI Age

## Overview

Revive and rewrite the Paperclip visual component builder with a fresh codebase, maintaining functional parity with the original while adding MCP (Model Context Protocol) integration for AI-assisted development.

**Vision:** Visual canvas + Code editor + AI assistant → Production components

**Target User:** Design engineers who want visual tools that produce real, production-quality code.

### Designer-Native UX (Figma/Framer Mental Model)

**.pc files are designed to map directly to Figma/Framer UX patterns.** The designer experience should feel immediately familiar to anyone who uses modern design tools.

| Figma/Framer Concept | Paperclip Equivalent |
|---------------------|---------------------|
| **Frame** | Component with `@frame(x, y, width, height)` |
| **Auto Layout** | `layout: vertical/horizontal` with gap/padding |
| **Component** | `public component Name { ... }` |
| **Instance** | `<ComponentName />` reference |
| **Variant** | `variant hover { ... }` with triggers |
| **Slot** | `slot children` / `slot header` |
| **Styles Panel** | `style { ... }` block |
| **Design Tokens** | `public token colors.primary = #007AFF` |
| **Constraints** | `width: fill` / `height: hug` |
| **Layers Panel** | Component tree in left sidebar |
| **Properties Panel** | Props pane showing expressions |

**Key UX Principles:**

1. **No learning curve for designers** - If you know Figma, you know Paperclip's canvas
2. **Same interactions** - Click to select, drag to move, handles to resize, double-click to edit text
3. **Same panels** - Layers left, properties right, canvas center
4. **Same shortcuts** - Cmd+D duplicate, Cmd+G group, V for move, T for text
5. **Frames = Artboards** - Multiple components visible on infinite canvas via `@frame`

**What designers DON'T need to know:**
- The .pc syntax (canvas edits it for them)
- How compilation works
- TypeScript/React internals
- Git (though it helps)

**What designers gain:**
- Their designs ARE the production code
- No handoff, no "rebuild in code"
- Real responsive behavior, not just mockups
- Engineers can't break the design (it's the source)

## Problem Statement

The design-to-code gap remains broken:
- Figma outputs garbage code
- Visual builders lock you into platforms
- AI generates code blindly without visual feedback
- No tool lets you fluidly move between visual, code, and AI

**The common objection:** *"Why not just use code + Storybook?"*

**The sharp answer:** Storybook documents components after they exist. Paperclip is where components are authored visually, with source-level traceability, before code hardens.

**The core insight:** Paperclip is a system that makes **UI causality visible**. Every pixel traces to editable source. Nothing happens by accident.

## Proposed Solution

A multi-modal component builder where:
1. **Visual** - Drag, drop, click, adjust on a canvas
2. **Code** - Write .pc files directly (or in IDE)
3. **AI** - Claude generates/edits via MCP tools

All three stay in sync. Output compiles to framework-native code (React, Yew, HTML/CSS) with zero runtime.

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│   Visual Canvas  ←→  .pc Source Files  ←→  Claude (MCP)│
│                                                         │
│                         ↓                               │
│                                                         │
│              React / Yew / HTML+CSS Output              │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## Designer-First Architecture (The Inverted Model)

### The Core Insight

Traditional tools: Engineers build components → Designers use them → Export doesn't work

**Paperclip inverts this:** Designers use .pc as the primary authoring environment → Engineers register "live components" for interactive behavior → .pc compiles to real framework code.

```
┌─────────────────────────────────────────────────────────────┐
│                     DESIGNER WORLD                          │
│                                                             │
│   .pc files = Source of truth for UI                        │
│   Designers "glue" components, wire props, build pages      │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                    COMPONENT REGISTRY                       │
│                                                             │
│   Engineers register: <Map>, <PaymentForm>, <DataTable>     │
│   With typed props, events, slots                           │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                     ENGINEER WORLD                          │
│                                                             │
│   Build behavior components, APIs, business logic           │
│   Never touch layout/styling                                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### The Boundary Between Designer and Engineer

```
┌─────────────────────────────────────────────────────────────┐
│                    DESIGNER (.pc)                           │
│                                                             │
│  ✓ Layout, styling, spacing                                 │
│  ✓ Wire props to registered components                      │
│  ✓ Simple events: navigate, toggle, show/hide               │
│  ✓ Iteration: for item in items { ... }                     │
│  ✓ Conditionals: if visible { ... }                         │
│  ✓ Bind to provided data: {user.name}                       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                    THE WALL                                 │
│         "If you need this, ask an engineer"                 │
├─────────────────────────────────────────────────────────────┤
│                    ENGINEER (code)                          │
│                                                             │
│  ✓ Fetch data, API calls                                    │
│  ✓ Complex state machines                                   │
│  ✓ Validation / business logic                              │
│  ✓ Side effects                                             │
│  ✓ Register new components                                  │
│  ✓ Custom hooks / behaviors                                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Live Components: Hybrid Rendering

Some UI elements need real JavaScript to render (maps, 3D viewers, charts). Paperclip uses a **hybrid system**:

- **Static .pc components** → Virtual DOM rendering (fast, <40ms)
- **Live components** → Real JS mounted at placeholder slots

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Preview Frame                                     │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Static Virtual DOM (fast patching)                            │ │
│  │  ┌──────────┐  ┌──────────────────────────────┐               │ │
│  │  │  Header  │  │  "Our Office Location"       │               │ │
│  │  └──────────┘  └──────────────────────────────┘               │ │
│  │  ┌────────────────────────────────────────────┐               │ │
│  │  │  ┌──────────────────────────────────────┐  │               │ │
│  │  │  │  LIVE COMPONENT SLOT                 │  │               │ │
│  │  │  │  <GoogleMap lat={} lng={} zoom={}>   │  │               │ │
│  │  │  │  (Real JS component mounted here)    │  │               │ │
│  │  │  └──────────────────────────────────────┘  │               │ │
│  │  └────────────────────────────────────────────┘               │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

**Designer syntax:**
```javascript
// Auto-discovered from components/GoogleMap.live.tsx
import { Map } from "@app/GoogleMap"

component LocationCard {
  render div {
    text "Our Office"
    Map(lat=37.7749, lng=-122.4194, zoom=12)
  }
}
```

**Engineer marks their component for Paperclip (choose your style):**
```typescript
// components/GoogleMap.tsx - normal component file

/** @paperclip */
export function GoogleMap({ lat, lng, zoom = 12, onMarkerClick }) {
  // ... normal React component
  return <GoogleMapImpl lat={lat} lng={lng} zoom={zoom} onClick={onMarkerClick} />
}

// That's it. Types inferred from TypeScript.
// Automatically appears in designer as @app/GoogleMap
```

Or with explicit metadata if needed:
```typescript
import { live } from '@paperclip/live'

export function GoogleMap({ lat, lng, zoom, onMarkerClick }) {
  // ...
}

live(GoogleMap, {
  props: { zoom: { default: 12, min: 1, max: 20 } },
  events: { onMarkerClick: { payload: 'marker' } },
})
```

### The Dual-View Model (Canvas + Props Pane)

**The Core Invariant:** Every pixel on canvas must trace to an editable source.

This is the same invariant that makes spreadsheets usable, shader editors intelligible, and Figma tolerable despite complexity.

```
┌─────────────────────────────────────────────────────────────────────┐
│ CANVAS (Rendered View)              │ PROPS PANE (Source View)      │
│                                     │                               │
│  ┌─────────────────────────────┐    │  content: {price * quantity}  │
│  │                             │    │           ▲                   │
│  │         $150                │────│───────────┘                   │
│  │           ▲                 │    │                               │
│  │     user selects this       │    │  Current value: $150          │
│  │                             │    │                               │
│  └─────────────────────────────┘    │  Bindings used:               │
│                                     │    price    → item.price = 30 │
│  Shows RESULT                       │    quantity → cart.qty = 5    │
│  But source is expression           │                               │
└─────────────────────────────────────────────────────────────────────┘
```

**Selection behavior:**

| Select on Canvas | What You See | What You Edit |
|------------------|--------------|---------------|
| Static text "Hello" | "Hello" | Inline or props pane |
| Computed "$150" | "$150" | Props pane: `{price * qty}` |
| One card in a loop | That card's content | Template (affects all iterations) |
| Live component | Placeholder/preview | Props pane: bound props |

**Visual indicators (non-optional, clickable):**

```
   Hello John  ← static, edit inline

   $150 ⚡     ← computed (expression icon)

   ┌─────┐ ┌─────┐ ┌─────┐
   │Card │ │Card │ │Card │ ∞   ← iterated (repeat icon)
   └─────┘ └─────┘ └─────┘

   ┌─────────────────────┐
   │  Map Component  🔌  │      ← live component (plug icon)
   └─────────────────────┘
```

**Clicking any indicator:**
- ⚡ → Opens props pane with expression editor, jumps to source line in Monaco
- ∞ → Opens loop template view, highlights the `for` block in source
- 🔌 → Opens live component props, shows link to engineer's source file

These icons are **enforcement without prohibition**. They say:
- "You're not editing a thing, you're editing a rule"
- "This thing exists because data exists"
- "This thing is powered by code you don't own"

### .pc Syntax ↔ Designer Visual Mapping

Every .pc syntax element maps to a familiar designer interaction:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  .pc Syntax                          Designer Action                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  /** @frame(x:100, y:50...) */      Drag frame on canvas                │
│  public component Card { }           Create component (Cmd+Alt+K)        │
│                                                                          │
│  render div {                        Frame/container in layers           │
│    style { ... }                     Properties panel on right           │
│  }                                                                       │
│                                                                          │
│  <Button />                          Drag component from library         │
│  <Button label="Save" />             Edit props in properties panel      │
│                                                                          │
│  variant hover {                     Add variant in variants panel       │
│    style { background: blue }        Edit variant styles                 │
│  }                                                                       │
│                                                                          │
│  slot children                       Add slot (like Figma component prop)│
│  insert children { <Icon /> }        Fill slot in instance               │
│                                                                          │
│  style {                             Edit in properties panel:           │
│    display: flex                     → Auto layout toggle                │
│    gap: 16px                         → Gap control                       │
│    padding: 24px                     → Padding controls                  │
│    background: token(colors.bg)      → Color picker → token              │
│  }                                                                       │
│                                                                          │
│  text "Hello {name}"                 Text layer, double-click to edit    │
│                                                                          │
│  if showBadge { <Badge /> }          Conditional visibility toggle       │
│                                                                          │
│  repeat items { <Item /> }           Shows one item, (∞) indicator       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**The .pc format IS the designer's document format** - just like .fig is Figma's. The designer never writes .pc directly; canvas interactions generate it.

### Expression Language (Formula-Like Only)

**The danger zone is expression power creep.** Expressions must stay formula-like, not code-like.

```
ALLOWED                          FORBIDDEN
─────────────────────────────────────────────────────────────
{price * quantity}               {if (x) { ... }}
{item.name}                      {items.map(i => ...)}
{user.firstName + " " + last}    {async () => await fetch()}
{formatCurrency(price)}          {let x = 1; x + 2}
{items.length > 0}               {import { foo } from '...'}
```

**Expression constraints:**
- Single-expression only
- No control flow
- No function definitions
- No side effects
- No async
- No imports

**If someone asks "Can I do X in an expression?" the answer is often: "Register an action or component."**

### Built-in Actions (No Code Needed)

Designers get a set of "safe" actions:

```javascript
// Navigation
onClick=navigate("/products/{id}")
onClick=back()
onClick=openUrl("https://...")

// UI State
onClick=toggle(menuOpen)
onClick=show(modal)
onClick=hide(dropdown)

// Notifications
onSuccess=showToast("Done!")
onError=showError(error.message)

// Simple state
onClick=set(selectedId, item.id)
onClick=append(items, newItem)
onClick=remove(items, index)
```

Anything beyond this → register a component or action.

### Sample Data (First-Class, Not Optional)

Sample data is design input, not mock data. It must be:
- First-class in the UI
- Versioned in the .pc file
- Intentional (presets for empty/single/many)

```
┌─────────────────────────────────────┐
│ SAMPLE DATA                    [x] │
├─────────────────────────────────────┤
│                                     │
│ data.items (showing 3 of 3)         │
│ ┌─────────────────────────────────┐ │
│ │ [                               │ │
│ │   { name: "Shirt", price: 30 }, │ │
│ │   { name: "Pants", price: 50 }, │ │
│ │   { name: "Shoes", price: 80 }  │ │
│ │ ]                               │ │
│ └─────────────────────────────────┘ │
│                                     │
│ Presets:                            │
│   ○ Empty list                      │
│   ○ Single item                     │
│   ● Multiple items (current)        │
│   ○ Many items (stress test)        │
│                                     │
└─────────────────────────────────────┘
```

### Preview Controls for Conditionals

Designers need to preview all conditional branches without changing source:

```
┌─────────────────────────────────────┐
│ CONDITIONALS                        │
├─────────────────────────────────────┤
│                                     │
│ ☑ user.isAdmin     [true ▼]        │ ← toggle to preview
│ ☐ user.isPremium   [false ▼]       │
│ ☑ cart.hasItems    [true ▼]        │
│                                     │
└─────────────────────────────────────┘
```

**Important:** Toggles are preview-only. They do NOT write back to source.

### Design Philosophy

> **"Nothing happens by accident."**

The system doesn't prevent people from doing powerful things. It forces power to show its source.

This is the difference between a tool becoming a framework vs staying a language.

**The throughline:** Design-time is permissive and explainable. Runtime is strict and explicit.
- Designers are allowed to be "wrong"
- The system tells them how
- Engineers decide what ships

---

## Technical Approach

### Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                           User Interfaces                               │
├──────────────────┬─────────────────┬────────────────┬──────────────────┤
│  Visual Designer │  Embedded Agent │  IDE / Editor  │  Claude (MCP)    │
│  (React Canvas)  │  (Cursor-style) │  (VS Code ext) │  (External)      │
└────────┬─────────┴────────┬────────┴───────┬────────┴────────┬─────────┘
         │                  │                │                 │
         │                  └────────────────┼─────────────────┘
         ▼                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        Workspace Server (Rust)                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────────────┐  │
│  │   Parser    │ │  Evaluator  │ │  Compilers  │ │    MCP Server    │  │
│  │   (.pc)     │ │  (Preview)  │ │ (React/Yew) │ │ (Designer+Claude)│  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └──────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────────────────┐
│                          File System                                    │
│  ┌─────────────┐ ┌─────────────┐ ┌───────────────────────────────────┐ │
│  │  .pc files  │ │   Compiled  │ │     paperclip.config.json         │ │
│  │  (source)   │ │   outputs   │ │     (project config)              │ │
│  └─────────────┘ └─────────────┘ └───────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

Note: The Embedded Agent in the designer uses the **same MCP tools** as external Claude Code.
This means AI-assisted workflows work identically whether you're using the built-in agent or Claude Code in your terminal.

### Core Components (from old codebase analysis)

| Component | Old Location | Lines | Purpose |
|-----------|--------------|-------|---------|
| Parser | `libs/parser/` | ~3K | .pc → AST |
| Proto/AST | `libs/proto/` | ~2K | AST definitions |
| Evaluator | `libs/evaluator/` | ~4K | AST → Virtual HTML/CSS |
| React Compiler | `libs/compiler-react/` | ~1K | Virtual → React code |
| Workspace Server | `libs/workspace/` | ~5K | gRPC server, file watching |
| Designer | `libs/designer/` | ~16K | React canvas UI |
| **MCP Server** | **NEW** | ~1K est. | AI tool integration |

---

## Implementation Phases

### Phase 0: Architecture Spikes (Validate Before Building)

Before full implementation, validate critical architectural decisions with focused prototypes.

#### Spike 0.1: Parser Performance (Rust)
**Goal:** Validate logos + chumsky + bumpalo hits performance targets

- [x] Implement minimal .pc parser with logos lexer
- [x] Add parser for component/style/token syntax (using recursive descent)
- [ ] Use bumpalo arena for AST allocation
- [ ] Benchmark: parse 1000-line file
- [ ] Target: <10ms parse time

**Output:** Parser skeleton meeting performance target, or alternative approach
**Status:** ✅ Parser implemented with logos lexer + recursive descent. All tests passing.

#### Spike 0.2: Evaluator + Virtual DOM (Rust)
**Goal:** Prove AST → Virtual DOM evaluation pipeline

- [x] Implement minimal evaluator that produces Virtual HTML/CSS
- [x] Handle imports and token resolution
- [x] Implement expression evaluation (simple formulas)
- [ ] Benchmark evaluation time
- [ ] Target: <20ms for medium component

**Output:** Working evaluator producing Virtual DOM
**Status:** ✅ Evaluator implemented with Virtual DOM output. Tests passing for components and styles.

#### Spike 0.3: gRPC Streaming Preview Loop (Rust + TypeScript)
**Goal:** Prove real-time edit → preview pipeline

- [ ] Set up Tonic gRPC server with streaming RPC
- [ ] Implement file watcher → parse → evaluate → stream
- [ ] Connect browser client via gRPC-web
- [ ] Implement Virtual DOM differ/patcher in browser
- [ ] Measure end-to-end latency: keystroke → preview update
- [ ] Target: <40ms total

**Output:** Working real-time preview loop

#### Spike 0.4: Roundtrip Serialization (Rust)
**Goal:** Prove AST edits preserve formatting

- [ ] Extend parser to capture whitespace/comments (CST or span-based)
- [ ] Make programmatic edit (add element, change prop)
- [ ] Serialize back to string
- [ ] Diff original vs output
- [ ] Verify only intended changes, formatting preserved

**Output:** Parser approach validated for lossless roundtrip

#### Spike 0.5: Live Component Preview Loading (TypeScript)
**Goal:** Prove we can load project components into preview

- [ ] Create minimal .pc file with live component reference
- [ ] Build component bundle from project's live components
- [ ] Load bundle into preview iframe/frame
- [ ] Mount component with props from Virtual DOM
- [ ] Verify component renders and responds to prop changes

**Output:** Working prototype or documented blockers

#### Spike 0.6: Controller + Data Flow (TypeScript)
**Goal:** Prove compiled output + controller pattern works

- [ ] Write sample .pc component with data bindings
- [ ] Manually write expected compiled output (View)
- [ ] Write controller that provides data via hooks
- [ ] Wire up in test React app
- [ ] Verify data flows correctly through nested components

**Output:** Validated compiled output format + runtime API

#### Spike 0.7: Sample Data in Doc-Comments
**Goal:** Prove sample data variants work in designer

- [ ] Define doc-comment format for `@sample`
- [ ] Parse and extract sample data variants from Rust parser
- [ ] Build UI to switch between variants
- [ ] Preview updates with selected sample data

**Output:** Sample data format spec + working switcher

#### Spike 0.8: Designer Panel in .pc (Dogfooding)
**Goal:** Build one real designer panel in .pc to validate the full stack

- [ ] Pick medium-complexity panel (e.g., component library sidebar)
- [ ] Build it in .pc with sample data variants
- [ ] Create live components for interactive parts
- [ ] Wire up with controller for real data
- [ ] Uses spikes 0.1-0.7 (parser, evaluator, gRPC, preview, controllers)
- [ ] Integrate into designer
- [ ] Document pain points / missing language features

**Output:** Confidence that .pc can build the designer, or list of gaps to address

#### Spike 0.9: Variant Combo System (Rust)
**Goal:** Prove multi-trigger variant combinations work correctly

Legacy Paperclip supports `variant hover + mobile` which applies styles when BOTH triggers are active simultaneously. This requires:

- [ ] Implement variant trigger parsing (`:hover`, `:focus`, `@media`, custom triggers)
- [ ] Support combo syntax: `variant hover + dark + mobile { ... }`
- [ ] Implement trigger state evaluation (AND logic for combos)
- [ ] Test complex combos with 3+ triggers
- [ ] Validate CSS output generates correct selectors

**Risk:** Combo permutations can explode; need to validate performance with many variants.

**Output:** Working variant combos + performance benchmarks

#### Spike 0.10: Override Path Resolution (Rust)
**Goal:** Prove deep instance override drilling works

Legacy Paperclip allows `override A.B.C.D { ... }` to drill into nested component instances. This is critical for:
- Customizing deeply nested component parts
- Designer selection of any instance at any depth
- Maintaining override chains through component composition

- [ ] Implement instance path encoding (e.g., `Button.0/Icon.1/Path.0`)
- [ ] Support override drilling syntax in parser
- [ ] Resolve override paths through component graph
- [ ] Handle shadow instances (instances of instances)
- [ ] Validate virtual IDs match override paths for designer selection

**Risk:** Path resolution through deep nesting is complex; legacy had edge cases with slots.

**Output:** Working override drilling + path resolution algorithm documented

#### Spike 0.11: Style Priority System (Rust)
**Goal:** Prove CSS cascade ordering matches designer expectations

Legacy Paperclip has 40+ priority levels for style ordering:
- Base styles < variant styles < override styles
- Explicit styles < inherited styles
- Within variants: default < hover < focus < active, etc.

- [ ] Document full priority ordering from legacy implementation
- [ ] Implement style priority sorting in evaluator
- [ ] Generate CSS with correct specificity/order
- [ ] Test complex scenarios (variant + override + trigger combos)
- [ ] Validate designer style inspector shows correct computed values

**Risk:** Getting this wrong causes "why isn't my style applying?" confusion.

**Output:** Priority system spec + test suite covering edge cases

#### Spike 0.12: Mutation System + Post-Effects (Rust)
**Goal:** Prove edit mutations maintain document integrity

Legacy Paperclip has 25+ mutation types with cascading post-effects:
- Moving a node may require re-parenting overrides
- Deleting a component may require cleanup of all instances
- Renaming propagates through the dependency graph

- [ ] Define core mutation types (insert, delete, move, update, reparent)
- [ ] Implement mutation application to AST
- [ ] Add post-effect system for cascading changes
- [ ] Handle undo/redo with mutation inverses
- [ ] Test complex sequences (move + rename + delete chain)

**Risk:** Missing post-effects = orphaned nodes / broken references.

**Output:** Mutation system spec + cascading effects documented

#### Spike 0.13: Copy/Paste Validation (TypeScript + Rust)
**Goal:** Prove clipboard operations respect structural rules

Legacy Paperclip enforces what can be pasted where:
- Can't paste a component inside itself (cycle detection)
- Styles can only be pasted into style containers
- Slots have placement restrictions
- Cross-file paste requires import resolution

- [ ] Define paste validation rules
- [ ] Implement structure validation in Rust
- [ ] Handle cross-file copy with import fixup
- [ ] Expose validation to TypeScript for designer feedback
- [ ] Test invalid paste scenarios show correct error messages

**Risk:** Invalid paste silently corrupting documents is hard to debug.

**Output:** Validation rules spec + error messages defined

#### Spike 0.14: Virtual ID Encoding (Rust + TypeScript)
**Goal:** Prove designer selection maps correctly to AST paths

The designer needs to select any element in the preview (including instances of instances). This requires:
- Encoding instance paths in virtual DOM node IDs
- Decoding click targets back to AST node paths
- Handling shadow DOM-like nesting through components

- [ ] Define virtual ID encoding scheme (e.g., `root/Card.0/Header.0/Title`)
- [ ] Implement encoding during Virtual DOM generation
- [ ] Implement decoding for click-to-select
- [ ] Support multi-select with shift/cmd-click
- [ ] Validate selection works at arbitrary nesting depth

**Risk:** ID mismatch = clicking in preview doesn't select correct node.

**Output:** ID encoding scheme spec + working click-to-select

#### Spike 0.15: React Compiler Type Inference (Rust)
**Goal:** Prove generated TypeScript types are correct and complete

Legacy Paperclip infers TypeScript types from .pc usage:
- Props derived from `{binding}` expressions
- Event handlers from `onClick={handler}` patterns
- Slot types from `slot` declarations
- Generic/conditional types for variant props

- [ ] Implement prop type inference from expressions
- [ ] Generate slot prop types (`renderHeader?: () => ReactNode`)
- [ ] Handle event handler types (onClick, onChange, etc.)
- [ ] Support optional vs required inference
- [ ] Validate generated types compile with strict TypeScript

**Risk:** Wrong types = engineer frustration, defeats type safety goal.

**Output:** Type inference algorithm spec + test suite

#### Spike 0.16: Feature Flags System (Rust)
**Goal:** Support experimental syntax without breaking existing files

Legacy Paperclip uses feature flags to gate experimental syntax:
- Allows gradual rollout of new features
- Parser can reject unknown syntax gracefully
- Enables A/B testing of language features

- [ ] Define feature flag declaration syntax (in config or file-level)
- [ ] Implement conditional parsing based on flags
- [ ] Provide clear error messages for flagged features
- [ ] Test flag propagation through imports

**Risk:** Without flags, can't iterate on syntax without breaking users.

**Output:** Feature flag spec + parser integration

#### Spike 0.17: Visual Diffing Pipeline (TypeScript + Rust)
**Goal:** Prove we can render components to PNGs and generate visual diffs

- [ ] Render single .pc component to PNG (headless Playwright/Puppeteer)
- [ ] Render all variants for a component (hover, focus, dark, mobile)
- [ ] Render all sample data variations (@sample default, @sample empty)
- [ ] Generate pixel-diff between two PNGs
- [ ] Output diff visualization (side-by-side with highlighted changes)
- [ ] Measure performance (can we render 100 components in <1 min?)

**Risk:** Headless rendering may have platform inconsistencies.

**Output:** Working snapshot CLI + diff algorithm

#### Spike 0.18: AI Vision + Reference System (TypeScript)
**Goal:** Prove AI agent can see the canvas and use reference materials

- [ ] Implement canvas screenshot capture (PNG from preview iframe)
- [ ] Feed screenshot to AI via multi-modal input
- [ ] Test AI understanding of visual layout vs code-only context
- [ ] Implement `.paperclip/references/` directory scanning
- [ ] Feed markdown/images from references to AI context
- [ ] Test AI using inspiration images to guide design decisions

**Risk:** Screenshot quality/size may impact AI performance.

**Output:** Working AI vision loop + reference integration

#### Spike 0.19: @frame Directive & Multi-Component Canvas (Rust + TypeScript)
**Goal:** Prove root components render to canvas with @frame positioning

A .pc file can have multiple root components, each rendered on the canvas at positions defined by `@frame`:

```pc
/**
 * @frame(x: 100, y: 50, width: 320, height: 480)
 */
public component MobileView { ... }

/**
 * @frame(x: 500, y: 50, width: 1024, height: 768)
 */
public component DesktopView { ... }
```

- [ ] Parse `@frame` directive from doc-comments
- [ ] Extract x, y, width, height values
- [ ] Include frame data in evaluated Virtual DOM output
- [ ] Render multiple root components on canvas at specified positions
- [ ] Update @frame when component is dragged on canvas
- [ ] Handle @frame roundtrip (edit position → update source)
- [ ] Support auto-height (width specified, height computed)

**Risk:** Keeping @frame in sync between canvas drag and source code.

**Output:** Working multi-component canvas with @frame positioning

#### Spike 0.20: File Navigator & Asset Drop (TypeScript)
**Goal:** Prove file tree navigation and asset drag-drop works

- [ ] Build file tree UI from project scan
- [ ] Open .pc file → loads into designer canvas
- [ ] Tab interface for multiple open files
- [ ] Scan and display asset files (images, SVGs)
- [ ] Drag image from asset panel onto canvas → creates `img` element
- [ ] Drag image onto existing element → sets background-image
- [ ] Handle relative path resolution for assets

**Risk:** Large projects may have slow file tree scanning.

**Output:** Working file navigator + asset drag-drop

#### Spike 0.21: Tools Layer + Rect Caching (TypeScript)
**Goal:** Prove tools overlay works with Redux-mediated rect caching (no globals)

Replicate the legacy designer pattern: measurements cached in state, tools receive positions as props.

- [ ] Set up preview in iframe with `id="_virtId"` attributes on elements
- [ ] Implement `getFrameRects()` - traverse DOM, call `getBoundingClientRect()`, convert to document coords
- [ ] Dispatch `ui/rectsCaptured` to store measurements in `state.rects`
- [ ] Implement `getSelectedNodeBox` selector to read from cached rects
- [ ] Draw `Selectable` component positioned via props (not global access)
- [ ] Implement coordinate transform: `((docX - scrollX) * zoom) + panX`
- [ ] Test resize handles update when `state.rects` changes
- [ ] Test click-to-select via `elementsFromPoint()` + id lookup
- [ ] Validate hover highlight works across iframe boundary
- [ ] Measure performance with 100+ elements (rect collection + rendering)

**Key Pattern to Validate:**
```typescript
// ❌ NOT: window.measurements or global bridge
// ✅ INSTEAD: dispatch({ type: "ui/rectsCaptured", payload: { rects } })
```

**Risk:** Rect collection timing - must recollect on preview re-render.

**Output:** Working tools overlay with state-mediated rect caching

**Success Criteria Phase 0:**
- [ ] All spikes completed with documented findings
- [ ] No architectural blockers discovered (or solutions identified)
- [ ] Confidence to proceed with full implementation

---

### Phase 1: Core Engine
**Goal:** Parse .pc files, evaluate to virtual DOM, compile to React

#### 1.1 Project Setup
- [ ] Initialize Cargo workspace structure
- [ ] Set up Protocol Buffer definitions (copy/adapt from old `libs/proto/`)
- [ ] Configure WASM compilation targets
- [ ] Create `paperclip.config.json` schema

**Files to create:**
```
Cargo.toml (workspace)
libs/
  proto/
    Cargo.toml
    src/
      ast/pc.proto       # Component AST definitions
      virt/html.proto    # Virtual HTML output
      virt/css.proto     # Virtual CSS output
      graph.proto        # Dependency graph
```

#### 1.2 Parser
- [ ] Implement tokenizer (`tokenizer.rs`) using `logos` crate
- [ ] Implement parser (`parser.rs`) using `chumsky` parser combinators
- [ ] Support full .pc syntax (components, styles, tokens, variants, slots)
- [ ] Error recovery with source positions and recovery tokens
- [ ] WASM compilation for browser use (`wasm-bindgen`)
- [ ] Arena allocation with `bumpalo` for zero-copy parsing

**Reference:** `../paperclip/libs/parser/src/pc/`

### Research Insights: Parser

**Best Practices (Rust Parser Research):**
- Use `logos` for lexing (10-100x faster than hand-rolled)
- Use `chumsky` for parsing (excellent error recovery, composable)
- `bumpalo` arena allocation eliminates per-node allocations
- Implement incremental parsing with `tree-sitter` for editor integration later

**Performance Considerations:**
- Target: Parse 1000-line file in <10ms (vs current 50ms target)
- Use `&str` slices into source instead of `String` copies
- Preallocate AST node vectors based on file size heuristics
- Consider CST (Concrete Syntax Tree) for lossless roundtripping

**Implementation Pattern:**
```rust
// Recommended lexer setup with logos
use logos::Logos;
use bumpalo::Bump;

#[derive(Logos, Debug, PartialEq)]
pub enum Token<'src> {
    #[token("component")]
    Component,
    #[token("style")]
    Style,
    #[token("token")]
    Token,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'src str),
    #[regex(r#""[^"]*""#)]
    String(&'src str),
    // ...
}

pub fn parse<'a>(bump: &'a Bump, source: &'a str) -> Result<Ast<'a>, ParseError> {
    let lexer = Token::lexer(source);
    // Use chumsky for recursive descent with error recovery
}
```

**Edge Cases:**
- Handle UTF-8 multi-byte characters in string positions
- Support nested template literals with proper escaping
- Implement error recovery at statement boundaries (semicolons, closing braces)

**.pc Syntax to Support:**
```javascript
// Imports
import "./theme.pc" as theme

// Tokens
public token primaryColor #3366FF
public token fontFamily Inter, sans-serif

// Style mixins
public style defaultFont {
  font-family: var(fontFamily)
  font-size: 14px
}

// Components
public component Button {
  variant hover trigger { ":hover" }
  variant mobile trigger { "@media (max-width: 600px)" }

  render button {
    style extends defaultFont {
      padding: 8px 16px
      background: var(primaryColor)
    }
    style variant hover {
      background: #2452CC
    }
    slot children {
      text "Click me"
    }
  }
}
```

**Files to create:**
```
libs/
  parser/
    Cargo.toml
    src/
      lib.rs
      tokenizer.rs
      parser.rs
      parser_context.rs
      string_scanner.rs
```

#### 1.3 Evaluator
- [ ] Implement HTML evaluator (AST → virtual HTML)
- [ ] Implement CSS evaluator (styles → virtual CSS)
- [ ] Handle imports and dependency resolution via GraphManager
- [ ] Resolve variants and conditions
- [ ] Source mapping for designer integration
- [ ] Incremental evaluation (only re-evaluate changed nodes)

**Reference:** `../paperclip/libs/evaluator/`

### Research Insights: Evaluator

**Architecture (Architecture Strategist):**
- Introduce `GraphManager` as centralized dependency tracker
- Track file→file and node→node dependencies
- Enable surgical re-evaluation on changes (not full rebuild)

```rust
pub struct GraphManager {
    /// File dependency graph
    file_deps: HashMap<PathBuf, HashSet<PathBuf>>,
    /// Node-level dependencies for fine-grained updates
    node_deps: HashMap<NodeId, HashSet<NodeId>>,
    /// Cached evaluation results
    cache: HashMap<PathBuf, EvaluatedModule>,
}

impl GraphManager {
    pub fn invalidate(&mut self, path: &Path) -> Vec<PathBuf> {
        // Returns all files that need re-evaluation
    }

    pub fn evaluate_incremental(&mut self, changed: &Path) -> Result<EvaluatedModule> {
        // Only re-evaluate affected subgraph
    }
}
```

**Performance Considerations:**
- Cache resolved token values (they rarely change)
- Parallelize independent branch evaluation with `rayon`
- Use structural sharing for virtual DOM (like Immer)

**Files to create:**
```
libs/
  evaluator/
    Cargo.toml
    src/
      lib.rs
      html/
        evaluator.rs
        virt.rs
      css/
        evaluator.rs
        virt.rs
        serializer.rs
```

#### 1.4 React Compiler
- [ ] Implement code compiler (virtual → React TSX)
- [ ] Generate TypeScript types for props
- [ ] Emit CSS (scoped, no conflicts)
- [ ] Source maps for debugging

**Reference:** `../paperclip/libs/compiler-react/src/code_compiler.rs`

**Output format:**
```tsx
// button.tsx (generated)
import styles from './button.module.css';

export interface ButtonProps {
  children?: React.ReactNode;
  className?: string;
}

export const Button: React.FC<ButtonProps> = ({ children, className }) => (
  <button className={`${styles.root} ${className || ''}`}>
    {children ?? 'Click me'}
  </button>
);
```

**Files to create:**
```
libs/
  compiler-react/
    Cargo.toml
    src/
      lib.rs
      code_compiler.rs
      definition_compiler.rs
```

#### 1.5 CLI
- [ ] `paperclip init` - Create project config
- [ ] `paperclip build` - Compile all .pc files
- [ ] `paperclip fmt` - Format .pc files
- [ ] `paperclip watch` - Watch and rebuild on change

**Files to create:**
```
libs/
  cli/
    Cargo.toml
    src/
      main.rs
      commands/
        init.rs
        build.rs
        fmt.rs
        watch.rs
```

**Success Criteria Phase 1:**
- [ ] Can parse valid .pc files to AST
- [ ] Can evaluate AST to virtual HTML/CSS
- [ ] Can compile to working React components
- [ ] CLI builds a sample project

---

### Phase 2: Workspace Server
**Goal:** gRPC server for live editing, file watching, and mutations

#### 2.1 Server Core
- [ ] Tokio async runtime setup
- [ ] File watcher (notify crate)
- [ ] Project graph management
- [ ] Undo/redo stack

**Reference:** `../paperclip/libs/workspace/`

#### 2.2 gRPC Service
- [ ] Define proto service (`designer.proto`)
- [ ] Implement Tonic gRPC server
- [ ] Add gRPC-web support for browser clients
- [ ] Event streaming for file changes

**Key RPCs:**
```protobuf
service Designer {
  rpc OpenFile(FileRequest) returns (FileResponse);
  rpc GetGraph(Empty) returns (stream Graph);
  rpc ApplyMutations(MutationsRequest) returns (MutationsResult);
  rpc UpdateFile(UpdateFileRequest) returns (Empty);
  rpc OnEvent(Empty) returns (stream DesignServerEvent);
  rpc Undo(Empty) returns (Empty);
  rpc Redo(Empty) returns (Empty);
}
```

#### 2.3 Mutation System
- [ ] Define mutation types (add, remove, update, move nodes)
- [ ] Apply mutations to AST
- [ ] Serialize back to .pc source
- [ ] Broadcast changes to clients

**Files to create:**
```
libs/
  workspace/
    Cargo.toml
    src/
      lib.rs
      server.rs
      engines/
        bootstrap.rs
        config.rs
        api.rs
        paperclip.rs
        local.rs
```

**Success Criteria Phase 2:**
- [ ] Server starts and watches project files
- [ ] Can open file and receive AST
- [ ] Can apply mutations and see file updates
- [ ] Multiple clients receive change events

---

### Phase 3: MCP Server (NEW)
**Goal:** AI integration via Model Context Protocol

#### 3.1 MCP Tool Definitions
- [ ] `list_components` - Browse component library
- [ ] `read_component` - Get source of a component
- [ ] `write_component` - Create or update component
- [ ] `delete_component` - Remove a component (**NEW**)
- [ ] `get_preview` - Screenshot for AI feedback
- [ ] `get_tokens` - List design tokens
- [ ] `create_token` - Add new design token (**NEW**)
- [ ] `update_token` - Modify existing token (**NEW**)
- [ ] `delete_token` - Remove a token (**NEW**)
- [ ] `compile` - Generate React/HTML output
- [ ] `validate` - Check syntax before saving
- [ ] `undo` - Revert last change (**NEW**)
- [ ] `redo` - Reapply reverted change (**NEW**)
- [ ] `get_context` - Return project structure for AI context (**NEW**)

### Research Insights: MCP Server

**Agent-Native Design (Agent-Native Reviewer):**
- Every UI action must have MCP tool equivalent
- Include `get_context` tool for AI to understand project structure
- Return structured errors with suggestions, not just failure messages
- Add completion signals: `{ success: true, next_steps: ["validate", "preview"] }`

**MCP Best Practices:**
- Use JSON Schema for tool input validation
- Implement idempotent operations where possible
- Include `dry_run` parameter for destructive operations
- Return diffs for write operations: `{ before, after, diff }`

**Security (Security Sentinel):**
- **CRITICAL:** Validate all paths against project root
- Reject paths containing `..` or absolute paths outside project
- Sanitize component names (alphanumeric + underscore only)
- Rate limit preview generation (expensive operation)

**Tool Schemas (Enhanced):**

```typescript
// list_components
Input: { path?: string }
Output: {
  components: [{ name, path, isPublic, description?, slots?, variants? }],
  context: { totalCount, projectRoot }
}

// read_component
Input: { path: string, includeAst?: boolean }
Output: { source: string, ast?: object, dependencies?: string[] }

// write_component
Input: { path: string, content: string, dryRun?: boolean }
Output: {
  success: boolean,
  errors?: [{ line, column, message, suggestion? }],
  diff?: { before, after },
  nextSteps?: string[]
}

// delete_component (NEW)
Input: { path: string, dryRun?: boolean }
Output: { success: boolean, dependents?: string[] }

// get_preview
Input: { path: string, variant?: string, width?: number, height?: number }
Output: { image: base64_png, renderTime: number }

// get_tokens
Input: { path?: string, type?: 'color'|'spacing'|'typography' }
Output: { tokens: [{ name, value, type, usageCount }] }

// create_token / update_token / delete_token (NEW)
Input: { name: string, value?: string, type?: string }
Output: { success: boolean, token?: object }

// compile
Input: { path: string, target: 'react'|'html', watch?: boolean }
Output: { files: [{ path, content }], warnings?: string[] }

// validate
Input: { content: string, path?: string }
Output: { valid: boolean, errors?: [{ line, column, message, code }] }

// undo / redo (NEW)
Input: { steps?: number }
Output: { success: boolean, currentState: string }

// get_context (NEW - for AI orientation)
Input: {}
Output: {
  projectRoot: string,
  components: [{ name, path }],
  tokens: [{ name, type }],
  recentChanges: string[]
}
```

**Path Validation (Security):**
```typescript
function validatePath(userPath: string, projectRoot: string): string {
  const resolved = path.resolve(projectRoot, userPath);
  if (!resolved.startsWith(projectRoot)) {
    throw new Error('Path traversal detected');
  }
  if (!resolved.endsWith('.pc')) {
    throw new Error('Only .pc files allowed');
  }
  return resolved;
}
```

#### 3.2 MCP Server Implementation
- [ ] Implement MCP protocol handler
- [ ] Connect to workspace server (or embed)
- [ ] Screenshot capture for previews (headless browser or canvas render)
- [ ] Stdin/stdout transport for Claude Code

**Files to create:**
```
libs/
  mcp-server/
    Cargo.toml
    src/
      lib.rs
      server.rs
      tools/
        list_components.rs
        read_component.rs
        write_component.rs
        get_preview.rs
        get_tokens.rs
        compile.rs
        validate.rs
```

**Example AI Workflow:**
```
User: "Create a notification banner with warning colors"

Claude:
1. get_tokens() → finds warning colors
2. write_component("NotificationBanner.pc", "...")
3. get_preview("NotificationBanner.pc") → verifies visually
4. "Created the component. Here's how it looks: [image]"
```

**Success Criteria Phase 3:**
- [ ] MCP server starts via Claude Code
- [ ] Can list, read, write components
- [ ] Can get visual previews
- [ ] Claude can create components end-to-end

---

### Phase 4: Visual Designer
**Goal:** React canvas for visual editing with dual-view model and live component support

**See:** [Designer-First Architecture](#designer-first-architecture-the-inverted-model) for the full design philosophy.

#### 4.1 Canvas Foundation
- [ ] React app setup (Vite + TypeScript strict mode)
- [ ] DOM-based canvas viewport with pan/zoom (NOT HTML Canvas)
- [ ] gRPC-web client connection with typed layer
- [ ] State management: **Machine + Engines pattern** (same as Shay app) — **designer app only, never in compiled output**
- [ ] **Dual-view model:** Canvas (rendered) + Props pane (source)
- [ ] **Visual indicators:** Computed (⚡), iterated (∞), live component (🔌) icons
- [ ] **Tools layer:** Separate overlay for selection, resize handles, guides

### Critical Architecture: Tools Layer Separation (from Legacy Designer)

**The tools (selection boxes, resize handles, etc.) are a SEPARATE layer from the preview.**

```
┌─────────────────────────────────────────────────────────────────┐
│                        Canvas Viewport                           │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Tools Layer (overlay)                   │  │
│  │    Selection boxes, resize handles, guides, rulers         │  │
│  │    Positioned via cached measurements from state.rects     │  │
│  ├───────────────────────────────────────────────────────────┤  │
│  │                    Preview Layer (iframe)                  │  │
│  │    Rendered .pc components with id="_virtId" attributes    │  │
│  │    Pure output - no designer artifacts                     │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

**Key Pattern: Redux-Mediated, Props-Driven Coordinate Mapping (NO GLOBALS)**

The legacy designer avoids globals entirely through this pattern:

```
┌─────────────────────────────────────┐
│     Tools Layer (Canvas/Tools)      │
│  - Selectable (selection boxes)     │
│  - Knobs & Edges (resize handles)   │
│  - Receives box/scroll/zoom as props│
└────────────────┬────────────────────┘
                 │
          Machine State (rects)
                 ↓
┌─────────────────────────────────────┐
│      State + Selectors              │
│  - getSelectedNodeBox()             │
│  - getHighlightedNodeBox()          │
│  - state.rects[virtId]              │
└────────────────┬────────────────────┘
                 │
        ui/rectsCaptured event
                 ↑
┌─────────────────────────────────────┐
│  Preview Layer (Frames/iframes)     │
│  - getBoundingClientRect() calls    │
│  - Dispatches measurements to state │
└─────────────────────────────────────┘
```

**Step 1: Preview Collects Measurements**

When frames load/update, traverse DOM and collect bounding rects:

```typescript
// libs/web-renderer/src/render.ts - getFrameRects()
export const getFrameRects = (
  mount: HTMLElement,
  info: PCModule,
  frameIndex: number
) => {
  const rects: Record<string, Box> = {};
  const bounds = getFrameBounds(frame); // Frame position on canvas

  // Traverse rendered DOM inside iframe
  const root = mount.childNodes[STAGE_INDEX].childNodes[0];

  traverseNativeNode(root, (node) => {
    if (node.nodeType !== 1) return;

    // Element id is "_virtId" - strip prefix
    const virtId = (node as HTMLElement).id?.substring(1);

    // THE KEY CALL: getBoundingClientRect()
    const clientRect = (node as Element).getBoundingClientRect();

    // Convert viewport coords to document coords (add frame offset)
    rects[virtId] = {
      width: clientRect.width,
      height: clientRect.height,
      x: bounds.x + clientRect.left,
      y: bounds.y + clientRect.top,
    };
  });

  return rects;
};
```

**Step 2: Dispatch to State (Not Globals)**

```typescript
// libs/designer/src/ui/logic/Editor/Canvas/Frames/index.tsx
const emitFrameRects = useCallback(
  (mount: HTMLElement, data: PCModule, frameIndex: number) => {
    const rects = getFrameRects(mount, data, frameIndex);

    // ❌ NOT: window.measurements = rects
    // ✅ INSTEAD: dispatch event to machine
    dispatch({
      type: "ui/rectsCaptured",
      payload: { frameIndex, rects },
    });
  },
  [dispatch]
);
```

**Step 3: Reducer Stores Measurements**

```typescript
// libs/designer/src/domains/ui/reducers/canvas.ts
case "ui/rectsCaptured":
  return produce(state, (draft) => {
    Object.assign(draft.rects, event.payload.rects);
  });
```

**Step 4: Selectors Compute Boxes**

```typescript
// libs/designer/src/state/pc.ts
export const getSelectedNodeBox = (state: DesignerState): Box =>
  getNodeBox(getTargetExprId(state), state);

export const getNodeBox = (virtId: string, state: DesignerState): Box => {
  // Read from cached measurements in state.rects
  return state.rects[virtId] ?? null;
};
```

**Step 5: Tools Layer Receives Props (Not Global Access)**

```typescript
// libs/designer/src/ui/logic/Editor/Canvas/Tools/Selectable/index.tsx
export const Selectable = React.memo(
  ({ box, canvasTransform, canvasScroll }: Props) => {
    // Calculate screen position from cached box + transform + scroll
    const left = (box.x - canvasScroll.x) * canvasTransform.z + canvasTransform.x;
    const top = (box.y - canvasScroll.y) * canvasTransform.z + canvasTransform.y;

    return (
      <div style={{
        transform: `translateX(${left}px) translateY(${top}px)`,
        width: box.width * canvasTransform.z,
        height: box.height * canvasTransform.z,
      }}>
        <ResizeHandle position="nw" />
        <ResizeHandle position="ne" />
        {/* ... */}
      </div>
    );
  }
);
```

**Three Coordinate Systems:**

| System | Source | Example |
|--------|--------|---------|
| **Viewport** | `getBoundingClientRect()` | `{ left: 245, top: 312 }` |
| **Document** | Stored in `state.rects` | `{ x: 100, y: 200 }` (with frame offset) |
| **Canvas Screen** | Computed for overlays | `((docX - scrollX) * zoom) + panX` |

**Why This Pattern (No Globals):**
1. **All state in reducer** - Easy to reset, serialize, debug
2. **Fully reactive** - Zoom/pan changes → positions recompute automatically
3. **Props-driven** - Components receive what they need, easy to test
4. **Time-decoupled** - Measurements cached, tools don't need live iframe access
5. **Iframe boundary clean** - Measurements taken once, stored in state

**Tools Layer Components:**
- `Selectable` - Selection box with resize handles
- `Knobs` - Corner resize handles (nw, ne, sw, se)
- `Edges` - Edge resize handles (n, s, e, w)
- `HoverHighlight` - Highlight on mouseover
- `InsertElement` - Drop zone indicators

**Legacy Codebase Reference:**

| Component | File | Key Lines |
|-----------|------|-----------|
| **getFrameRects** | `libs/web-renderer/src/render.ts` | 189-237 |
| **Selectable** | `libs/designer/src/ui/logic/Editor/Canvas/Tools/Selectable/index.tsx` | 31-250 |
| **Tools parent** | `libs/designer/src/ui/logic/Editor/Canvas/Tools/index.tsx` | 131-132 |
| **Rect capture dispatch** | `libs/designer/src/ui/logic/Editor/Canvas/Frames/index.tsx` | 150-174 |
| **Canvas reducer** | `libs/designer/src/domains/ui/reducers/canvas.ts` | 223-236 |
| **Box selectors** | `libs/designer/src/state/pc.ts` | 1178-1195 |
| **calcExprBox** | `libs/designer/src/state/pc.ts` | 822-887 |
| **startDOMDrag utility** | `libs/designer/src/ui/logic/utils/dnd.ts` | 3-50 |
| **useFrame hook** | `libs/designer/src/hooks/useFrame/index.ts` | - |
| **useFrameContainer** | `libs/designer/src/hooks/useFrameContainer/index.tsx` | - |
| **FrameContainer** | `libs/designer/src/ui/logic/FrameContainer/index.tsx` | - |

### Additional Legacy Patterns to Preserve

#### Pattern 1: Undo/Redo via Browser History API

**Location:** `libs/designer/src/domains/history/`

Rather than maintaining an in-memory history stack, the legacy designer delegates to the browser's native history API:

```typescript
// history.ts
export class History {
  private _em: EventEmitter;
  constructor() {
    this._em = new EventEmitter();
    window.addEventListener("popstate", () => {
      this._em.emit("change");
    });
  }
  redirect(url: string) {
    history.pushState(null, null, url);
    this._em.emit("change");
  }
}
```

**Key Insight:** Application state is serializable to URL. Browser manages history stack for free.

#### Pattern 2: Keyboard Shortcuts as Data (Menu-Based)

**Location:** `libs/designer/src/domains/shortcuts/`

Shortcuts are declared as a menu structure, not hardcoded:

```typescript
export const getGlobalShortcuts = (state: DesignerState): MenuItem<ShortcutCommand>[] => [
  {
    kind: MenuItemKind.Option,
    label: "Undo",
    shortcut: ["meta", "z"],
    command: ShortcutCommand.Undo,
  },
  // ... more shortcuts
];
```

**Key Insight:** Shortcuts are data, enabling conditional enabling/disabling and context menus.

#### Pattern 3: Copy/Paste with Typed Payloads

**Location:** `libs/designer/src/domains/clipboard/`

```typescript
const handleCopy = async (command: ShortcutCommand, state: DesignerState) => {
  let payload: ClipboardPayload;

  if (command === ShortcutCommand.CopyStyles) {
    payload = { data: computeElementStyle(...), type: command };
  } else {
    payload = { data: await api.copyExpression(...), type: command };
  }

  navigator.clipboard.writeText(JSON.stringify(payload));
};
```

**Key Insight:** Payload tagged with `type` enables different paste behaviors.

#### Pattern 4: Throttled Drag with Delta Tracking

**Location:** `libs/designer/src/ui/logic/utils/dnd.ts`

```typescript
export const startDOMDrag = (startEvent, onStart, update, stop) => {
  const sx = startEvent.clientX;
  const sy = startEvent.clientY;
  let _started = false;

  const drag = throttle((event) => {
    if (!_started) {
      _started = true;
      onStart?.(event);
    }
    update(event, {
      delta: { x: event.clientX - sx, y: event.clientY - sy },
    });
  }, 10);

  doc.addEventListener("mousemove", drag);
  doc.addEventListener("mouseup", onMouseUp);
};
```

**Key Insight:** `_started` flag prevents accidental drags on double-click.

#### Pattern 5: Zoom-to-Point Canvas Transform

**Location:** `libs/designer/src/state/geom.ts`

```typescript
export const centerTransformZoom = (
  translate: Transform,
  bounds: Box,
  nz: number,
  point?: Point
): Transform => {
  const oz = translate.z;
  const zd = nz / oz;

  // Center is based on mouse position
  const v1px = point ? point.x / bounds.width : 0.5;
  const v1py = point ? point.y / bounds.height : 0.5;

  // ... perspective-correct zoom math
  return { x: left, y: top, z: nz };
};
```

**Key Insight:** Zoom toward cursor position, not canvas center.

#### Pattern 6: Variant-Aware Style Cascade

**Location:** `libs/core/src/proto/ast/pc-utils.ts` (478-612)

```typescript
export const computeElementStyle = memoize((exprId, graph, activeVariantIds) => {
  // Styles are computed per variant combo
  // Variant specificity > general styles
  // Each property tracks previous values in cascade (for UI display)
});

const overrideComputedStyles = (computedStyles, overrides) => {
  // Determines precedence by variant specificity count
  const [low, high] = override.variantIds.length >= prev.variantIds.length
    ? [computedStyles, overrides]
    : [overrides, computedStyles];
};
```

**Key Insight:** Variant matching is AND operation (all variants in combo must be active).

#### Pattern 7: Incremental Frame Patching

**Location:** `libs/designer/src/hooks/useFrameMount/index.ts`

```typescript
useEffect(() => {
  if (state?.mount && frameIndex === state.frameIndex) {
    try {
      patchFrame(state.mount, frameIndex, state.pcData, pcData, options);
    } catch (e) {
      // Fall back to full re-render on patch failure
      mount = renderFrame(pcData, frameIndex, options);
    }
  } else {
    mount = renderFrame(pcData, frameIndex, options);
  }
}, [pcData, variantIds]);
```

**Key Insight:** Try incremental patch, fall back to full render on failure.

#### Pattern 8: Engine Composition via Combinator

**Location:** `libs/designer/src/engine/index.ts`

```typescript
export const createEngine = (options, ...otherEngineCreators) =>
  (dispatch, getState) => {
    return combineEngineCreators(
      createDesignerEngine(apiClient),      // API operations
      createShortcutsEngine(apiClient),     // Keyboard & menu
      createClipboardEngine,                // Copy/paste
      createHistoryEngine(options.history), // Undo/redo
      createKeyboardEngine,                 // Raw keyboard
      createUIEngine(options.history),      // Canvas & panels
      ...otherEngineCreators
    )(dispatch, getState);
  };
```

**Key Insight:** Each domain is independent engine; compose via `combineEngineCreators`.

#### Pattern 9: Cross-Browser Wheel Normalization

**Location:** `libs/designer/src/ui/logic/Editor/Canvas/normalize-wheel.ts`

~160 lines handling Safari, Chrome, Firefox, IE wheel event variations.

```typescript
export const ZOOM_SENSITIVITY = IS_WINDOWS ? 2500 : 250;
export const PAN_X_SENSITIVITY = IS_WINDOWS ? 0.05 : 1;
```

**Key Insight:** Windows/macOS have different wheel deltas; adapt sensitivity.

#### Pattern 10: Shadow ID Pattern for Instances

```typescript
export const getShadowExprId = (id: string) => id.split(".").pop();

// Check if instance vs definition
const isInstance = id.includes(".");

// Instance path: "ComponentId.0.ChildId.1.GrandchildId"
```

**Key Insight:** Dot notation encodes instance path through component hierarchy.

#### Pattern 11: Memoization Throughout

Extensive use of `memoize` utility caches AST traversals:

```typescript
export const getChildren = memoize((node, graph) => { ... });
export const getChildParentMap = memoize((graph) => { ... });
export const getComponentVariants = memoize((component) => { ... });
export const computeElementStyle = memoize((exprId, graph, activeVariantIds) => { ... });
```

**Key Insight:** Critical for performance - many AST queries happen on every state update.

#### Legacy Pattern Reference Table

| Pattern | Location | Key File |
|---------|----------|----------|
| **Undo/Redo** | `domains/history/` | `history.ts`, `reducer.ts` |
| **Keyboard Shortcuts** | `domains/shortcuts/` | `state.ts`, `engine.ts` |
| **Copy/Paste** | `domains/clipboard/` | `engine.ts` |
| **Drag & Drop** | `ui/logic/utils/` | `dnd.ts` |
| **Canvas Transform** | `state/` | `geom.ts` |
| **Style Computation** | `core/proto/ast/` | `pc-utils.ts:478-612` |
| **Variant Application** | `core/proto/ast/` | `pc-utils.ts:505-569` |
| **Slot Resolution** | `core/proto/ast/` | `pc-utils.ts:695-710` |
| **Import Resolution** | `core/proto/ast/` | `pc-utils.ts:355-372` |
| **Frame Patching** | `hooks/useFrameMount/` | `index.ts` |
| **Engine Composition** | `engine/` | `index.ts` |
| **Wheel Normalization** | `ui/logic/Editor/Canvas/` | `normalize-wheel.ts` |
| **Double-Click Detection** | `domains/ui/` | `state.ts:142-170` |

### Research Insights: Designer Architecture

**Canvas Approach (React Canvas Research):**
- Use DOM-based rendering, not HTML Canvas
- Better accessibility, native browser events, CSS styling
- Libraries: `@dnd-kit/core` for drag/drop (not react-dnd)
- Pan/zoom via CSS transforms on container element

**State Management: Machine + Engines Pattern**

The designer uses the same architecture as the Shay app (`~/Developer/fourthplaces/shay/packages/app`) and the legacy Paperclip designer:

```
┌─────────────────────────────────────────────────────────────────┐
│                         DesignerMachine                          │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                      Root Reducer                            ││
│  │  (chains: canvas, selection, document, ui, history reducers) ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Engine Composition                        ││
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐││
│  │  │   API   │ │Shortcuts│ │ History │ │Keyboard │ │   AI   │││
│  │  │ Engine  │ │ Engine  │ │ Engine  │ │ Engine  │ │ Engine │││
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

**Core Types:**
```typescript
// Machine orchestrates state + side effects
type Machine<State, Event> = {
  dispatch(event: Event): void
  getState(): State
  subscribe(listener: (state: State) => void): Unsubscribe
}

// Engine handles side effects for a domain
type Engine<State, Event> = {
  start?(): void
  handleEvent(event: Event, currState: State, prevState: State): void
  dispose?(): void
}

// Engine factory receives machine handle
type EngineFactory<State, Event, Props> = (
  props: Props,
  machine: MachineHandle<State, Event>
) => Engine<State, Event>

type MachineHandle<State, Event> = {
  dispatch(event: Event): void
  getState(): State
}
```

**Event Flow:**
```
User clicks element on canvas
       ↓
Component dispatches: { type: "canvas/elementClicked", payload: { id } }
       ↓
Root Reducer processes event
  - selectionReducer: state.selection = [id]
  - uiReducer: state.inspectorTab = "properties"
       ↓
All Engines receive event
  - apiEngine: no-op (selection is UI-only)
  - shortcutsEngine: updates available shortcuts for selection
  - historyEngine: no-op (selection doesn't affect history)
       ↓
State listeners notified → React re-renders
```

**Engine Examples:**

```typescript
// API Engine - handles backend communication
const apiEngine: EngineFactory<DesignerState, DesignerEvent, APIProps> =
  (props, machine) => {
    const { client } = props
    const { dispatch, getState } = machine

    return {
      async handleEvent(event, currState, prevState) {
        switch (event.type) {
          case "canvas/styleChanged":
            // Optimistic update already in state via reducer
            try {
              await client.applyMutations([{
                type: "setStyle",
                nodeId: event.payload.nodeId,
                style: event.payload.style
              }])
              dispatch({ type: "api/mutationSucceeded" })
            } catch (error) {
              dispatch({ type: "api/mutationFailed", payload: { error } })
            }
            break
        }
      }
    }
  }

// AI Engine - handles embedded agent chat
const aiEngine: EngineFactory<DesignerState, DesignerEvent, AIProps> =
  (props, machine) => {
    const { mcpClient } = props
    const { dispatch, getState } = machine

    return {
      async handleEvent(event, currState, prevState) {
        switch (event.type) {
          case "ai/promptSubmitted":
            dispatch({ type: "ai/responseStarted" })

            // Build context from current state
            const context = {
              selection: currState.selection,
              filePath: currState.document?.path,
              canvasState: currState.canvas
            }

            // Stream response from AI
            for await (const chunk of mcpClient.chat(event.payload.prompt, context)) {
              dispatch({ type: "ai/responseChunk", payload: { chunk } })
            }

            dispatch({ type: "ai/responseComplete" })
            break

          case "ai/applyChangesConfirmed":
            // Apply the previewed changes
            for (const mutation of event.payload.mutations) {
              dispatch({ type: "canvas/applyMutation", payload: mutation })
            }
            break
        }
      }
    }
  }
```

**Machine Composition:**
```typescript
// machine/index.ts
export const DesignerMachine = defineMachine<DesignerEvent, DesignerState, DesignerProps>({
  reducer: rootReducer,
  engine: (props, machine) => {
    const api = apiEngine(props, machine)
    const shortcuts = shortcutsEngine(props, machine)
    const history = historyEngine(props, machine)
    const keyboard = keyboardEngine(props, machine)
    const ai = aiEngine(props, machine)
    const clipboard = clipboardEngine(props, machine)

    return {
      start() {
        api.start?.()
        shortcuts.start?.()
        // ... all engines
      },
      handleEvent(event, currState, prevState) {
        api.handleEvent(event, currState, prevState)
        shortcuts.handleEvent(event, currState, prevState)
        history.handleEvent(event, currState, prevState)
        keyboard.handleEvent(event, currState, prevState)
        ai.handleEvent(event, currState, prevState)
        clipboard.handleEvent(event, currState, prevState)
      },
      dispose() {
        api.dispose?.()
        shortcuts.dispose?.()
        // ... cleanup
      }
    }
  },
  initialState
})
```

**React Integration:**
```typescript
// App.tsx
const machine = DesignerMachine.createInstance(props, dispatcher.dispatch)

export const App = () => (
  <DesignerMachine.Provider instance={machine}>
    <Canvas />
    <Inspector />
    <AIChat />
  </DesignerMachine.Provider>
)

// Canvas.tsx
const Canvas = () => {
  const selection = DesignerMachine.useSelector(state => state.selection)
  const dispatch = useDispatch<DesignerEvent>()

  const handleClick = (nodeId: string) => {
    dispatch({ type: "canvas/elementClicked", payload: { id: nodeId } })
  }

  return <CanvasViewport selection={selection} onClick={handleClick} />
}
```

**Why This Pattern (Not Zustand/Redux):**

| Concern | Zustand/Redux | Machine + Engines |
|---------|---------------|-------------------|
| Side effects | Middleware or thunks | Dedicated engines |
| Domain isolation | Slices | Engine per domain |
| Event tracing | Harder | All events flow through machine |
| Testing | Mock store | Mock engine props |
| Async operations | Awkward | Natural in engines |
| Multiple side effects per event | Multiple dispatches | All engines see same event |

The Machine + Engines pattern provides cleaner separation between:
- **Reducers** = pure state transitions (sync, testable)
- **Engines** = side effects (async, I/O, subscriptions)

**Race Condition Prevention (Frontend Races Reviewer):**
- Implement optimistic UI with rollback on server rejection
- Queue rapid mutations, debounce to single server call
- Lock interactions during pending mutations (show spinner)
- Use `AbortController` for cancellable requests

#### 4.2 Frame Rendering & Live Preview
- [ ] Render evaluated HTML/CSS in frames via Virtual DOM
- [ ] Multiple component previews on canvas
- [ ] Responsive preview breakpoints
- [ ] **Hybrid rendering:** Virtual DOM for static + real JS for live components
- [ ] **Live component registry:** Load and mount registered components at placeholders
- [ ] **Selection behavior:** Static → inline edit, Computed → props pane, Loop item → edit template
- [ ] **Sample data panel:** Inline data editing with presets (empty/single/many)
- [ ] **Conditional preview toggles:** Preview all branches without changing source

### Research Insights: Live Component Preview

**CRITICAL: Real-time Performance Architecture**

The preview does NOT re-compile to React on each change. Instead:

1. **Evaluator outputs Virtual HTML/CSS** (fast, in-memory)
2. **Preview renders Virtual DOM directly** (no compilation step)
3. **Changes are diffed/patched** (only update what changed)

```
User edits .pc file
       ↓
    Parser (< 10ms)
       ↓
   Evaluator (< 20ms) → Virtual HTML + Virtual CSS
       ↓
   Diff against previous Virtual DOM
       ↓
   Patch preview iframe (< 5ms)

Total: < 35ms per keystroke
```

**gRPC Streaming for Real-time Updates:**
```protobuf
service Designer {
  // Stream evaluated changes to clients
  rpc OnEvaluatedChange(Empty) returns (stream EvaluatedModule);
}

message EvaluatedModule {
  string path = 1;
  VirtualHTML html = 2;
  VirtualCSS css = 3;
  repeated Patch patches = 4;  // Delta from previous state
}

message Patch {
  enum Op { INSERT, UPDATE, REMOVE, MOVE }
  Op operation = 1;
  string target_path = 2;  // DOM path like "0/2/1"
  bytes payload = 3;       // New node data if INSERT/UPDATE
}
```

**Preview Renderer (Browser) - Islands Architecture:**

The preview uses **islands of React in a sea of Virtual DOM**:
- Static .pc content → Virtual DOM (fast patching, no React overhead)
- Live component slots → Actual React roots mounted into placeholders

```
┌────────────────────────────────────────────────────────────────┐
│  <div>                           ← Virtual DOM (fast patches) │
│    <h1>Our Office</h1>           ← Virtual DOM                 │
│    <div data-live-component>     ← Placeholder div             │
│      ┌────────────────────┐                                    │
│      │ React.createRoot() │      ← Real React component        │
│      │ <GoogleMap ... />  │         mounted here               │
│      └────────────────────┘                                    │
│    </div>                                                      │
│    <button>Contact</button>      ← Virtual DOM                 │
│  </div>                                                        │
└────────────────────────────────────────────────────────────────┘
```

```typescript
class PreviewRenderer {
  private root: HTMLElement
  private currentVdom: VirtualNode | null = null
  private liveRoots: Map<string, ReactRoot> = new Map()
  private registry: ComponentRegistry

  render(vdom: VirtualNode, css: VirtualCSS) {
    // 1. Patch static Virtual DOM (fast path)
    if (this.currentVdom) {
      const patches = diff(this.currentVdom, vdom)
      applyPatches(this.root, patches)
    } else {
      this.root.innerHTML = ''
      this.root.appendChild(vdomToHtml(vdom))
    }
    this.updateStyles(css)
    this.currentVdom = vdom

    // 2. Mount/update React islands for live components
    const liveSlots = this.root.querySelectorAll('[data-live-component]')
    for (const slot of liveSlots) {
      const componentId = slot.dataset.liveComponent  // e.g., "@app/GoogleMap"
      const props = JSON.parse(slot.dataset.props)

      // Create React root if first time
      if (!this.liveRoots.has(slot.id)) {
        this.liveRoots.set(slot.id, createRoot(slot))
      }

      // Render the actual React component
      const Component = this.registry.get(componentId)
      this.liveRoots.get(slot.id).render(<Component {...props} />)
    }

    // 3. Cleanup unmounted live components
    for (const [id, root] of this.liveRoots) {
      if (!document.getElementById(id)) {
        root.unmount()
        this.liveRoots.delete(id)
      }
    }
  }
}
```

This is similar to **Astro's islands architecture** - static HTML with interactive "islands" where needed. Benefits:
- Fast updates for static content (Virtual DOM patching)
- Real React behavior for interactive components
- Clean separation of concerns

**Why NOT Sandpack for Real-time:**
- Sandpack requires full recompilation (100-500ms)
- Only use Sandpack for "Export to React" preview
- Designer preview uses direct Virtual DOM rendering

**Performance Targets:**
| Operation | Target | Approach |
|-----------|--------|----------|
| Parse | < 10ms | logos + bumpalo arena |
| Evaluate | < 20ms | Incremental, cached tokens |
| Diff | < 3ms | Virtual DOM comparison |
| Patch | < 5ms | Direct DOM mutations |
| **Total** | **< 40ms** | 25 FPS editing |

#### 4.3 Selection & Editing
- [ ] Click to select elements
- [ ] Multi-select with shift/drag
- [ ] Property inspector panel
- [ ] Inline text editing

#### 4.4 Manipulation Tools
- [ ] Drag to reorder/reparent
- [ ] Resize handles
- [ ] Spacing/margin adjusters
- [ ] Style property editors

#### 4.5 Component Tools
- [ ] Component library panel
- [ ] Drag components onto canvas
- [ ] Slot filling interface
- [ ] Variant switcher

#### 4.6 File Navigator & Asset Browser
- [ ] File tree panel (project .pc files)
- [ ] Open .pc file → loads into canvas
- [ ] Multiple open files (tabs)
- [ ] Asset browser (images, fonts, icons)
- [ ] Drag-drop assets onto canvas
- [ ] Image preview on hover
- [ ] Create new .pc file from navigator
- [ ] Rename/delete files with confirmation

**Root Component Rendering:**

When a .pc file is opened, **root-level components render directly on the canvas**. Each component uses an `@frame` directive to specify its canvas position:

```pc
/**
 * @frame(x: 100, y: 50, width: 320, height: 480)
 */
public component MobileCard {
  render div {
    ...
  }
}

/**
 * @frame(x: 500, y: 50, width: 800, height: 600)
 */
public component DesktopCard {
  render div {
    ...
  }
}
```

This enables:
- Multiple component variants visible at once
- Artboard-style design (like Figma frames)
- Responsive previews side-by-side
- Canvas position persisted in source code

**@frame Directive:**

| Property | Description |
|----------|-------------|
| `x` | Canvas X position (px) |
| `y` | Canvas Y position (px) |
| `width` | Frame width (px) |
| `height` | Frame height (px, optional - can auto-size) |

**Asset Drag-Drop:**

```
┌──────────────────┐
│ Assets           │
├──────────────────┤
│ 📁 images/       │
│   🖼 hero.png    │  ←── drag onto canvas
│   🖼 logo.svg    │
│   🖼 avatar.jpg  │
│ 📁 icons/        │
│   📄 check.svg   │
└──────────────────┘

Drop on canvas → creates:
  img src="./images/hero.png"

Drop on existing element → sets background:
  style {
    background: url("./images/hero.png")
  }
```

#### 4.7 Integrated Code Editor
- [ ] Monaco Editor integration (VSCode core)
- [ ] Collapsible bottom panel + split view mode
- [ ] .pc syntax highlighting
- [ ] Autocomplete from language server
- [ ] Inline error display
- [ ] Click-to-source (canvas element → code line)
- [ ] Live sync (edit code → canvas updates)

#### 4.8 AI Agent Chat (Cursor-style)
- [ ] Chat panel UI (collapsible, dockable)
- [ ] Message threading with history
- [ ] Context injection (selection, canvas state, file path)
- [ ] Streaming responses with markdown rendering
- [ ] Preview-before-apply for changes
- [ ] MCP tool integration (same tools as external Claude)
- [ ] Multi-modal input (text, paste images, drag files)
- [ ] Suggested actions based on selection
- [ ] Keyboard shortcut (Cmd+K) for quick prompt
- [ ] Inline prompt mode (select element → type to change)

**Files to create:**
```
libs/
  common/                           # Shared utilities (port from @shay/common)
    src/
      machine/
        core.ts                     # Machine implementation
        store.ts                    # State store
        types.ts                    # Engine, MachineHandle types
      index.ts

  designer/
    package.json
    vite.config.ts
    src/
      index.tsx
      App.tsx

      # Machine + Engines (core architecture)
      core/
        machine.ts                  # DesignerMachine definition
        state/
          types.ts                  # DesignerState shape
          events.ts                 # All event type definitions
          initial.ts                # Initial state
          reducers/
            index.ts                # Root reducer
            canvas.ts               # Canvas/viewport state
            selection.ts            # Selection state
            document.ts             # Document/AST state
            ui.ts                   # UI panels state
            history.ts              # Undo/redo state
            ai.ts                   # AI chat state
        engines/
          index.ts                  # Engine composition
          api.ts                    # Backend communication (gRPC)
          shortcuts.ts              # Keyboard shortcuts
          history.ts                # Browser history/undo
          keyboard.ts               # Low-level keyboard events
          clipboard.ts              # Copy/paste
          ai.ts                     # AI agent chat
          preview.ts                # Live component mounting

      # React Components
      components/
        Canvas/
          Canvas.tsx
          Frame.tsx
          Viewport.tsx
          LiveComponentSlot.tsx
        Inspector/
          PropertyPanel.tsx
          StyleEditor.tsx
          ExpressionEditor.tsx
        Library/
          ComponentList.tsx
          TokenList.tsx
          LiveComponentPalette.tsx
        AIChat/
          ChatPanel.tsx
          MessageList.tsx
          PromptInput.tsx
          DiffPreview.tsx
        CodeEditor/
          MonacoEditor.tsx
          SyntaxHighlighting.ts
        Tools/
          Selection.tsx
          TextEditor.tsx

      # API Layer
      api/
        grpc-client.ts
        mcp-client.ts               # For AI agent
```

**Reference:** `../paperclip/libs/designer/` (135 TSX files, 16K lines)

### Research Insights: Designer Visual Design

**Color Palette (Frontend Design Skill):**
```css
:root {
  /* Warm graphite theme - professional, not cold */
  --bg-canvas: #1a1a1f;
  --bg-panel: #242428;
  --bg-elevated: #2d2d32;
  --border-subtle: #3a3a40;
  --text-primary: #e8e8ec;
  --text-secondary: #9898a0;
  --accent-primary: #6366f1;  /* Indigo */
  --accent-hover: #818cf8;
}
```

**Typography:**
- Font: JetBrains Mono for code, Inter for UI
- Code: 13px, UI: 14px base

**Component Structure:**
```
┌─────────────────────────────────────────────────────────────┐
│ Toolbar (fixed top)                                         │
├───────────┬─────────────────────────────────┬───────────────┤
│ Component │                                 │   Inspector   │
│  Library  │         Canvas Area             │    Panel      │
│   Panel   │     (pan/zoom viewport)         │ (properties)  │
│  (240px)  │                                 │   (320px)     │
├───────────┴─────────────────────────────────┴───────────────┤
│ Code Editor (Monaco) - collapsible bottom panel             │
├─────────────────────────────────────────────────────────────┤
│ Status Bar (file path, zoom level, selection info)          │
└─────────────────────────────────────────────────────────────┘
```

**Integrated Code Editor:**
The designer includes Monaco Editor (VSCode's core) so users never need to leave:

- **Toggle with keyboard shortcut** (Cmd+E)
- **Split view option** (code + canvas side by side)
- **Full syntax highlighting** for .pc files
- **Autocomplete** from Paperclip language server
- **Inline errors** as you type
- **Click element → jumps to source line**
- **Edit source → canvas updates live**

This makes Paperclip fully self-contained. No VSCode required.

**Integrated AI Agent Chat (Cursor-style):**
An embedded AI assistant that understands the visual context:

```
┌─────────────────────────────────────────────────────────────┐
│ Toolbar                                                      │
├───────────┬─────────────────────────────────┬───────────────┤
│ Component │                                 │   Inspector   │
│  Library  │         Canvas Area             │    Panel      │
│   Panel   │                                 ├───────────────┤
│           │                                 │   AI Chat     │
│           │                                 │   ┌─────────┐ │
│           │                                 │   │ 💬      │ │
│           │                                 │   │ "Make   │ │
│           │                                 │   │ this    │ │
│           │                                 │   │ button  │ │
│           │                                 │   │ bigger" │ │
│           │                                 │   └─────────┘ │
├───────────┴─────────────────────────────────┴───────────────┤
│ Code Editor (Monaco) - collapsible                           │
└─────────────────────────────────────────────────────────────┘
```

**Agent capabilities:**
- **Context-aware:** Sees current selection, canvas state, component tree
- **Visual feedback:** Shows what it's about to change before applying
- **MCP-powered:** Uses same tools as external Claude Code
- **Conversational:** Can discuss design decisions, explain choices
- **Multi-modal input:** Accepts text, screenshots, Figma links
- **Canvas Vision:** Can take screenshots of the canvas to SEE what it's working on
- **Reference Directory:** Reads project's `.paperclip/references/` for design inspiration

**AI Vision System:**

The agent doesn't just infer from code - it can *see* the canvas:

```
┌─────────────────────────────────────────────────────────────┐
│  Agent Loop                                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 1. User prompt: "Make this look more modern"         │   │
│  │ 2. Agent takes canvas screenshot (PNG)               │   │
│  │ 3. Agent analyzes: "I see a card with dated styling" │   │
│  │ 4. Agent checks .paperclip/references/ for context   │   │
│  │ 5. Agent applies changes                             │   │
│  │ 6. Agent takes new screenshot to verify              │   │
│  │ 7. Agent: "Here's what I changed. Better?"           │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

This is critical because:
- Code inference misses visual nuance (spacing feels off, colors clash)
- Screenshots capture the *actual* rendered output
- Agent can verify its changes look correct before committing
- Enables "make this look like X" where X is a reference image

**Design Reference Directory:**

Engineers/designers can populate a `.paperclip/` directory with design context:

```
.paperclip/
  references/
    brand-guidelines.md      # Brand colors, typography, voice
    component-patterns.md    # When to use which components
    inspiration/
      stripe-dashboard.png   # Screenshot of Stripe
      linear-sidebar.png     # Screenshot of Linear
      notion-cards.png       # Screenshot of Notion
    figma-exports/
      design-tokens.json     # Exported from Figma
      component-specs.md     # Component specifications
  prompts/
    default-system.md        # Custom system prompt additions
    code-style.md            # How agent should write .pc code
```

Agent reads these automatically and incorporates them:

```
User: "Add a settings panel"
Agent: [reads brand-guidelines.md, sees inspiration/linear-sidebar.png]
       "Based on your brand guidelines and the Linear-inspired
        direction in your references, I'll create a settings panel
        with your primary color and sidebar navigation pattern."
```

This makes the agent project-aware, not generic

**Example interactions:**
```
User: "Make this card look more like the Stripe dashboard"
Agent: [Shows preview] "I'll update the card with:
       - Subtle shadow instead of border
       - 16px border radius
       - More vertical padding
       Apply changes?"

User: "Add a loading state to this button"
Agent: [Creates variant] "Added 'loading' variant with:
       - Spinner icon
       - Disabled state
       - 'Loading...' text
       Switch to preview it?"

User: "Why is this text truncated?"
Agent: "The parent container has overflow:hidden and
       the text is 240px but container is 200px.
       Want me to either:
       1. Widen the container
       2. Add text-overflow: ellipsis
       3. Wrap to multiple lines"
```

**What sets this apart from Tambo:**
| Tambo | Paperclip Agent |
|-------|-----------------|
| Runtime chat in your app | Design-time assistant |
| Generates components on-the-fly | Edits .pc source files |
| For end users | For designers/developers |
| Output is ephemeral | Output is committed code |
| Chat IS the product | Canvas IS the product, chat assists |

Paperclip's agent is about **designing better**, not runtime AI features.

**Success Criteria Phase 4:**
- [ ] Can view components on canvas
- [ ] Can select and see properties
- [ ] Can edit styles visually
- [ ] Changes sync to .pc files
- [ ] **Dual-view works:** Selecting computed value shows expression in props pane
- [ ] **Visual indicators:** ⚡ ∞ 🔌 icons appear for computed/iterated/live elements
- [ ] **Live components render:** Registered components mount and update with props
- [ ] **Sample data works:** Can switch presets and see preview change
- [ ] **Loop editing:** Selecting item in loop edits template, not instance
- [ ] **Expression editing:** Props pane shows and edits expressions, canvas shows results
- [ ] **Code editor works:** Can toggle Monaco, edit .pc, see canvas update
- [ ] **AI agent works:** Can chat, agent sees selection, applies changes via MCP

---

### Phase 5: Integration & Polish
**Goal:** Production-ready tooling

#### 5.1 VSCode Extension (Realtime Sync)
- [ ] Language server for .pc files
- [ ] Syntax highlighting
- [ ] Autocomplete
- [ ] Go to definition
- [ ] **WebSocket connection to Paperclip server**
- [ ] **Bidirectional realtime sync** (VSCode ↔ Designer)
- [ ] **Inline errors as you type** (from server evaluation)
- [ ] **Cursor position sync** (see where designer is editing)
- [ ] Embedded designer preview (optional)

#### 5.2 Build Integration
- [ ] Vite plugin
- [ ] Webpack loader
- [ ] Watch mode with HMR

#### 5.3 Documentation
- [ ] .pc syntax reference
- [ ] Getting started guide
- [ ] API reference for MCP tools
- [ ] Example projects

#### 5.4 Testing
- [ ] Parser test suite
- [ ] Compiler snapshot tests
- [ ] Designer E2E tests

#### 5.5 Visual Diffing Tool (CI/CD Integration)
**Goal:** Catch visual regressions before they ship

Render all .pc components as PNGs and show side-by-side diffs for code review:

```
┌─────────────────────────────────────────────────────────────┐
│  PR #1234: Update Button component                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Button.pc - 3 visual changes detected                       │
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │   Before    │    │    After    │    │    Diff     │      │
│  │  ┌──────┐   │    │  ┌──────┐   │    │  ┌──────┐   │      │
│  │  │ Save │   │    │  │ Save │   │    │  │██████│   │      │
│  │  └──────┘   │    │  └──────┘   │    │  └──────┘   │      │
│  │   default   │    │   default   │    │  +8px pad   │      │
│  └─────────────┘    └─────────────┘    └─────────────┘      │
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │  ┌──────┐   │    │  ┌──────┐   │    │  ┌──────┐   │      │
│  │  │ Save │   │ →  │  │ Save │   │    │  │██████│   │      │
│  │  └──────┘   │    │  └──────┘   │    │  └──────┘   │      │
│  │    hover    │    │    hover    │    │  new shadow │      │
│  └─────────────┘    └─────────────┘    └─────────────┘      │
│                                                              │
│  [✓ Approve]  [✗ Reject]  [💬 Comment]                       │
└─────────────────────────────────────────────────────────────┘
```

**Implementation:**
- [ ] CLI command: `paperclip snapshot` - renders all components to PNGs
- [ ] Snapshot storage (local `.paperclip/snapshots/` or cloud)
- [ ] Diff algorithm (pixel-diff with configurable threshold)
- [ ] GitHub Action integration
- [ ] PR comment bot with visual diff gallery
- [ ] Variant matrix rendering (default, hover, focus, dark, mobile, etc.)
- [ ] Baseline management (approve new baselines)
- [ ] Headless rendering (using Playwright or Puppeteer)

**Sample data integration:**
```bash
# Render all variants for all sample data combinations
paperclip snapshot --variants --samples

# Output structure:
.paperclip/snapshots/
  Button/
    default.png
    default@hover.png
    default@dark.png
    default@mobile.png
    empty.png           # @sample empty
    loading.png         # @sample loading
  Card/
    ...
```

**GitHub Action:**
```yaml
- name: Paperclip Visual Diff
  uses: paperclip/visual-diff@v1
  with:
    base: main
    threshold: 0.1%  # Pixel difference threshold
```

**Why this is powerful:**
- Catches unintended style changes (broke button on hover? diff shows it)
- Documents visual changes for reviewers (don't just read code, see it)
- Works with CI/CD (block merge if unapproved visual changes)
- Covers all variant/sample combinations automatically

**Success Criteria Phase 5:**
- [ ] VSCode extension installable
- [ ] Can integrate into Vite/webpack project
- [ ] Documentation complete
- [ ] CI/CD pipeline passing
- [ ] **Visual diff CLI works:** Can render snapshots, generate diffs
- [ ] **GitHub integration works:** PR comments with visual changes

---

## ERD: Core Data Model

```mermaid
erDiagram
    Project ||--o{ Document : contains
    Document ||--o{ Node : has_root
    Node ||--o{ Node : children
    Node ||--o{ Style : has_styles
    Node ||--o{ Variant : has_variants

    Project {
        string config_path
        string src_dir
        string designs_dir
        json compiler_options
    }

    Document {
        string path
        string content
        Node ast
        VirtualModule evaluated
    }

    Node {
        string id
        enum type
        string name
        map properties
    }

    Style {
        string id
        list declarations
        string extends
    }

    Variant {
        string name
        string trigger
        list style_overrides
    }

    VirtualModule {
        VirtualHTML html
        VirtualCSS css
        list imports
    }
```

---

## Risk Analysis & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Parser complexity underestimated | Medium | High | Use logos+chumsky (proven combo), start with subset |
| WASM performance issues | Low | Medium | Use bumpalo arena, profile early |
| Designer UX too complex | Medium | High | Focus on core flows, defer advanced features |
| MCP integration gaps | Low | Medium | Test with Claude Code early, add get_context tool |
| Scope creep | High | High | Strict phase gates, defer nice-to-haves |
| Race conditions in designer | Medium | High | Optimistic UI with rollback, operation queuing |
| Path traversal attacks | Medium | Critical | Validate all paths against project root |
| Live preview XSS | Medium | High | Sandpack iframe isolation |

### Research-Discovered Risks

**Simplicity Reviewer Warning:**
> Consider 50-60% scope reduction for true MVP. The core value is parse→evaluate→compile. Designer and MCP can be Phase 2 after the core loop works end-to-end.

**Recommended MVP Scope:**
1. Parser (full .pc syntax)
2. Evaluator (HTML/CSS output)
3. React compiler
4. CLI (build/watch)
5. *Defer:* Designer, MCP, VSCode extension

This lets you ship something usable in days, not weeks.

---

## Implementation Priority Tiers

Based on language design analysis, these are the priorities for the designer-first features:

### Tier 1 — Must nail early (trust depends on these)

| Feature | Why Critical | Approach |
|---------|--------------|----------|
| **Type contracts + warnings** | Silent coercion destroys trust | Warnings not blocks; red squiggle + icon; CI can optionally fail |
| **Error states in preview** | If preview lies, people stop trusting | Canvas never crashes; errors visible but contained; always show something |
| **Empty states (`for...empty`)** | Designers will hardcode placeholders otherwise | `for item in data { } empty { "No items" }` syntax |
| **Preview controls for conditionals** | Without this, designers duplicate components | Toggle UI is preview-only, never writes back |

### Tier 2 — Important, can evolve

| Feature | Notes |
|---------|-------|
| **Slot/composition boundaries** | Dashed outlines, slot labels on hover, "You're editing slot: footer" |
| **Responsive/variants** | Must feel orthogonal to data; no "responsive logic" creep |
| **Git diffs** | Stable formatting, deterministic ordering, no canvas-only metadata in source |

### Tier 3 — Can come later

| Feature | Notes |
|---------|-------|
| **Workflow metadata** | JSDoc-like descriptions in component registry |
| **Sample data extraction** | Inline for now, extract to files later |
| **Runtime vs design-time toggle** | Implicit preview mode initially; add explicit toggle only when confusion appears |

---

## Dependencies & Prerequisites

**Rust Ecosystem (2026 Recommendations):**
- `logos` for lexing (fastest Rust lexer)
- `chumsky` for parser combinators (excellent error recovery)
- `bumpalo` for arena allocation
- `serde` / `prost` for serialization
- `tokio` 1.x for async runtime
- `tonic` 0.12+ for gRPC
- `notify` 6.x for file watching
- `wasm-bindgen` 0.2.90+ for WASM
- `rayon` for parallel evaluation

**JavaScript/TypeScript (2026 Recommendations):**
- React 19+ (use React Compiler if stable)
- Vite 6.x for bundling
- `@connectrpc/connect-web` for gRPC (replaces grpc-web)
- `@dnd-kit/core` for drag/drop (not react-dnd)
- `immer` for immutable state updates in reducers
- Machine + Engines pattern from `@shay/common` (or port to `@paperclip/common`)
- `@codesandbox/sandpack-react` for live preview

**External:**
- Claude Code for MCP testing
- Playwright for screenshots (or Sandpack preview capture)

---

## Success Metrics

| Metric | Target | Research-Informed |
|--------|--------|-------------------|
| Parse time (1000 line file) | < 10ms | logos+bumpalo enables this |
| Compile time (single component) | < 50ms | Incremental compilation |
| Designer load time | < 1.5s | Machine pattern + code splitting |
| MCP tool response time | < 300ms | Cached graph manager |
| File sync latency | < 50ms | Debounced mutations |
| Live preview update | < 200ms | Sandpack HMR |
| Memory (designer idle) | < 150MB | Arena allocation |

---

## Security Considerations (Security Sentinel Review)

### Critical Security Requirements

**1. Path Traversal Prevention**
```rust
// REQUIRED: Validate all file paths
fn validate_path(user_path: &str, project_root: &Path) -> Result<PathBuf, SecurityError> {
    let resolved = project_root.join(user_path).canonicalize()?;
    if !resolved.starts_with(project_root) {
        return Err(SecurityError::PathTraversal);
    }
    Ok(resolved)
}
```

**2. Live Preview Sandboxing**
- Use Sandpack iframe isolation (separate origin)
- Apply Content Security Policy: `script-src 'self'`
- Never execute user code in main thread

**3. MCP Input Validation**
- Validate all tool inputs against JSON Schema
- Sanitize component names: `/^[a-zA-Z][a-zA-Z0-9_]*$/`
- Limit file sizes (e.g., 1MB max for .pc files)
- Rate limit expensive operations (preview generation)

**4. gRPC Security**
- Use TLS for production deployments
- Implement request authentication tokens
- Validate message sizes (prevent DoS)

---

## Platform Agnosticism

Paperclip is a **platform-agnostic design language**. The .pc syntax describes UI structure and styling; compilers and preview runtimes are pluggable.

```
                         .pc source (universal)
                                │
            ┌───────────────────┼───────────────────┐
            ▼                   ▼                   ▼
      Web Compiler        RN Compiler         Swift Compiler
            │                   │                   │
            ▼                   ▼                   ▼
    React/Vue/Svelte      React Native         SwiftUI
            │                   │                   │
            ▼                   ▼                   ▼
      Browser Preview     Expo/Simulator      Xcode Preview
```

### Configuration

```typescript
// paperclip.config.ts
export default {
  platform: 'web',        // 'web' | 'react-native' | 'ios' | 'android'
  framework: 'react',     // web: react/vue/svelte/solid
                          // ios: swiftui/uikit
                          // android: compose/views
}
```

### Live Components Per Platform

Same .pc file, different live component implementations per platform:

```typescript
// components/Map.tsx (web)
/** @paperclip platform:web */
export function Map({ lat, lng, onMarkerClick }) {
  return <GoogleMapWeb lat={lat} lng={lng} onMarkerClick={onMarkerClick} />
}
```

```typescript
// components/Map.native.tsx (React Native)
/** @paperclip platform:react-native */
import MapView from 'react-native-maps'

export function Map({ lat, lng, onMarkerClick }) {
  return (
    <MapView
      region={{ latitude: lat, longitude: lng, latitudeDelta: 0.01, longitudeDelta: 0.01 }}
      onMarkerPress={onMarkerClick}
    />
  )
}
```

```swift
// components/Map.swift (iOS/SwiftUI)
/// @paperclip platform:ios
import SwiftUI
import MapKit

struct MapLive: View {
    let lat: Double
    let lng: Double
    let onMarkerClick: ((Marker) -> Void)?

    var body: some View {
        Map(coordinateRegion: .constant(MKCoordinateRegion(
            center: CLLocationCoordinate2D(latitude: lat, longitude: lng),
            span: MKCoordinateSpan(latitudeDelta: 0.01, longitudeDelta: 0.01)
        )))
    }
}
```

### Compiled Output Per Platform

**Command:**
```bash
paperclip compile --platform web          # → React/Vue/Svelte
paperclip compile --platform react-native # → React Native
paperclip compile --platform ios          # → SwiftUI
paperclip compile --platform android      # → Jetpack Compose
```

**iOS/SwiftUI output:**
```swift
// Generated: ProductCard.swift
import SwiftUI
import PaperclipRuntime

struct ProductCard: View {
    let product: Product

    var body: some View {
        VStack(spacing: 16) {
            Text(product.name)
                .font(.system(size: 18, weight: .semibold))
                .foregroundColor(Color(hex: "#1a1a1f"))

            // Live component
            MapLive(lat: product.storeLat, lng: product.storeLng)

            Button(action: { PaperclipActions.showToast("Added!") }) {
                Text("Add to Cart")
            }
            .buttonStyle(PrimaryButtonStyle())
        }
        .padding(16)
        .background(Color.white)
        .cornerRadius(8)
    }
}
```

### Preview Per Platform

| Platform | Preview Method | Live Component Runtime |
|----------|----------------|------------------------|
| Web | Browser | React/Vue/Svelte islands |
| React Native | Expo Go / RN Dev Client | RN components |
| iOS | Xcode Previews / Simulator | SwiftUI views |
| Android | Android Studio / Emulator | Compose components |

### Preview Architecture

**MVP: Web-Based Preview (All Platforms)**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Paperclip Workspace Server                    │
│  (Rust: parser, evaluator, file watching, gRPC)                 │
└──────────────────────────┬──────────────────────────────────────┘
                           │ gRPC streaming
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Browser Preview (MVP)                         │
│                                                                  │
│   - Virtual DOM rendering (fast, <40ms)                         │
│   - Platform-specific CSS (approximate iOS/Android look)        │
│   - Live components: placeholder or web fallback                │
│   - Device frames: iPhone, Pixel, etc. (visual only)           │
│                                                                  │
│   Good for: Layout, spacing, typography, colors                 │
│   Limited: Real native components, gestures, animations         │
└─────────────────────────────────────────────────────────────────┘

Optional: Connect local simulator (if user has Xcode/Android Studio)
```

**Future (v2): Cloud-Rendered Native Previews**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Paperclip Cloud (v2)                          │
│                                                                  │
│   - Real iOS Simulators on Mac Minis                            │
│   - Real Android Emulators on Linux VMs                         │
│   - Video streamed to browser via WebRTC                        │
│   - No local Xcode/Android Studio required                      │
│                                                                  │
│   Enables: Real SwiftUI, MapKit, Compose, gestures, haptics    │
└─────────────────────────────────────────────────────────────────┘
```

### Multi-Platform Targeting

**Platforms CAN be mixed** - compile the same .pc source to multiple targets:

```typescript
// paperclip.config.ts
export default {
  targets: [
    { platform: 'web', framework: 'react', outDir: 'src/web/components' },
    { platform: 'ios', framework: 'swiftui', outDir: 'ios/App/Generated' },
    { platform: 'android', framework: 'compose', outDir: 'android/app/generated' },
    { platform: 'react-native', outDir: 'src/native/components' },
  ]
}
```

**Platform-specific overrides in .pc:**

```javascript
component Button {
  render button {
    style {
      padding: 16px
      border-radius: 8px
    }

    // iOS needs more touch area (44pt minimum)
    style platform:ios {
      padding: 20px
    }

    // Android uses Material Design radii
    style platform:android {
      border-radius: 4px
    }

    // React Native needs explicit flex
    style platform:react-native {
      flex-direction: row
    }
  }
}
```

**Live components resolve by platform suffix:**

```
components/
  Map.web.tsx           → web target
  Map.native.tsx        → react-native target
  Map.swift             → ios target
  Map.kt                → android target

  // Or in platform folders:
  web/Map.tsx
  ios/Map.swift
  android/Map.kt
```

**Build commands:**

```bash
paperclip compile                    # All targets
paperclip compile --platform web     # Just web
paperclip compile --platform ios     # Just iOS
paperclip watch --platform web       # Watch mode for web
```

### Framework Constraint (Within a Platform)

**No mixing frameworks within a single platform target.** Pick one per platform:
- Web: React OR Vue OR Svelte (not mixed)
- iOS: SwiftUI OR UIKit (not mixed)
- Android: Compose OR Views (not mixed)

This keeps compiled output clean. But you CAN target multiple platforms from the same .pc source.

---

## Distribution & Developer Experience

### Single Rust Executable

The entire Paperclip system runs from a single binary:

```bash
# Download and run - that's it
./paperclip

# Opens visual designer in browser at localhost:3000
# Watches current directory for .pc files
# gRPC server starts automatically
```

No Node, no npm, no dependencies. Just download and run.

**Commands:**
```bash
./paperclip              # Start designer UI + server
./paperclip build        # Compile all .pc files
./paperclip watch        # Watch and rebuild
./paperclip init         # Initialize new project
./paperclip fmt          # Format .pc files
```

### Downloadable Desktop App

A native desktop wrapper around the web experience:

- **macOS:** `Paperclip.app` (universal binary)
- **Windows:** `Paperclip.exe`
- **Linux:** `paperclip.AppImage`

Built with Tauri (Rust + webview) for minimal size (~10MB).

**Features:**
- Native file dialogs
- System menu integration
- Auto-updates
- Offline-first (no account required)

### VSCode Extension with Realtime Sync

The VSCode extension connects to the Paperclip server and syncs changes in realtime:

```
┌─────────────────┐     WebSocket      ┌─────────────────┐
│   VSCode        │◄──────────────────►│  Paperclip      │
│   Extension     │   bidirectional    │  Server         │
│                 │   sync             │                 │
│  - Edit .pc     │                    │  - Parse        │
│  - See errors   │                    │  - Evaluate     │
│  - Autocomplete │                    │  - Preview      │
└─────────────────┘                    └─────────────────┘
```

**Realtime sync means:**
- Changes in VSCode instantly appear in designer preview
- Changes in designer instantly appear in VSCode
- Cursor position synced (see where designer is editing)
- Errors appear inline as you type

---

## Figma Import

Paperclip is meant to **replace Figma** for component work, but Figma import helps with migration:

```bash
./paperclip import figma <figma-url>
```

**Import process:**
1. Fetch Figma file via API
2. Convert frames to .pc components
3. Extract colors/fonts as tokens
4. Generate initial component structure

**What imports well:**
- Layout structure (frames, auto-layout → flexbox)
- Design tokens (colors, typography, spacing)
- Component hierarchy
- Basic styles

**What needs manual cleanup:**
- Complex interactions
- Responsive breakpoints
- Slot definitions
- Live component wiring

**Philosophy:** Import gets you 70% there, you refine the rest in Paperclip.

---

## Doc-Comments for Metadata

Use doc-comments to add metadata that the UI can read:

```javascript
/**
 * Primary action button
 * @locked Approved by design team 2026-01-15
 * @author jane@company.com
 * @figma https://figma.com/file/xxx
 */
public component Button {
  render button { ... }
}
```

**Supported annotations:**
| Annotation | Purpose |
|------------|---------|
| `@locked <reason>` | Prevent accidental edits in designer |
| `@deprecated <message>` | Show warning when used |
| `@author <email>` | Track ownership |
| `@figma <url>` | Link to original Figma design |
| `@see <component>` | Related components |
| `@example` | Usage examples shown in component palette |

**In the designer:**
- Locked components show a lock icon
- Editing locked components requires explicit unlock
- Deprecation warnings shown when dragging deprecated components

---

## Future Considerations

**Deferred to v2:**
- **Cloud-rendered native previews** - Real SwiftUI/Compose streamed from cloud workers (no local Xcode/Android Studio needed)
- **CRDT-based collaboration** - Real-time multiplayer editing
- **CRDT for AST patches** - Efficient incremental updates
- **PHP compiler** - Server-side rendering
- **Python compiler** - Django/Flask templates
- **Ruby compiler** - Rails views
- Yew compiler (Rust/WASM output)
- Android/Jetpack Compose compiler
- UIKit compiler (non-SwiftUI iOS)
- Design system marketplace
- Advanced animations/transitions
- AI-powered design suggestions

**Stretch Goal: End-to-End Website Builder**

The north star vision: **Designers never leave Paperclip. Engineers give them components. Designers ship websites.**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Paperclip Website Builder                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Engineer provides:           Designer creates:                  │
│  ┌─────────────────┐         ┌─────────────────────────────┐    │
│  │ <Header />      │         │ Landing page using Header,  │    │
│  │ <ProductCard /> │    →    │ ProductCard, Footer, etc.   │    │
│  │ <Footer />      │         │                              │    │
│  │ <ContactForm /> │         │ Blog page                    │    │
│  │ ...             │         │ Pricing page                 │    │
│  └─────────────────┘         │ About page                   │    │
│                              └─────────────────────────────┘    │
│                                                                  │
│  Designer ships directly:                                        │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ [Preview] [Publish to staging] [Deploy to production]   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**How it works:**
1. Engineers build components in .pc (Header, Cards, Forms, etc.)
2. Engineers expose components with clear props + sample data
3. Designers compose pages using those components in the canvas
4. Designers bind to CMS data (Contentful, Sanity, etc.) via visual controls
5. Designers preview with real content
6. Designers publish directly (static site gen or SSR)

**Why this is possible:**
- Paperclip already compiles to React/etc. - can compile to static HTML
- Sample data mechanism maps directly to CMS data binding
- Live components handle the interactive bits
- AI agent can help with complex layouts

**What engineers still do:**
- Build the core component library
- Create live components for interactive features
- Set up CMS integrations
- Configure deploy targets

**What designers can do alone:**
- Create new pages from component palette
- Arrange layouts on the canvas
- Bind content to CMS fields
- Adjust responsive breakpoints
- Publish changes without engineering help

This makes Paperclip a **Webflow/Framer alternative** that outputs real code, not proprietary runtime

---

## References & Research

### Internal References
- Old parser: `../paperclip/libs/parser/src/pc/parser.rs`
- Old evaluator: `../paperclip/libs/evaluator/src/`
- Old compiler: `../paperclip/libs/compiler-react/src/code_compiler.rs`
- Old workspace: `../paperclip/libs/workspace/`
- Old designer: `../paperclip/libs/designer/`
- DSL syntax: `../paperclip/docs/syntax.md`
- Proto definitions: `../paperclip/libs/proto/src/`

### UX Reference Products (Ideal DX Inspiration)

**[Framer](https://www.framer.com/)** - Primary UX reference for Paperclip designer

| Feature | Framer Approach | Paperclip Equivalent |
|---------|-----------------|---------------------|
| **Canvas** | Free-form visual canvas, familiar to Figma users | DOM-based canvas with pan/zoom |
| **Design → Live** | What you design IS the final website, no handoff | .pc compiles to real components |
| **Responsive** | Breakpoints directly in canvas | Responsive preview modes |
| **Animations** | Built-in smooth effects, no external tools | Variants + CSS transitions |
| **AI Generation** | Describe site → generates designed, animated version | AI agent generates .pc code |
| **Publish** | Single-click publish to live site | Compile + deploy workflow |
| **Figma Import** | Seamless import from Figma | Potential future feature |

Key Framer DX principles to emulate:
- **No blank canvas anxiety** - AI generates starting point
- **What you see is what ships** - No design-to-code translation loss
- **Single tool** - Design, prototype, publish in one place
- **Delightful interactions** - Smooth animations, responsive feedback

**[Pencil.dev](https://www.pencil.dev/)** - "Design on canvas. Land in code."

| Feature | Pencil Approach | Paperclip Overlap |
|---------|-----------------|-------------------|
| **IDE Integration** | Design directly in your preferred IDE | VSCode extension + standalone |
| **No Context Switch** | Stay in code editor, design visually | Integrated code editor in designer |
| **Output is Code** | Designs land as real code files | .pc files are the source of truth |

Key Pencil principle: **Bring design into where developers already work.**

**[Builder.io](https://www.builder.io/)** - AI Frontend Engineer

| Feature | Builder Approach | Paperclip Overlap |
|---------|-----------------|-------------------|
| **Figma → Code** | Plugin converts Figma to production code | Could import Figma as .pc |
| **Visual Copilot** | AI generates and refines code | AI agent with MCP tools |
| **Framework Agnostic** | Works with any modern framework | Compiles to React, Vue, Svelte, etc. |
| **Living Code** | Generates maintainable code, not static exports | .pc is the source, compiles clean |

Key Builder principle: **Bridge design and code without duplication.**

### How Paperclip Differentiates

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Design Tool Landscape                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Figma ──────────── Design only, needs dev handoff                       │
│                                                                          │
│  Framer ─────────── Design + publish (their runtime)                     │
│                                                                          │
│  Builder.io ─────── Figma import + AI code gen                           │
│                                                                          │
│  Pencil.dev ─────── IDE-integrated visual editing                        │
│                                                                          │
│  Paperclip ──────── Design + compile to YOUR codebase                    │
│                     ├─ Outputs real React/Vue/Svelte                     │
│                     ├─ No vendor runtime lock-in                         │
│                     ├─ AI agent that edits .pc source                    │
│                     ├─ Multi-platform (web, iOS, Android)                │
│                     └─ Engineer + Designer same tool                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**Paperclip's unique position:** Visual design tool that outputs to YOUR tech stack, not a proprietary runtime. Designers and engineers share the same source of truth (.pc files).

### External References
- [MCP Specification](https://modelcontextprotocol.io/)
- [Tonic gRPC](https://github.com/hyperium/tonic)
- [WASM Bindgen](https://rustwasm.github.io/wasm-bindgen/)

### Research References (from /deepen-plan)
- [Logos Lexer](https://github.com/maciejhirsz/logos) - Fast Rust lexer generator
- [Chumsky Parser](https://github.com/zesterer/chumsky) - Parser combinator with error recovery
- [Bumpalo Arena](https://github.com/fitzgen/bumpalo) - Fast bump allocation
- [Sandpack](https://sandpack.codesandbox.io/) - Live component preview
- Machine + Engines pattern - See `~/Developer/fourthplaces/shay/packages/app/src/core/` and legacy Paperclip `libs/designer/src/machine/`
- [@dnd-kit](https://dndkit.com/) - Modern drag and drop for React
- [Connect-Web](https://connectrpc.com/docs/web/getting-started) - Modern gRPC-web client

---

## Appendix: .pc Syntax Quick Reference

```javascript
// === IMPORTS ===
import "./tokens.pc" as tokens
import "./button.pc" as button

// === LIVE COMPONENT IMPORTS (NEW) ===
// Auto-discovered from *.live.tsx files
import { Map } from "@app/GoogleMap"
import { ModelViewer } from "@app/ModelViewer"
import { PaymentForm, AddToCartButton } from "@app/components"

// External packages
import { Chart } from "@acme/charts"

// === TOKENS (design primitives) ===
public token primaryColor #3366FF
public token spacing04 16px
public token fontFamily Inter, sans-serif

// === STYLES (reusable mixins) ===
public style bodyText {
  font-family: var(fontFamily)
  font-size: 14px
  line-height: 1.5
}

public style heading extends bodyText {
  font-weight: 600
  font-size: 24px
}

// === COMPONENTS ===
public component Card {
  // Variants with CSS selector triggers
  variant elevated trigger { ".elevated" }
  variant hover trigger { ":hover" }
  variant mobile trigger { "@media (max-width: 600px)" }

  // Render tree
  render div root {
    style {
      background: white
      border-radius: 8px
      padding: var(spacing04)
    }
    style variant elevated {
      box-shadow: 0 4px 12px rgba(0,0,0,0.15)
    }
    style variant mobile {
      padding: 8px
    }

    // Named slots for composition
    slot header
    slot children
  }
}

// === USAGE ===
Card elevated {
  insert header {
    text "Card Title"
  }
  insert children {
    text "Card content goes here"
  }
}

// === EXPRESSIONS (NEW - formula-like only) ===
// Allowed:
text {item.name}                      // Data binding
text {price * quantity}               // Arithmetic
text {user.firstName + " " + user.lastName}  // Concatenation
text {formatCurrency(price)}          // Registered formatters
text {items.length > 0 ? "Has items" : "Empty"}  // Simple ternary

// NOT allowed (register a component/action instead):
// - Multi-line expressions
// - Control flow (if/for inside expression)
// - Function definitions
// - Async/await
// - Side effects

// === ITERATION (NEW) ===
public component ProductGrid {
  render div.grid {
    for product in data.products {
      div.card {
        img(src=product.image)
        text.title {product.name}
        text.price {"$" + product.price}
      }
    } empty {
      // Designer-defined empty state
      text.empty "No products found"
    }
  }
}

// === CONDITIONALS (NEW) ===
public component ProductCard {
  render div {
    text.title {product.name}

    if product.inStock {
      AddToCartButton(
        productId=product.id,
        onSuccess=showToast("Added!")
      )
    } else {
      text.soldout "Sold Out"
    }

    if product.onSale {
      span.badge "SALE"
    }
  }
}

// === LIVE COMPONENTS (NEW) ===
// Import auto-discovered components
import { Map } from "@app/GoogleMap"
import { Modal } from "@app/Modal"

public component LocationPage {
  render div {
    text.heading "Our Office"

    // Live component with props (Map.live.tsx auto-discovered)
    Map(
      lat=37.7749,
      lng=-122.4194,
      zoom=12,
      markers=data.locations,
      onMarkerClick=showPopup(marker.id)
    )

    // Live component with slot content
    Modal(isOpen=showDetails) {
      slot header {
        text "Location Details"
      }
      slot body {
        text {selectedLocation.address}
      }
    }
  }
}

// === BUILT-IN ACTIONS (NEW) ===
// Navigation
Button(onClick=navigate("/products/{id}")) { text "View" }
Button(onClick=back()) { text "Go Back" }
Button(onClick=openUrl("https://example.com")) { text "External" }

// UI State
Button(onClick=toggle(menuOpen)) { text "Toggle Menu" }
Button(onClick=show(modal)) { text "Open Modal" }
Button(onClick=hide(dropdown)) { text "Close" }

// Notifications
AddToCartButton(
  productId=item.id,
  onSuccess=showToast("Added to cart!"),
  onError=showError(error.message)
)

// Simple state mutations
Button(onClick=set(selectedId, item.id)) { text "Select" }
Button(onClick=append(items, newItem)) { text "Add" }
Button(onClick=remove(items, index)) { text "Remove" }

// === SAMPLE DATA (NEW - inline in .pc file) ===
sample data.products [
  { id: "1", name: "Shirt", price: 30, inStock: true },
  { id: "2", name: "Pants", price: 50, inStock: true },
  { id: "3", name: "Shoes", price: 80, inStock: false }
]

sample data.user {
  firstName: "John",
  lastName: "Doe",
  isAdmin: false,
  isPremium: true
}

// Presets for testing
sample preset "empty" data.products []
sample preset "single" data.products [{ id: "1", name: "Shirt", price: 30 }]
sample preset "stress" data.products [...100 items...]
```

---

## Appendix: Component Registry (Auto-Discovery)

**No `defineConfig` needed.** Developers choose how to mark components for Paperclip.

### Developer Chooses the Registration Style

**Option A: JSDoc marker (zero imports)**
```typescript
// components/GoogleMap.tsx - normal component file

/** @paperclip */
export function GoogleMap({ lat, lng, zoom, onMarkerClick }) {
  // ... your normal React component
  return <div>...</div>
}
```

**Option B: Simple registration call**
```typescript
// components/GoogleMap.tsx
import { live } from '@paperclip/live'

export function GoogleMap({ lat, lng, zoom, onMarkerClick }) {
  // ... normal React component
}

// One line - types inferred from TypeScript
live(GoogleMap)
```

**Option C: With explicit metadata (when inference isn't enough)**
```typescript
// components/GoogleMap.tsx
import { live } from '@paperclip/live'

export function GoogleMap({ lat, lng, zoom, onMarkerClick }) {
  // ...
}

live(GoogleMap, {
  props: {
    zoom: { default: 12, min: 1, max: 20 },  // Add constraints
  },
  events: {
    onMarkerClick: { payload: 'marker' },
  },
  slots: ['popup'],
  description: 'Interactive map with markers',
})
```

**Option D: File convention (if you prefer separation)**
```
components/
  GoogleMap.live.tsx      ← Auto-discovered as @app/GoogleMap
  PaymentForm.live.tsx
```

### Config: Tell Paperclip What to Scan For

```typescript
// paperclip.config.ts
export default {
  live: {
    // Pick your style (or combine them):
    markers: ['@paperclip', '@live'],     // JSDoc tags to look for
    imports: ['@paperclip/live'],          // Scan files with this import
    patterns: ['**/*.live.tsx'],           // File naming convention

    // Where to scan (defaults to src/)
    include: ['src/components', 'src/features'],
    exclude: ['**/*.test.tsx', '**/*.stories.tsx'],
  }
}
```

### How Discovery Works

```
paperclip build (or watch)
  ↓
Scans files matching config criteria
  ↓
Finds components with markers/live() calls
  ↓
Extracts types from TypeScript
  ↓
Generates registry manifest (.paperclip/registry.json)
  ↓
Designer loads manifest → shows in component palette
```

### TypeScript Does the Heavy Lifting

With proper TypeScript, Paperclip infers most metadata:

```typescript
// components/AddToCartButton.tsx

interface Props {
  productId: string              // → required string prop
  quantity?: number              // → optional number prop
  onSuccess?: () => void         // → event with no payload
  onError?: (error: Error) => void  // → event with Error payload
}

/** @paperclip */
export function AddToCartButton({ productId, quantity = 1, onSuccess, onError }: Props) {
  // ...
}

// Paperclip infers:
// - props: productId (required), quantity (optional, default: 1)
// - events: onSuccess, onError
// - No explicit live() call needed
```

### In the Designer

Components appear automatically in the component palette:

```
┌─────────────────────────────────────┐
│ COMPONENTS                          │
├─────────────────────────────────────┤
│ 📦 @app                             │
│   ├── AddToCartButton              │
│   ├── GoogleMap                    │
│   ├── PaymentForm                  │
│   └── ModelViewer                  │
│                                     │
│ 📦 @paperclip/ui (built-in)        │
│   ├── Button                       │
│   ├── Input                        │
│   └── Modal                        │
└─────────────────────────────────────┘
```

### Data Sources (Also Auto-Discovered)

```typescript
// data/products.data.ts
import { dataSource } from '@paperclip/live'

export const products = dataSource({
  fetch: () => api.getProducts(),
  refresh: 30_000,

  // Sample data for designer preview
  sample: [
    { id: '1', name: 'Shirt', price: 30 },
    { id: '2', name: 'Pants', price: 50 },
  ]
})
```

### Formatters (Also Auto-Discovered)

```typescript
// formatters/currency.formatter.ts
import { formatter } from '@paperclip/live'

export const formatCurrency = formatter((value: number) =>
  `$${value.toFixed(2)}`
)

// Now usable in .pc expressions:
// text {formatCurrency(price)}
```

### Config File (Optional, Minimal)

```typescript
// paperclip.config.ts - only if you need to customize
export default {
  // Where to scan (defaults to src/)
  scan: ['src/components', 'src/data'],

  // External packages with live components
  packages: ['@acme/maps', '@acme/charts'],
}
```

### How Compilation Works

The `liveComponent` wrapper is **transparent at compile time**. It only provides metadata for the designer - the compiled output imports the underlying component directly.

**.pc source:**
```javascript
import { Map } from "@app/GoogleMap"
import { AddToCartButton } from "@app/AddToCartButton"

component ProductPage {
  render div {
    text.title {product.name}
    Map(lat=store.lat, lng=store.lng, onMarkerClick=selectStore(marker.id))
    AddToCartButton(productId=product.id, onSuccess=showToast("Added!"))
  }
}
```

**Compiled React output:**
```tsx
// ProductPage.tsx (generated)
import styles from './ProductPage.module.css'
import { Map } from '../components/GoogleMap.live'        // Direct import to .live.tsx
import { AddToCartButton } from '../components/AddToCartButton.live'
import { useData, useActions } from '@paperclip/runtime'

export interface ProductPageProps {
  product: { id: string; name: string }
  store: { lat: number; lng: number }
}

export function ProductPage({ product, store }: ProductPageProps) {
  const { selectStore, showToast } = useActions()

  return (
    <div className={styles.root}>
      <span className={styles.title}>{product.name}</span>
      <Map
        lat={store.lat}
        lng={store.lng}
        onMarkerClick={(marker) => selectStore(marker.id)}
      />
      <AddToCartButton
        productId={product.id}
        onSuccess={() => showToast("Added!")}
      />
    </div>
  )
}
```

**Key points:**
- `@app/GoogleMap` → resolves to `../components/GoogleMap.live.tsx`
- `liveComponent()` wrapper is stripped - just imports the React component
- Props are passed through directly
- Event handlers are converted to arrow functions
- Data bindings become props on the generated component
- Built-in actions (`showToast`, etc.) come from `@paperclip/runtime`
```

---

## Appendix: What Paperclip Will Never Support

Defining the negative space is clarifying:

| Feature | Why Not |
|---------|---------|
| **Multi-statement expressions** | Expressions are formulas, not code |
| **Async in expressions** | Side effects belong in registered components |
| **Component-internal state** | State lives in registered components or data sources |
| **Imperative event handlers** | Only declarative actions (navigate, toggle, show) |
| **CSS-in-JS runtime** | CSS compiles away, no runtime |
| **Per-instance overrides in loops** | Edit the template, not instances |
| **Hidden computed values** | All computations show visual indicators |
| **Complex conditionals** | `if/else` only, no switch/match/pattern matching |
| **Nested iteration** | One level of `for` at a time (nest via components) |

This keeps Paperclip a **language**, not a **framework**.

### Why .pc Will Stay Stable (The Language Won't Sprawl)

Skeptics will ask: *"This is just another DSL that will rot."*

Here's why .pc is small enough to remain stable:

| Constraint | Why It Matters |
|------------|----------------|
| **Declarative only** | No execution model to version |
| **No user-defined functions** | Can't accumulate language features |
| **No imports of code** | Only .pc files and registered components |
| **No side effects** | Nothing to debug at runtime |
| **No async** | No execution timing semantics |
| **No runtime semantics** | Compiles away completely |

The entire .pc language fits on one page. That's intentional. **If a feature would require a second page, it belongs in a registered component instead.**

---

## Appendix: Concrete "Forbidden → Do This Instead" Examples

Abstract rules need concrete examples to be felt:

### Example 1: Debounced Search

**Designer wants:** Search input that waits 300ms before filtering

```javascript
// ❌ NOT allowed in .pc (would require async/timing)
input(onInput=debounce(filter, 300))
```

```javascript
// ✅ Engineer registers a live component
// components/SearchInput.live.tsx
/** @paperclip */
export function SearchInput({ onSearch, debounceMs = 300 }) {
  const [value, setValue] = useState('')
  const debouncedSearch = useDebouncedCallback(onSearch, debounceMs)

  return <input value={value} onChange={e => {
    setValue(e.target.value)
    debouncedSearch(e.target.value)
  }} />
}
```

```javascript
// Designer uses it in .pc
SearchInput(onSearch=setFilter(query))
```

### Example 2: Form Validation

**Designer wants:** Show error if email is invalid

```javascript
// ❌ NOT allowed (complex conditional logic)
if !isValidEmail(email) {
  text.error "Invalid email"
}
```

```javascript
// ✅ Engineer provides validated input
// components/EmailInput.live.tsx
/** @paperclip */
export function EmailInput({ value, onChange, showError = true }) {
  const isValid = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)
  return (
    <div>
      <input type="email" value={value} onChange={e => onChange(e.target.value)} />
      {showError && !isValid && value && <span className="error">Invalid email</span>}
    </div>
  )
}
```

```javascript
// Designer uses it in .pc - error handling is encapsulated
EmailInput(value=form.email, onChange=set(form.email, value))
```

### Example 3: API Data Fetching

**Designer wants:** Show list of products from API

```javascript
// ❌ NOT allowed (async, side effects)
products = await fetch("/api/products")
for product in products { ... }
```

```javascript
// ✅ Engineer registers a data source
// data/products.data.ts
import { dataSource } from '@paperclip/live'

export const products = dataSource({
  fetch: () => api.getProducts(),
  sample: [
    { id: '1', name: 'Shirt', price: 30 },
    { id: '2', name: 'Pants', price: 50 },
  ]
})
```

```javascript
// Designer uses sample data in preview, real data at runtime
for product in data.products {
  ProductCard(product=product)
} empty {
  text "No products found"
}
```

---

## Appendix: Multi-Platform Compilation Limits (Honest Admission)

We support React, Vue, Svelte, SwiftUI, Compose—but **not everything maps cleanly**.

### Layout Primitives That Don't Translate 1:1

| .pc Construct | Web | iOS | Android | Notes |
|---------------|-----|-----|---------|-------|
| `display: flex` | ✅ Native | ✅ SwiftUI Stacks | ✅ Compose Row/Column | Clean mapping |
| `position: absolute` | ✅ Native | ⚠️ ZStack + offset | ⚠️ Box with offset | Approximation |
| `overflow: scroll` | ✅ Native | ⚠️ ScrollView wrapper | ⚠️ LazyColumn/Row | Behavioral differences |
| `box-shadow` | ✅ Native | ⚠️ .shadow() modifier | ⚠️ elevation | Visual differences |
| `backdrop-filter` | ⚠️ Browser support | ✅ .blur() | ❌ Not supported | Platform gap |

### What We Do About It

1. **Warn at compile time** if a style won't translate to target platform
2. **Platform-specific overrides** let you handle edge cases:
   ```javascript
   style { box-shadow: 0 4px 12px rgba(0,0,0,0.15) }
   style platform:ios { /* uses .shadow() */ }
   style platform:android { elevation: 8 }
   ```
3. **Document known gaps** in compiler output comments

### Animation & Transition Differences

Animations are **not supported in v1**. Each platform has fundamentally different animation models:
- Web: CSS transitions, keyframes
- iOS: SwiftUI animations, Core Animation
- Android: Compose animations, MotionLayout

We'll address animations in v2 with a platform-aware animation spec.

### Text Rendering Differences

Font rendering varies by platform. Paperclip handles this by:
- Using system font stacks by default
- Allowing platform-specific font overrides
- Warning when a font isn't available on target platform

**The honest truth:** Multi-platform from one source is 80% parity, not 100%. The remaining 20% requires platform-specific overrides or accepting visual differences.

---

## Appendix: Runtime Clarification

When we say **"zero runtime"**, we mean:

> **No Paperclip runtime.** The compiled output is pure framework code.

What EXISTS at runtime:
- Your framework (React, SwiftUI, etc.)
- Your live components (they're real components)
- A tiny `@paperclip/runtime` helper for built-in actions (~2KB)

What does NOT exist at runtime:
- Paperclip interpreter
- Virtual DOM diffing engine
- .pc parser
- Any Paperclip-specific reconciliation

The `.pc` file is fully compiled away. If you delete Paperclip from your machine, your compiled components keep working.

---

## Appendix: AI Agent Constraints (Operational Limits)

The embedded AI agent is powerful but constrained:

### What AI Can Do
- ✅ Edit styles (colors, spacing, typography)
- ✅ Add/remove/reorder elements
- ✅ Wire props to existing data bindings
- ✅ Use registered live components
- ✅ Use built-in actions (navigate, toggle, show)
- ✅ Modify sample data

### What AI Cannot Do
- ❌ **Introduce new live components** (must be registered by engineer first)
- ❌ **Widen expression power** (can't add async, imports, functions)
- ❌ **Write imperative logic** (only declarative .pc syntax)
- ❌ **Modify engineer code** (live components are read-only to AI)
- ❌ **Hide changes** (all changes are diff-visible in source)

### Preview-Before-Apply (Non-Negotiable)

Every AI change shows a preview diff before applying:

```
┌─────────────────────────────────────────────────────────┐
│ AI wants to change:                                      │
├─────────────────────────────────────────────────────────┤
│  - padding: 16px                                         │
│  + padding: 24px                                         │
│                                                          │
│  + border-radius: 8px                                    │
├─────────────────────────────────────────────────────────┤
│           [Preview]  [Apply]  [Reject]                   │
└─────────────────────────────────────────────────────────┘
```

**No silent mutations.** This is how you keep AI trustworthy.

---

## Appendix: Permissive vs Strict (Naming the Distinction)

Paperclip is:
- **Structurally permissive** — designers can arrange anything, wire any props, make "wrong" layouts
- **Semantically strict** — expressions are formulas only, no logic creep, no hidden computation

This is intentional. The system allows creative freedom while preventing architectural chaos.

**Analogy:** A spreadsheet lets you put any formula in any cell (structurally permissive), but formulas can't launch missiles or read files (semantically strict).

---

## Appendix: Why Not Just Code + Storybook?

This is the most common objection. Here's the sharp answer:

| Aspect | Storybook | Paperclip |
|--------|-----------|-----------|
| **When** | After components exist | Where components are authored |
| **Source of truth** | Code files | .pc files (visual-first) |
| **Designer access** | View only | Full authoring |
| **Variants** | Written in code | Defined visually |
| **Sample data** | Mock files | First-class, inline |
| **Real-time preview** | Requires rebuild | <40ms live updates |

**Storybook documents components after they exist.**

**Paperclip is where components are authored visually, with source-level traceability, before code hardens.**

The workflow difference:

```
Storybook:
  Engineer writes component → Engineer writes stories → Designer views in Storybook
  (Designer is consumer)

Paperclip:
  Designer authors in .pc → Engineer registers live components → Compiles to code
  (Designer is author)
```

---

## Appendix: Design Tokens Access (No Imports, But Globals)

You asked: *"If no imports, how do designers access design tokens?"*

Tokens are imported from other .pc files (not code files):

```javascript
// tokens.pc
public token primaryColor #3366FF
public token spacing04 16px
public token fontFamily Inter, sans-serif
```

```javascript
// button.pc
import "./tokens.pc" as tokens

component Button {
  render button {
    style {
      background: var(tokens.primaryColor)
      padding: var(tokens.spacing04)
      font-family: var(tokens.fontFamily)
    }
  }
}
```

Additionally, a global `theme` object is injected from configuration:

```typescript
// paperclip.config.ts
export default {
  theme: {
    colors: {
      primary: '#3366FF',
      secondary: '#6B7280',
    },
    spacing: {
      sm: '8px',
      md: '16px',
      lg: '24px',
    }
  }
}
```

```javascript
// Accessible in any .pc file without import
style {
  background: var(theme.colors.primary)
  padding: var(theme.spacing.md)
}
```

---

## Appendix: Component Discovery Manifest

The CLI outputs a manifest of discovered components so designers know what's available:

```bash
paperclip registry --output .paperclip/registry.json
```

```json
{
  "components": [
    {
      "name": "GoogleMap",
      "path": "@app/GoogleMap",
      "source": "src/components/GoogleMap.live.tsx",
      "props": {
        "lat": { "type": "number", "required": true },
        "lng": { "type": "number", "required": true },
        "zoom": { "type": "number", "default": 12 }
      },
      "events": ["onMarkerClick"],
      "description": "Interactive Google Map"
    },
    {
      "name": "AddToCartButton",
      "path": "@app/AddToCartButton",
      "source": "src/components/AddToCartButton.live.tsx",
      "props": {
        "productId": { "type": "string", "required": true },
        "quantity": { "type": "number", "default": 1 }
      },
      "events": ["onSuccess", "onError"]
    }
  ],
  "dataSources": [
    {
      "name": "products",
      "path": "data.products",
      "source": "src/data/products.data.ts",
      "sampleCount": 3
    }
  ],
  "formatters": [
    {
      "name": "formatCurrency",
      "source": "src/formatters/currency.formatter.ts"
    }
  ]
}
```

This manifest:
- Powers the Component Library panel in the designer
- Enables autocomplete in Monaco editor
- Lets designers discover what engineers have registered
- Updates automatically on `paperclip watch`
