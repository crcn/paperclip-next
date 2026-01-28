# Serializable Patches Refactor

## Overview

Refactored the client library from a tightly-coupled DOM-based diff/patch system to a **path-based, serializable patch system** with pure functions.

**Note**: While this architecture uses path-based patches similar to Operational Transformation (OT) systems, it does NOT implement full OT. There are no transformation functions for concurrent operations. This is designed for **single-user HMR (Hot Module Replacement)** scenarios, not multi-user collaborative editing.

## What Changed

### Before (DOM-coupled)

```typescript
// Patches contained DOM element references
const patches = diff(oldVNode, newVNode, domElement);
patch(patches); // Mutates DOM directly
```

**Problems:**
- ❌ Diff required DOM elements as input
- ❌ Patches contained direct DOM references
- ❌ Not serializable (can't send over network)
- ❌ Tightly coupled to browser DOM
- ❌ Hard to test without DOM
- ❌ Encouraged global state pattern (DOM elements stored globally)

### After (Path-based, Serializable)

```typescript
// Pure diffing - no DOM needed
const patches = diff(oldVNode, newVNode);

// Apply with pluggable applier
const newElement = patch(patches, element, domPatchApplier());
```

**Benefits:**
- ✅ Pure functions with no side effects
- ✅ Patches are serializable JSON (perfect for gRPC!)
- ✅ Platform-agnostic (DOM, SSR, React, Canvas, etc.)
- ✅ Testable without browser
- ✅ Composable appliers
- ✅ Ready for single-user HMR streaming

## API Changes

### 1. Patch Type (Now Path-Based)

```typescript
// Old: DOM references
type Patch =
  | { type: "UPDATE_ATTRS"; element: Element; attributes: {...} }
  | { type: "REMOVE"; element: Node }

// New: Path-based addressing
type Patch =
  | { type: "UPDATE_ATTRS"; path: number[]; attributes: {...} }
  | { type: "REMOVE"; path: number[] }
```

**Path format:** `[0, 2, 1]` = first child → third child → second child

### 2. diff() Function

```typescript
// Old signature
function diff(
  oldNode: VNode | null,
  newNode: VNode | null,
  element: Node | null  // ❌ Required DOM element
): Patch[]

// New signature
function diff(
  oldNode: VNode | null,
  newNode: VNode | null,
  path?: number[]  // ✅ Optional path (defaults to [])
): Patch[]
```

### 3. patch() Function

```typescript
// Old signature
function patch(patches: Patch[]): void

// New signature (generic!)
function patch<T>(
  patches: Patch[],
  target: T,
  applier: PatchApplier<T>
): T
```

### 4. New PatchApplier Interface

```typescript
interface PatchApplier<T> {
  apply(patches: Patch[], target: T): T;
}

// Factory function for DOM applier
function domPatchApplier(): PatchApplier<Element>
```

## Usage Examples

### Basic Usage

```typescript
import { diff, patch, domPatchApplier } from "./vdom";

// 1. Pure diffing (no DOM)
const patches = diff(oldVNode, newVNode);

// 2. Serialize for network transmission
const json = JSON.stringify(patches);
// Send over gRPC, WebSocket, etc.

// 3. Deserialize and apply
const received = JSON.parse(json);
patch(received, domElement, domPatchApplier());
```

### Multiple Root Nodes

```typescript
const patches = [];
for (let i = 0; i < nodes.length; i++) {
  patches.push(...diff(oldNodes[i], newNodes[i], [i]));
}
patch(patches, container, domPatchApplier());
```

### Custom Appliers

```typescript
// Server-side rendering applier
const ssrApplier: PatchApplier<string> = {
  apply(patches, html) {
    // Generate HTML string from patches
    return updatedHtml;
  }
};

// React applier
const reactApplier: PatchApplier<ReactElement> = {
  apply(patches, element) {
    // Update React tree
    return updatedElement;
  }
};
```

## Example Output

```typescript
const patches = diff(oldTree, newTree);
console.log(JSON.stringify(patches, null, 2));
```

```json
[
  {
    "type": "UPDATE_ATTRS",
    "path": [],
    "attributes": {
      "class": "btn-primary",
      "disabled": "true"
    }
  },
  {
    "type": "UPDATE_STYLES",
    "path": [],
    "styles": {
      "padding": "12px",
      "background": "red"
    }
  },
  {
    "type": "UPDATE_TEXT",
    "path": [0],
    "content": "Click me now!"
  }
]
```

## Perfect for Your gRPC Architecture

This refactor is ideal for the Paperclip workspace server architecture:

```typescript
// Server: Generate patches from .pc file changes
const patches = diff(oldVNode, newVNode);

// Stream over gRPC
stream.send(JSON.stringify(patches));

// Client: Receive and apply
stream.on('data', (json) => {
  const patches = JSON.parse(json);
  patch(patches, rootElement, domPatchApplier());
});
```

## Future Possibilities

With serializable, path-based patches, you can now:

1. **Server-side diffing** - Rust evaluator could generate patches directly
2. **Time travel** - Store and replay patch sequences for undo/redo
3. **Multi-platform** - Same patches work in browser, Node, native apps
4. **Optimizations** - Batch, deduplicate, or compress patches before sending
5. **Collaborative editing** - Add OT transformation layer or use CRDT (Automerge, Yjs) on top of patches

## Testing

Run the demonstration:

```bash
npx tsx src/test-patches-serializable.ts
```

## Files Changed

- `packages/client/src/vdom.ts` - Core refactor
- `packages/client/src/main.ts` - Updated to use new API
- `packages/client/src/test-patches-serializable.ts` - Demonstration

## No More Globals!

This refactor eliminates the need for global state in web-related code:

### Before: Encouraged Globals

```typescript
// ❌ Old pattern encouraged storing DOM globally
let currentElement: Element | null = null;

function init() {
  currentElement = document.getElementById('root');
  const patches = diff(oldNode, newNode, currentElement);
  patch(patches); // Uses global state implicitly
}
```

### After: Explicit Parameters

```typescript
// ✅ New pattern uses explicit parameters
function init() {
  const element = document.getElementById('root');
  const patches = diff(oldNode, newNode); // Pure!
  patch(patches, element, domPatchApplier()); // Explicit!
}
```

**Key principle:** Avoid globals in web-related code. Pass dependencies explicitly instead.

See **DEVELOPMENT.md** for complete guidelines on avoiding global state.

## Migration Notes

If you have existing code using the old API:

```typescript
// Old
const patches = diff(oldNode, newNode, element);
patch(patches);

// New
const patches = diff(oldNode, newNode);
patch(patches, element, domPatchApplier());
```

The change is minimal but the benefits are substantial! 🎉
