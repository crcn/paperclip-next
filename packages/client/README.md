# paperclip-client

TypeScript browser client with path-based, serializable Virtual DOM differ and patcher.

## Features

- ⚡ **Serializable patches** - Path-based addressing, JSON serializable
- 🔄 **Pure diffing** - No side effects, no DOM required for diffing
- 📦 **Zero dependencies** - Pure TypeScript Virtual DOM implementation
- 🌐 **Platform-agnostic** - Works with DOM, SSR, React, or custom renderers
- 🎨 **Complete coverage** - CREATE, REMOVE, REPLACE, UPDATE patches
- ✅ **Type-safe** - Full TypeScript types
- 🚫 **No globals** - Explicit dependencies, easy to test
- 🎯 **Single-user HMR** - Designed for hot module replacement, not collaborative editing

## Important Guidelines

**⚠️ Avoid globals in web-related code!** This library uses explicit parameters instead of global state. See **DEVELOPMENT.md** for complete guidelines on writing composable, testable code.

## Installation

```bash
yarn install
```

## Usage

### Basic Usage

```typescript
import { VNode, VDocument, createElement, diff, patch } from "./src/vdom";

// Virtual DOM document
const vdoc: VDocument = {
  nodes: [
    {
      type: "Element",
      tag: "button",
      attributes: {},
      styles: {
        padding: "8px 16px",
        background: "#3366FF",
        color: "white",
      },
      children: [
        {
          type: "Text",
          content: "Click me",
        },
      ],
    },
  ],
  styles: [],
};

// Create real DOM element
const element = createElement(vdoc.nodes[0]);
document.body.appendChild(element);
```

### Diffing and Patching (Path-Based)

```typescript
import { diff, patch, domPatchApplier } from "./src/vdom";

// Old Virtual DOM
const oldVNode: VNode = {
  type: "Element",
  tag: "button",
  attributes: {},
  styles: { background: "#3366FF" },
  children: [{ type: "Text", content: "Click me" }],
};

// New Virtual DOM
const newVNode: VNode = {
  type: "Element",
  tag: "button",
  attributes: {},
  styles: { background: "#FF3366" }, // Changed color
  children: [{ type: "Text", content: "Click me now!" }], // Changed text
};

// Compute patches (pure - no DOM needed!)
const patches = diff(oldVNode, newVNode);

// Patches are now serializable!
const json = JSON.stringify(patches);
console.log(json); // Can send over network!

// Get existing DOM element
const element = document.querySelector("button");

// Apply patches using DOM applier (only updates what changed)
patch(patches, element, domPatchApplier());
```

### Complete Example

```typescript
import { VDocument, createElement, diff, patch } from "./src/vdom";

let currentVDoc: VDocument | null = null;
let currentRoot: HTMLElement | null = null;

// Mount initial Virtual DOM
function mount(container: HTMLElement, vdoc: VDocument) {
  currentVDoc = vdoc;
  currentRoot = container;

  container.innerHTML = "";

  for (const node of vdoc.nodes) {
    const element = createElement(node);
    container.appendChild(element);
  }
}

// Update to new Virtual DOM
function update(vdoc: VDocument) {
  if (!currentVDoc || !currentRoot) return;

  const patches = [];
  const maxLength = Math.max(currentVDoc.nodes.length, vdoc.nodes.length);

  for (let i = 0; i < maxLength; i++) {
    const oldNode = currentVDoc.nodes[i] || null;
    const newNode = vdoc.nodes[i] || null;

    // Pure diff - no DOM element needed!
    patches.push(...diff(oldNode, newNode, [i]));
  }

  // Apply patches using DOM applier
  patch(patches, currentRoot, domPatchApplier());
  currentVDoc = vdoc;
}

// Usage
const container = document.getElementById("app")!;

const vdoc1: VDocument = {
  nodes: [
    {
      type: "Element",
      tag: "div",
      attributes: {},
      styles: { padding: "16px" },
      children: [{ type: "Text", content: "Hello" }],
    },
  ],
  styles: [],
};

mount(container, vdoc1);

// Later, update
const vdoc2: VDocument = {
  nodes: [
    {
      type: "Element",
      tag: "div",
      attributes: {},
      styles: { padding: "24px" }, // Changed
      children: [{ type: "Text", content: "Hello World" }], // Changed
    },
  ],
  styles: [],
};

update(vdoc2);
```

## API Reference

### Types

#### `VNode`

Virtual DOM node (discriminated union):

```typescript
type VNode =
  | {
      type: "Element";
      tag: string;
      attributes: Record<string, string>;
      styles: Record<string, string>;
      children: VNode[];
      id?: string;
    }
  | {
      type: "Text";
      content: string;
    }
  | {
      type: "Comment";
      content: string;
    };
```

#### `VDocument`

Virtual document:

```typescript
interface VDocument {
  nodes: VNode[];
  styles: CssRule[];
}
```

#### `CssRule`

CSS rule:

```typescript
interface CssRule {
  selector: string;
  properties: Record<string, string>;
}
```

#### `Patch`

Path-based patch operation (discriminated union):

```typescript
type Patch =
  | { type: "CREATE"; path: number[]; node: VNode; index: number }
  | { type: "REMOVE"; path: number[] }
  | { type: "REPLACE"; path: number[]; newNode: VNode }
  | { type: "UPDATE_ATTRS"; path: number[]; attributes: Record<string, string> }
  | { type: "UPDATE_STYLES"; path: number[]; styles: Record<string, string> }
  | { type: "UPDATE_TEXT"; path: number[]; content: string };
```

**Path format:** `[0, 2, 1]` means first child → third child → second child

**Key feature:** No DOM references! Patches are pure data and fully serializable.

### Functions

#### `createElement(vnode: VNode): Node`

Create a real DOM node from a Virtual DOM node.

```typescript
const vnode: VNode = {
  type: "Element",
  tag: "button",
  attributes: { class: "btn" },
  styles: { padding: "8px" },
  children: [{ type: "Text", content: "Click" }],
};

const element = createElement(vnode);
document.body.appendChild(element);
```

#### `diff(oldNode: VNode | null, newNode: VNode | null, path?: number[]): Patch[]`

**Pure function** - Computes patches to transform oldNode into newNode without any DOM access.

```typescript
const patches = diff(oldVNode, newVNode);
// Returns array of serializable patch operations
```

#### `patch<T>(patches: Patch[], target: T, applier: PatchApplier<T>): T`

Apply patches to a target using the given applier strategy.

```typescript
const patches = diff(oldVNode, newVNode);
patch(patches, element, domPatchApplier()); // DOM is now updated
```

#### `domPatchApplier(): PatchApplier<Element>`

Factory function that returns a DOM patch applier.

```typescript
const applier = domPatchApplier();
patch(patches, element, applier);
```

#### `PatchApplier<T>` interface

Allows different patch application strategies:

```typescript
interface PatchApplier<T> {
  apply(patches: Patch[], target: T): T;
}

// You can create custom appliers!
const ssrApplier: PatchApplier<string> = {
  apply(patches, html) {
    // Generate HTML string from patches
    return updatedHtml;
  }
};
```

## Patch Types (Path-Based)

All patches use **path-based addressing** instead of DOM references. This enables serialization over the network for HMR streaming.

### CREATE

Create a new node at the specified path and index.

```typescript
{
  type: "CREATE",
  path: [0, 2],  // Parent's path
  node: { type: "Element", tag: "div", ... },
  index: 2  // Insert at this index
}
```

### REMOVE

Remove a node at the specified path.

```typescript
{
  type: "REMOVE",
  path: [0, 2, 1]  // Path to node to remove
}
```

### REPLACE

Replace a node at the specified path with a new one.

```typescript
{
  type: "REPLACE",
  path: [0, 1],  // Path to node to replace
  newNode: { type: "Element", tag: "span", ... }
}
```

### UPDATE_ATTRS

Update element attributes at the specified path.

```typescript
{
  type: "UPDATE_ATTRS",
  path: [0],  // Path to element
  attributes: { class: "active", "data-id": "123" }
}
```

### UPDATE_STYLES

Update inline styles at the specified path.

```typescript
{
  type: "UPDATE_STYLES",
  path: [0, 2],  // Path to element
  styles: { padding: "16px", background: "#FF0000" }
}
```

### UPDATE_TEXT

Update text content at the specified path.

```typescript
{
  type: "UPDATE_TEXT",
  path: [0, 1, 0],  // Path to text node
  content: "New text content"
}
```

### Why Path-Based?

- ✅ **Serializable** - Can send over network (gRPC, WebSocket)
- ✅ **Testable** - No DOM needed for testing
- ✅ **HMR-ready** - Perfect for hot module replacement streaming
- ✅ **Platform-agnostic** - Works with DOM, SSR, React, etc.
- ✅ **Extensible** - Can add OT/CRDT layer for collaborative editing (future)

## Demo

Run the interactive demo:

```bash
yarn dev
```

Open http://localhost:3000 to see the Virtual DOM differ/patcher in action.

The demo shows:
- Three example Virtual DOM documents
- Click buttons to cycle through them
- Watch efficient DOM updates (only changed properties)
- Real-time visual feedback

## Performance

The differ/patcher is optimized for:
- **Minimal comparisons** - Only compares changed properties
- **Efficient patching** - Only touches changed DOM nodes
- **No full re-renders** - Surgical updates only

Example:
```typescript
// Only the text changes
oldVNode: { type: "Text", content: "Hello" }
newVNode: { type: "Text", content: "World" }

// Result: 1 UPDATE_TEXT patch (not a full element replacement)
```

## TypeScript Support

Full type checking:

```typescript
import { VNode } from "./src/vdom";

// TypeScript will catch type errors
const validNode: VNode = {
  type: "Element",
  tag: "div",
  attributes: {},
  styles: {},
  children: [],
};

// Error: Property 'tag' is missing
const invalidNode: VNode = {
  type: "Element",
  attributes: {},
  styles: {},
  children: [],
};
```

## Testing

Build TypeScript:

```bash
yarn build
```

Run in browser:

```bash
yarn dev
```

## Integration with Paperclip Server

To connect to the gRPC server (future):

```typescript
// Via gRPC-web (to be implemented)
import { WorkspaceServiceClient } from "./proto/workspace_pb_service";

const client = new WorkspaceServiceClient("http://localhost:50051");

const request = new PreviewRequest();
request.setFilePath("button.pc");

const stream = client.streamPreview(request, {});

stream.on("data", (update) => {
  const vdoc = JSON.parse(update.getVdomJson());
  updatePreview(vdoc);
});

stream.on("error", (err) => {
  console.error("Stream error:", err);
});
```

## Browser Compatibility

Works in all modern browsers:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

Uses standard DOM APIs:
- `document.createElement`
- `document.createTextNode`
- `element.setAttribute`
- `element.style`
- `element.appendChild`
- `element.removeChild`

## Development

Project structure:

```
client/
├── src/
│   ├── vdom.ts       # Virtual DOM types and functions
│   └── main.ts       # Demo application
├── index.html        # Demo page
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

Build for production:

```bash
yarn build
```

Preview production build:

```bash
yarn preview
```

## Example Virtual DOM JSON

Input from Paperclip server:

```json
{
  "nodes": [
    {
      "type": "Element",
      "tag": "button",
      "attributes": {
        "class": "btn-primary"
      },
      "styles": {
        "padding": "8px 16px",
        "background": "#3366FF",
        "color": "white",
        "border": "none",
        "border-radius": "4px"
      },
      "children": [
        {
          "type": "Text",
          "content": "Click me"
        }
      ]
    }
  ],
  "styles": []
}
```

Result in browser:

```html
<button class="btn-primary" style="padding: 8px 16px; background: #3366FF; color: white; border: none; border-radius: 4px;">
  Click me
</button>
```

## Development Guidelines

See **DEVELOPMENT.md** for important guidelines on:
- ⚠️ Avoiding global state in web code
- Writing pure, composable functions
- Using explicit dependencies instead of singletons
- Testing strategies for pure functions

See **OT_REFACTOR.md** for details on the path-based, serializable patch architecture.

## gRPC Client

A Node.js gRPC client is provided for connecting to the Paperclip workspace server.

### Running the Client

Start the workspace server:

```bash
cd ../../
cargo run --bin paperclip-server examples
```

In another terminal, run the gRPC client:

```bash
yarn grpc-client button.pc
```

This connects to the server and streams preview updates for the specified file.

### Latency Testing

Measure end-to-end latency from file write to preview update:

```bash
yarn test:latency
```

**Results from Spike 0.3:**
- Average latency: **12.67ms**
- Min latency: **6ms**
- Max latency: **25ms**
- Pass rate: **100%** (all tests < 40ms target)

The test measures the time from writing a `.pc` file to receiving the preview update via gRPC streaming. This validates that the full pipeline (file write → watcher → parse → evaluate → stream → receive) meets performance targets.

## License

MIT
