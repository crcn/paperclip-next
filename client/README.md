# paperclip-client

TypeScript browser client with efficient Virtual DOM differ and patcher.

## Features

- ⚡ **Efficient diffing** - Minimal DOM updates
- 🔄 **Smart patching** - Only updates what changed
- 📦 **Zero dependencies** - Pure TypeScript Virtual DOM implementation
- 🎨 **Complete coverage** - CREATE, REMOVE, REPLACE, UPDATE patches
- ✅ **Type-safe** - Full TypeScript types

## Installation

```bash
npm install
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

### Diffing and Patching

```typescript
import { diff, patch } from "./src/vdom";

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

// Get existing DOM element
const element = document.querySelector("button");

// Compute patches
const patches = diff(oldVNode, newVNode, element);

// Apply patches (only updates what changed)
patch(patches);
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
    const element = currentRoot.childNodes[i] || null;

    patches.push(...diff(oldNode, newNode, element));
  }

  patch(patches);
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

DOM patch operation (discriminated union):

```typescript
type Patch =
  | { type: "CREATE"; node: VNode; parent: Node; index: number }
  | { type: "REMOVE"; element: Node }
  | { type: "REPLACE"; oldElement: Node; newNode: VNode }
  | { type: "UPDATE_ATTRS"; element: Element; attributes: Record<string, string> }
  | { type: "UPDATE_STYLES"; element: Element; styles: Record<string, string> }
  | { type: "UPDATE_TEXT"; element: Text; content: string };
```

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

#### `diff(oldNode: VNode | null, newNode: VNode | null, element: Node | null): Patch[]`

Compute a list of patches to transform oldNode into newNode.

```typescript
const patches = diff(oldVNode, newVNode, existingElement);
// Returns array of patch operations
```

#### `patch(patches: Patch[]): void`

Apply a list of patches to the DOM.

```typescript
const patches = diff(oldVNode, newVNode, element);
patch(patches); // DOM is now updated
```

## Patch Types

### CREATE

Create a new DOM node and insert it at the specified index.

```typescript
{
  type: "CREATE",
  node: { type: "Element", tag: "div", ... },
  parent: parentElement,
  index: 2
}
```

### REMOVE

Remove a DOM node from the tree.

```typescript
{
  type: "REMOVE",
  element: domElement
}
```

### REPLACE

Replace an entire DOM node with a new one.

```typescript
{
  type: "REPLACE",
  oldElement: oldDomElement,
  newNode: { type: "Element", tag: "span", ... }
}
```

### UPDATE_ATTRS

Update element attributes without replacing the element.

```typescript
{
  type: "UPDATE_ATTRS",
  element: domElement,
  attributes: { class: "active", "data-id": "123" }
}
```

### UPDATE_STYLES

Update inline styles without replacing the element.

```typescript
{
  type: "UPDATE_STYLES",
  element: domElement,
  styles: { padding: "16px", background: "#FF0000" }
}
```

### UPDATE_TEXT

Update text content of a text node.

```typescript
{
  type: "UPDATE_TEXT",
  element: textNode,
  content: "New text content"
}
```

## Demo

Run the interactive demo:

```bash
npm run dev
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
npm run build
```

Run in browser:

```bash
npm run dev
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
npm run build
```

Preview production build:

```bash
npm run preview
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

## License

MIT
