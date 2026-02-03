---
title: "feat: Add Style Editing Pane with Layer System"
type: feat
date: 2026-02-03
deepened: 2026-02-03
---

# Add Style Editing Pane with Layer System

## Enhancement Summary

**Deepened on:** 2026-02-03
**Research agents used:** 11 (Frontend Design, TypeScript Reviewer, Architecture Strategist, Performance Oracle, Security Sentinel, Code Simplicity Reviewer, Pattern Recognition, Frontend Races Reviewer, Agent-Native Reviewer, Best Practices Researcher, Context7 MCP)

### Key Improvements from Research

1. **Simplification**: Defer variants, tokens, and mixins to Phase 2 - reduces initial scope by 60-65%
2. **Security hardening**: CSS injection prevention, XSS mitigation, input sanitization
3. **Performance optimizations**: Debounced updates, virtualized lists, memoized selectors
4. **Race condition prevention**: AbortController for fetch cleanup, proper async state management
5. **Architecture gap**: Engine composition infrastructure only supports 1 engine per machine

### Critical Findings

| Finding | Source | Priority |
|---------|--------|----------|
| Engine composition gap - can't run multiple engines | Architecture Strategist | **Critical** |
| CSS injection vulnerability in style values | Security Sentinel | **Critical** |
| Missing AbortController for fetch cleanup | Frontend Races Reviewer | **High** |
| No debouncing on style changes | Performance Oracle | **High** |
| Agent-native score 4/10 - no programmatic API | Agent-Native Reviewer | **Medium** |

### Recommended Phase 1 Scope Reduction

Per Code Simplicity Reviewer, defer to Phase 2:
- ❌ Variant support (complex UI, mutation format unclear)
- ❌ Token autocomplete (requires token list API)
- ❌ Mixin selector (requires mixin resolution)
- ❌ Computed style origins (nice-to-have)
- ✅ Basic inline style editing only

---

## Overview

Implement a style editing pane (right sidebar) and layer/file navigation panel (left sidebar) for the paperclip-next designer, modeled after the original paperclip designer at `~/Developer/crcn/paperclip`. This enables visual CSS property editing with real-time preview, variant-aware styling, token autocomplete, and hierarchical document navigation.

## Problem Statement / Motivation

Currently, the paperclip-next designer only supports:
- Canvas pan/zoom
- Frame selection and resizing
- Frame position mutations

Users cannot:
- Select individual elements within frames
- View or edit CSS properties visually
- Navigate the document tree
- Work with variants or style mixins

The original paperclip designer has these features fully implemented. We need feature parity while adapting to paperclip-next's architecture (Machine + Engines pattern, CRDT-backed documents, SSE-based updates).

### Research Insights: Scope Reduction

**Code Simplicity Reviewer** recommends a minimal Phase 1:

> The current plan overengineers by bundling variants, tokens, and mixins into Phase 1. These add complexity without validating the core mutation flow. Start with inline styles only.

**Simplified Phase 1 Scope:**
- ✅ Element selection (canvas click + layer tree)
- ✅ Display inline styles for selected element
- ✅ Edit existing style properties
- ✅ Add new style properties
- ✅ Remove style properties
- ❌ Variants → Phase 2
- ❌ Token autocomplete → Phase 2
- ❌ Mixins → Phase 2
- ❌ Computed style origins → Phase 2

## Research Findings

### Original Paperclip Designer Analysis

**Style Panel Architecture** (`~/Developer/crcn/paperclip/libs/designer/src/ui/logic/Editor/EditorPanels/RightSidebar/StylePanel/`):

| Component | Purpose | File |
|-----------|---------|------|
| `StylePanel` | Container with Variants, Mixins, Declarations sections | `index.tsx` |
| `Declarations` | Lists all CSS properties for selected element | `Declarations/index.tsx` |
| `Declaration` | Single property row with name/value editing | `Declarations/Declaration.tsx` |
| `DeclarationValue` | Value input with autocomplete and color picker | `Declarations/DeclarationValue/index.tsx` |
| `Mixins` | Style extends/mixin management | `Mixins/index.tsx` |
| `Variants` | Variant combination selector | `Variants/index.tsx` |

**Key State Pattern**:
```typescript
// Original uses Redux-style selectors
const style = useSelector(getSelectedExprStyles);  // ComputedStyleMap
const targetId = useSelector(getStyleableTargetId);

// Dispatches semantic events
dispatch({
  type: "ui/styleDeclarationsChangeCompleted",
  payload: { values: { [property]: value }, imports }
});
```

**Layer Panel Architecture** (`~/Developer/crcn/paperclip/libs/designer/src/ui/logic/Editor/EditorPanels/LeftSidebar/Layers/`):
- Recursive tree rendering from AST
- Supports: Components, Elements, Text, Slots, Inserts, Instances
- Click-to-select synchronizes with canvas

### DSL Gaps Identified

| Feature | Original Paperclip | paperclip-next | Gap |
|---------|-------------------|----------------|-----|
| Token reference syntax | `var(token.name)` | `{tokenName}` in evaluator | Different syntax |
| SetInlineStyle mutation | Fully implemented | Returns `Noop` | **Critical - not implemented** |
| Style block indexing | Part of AST | Placeholder only | **Critical - needed for CRDT** |
| Variant-specific mutations | Supports `variant_ids` field | No variant field | **Important - needed for variants** |
| Computed style origins | `InheritedDeclInfo` | Not tracked | Nice-to-have |

### paperclip-next Patterns to Follow

**Machine + Engines Pattern** (NOT Redux/Zustand):
```typescript
// packages/designer/src/machine/index.ts
export const DesignerMachine = defineMachine<DesignerEvent, DesignerState, Props>({
  reducer,           // Pure state transitions
  initialState,
  engine: createSSEEngine,  // Side effects
});

// Usage
const styles = DesignerMachine.useSelector((s) => s.selectedStyles);
const dispatch = useDispatch<DesignerEvent>();
```

**Mutation Flow**:
1. UI dispatches event (e.g., `style/changed`)
2. Reducer applies optimistic update
3. Engine sends mutation to server via `sendMutation()`
4. Server applies to CRDT, re-evaluates, broadcasts VDOM patches
5. SSE delivers patches, reducer merges with pending mutations

### Research Insights: Architecture Gap

**Architecture Strategist** identified a critical infrastructure limitation:

> The current `defineMachine` only supports a single engine. Style editing will need a separate "StyleEngine" for debouncing and batching, but the infrastructure can't compose multiple engines.

**Current limitation** (`packages/common/src/machine/types.ts`):
```typescript
// Only supports ONE engine
engine: (machine: Machine<E, S>, propsRef: ...) => () => void;
```

**Options:**
1. **Workaround (recommended for Phase 1)**: Put all side effects in single SSE engine
2. **Fix infrastructure (Phase 2)**: Support `engines: Engine[]` array

**Pattern Recognition Specialist** also noted:
> `calculateResizedBounds` is duplicated in reducers.ts. Extract to shared utility before adding more frame/element logic.

## Proposed Solution

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Designer Layout                          │
├──────────────┬────────────────────────────┬────────────────────┤
│  Left Panel  │         Canvas             │    Right Panel     │
│              │                            │                    │
│  ┌────────┐  │  ┌──────────────────────┐  │  ┌──────────────┐  │
│  │ Layers │  │  │    Frame 1           │  │  │ Style Panel  │  │
│  │  Tree  │  │  │  ┌────────────────┐  │  │  │              │  │
│  │        │  │  │  │ Selected Elem  │  │  │  │ - Variants   │  │
│  ├────────┤  │  │  │    [div]       │  │  │  │ - Mixins     │  │
│  │ Files  │  │  │  └────────────────┘  │  │  │ - Properties │  │
│  │  Tree  │  │  └──────────────────────┘  │  │   - color    │  │
│  └────────┘  │                            │  │   - padding  │  │
│              │                            │  │   - ...      │  │
└──────────────┴────────────────────────────┴──────────────────────┘
```

### Data Flow

```
User edits "color: red" → "color: blue"
         │
         ▼
┌─────────────────────────────────────┐
│  dispatch({ type: "style/changed",  │
│    payload: { property: "color",    │
│               value: "blue" }})     │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Reducer: Optimistic update         │
│  state.editingStyles["color"] =     │
│    "blue"                           │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Engine: sendMutation({             │
│    type: "setInlineStyle",          │
│    node_id: sourceId,               │
│    property: "color",               │
│    value: "blue",                   │
│    variants: ["hover"]  // optional │
│  })                                 │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Server: Apply to Y.Text            │
│  Re-parse → Re-evaluate → VDOM      │
│  Broadcast patches via SSE          │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  SSE: document/loaded event         │
│  Reducer merges VDOM with pending   │
│  mutations                          │
└─────────────────────────────────────┘
```

## Technical Approach

### Phase 1: Backend Infrastructure

#### 1.1 Implement SetInlineStyle Mutation

**File:** `packages/workspace/src/mutation_handler.rs`

```rust
// Add to MutationHandler impl
fn apply_set_inline_style(
    &mut self,
    mutation_id: u64,
    node_id: &str,
    property: &str,
    value: &str,
    variants: Option<Vec<String>>,
    crdt_doc: &Doc,
) -> Result<MutationResult, MutationError> {
    let txn = crdt_doc.transact_mut();

    // 1. Find node in AST index
    let node_info = self.ast_index.get_node(node_id)
        .ok_or(MutationError::NodeNotFound(node_id.to_string()))?;

    // 2. Find or create style block
    let style_block = self.find_or_create_style_block(
        &node_info,
        variants.as_deref()
    )?;

    // 3. Apply edit via StickyIndex
    let edit = if value.is_empty() {
        // Remove property
        self.generate_remove_property_edit(&style_block, property)
    } else {
        // Set/update property
        self.generate_set_property_edit(&style_block, property, value)
    };

    // 4. Apply to Y.Text
    self.apply_edit_to_crdt(&txn, edit)?;

    Ok(MutationResult::Applied { mutation_id, version: self.version })
}
```

#### Research Insights: Security Hardening

**Security Sentinel** identified CSS injection risk:

> **CRITICAL**: Style values from user input must be sanitized to prevent CSS injection attacks. Malicious values like `red; } body { display: none }` could break the entire document.

**Required validation** (add to `apply_set_inline_style`):
```rust
// Validate property name - alphanumeric and hyphens only
fn validate_css_property(property: &str) -> Result<(), MutationError> {
    if !property.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(MutationError::InvalidProperty(property.to_string()));
    }
    Ok(())
}

// Validate value - no braces, no semicolons that could escape
fn validate_css_value(value: &str) -> Result<(), MutationError> {
    if value.contains('{') || value.contains('}') || value.contains(';') {
        return Err(MutationError::InvalidValue(value.to_string()));
    }
    Ok(())
}
```

**Additional security checks from Security Sentinel:**
- Rate limit mutations per client (prevent DoS)
- Validate node_id format (path traversal prevention)
- Log mutation attempts for audit trail

#### 1.2 Extend AST Index for Style Blocks

**File:** `packages/workspace/src/ast_index.rs`

```rust
pub struct StyleBlockInfo {
    pub node_id: String,           // Parent element's source_id
    pub variants: Vec<String>,     // e.g., ["hover", "disabled"]
    pub properties_start: usize,   // Position after opening {
    pub properties_end: usize,     // Position before closing }
    pub sticky_start: StickyIndex,
    pub sticky_end: StickyIndex,
}

impl AstIndex {
    pub fn index_style_blocks(&mut self, element: &Element, source: &str) {
        for style in &element.styles {
            let block_info = StyleBlockInfo {
                node_id: element.span.id.clone(),
                variants: style.variants.clone(),
                properties_start: style.span.start + "style ".len() + variants_len,
                properties_end: style.span.end - 1,
                // Create sticky indices for CRDT safety
                sticky_start: StickyIndex::from_position(properties_start),
                sticky_end: StickyIndex::from_position(properties_end),
            };
            self.style_blocks.insert(
                (element.span.id.clone(), style.variants.clone()),
                block_info
            );
        }
    }
}
```

#### 1.3 Add Mutation Type to Proto

**File:** `packages/proto/src/workspace.proto`

```protobuf
message SetInlineStyle {
  string node_id = 1;           // source_id of element
  string property = 2;          // CSS property name
  string value = 3;             // CSS value (empty = remove)
  repeated string variants = 4; // Optional variant combination
}

message RemoveInlineStyle {
  string node_id = 1;
  string property = 2;
  repeated string variants = 3;
}
```

### Phase 2: Designer State Extension

#### 2.1 State Types

**File:** `packages/designer/src/machine/state.ts`

```typescript
// Element selection within a frame
export interface ElementSelection {
  frameIndex: number;
  nodeId: string;        // semantic_id from VDOM
  sourceId: string;      // source_id for mutations
}

// Style value with origin tracking
export interface ComputedStyleValue {
  value: string;
  origin: "inline" | "mixin" | "inherited" | "default";
  sourceId?: string;     // Where the style comes from
  variants?: string[];   // If variant-specific
}

// Extend DesignerState
export interface DesignerState {
  // Existing...
  canvas: CanvasState;
  tool: ToolState;
  frames: Frame[];
  document: VDocument | null;

  // New: Element selection
  selectedElement: ElementSelection | null;

  // New: Active variant editing context
  activeVariants: string[];
  availableVariants: VariantInfo[];

  // New: Style editing state
  computedStyles: Record<string, ComputedStyleValue>;
  editingProperty: string | null;
  pendingStyleChanges: Record<string, string>;  // Optimistic

  // New: Available tokens for autocomplete
  availableTokens: TokenInfo[];

  // New: Layer panel state
  expandedNodes: Set<string>;
}

export interface VariantInfo {
  id: string;
  name: string;
  trigger?: string;  // ":hover", "@media (...)"
}

export interface TokenInfo {
  name: string;
  value: string;
  type: "color" | "spacing" | "typography" | "other";
  source: string;  // File path
}
```

#### 2.2 Events

**File:** `packages/designer/src/machine/state.ts`

```typescript
export type DesignerEvent =
  // Existing events...
  | BaseEvent<"canvas/panStart", Point>
  | BaseEvent<"frame/selected", { index: number }>
  // ...

  // New: Element selection
  | BaseEvent<"element/selected", ElementSelection>
  | BaseEvent<"element/deselected">
  | BaseEvent<"element/hovered", { nodeId: string } | null>

  // New: Style editing
  | BaseEvent<"style/propertyFocused", { property: string }>
  | BaseEvent<"style/propertyBlurred">
  | BaseEvent<"style/changed", { property: string; value: string }>
  | BaseEvent<"style/removed", { property: string }>

  // New: Variant selection
  | BaseEvent<"variant/toggled", { variantId: string }>
  | BaseEvent<"variant/cleared">

  // New: Layer panel
  | BaseEvent<"layer/nodeExpanded", { nodeId: string }>
  | BaseEvent<"layer/nodeCollapsed", { nodeId: string }>
  | BaseEvent<"layer/nodeClicked", { nodeId: string; sourceId: string }>;
```

#### 2.3 Reducers

**File:** `packages/designer/src/machine/reducers.ts`

```typescript
// Add to reducer switch
case "element/selected": {
  return {
    ...state,
    selectedElement: event.payload,
    // Fetch computed styles for the element
    computedStyles: extractStylesForElement(
      state.document,
      event.payload.nodeId,
      state.activeVariants
    ),
  };
}

case "style/changed": {
  const { property, value } = event.payload;
  return {
    ...state,
    pendingStyleChanges: {
      ...state.pendingStyleChanges,
      [property]: value,
    },
    // Update computed styles optimistically
    computedStyles: {
      ...state.computedStyles,
      [property]: {
        value,
        origin: "inline",
        variants: state.activeVariants,
      },
    },
  };
}

case "variant/toggled": {
  const { variantId } = event.payload;
  const activeVariants = state.activeVariants.includes(variantId)
    ? state.activeVariants.filter(v => v !== variantId)
    : [...state.activeVariants, variantId];

  return {
    ...state,
    activeVariants,
    // Recompute styles for new variant combination
    computedStyles: extractStylesForElement(
      state.document,
      state.selectedElement?.nodeId,
      activeVariants
    ),
  };
}
```

#### 2.4 Engine Side Effects

**File:** `packages/designer/src/machine/engines.ts`

```typescript
// Add to handleEvent in createSSEEngine
if (event.type === "style/changed") {
  const { property, value } = event.payload;
  const selected = prevState.selectedElement;

  if (!selected?.sourceId) {
    console.warn("[API] No sourceId for style mutation");
    return;
  }

  const mutationId = `mut-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

  machine.dispatch({
    type: "mutation/started",
    payload: {
      mutation: {
        mutationId,
        type: "setInlineStyle",
        nodeId: selected.sourceId,
        property,
        value,
      },
    },
  });

  const serverUrl = propsRef.current?.serverUrl || "";
  const filePath = propsRef.current?.filePath;

  if (filePath) {
    sendMutation(serverUrl, filePath, {
      type: "setInlineStyle",
      node_id: selected.sourceId,
      property,
      value,
      variants: prevState.activeVariants.length > 0
        ? prevState.activeVariants
        : undefined,
    }).then((response) => {
      if (response.success) {
        machine.dispatch({
          type: "mutation/acknowledged",
          payload: { mutationId, version: response.version },
        });
      } else {
        machine.dispatch({
          type: "mutation/failed",
          payload: { mutationId, error: response.error || "Unknown error" },
        });
      }
    });
  }
}
```

#### Research Insights: Race Conditions & Performance

**Frontend Races Reviewer** identified critical issues:

> **Missing AbortController**: The current `sendMutation` doesn't support cancellation. Rapid edits can result in out-of-order responses. Additionally, component unmount doesn't cancel pending fetches.

**Required fix - add AbortController support:**
```typescript
// Track pending mutations for cancellation
const pendingMutations = new Map<string, AbortController>();

// In style/changed handler:
if (event.type === "style/changed") {
  // Cancel any pending mutation for the same property
  const existingController = pendingMutations.get(property);
  if (existingController) {
    existingController.abort();
  }

  const controller = new AbortController();
  pendingMutations.set(property, controller);

  sendMutation(serverUrl, filePath, mutation, { signal: controller.signal })
    .then(...)
    .catch((err) => {
      if (err.name === 'AbortError') {
        // Expected - newer mutation superseded this one
        return;
      }
      // Handle actual errors
    })
    .finally(() => {
      pendingMutations.delete(property);
    });
}

// In cleanup function:
return () => {
  // Cancel ALL pending mutations on unmount
  pendingMutations.forEach(controller => controller.abort());
  pendingMutations.clear();
};
```

**Performance Oracle** recommends debouncing:

> Style edits can fire rapidly during typing. Debounce mutations to reduce server load.

```typescript
// Debounce style mutations (300ms recommended)
const debouncedStyleMutation = debounce((property: string, value: string) => {
  sendMutation(/* ... */);
}, 300);

// But update optimistic state immediately
machine.dispatch({ type: "style/optimisticUpdate", payload: { property, value } });
debouncedStyleMutation(property, value);
```

### Phase 3: UI Components

#### Research Insights: React 2026 Best Practices

**Best Practices Researcher** findings on React 2026:

> - **React 19 `useOptimistic`**: Built-in hook for optimistic updates - cleaner than manual state tracking
> - **React 19 `useTransition`**: Mark style updates as non-urgent to avoid blocking input
> - **Memoization**: Use `useMemo` for `sortedProperties` to avoid re-sorting on every render
> - **Component composition**: Keep StylePanel thin, delegate to specialized components

**Frontend Design Skill** recommendations:

> Use CSS custom properties (design tokens) for consistent styling:
> ```css
> :root {
>   --panel-bg: #1e1e1e;
>   --panel-text: #e0e0e0;
>   --input-bg: #2d2d2d;
>   --input-border: #404040;
>   --input-focus: #007acc;
>   --row-hover: rgba(255, 255, 255, 0.05);
> }
> ```

#### 3.1 Style Panel Component (Simplified for Phase 1)

**File:** `packages/designer/src/components/StylePanel.tsx`

```typescript
"use client";

import React, { useCallback, useMemo, useTransition } from "react";
import { useDispatch } from "@paperclip/common";
import { DesignerMachine, DesignerEvent } from "../machine";
import { StylePropertyRow } from "./StylePropertyRow";
// NOTE: VariantSelector and MixinSelector deferred to Phase 2

export function StylePanel() {
  const dispatch = useDispatch<DesignerEvent>();
  const [isPending, startTransition] = useTransition();

  const selectedElement = DesignerMachine.useSelector((s) => s.selectedElement);
  const computedStyles = DesignerMachine.useSelector((s) => s.computedStyles);

  // Memoize sorted properties to avoid re-sorting on every render
  const sortedProperties = useMemo(() => {
    return Object.entries(computedStyles).sort(([a], [b]) => a.localeCompare(b));
  }, [computedStyles]);

  const handleStyleChange = useCallback((property: string, value: string) => {
    // Use transition to mark as non-urgent (won't block input)
    startTransition(() => {
      dispatch({
        type: "style/changed",
        payload: { property, value },
      });
    });
  }, [dispatch]);

  const handleStyleRemove = useCallback((property: string) => {
    dispatch({
      type: "style/removed",
      payload: { property },
    });
  }, [dispatch]);

  if (!selectedElement) {
    return (
      <div className="style-panel style-panel--empty">
        <p>Select an element to edit styles</p>
      </div>
    );
  }

  return (
    <div className="style-panel">
      {/* Phase 2: Variant Selector will go here */}
      {/* Phase 2: Mixin Selector will go here */}

      {/* Properties Section */}
      <section className="style-panel__section">
        <h3>Styles {isPending && <span className="loading-indicator" />}</h3>
        <div className="style-panel__properties">
          {sortedProperties.map(([property, styleValue]) => (
            <StylePropertyRow
              key={property}
              property={property}
              value={styleValue.value}
              origin={styleValue.origin}
              onChange={(value) => handleStyleChange(property, value)}
              onRemove={() => handleStyleRemove(property)}
            />
          ))}

          {/* Add new property row */}
          <StylePropertyRow
            property=""
            value=""
            isNew
            onChange={handleStyleChange}
          />
        </div>
      </section>
    </div>
  );
}
```

#### 3.2 Style Property Row Component (Simplified for Phase 1)

**File:** `packages/designer/src/components/StylePropertyRow.tsx`

**TypeScript Reviewer** findings:
> - Use `useId()` for accessible input labels
> - Make `onChange` signature consistent (always `(property, value)` or always `(value)`)
> - Sync `editingValue` state when `value` prop changes (controlled component pattern)

```typescript
"use client";

import React, { useState, useCallback, useRef, useEffect, useId } from "react";
// NOTE: TokenAutocomplete and ColorPicker deferred to Phase 2

interface StylePropertyRowProps {
  property: string;
  value: string;
  origin?: "inline" | "mixin" | "inherited" | "default";
  isNew?: boolean;
  onChange: (property: string, value: string) => void;
  onRemove?: () => void;
}

export function StylePropertyRow({
  property,
  value,
  origin = "inline",
  isNew = false,
  onChange,
  onRemove,
}: StylePropertyRowProps) {
  const [editingProperty, setEditingProperty] = useState(property);
  const [editingValue, setEditingValue] = useState(value);
  const inputRef = useRef<HTMLInputElement>(null);
  const inputId = useId();

  // Sync local state when prop changes (controlled component pattern)
  useEffect(() => {
    setEditingValue(value);
  }, [value]);

  useEffect(() => {
    setEditingProperty(property);
  }, [property]);

  const handleValueChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setEditingValue(e.target.value);
  }, []);

  const handlePropertyChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setEditingProperty(e.target.value);
  }, []);

  const handleBlur = useCallback(() => {
    // Only trigger change if value actually changed
    if (editingValue !== value || editingProperty !== property) {
      if (editingProperty && editingValue) {
        onChange(editingProperty, editingValue);
      }
    }
  }, [editingProperty, editingValue, property, value, onChange]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      inputRef.current?.blur();
    } else if (e.key === "Escape") {
      setEditingValue(value);
      setEditingProperty(property);
    }
  }, [value, property]);

  const originClass = `style-row--${origin}`;

  return (
    <div className={`style-row ${originClass} ${isNew ? "style-row--new" : ""}`}>
      <label htmlFor={inputId} className="style-row__property">
        {isNew ? (
          <input
            type="text"
            value={editingProperty}
            onChange={handlePropertyChange}
            onBlur={handleBlur}
            placeholder="property"
            className="style-row__property-input"
            aria-label="CSS property name"
          />
        ) : (
          property
        )}
      </label>

      <span className="style-row__value">
        <input
          id={inputId}
          ref={inputRef}
          type="text"
          value={editingValue}
          onChange={handleValueChange}
          onBlur={handleBlur}
          onKeyDown={handleKeyDown}
          placeholder={isNew ? "value" : undefined}
          className="style-row__value-input"
          aria-label={`Value for ${property || "new property"}`}
        />

        {/* Phase 2: Color picker for color properties */}
        {/* Phase 2: Token autocomplete */}
      </span>

      {/* Remove button for inline styles */}
      {!isNew && origin === "inline" && onRemove && (
        <button
          className="style-row__remove"
          onClick={onRemove}
          title="Remove property"
          aria-label={`Remove ${property}`}
        >
          ×
        </button>
      )}

      {/* Origin indicator */}
      {origin !== "inline" && (
        <span className="style-row__origin" title={`From ${origin}`}>
          {origin === "mixin" ? "M" : origin === "inherited" ? "I" : "D"}
        </span>
      )}
    </div>
  );
}
```

#### 3.3 Layer Panel Component

**File:** `packages/designer/src/components/LayerPanel.tsx`

```typescript
"use client";

import React, { useCallback } from "react";
import { useDispatch } from "@paperclip/common";
import { DesignerMachine, DesignerEvent } from "../machine";
import { VNode } from "@paperclip/proto";

export function LayerPanel() {
  const dispatch = useDispatch<DesignerEvent>();
  const document = DesignerMachine.useSelector((s) => s.document);
  const selectedElement = DesignerMachine.useSelector((s) => s.selectedElement);
  const expandedNodes = DesignerMachine.useSelector((s) => s.expandedNodes);

  if (!document) {
    return (
      <div className="layer-panel layer-panel--empty">
        <p>No document loaded</p>
      </div>
    );
  }

  return (
    <div className="layer-panel">
      <h3>Layers</h3>
      <div className="layer-tree">
        {document.nodes.map((node, index) => (
          <LayerNode
            key={node.element?.semanticId || `node-${index}`}
            node={node}
            depth={0}
            selectedId={selectedElement?.nodeId}
            expandedNodes={expandedNodes}
            onSelect={(nodeId, sourceId) => {
              dispatch({
                type: "element/selected",
                payload: {
                  frameIndex: index,
                  nodeId,
                  sourceId,
                },
              });
            }}
            onToggleExpand={(nodeId) => {
              if (expandedNodes.has(nodeId)) {
                dispatch({ type: "layer/nodeCollapsed", payload: { nodeId } });
              } else {
                dispatch({ type: "layer/nodeExpanded", payload: { nodeId } });
              }
            }}
          />
        ))}
      </div>
    </div>
  );
}

interface LayerNodeProps {
  node: VNode;
  depth: number;
  selectedId: string | undefined;
  expandedNodes: Set<string>;
  onSelect: (nodeId: string, sourceId: string) => void;
  onToggleExpand: (nodeId: string) => void;
}

function LayerNode({
  node,
  depth,
  selectedId,
  expandedNodes,
  onSelect,
  onToggleExpand,
}: LayerNodeProps) {
  // Handle different node types
  if (node.element) {
    const { tag, semanticId, sourceId, children } = node.element;
    const hasChildren = children && children.length > 0;
    const isExpanded = expandedNodes.has(semanticId);
    const isSelected = selectedId === semanticId;

    return (
      <div className="layer-node">
        <div
          className={`layer-node__row ${isSelected ? "layer-node__row--selected" : ""}`}
          style={{ paddingLeft: `${depth * 16}px` }}
          onClick={() => onSelect(semanticId, sourceId || semanticId)}
        >
          {hasChildren && (
            <button
              className="layer-node__expand"
              onClick={(e) => {
                e.stopPropagation();
                onToggleExpand(semanticId);
              }}
            >
              {isExpanded ? "▼" : "▶"}
            </button>
          )}
          <span className="layer-node__icon">◻</span>
          <span className="layer-node__name">{tag}</span>
        </div>

        {hasChildren && isExpanded && (
          <div className="layer-node__children">
            {children.map((child, index) => (
              <LayerNode
                key={child.element?.semanticId || `child-${index}`}
                node={child}
                depth={depth + 1}
                selectedId={selectedId}
                expandedNodes={expandedNodes}
                onSelect={onSelect}
                onToggleExpand={onToggleExpand}
              />
            ))}
          </div>
        )}
      </div>
    );
  }

  if (node.text) {
    return (
      <div
        className="layer-node layer-node--text"
        style={{ paddingLeft: `${depth * 16}px` }}
      >
        <span className="layer-node__icon">T</span>
        <span className="layer-node__name">
          {node.text.content.slice(0, 20)}
          {node.text.content.length > 20 ? "..." : ""}
        </span>
      </div>
    );
  }

  return null;
}
```

### Phase 4: Integration

#### 4.1 Update Designer Layout

**File:** `packages/designer/src/components/Designer.tsx`

```typescript
"use client";

import React from "react";
import { DispatchProvider } from "@paperclip/common";
import { DesignerMachine, SSEEngineProps } from "../machine";
import { Canvas } from "./Canvas";
import { LayerPanel } from "./LayerPanel";
import { StylePanel } from "./StylePanel";

export interface DesignerProps extends SSEEngineProps {
  showPanels?: boolean;
}

export function Designer({ filePath, serverUrl, showPanels = true }: DesignerProps) {
  return (
    <DispatchProvider>
      <DesignerMachine.Provider props={{ filePath, serverUrl }}>
        <div className="designer">
          {showPanels && (
            <aside className="designer__left-panel">
              <LayerPanel />
            </aside>
          )}

          <main className="designer__canvas">
            <Canvas />
          </main>

          {showPanels && (
            <aside className="designer__right-panel">
              <StylePanel />
            </aside>
          )}
        </div>
      </DesignerMachine.Provider>
    </DispatchProvider>
  );
}
```

## Acceptance Criteria

### Functional Requirements

- [ ] **Element Selection**: Click element in canvas or layer tree to select it
- [ ] **Style Display**: Selected element's computed styles shown in right panel
- [ ] **Style Editing**: Edit CSS property values with live preview
- [ ] **Style Addition**: Add new CSS properties to elements
- [ ] **Style Removal**: Remove CSS properties from elements
- [ ] **Variant Support**: Toggle variants to edit variant-specific styles
- [ ] **Token Autocomplete**: Suggest available tokens when editing values
- [ ] **Color Picker**: Visual color picker for color properties
- [ ] **Layer Tree**: Hierarchical view of document structure
- [ ] **Layer Selection**: Click layer to select element

### Non-Functional Requirements

- [ ] Style changes reflected in preview within 100ms
- [ ] Optimistic updates for responsive feel
- [ ] Graceful error handling with rollback
- [ ] Works with CRDT concurrent editing

### Quality Gates

- [ ] Unit tests for reducers
- [ ] Integration tests for mutation flow
- [ ] E2E test for style edit → preview update

## Dependencies & Prerequisites

### Backend Work Required First

1. **SetInlineStyle mutation** - Currently returns Noop
2. **Style block indexing** - AST index needs extension
3. **Token list API** - Need to expose available tokens

### Dependencies

- `packages/proto` - Mutation type definitions
- `packages/workspace` - Mutation handler
- `packages/editor` - AST mutations
- `packages/evaluator` - Style computation
- `@paperclip/common` - Machine + Engines pattern

## Risk Analysis & Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Style block creation complexity | High | Medium | Start with existing blocks only, add creation later |
| Variant targeting ambiguity | Medium | High | **Deferred to Phase 2** |
| Performance with many properties | Medium | Low | Virtualize property list, debounce updates |
| CRDT conflict with style edits | High | Low | Use StickyIndex for position tracking |
| CSS injection attacks | **Critical** | Medium | Validate property/value before mutation |
| Race conditions on rapid edits | High | High | AbortController + debouncing |
| Engine composition limitation | Medium | High | Use single SSE engine for Phase 1 |

### Research-Identified Risks (from Security Sentinel)

| Vulnerability | Severity | Mitigation |
|--------------|----------|------------|
| CSS injection in style values | Critical | Validate no `{`, `}`, `;` in values |
| XSS in iframe preview | Critical | CSP headers, sandbox iframe |
| Path traversal in node_id | High | Validate node_id format |
| Missing rate limiting | Medium | Add per-client mutation throttling |

## Open Questions

### Critical (Must Answer Before Implementation)

1. **Q: How should SetInlineStyle locate style blocks in source?**
   - Proposed: Extend AstIndex with StyleBlockInfo
   - Need: Confirmation this approach works with CRDT

2. **Q: What is the mutation format for variant-specific edits?**
   - Proposed: `variants: string[]` optional field on SetInlineStyle
   - Need: Confirmation from original paperclip team

3. **Q: How does the UI get raw style data (not computed CSS)?**
   - Option A: Include in VDOM metadata
   - Option B: Separate API endpoint
   - Recommendation: VDOM metadata for consistency

### Important (Should Answer Soon)

4. **Q: What happens when editing styles on repeat item?**
   - Current: CannotEditRepeatInstance error
   - Need: Clarify if template editing is allowed

5. **Q: How should style block creation work?**
   - Proposed: Auto-create on first property edit
   - Need: Confirm insertion point in source

## References & Research

### Internal References

- Machine pattern: `packages/common/src/machine/react/defineMachine.tsx`
- Existing mutations: `packages/editor/src/mutations.rs`
- Mutation handler: `packages/workspace/src/mutation_handler.rs`
- Designer state: `packages/designer/src/machine/state.ts`

### External References (Original Paperclip)

- Style panel: `~/Developer/crcn/paperclip/libs/designer/src/ui/logic/Editor/EditorPanels/RightSidebar/StylePanel/`
- Declarations: `~/Developer/crcn/paperclip/libs/designer/src/ui/logic/Editor/EditorPanels/RightSidebar/StylePanel/Declarations/`
- Layer panel: `~/Developer/crcn/paperclip/libs/designer/src/ui/logic/Editor/EditorPanels/LeftSidebar/Layers/`
- Mutations proto: `~/Developer/crcn/paperclip/libs/proto/src/ast_mutate/mod.proto`
- Set style declarations: `~/Developer/crcn/paperclip/libs/core/src/proto/ast_mutate/set_style_declarations.rs`

### Institutional Learnings Applied

- Use Machine + Engines pattern (NOT Redux/Zustand) - from architecture docs
- Semantic mutations over raw edits - from collaboration.md
- Server owns Y.Text, clients send mutations - from multi-session sync plan
- Use semantic_id for stable element identity - from Architecture Constitution
