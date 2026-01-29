---
title: Design Workspace Client and Rendering Client Libraries with Redux Integration
type: feat
date: 2026-01-28
---

# Design Workspace Client and Rendering Client Libraries with Redux Integration

## Overview

Design and implement two complementary TypeScript libraries for Paperclip:

1. **Workspace Client Library** - Manages connection to workspace server, handles mutations, document state, and AST access. Integrates with Redux via event-driven architecture.

2. **Rendering Client Library** - Pure React component that efficiently diffs VDOM and patches DOM/CSSOM using reference equality checks. Manages iframe isolation per root node and style injection via CSSOM.

Both libraries work in universal environments (Browser, Node.js, WebWorker) with multi-transport support (gRPC for Node, gRPC-web for browser).

**Design Inspiration:** The rendering client architecture is heavily inspired by the existing `~/Developer/crcn/paperclip/libs/web-renderer`, which uses Immer for structural sharing and `===` checks for efficient patching. This proven pattern is adapted for the new Redux-based workspace architecture.

## Problem Statement / Motivation

### Current State

The existing `packages/client` provides:
- ✅ VDOM types and basic patch applier
- ✅ Basic Node.js gRPC client
- ❌ No editing/mutation support
- ❌ No connection management (reconnection, health checks)
- ❌ No Redux integration
- ❌ No document outline access (needed for layers panel)
- ❌ No optimistic update handling

### Current Protocol Limitations

The `workspace.proto` only supports **read-only preview streaming**:
- `StreamPreview` - patches FROM server
- `WatchFiles` - file system events

**Missing:**
- `ApplyMutation` - send semantic edits TO server
- `GetDocumentOutline` - fetch AST structure for UI
- Mutation acknowledgment/rebase in preview stream
- CRDT synchronization (handled internally by server)

### Why This Matters

**For VSCode Extension:**
- Needs programmatic editing (not just file writes)
- Needs document outline for layers panel/tree view
- Needs optimistic updates for responsive UX

**For Designer (Embedded Web):**
- Same requirements plus Redux state management
- Visual editing must sync with AST mutations
- Real-time collaboration ready

**For Future Tools:**
- Language server
- CLI linting/formatting
- Testing frameworks

A unified, well-designed client library prevents fragmentation and ensures consistent behavior across all tools.

## Proposed Solution

### High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Application Layer                          │
│              (Designer, VSCode Extension)                     │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Redux Store (with Immer)                │    │
│  │                                                       │    │
│  │  documents: {                                        │    │
│  │    "button.pc": {                                    │    │
│  │      vdom: PCModule,  // ← Structural sharing!      │    │
│  │      version: 42                                     │    │
│  │    }                                                 │    │
│  │  }                                                   │    │
│  └──────────────┬─────────────────────┬────────────────┘    │
│                 │                     │                       │
│       ┌─────────▼─────────┐  ┌───────▼────────────┐        │
│       │ Workspace Client  │  │ Rendering Component │        │
│       │                   │  │  (Pure React)       │        │
│       │ • Dispatch events │  │                     │        │
│       │ • Send mutations  │  │ const prev =        │        │
│       │                   │  │   usePrevious(vdom) │        │
│       │                   │  │                     │        │
│       │                   │  │ if (prev !== vdom)  │        │
│       │                   │  │   patchDOM()        │        │
│       └─────────┬─────────┘  └─────────────────────┘        │
└─────────────────┼──────────────────────────────────────────┘
                  │ gRPC / gRPC-web
                  │
        ┌─────────▼─────────┐
        │ Workspace Server  │
        │  (Rust/gRPC)      │
        │                   │
        │ • Parse/Evaluate  │
        │ • CRDT (internal) │
        │ • Diff → Patches  │
        └───────────────────┘
```

### Core Architecture Insight

**The key innovation: Redux + Immer + Reference Equality**

```typescript
// 1. Server sends protocol patches
server → PATCHES_RECEIVED { patches: [...] }

// 2. Redux reducer applies patches with Immer (structural sharing!)
produce(draft, (vdom) => {
  applyPatches(vdom, patches);  // Only mutated parts get new references
});

// 3. React component receives VDOM
<PaperclipRenderer vdom={vdom} />

// 4. Renderer does efficient diffing via === checks
if (prevNode === currNode) {
  return;  // Nothing changed - skip all work!
}
```

**Why this is elegant:**
- Immer gives structural sharing (unchanged = same reference)
- `===` checks are O(1) and skip entire unchanged subtrees
- Protocol patches are just for Redux updates, not DOM manipulation
- Renderer is pure - receives data, not patch commands

### Core Principles

1. **Event-Driven Architecture** - Workspace client emits events (past tense) describing what happened, not commands
2. **Server Authority** - Server is source of truth, clients rebase on conflict
3. **Optimistic Updates** - Apply mutations locally immediately, reconcile with server
4. **Structural Sharing via Immer** - Unchanged VDOM parts keep same references for efficient `===` checks
5. **Pure Rendering Component** - Receives VDOM data, not client instances or patch commands
6. **Reference Equality Diffing** - Inspired by existing `libs/web-renderer`, uses `prev === curr` checks
4. **Redux Integration** - All workspace state flows through Redux store
5. **Protocol-First** - Design RPCs before client API to prevent leaky abstractions
6. **TypeScript-First** - Excellent type safety and DX with full autocomplete
7. **Universal** - Works in Browser, Node.js, and WebWorker contexts

## Technical Approach

### Phase 1: Protocol Extension

Extend `workspace.proto` with editing capabilities.

#### New RPC: ApplyMutation

```protobuf
// New in workspace.proto
rpc ApplyMutation(MutationRequest) returns (MutationResponse);

message MutationRequest {
  string client_id = 1;
  string file_path = 2;
  Mutation mutation = 3;
  uint64 expected_version = 4;  // Optimistic concurrency control
}

message MutationResponse {
  oneof result {
    MutationAck ack = 1;
    MutationRebased rebased = 2;
    MutationNoop noop = 3;
  }
}

message MutationAck {
  string mutation_id = 1;
  uint64 new_version = 2;
  int64 timestamp = 3;
}

message MutationRebased {
  string original_mutation_id = 1;
  Mutation transformed_mutation = 2;
  uint64 new_version = 3;
  string reason = 4;
}

message MutationNoop {
  string mutation_id = 1;
  string reason = 2;  // e.g., "node already deleted"
}

// Mutation types (semantic AST operations)
message Mutation {
  string mutation_id = 1;
  int64 timestamp = 2;

  oneof mutation_type {
    MoveElement move_element = 3;
    UpdateText update_text = 4;
    SetInlineStyle set_inline_style = 5;
    SetAttribute set_attribute = 6;
    RemoveNode remove_node = 7;
    InsertElement insert_element = 8;
  }
}

message MoveElement {
  string node_id = 1;
  string new_parent_id = 2;
  uint32 index = 3;
}

message UpdateText {
  string node_id = 1;
  string content = 2;
}

message SetInlineStyle {
  string node_id = 1;
  string property = 2;
  string value = 3;
}

message SetAttribute {
  string node_id = 1;
  string name = 2;
  string value = 3;
}

message RemoveNode {
  string node_id = 1;
}

message InsertElement {
  string parent_id = 1;
  uint32 index = 2;
  ASTElement element = 3;  // From parser types
}
```

#### New RPC: GetDocumentOutline

```protobuf
rpc GetDocumentOutline(OutlineRequest) returns (OutlineResponse);

message OutlineRequest {
  string file_path = 1;
}

message OutlineResponse {
  repeated OutlineNode nodes = 1;
  uint64 version = 2;
}

message OutlineNode {
  string node_id = 1;
  NodeType type = 2;
  optional string parent_id = 3;
  repeated string child_ids = 4;
  SourceSpan span = 5;
  optional string label = 6;  // e.g., component name, tag name
}

enum NodeType {
  COMPONENT = 0;
  ELEMENT = 1;
  TEXT = 2;
  CONDITIONAL = 3;
  REPEAT = 4;
  INSERT = 5;
}

message SourceSpan {
  uint32 start_line = 1;
  uint32 start_col = 2;
  uint32 end_line = 3;
  uint32 end_col = 4;
}
```

#### Extended StreamPreview

Extend existing `PreviewUpdate` to include mutation metadata:

```protobuf
message PreviewUpdate {
  string file_path = 1;
  repeated paperclip.patches.VDocPatch patches = 2;
  optional string error = 3;
  int64 timestamp = 4;
  uint64 version = 5;

  // NEW: Include mutation acknowledgments
  repeated string acknowledged_mutation_ids = 6;
  optional string changed_by_client_id = 7;
}
```

### Phase 2: Workspace Client Library

#### Package Structure

```
packages/workspace-client/
├── src/
│   ├── index.ts                    # Public API
│   ├── client.ts                   # Main WorkspaceClient class
│   ├── connection.ts               # Connection management
│   ├── transport/
│   │   ├── transport.ts            # Transport interface
│   │   ├── grpc-transport.ts       # Node.js gRPC
│   │   └── grpc-web-transport.ts   # Browser gRPC-web
│   ├── mutations.ts                # Mutation types and builders
│   ├── events.ts                   # Event types
│   ├── state/
│   │   ├── reducer.ts              # Redux reducer
│   │   ├── actions.ts              # Action creators (commands)
│   │   └── selectors.ts            # State selectors
│   └── middleware.ts               # Redux middleware
├── tests/
├── package.json
├── tsconfig.json
└── README.md
```

#### Core API

```typescript
// src/index.ts

export interface WorkspaceClientConfig {
  serverAddress: string;
  transport: 'grpc' | 'grpc-web';
  clientId?: string;
  dispatch?: ReduxDispatch;  // Optional Redux integration
  reconnect?: {
    maxAttempts: number;
    delayMs: number;
  };
}

export interface WorkspaceClient {
  // Connection
  connect(): Promise<void>;
  disconnect(): void;
  readonly connectionState: ConnectionState;

  // Mutations (returns mutation ID for tracking)
  applyMutation(filePath: string, mutation: Mutation, expectedVersion: number): Promise<string>;

  // Document access
  getOutline(filePath: string): Promise<DocumentOutline>;

  // Streaming
  streamPreview(filePath: string): AsyncIterableIterator<PreviewUpdate>;

  // Event subscription (if not using Redux)
  on(event: string, handler: EventHandler): () => void;
}

// Factory function (follows existing patterns)
export function createWorkspaceClient(config: WorkspaceClientConfig): WorkspaceClient;
```

#### Event Types (Past Tense)

```typescript
// src/events.ts

export type WorkspaceEvent =
  | { type: 'WORKSPACE_CONNECTED'; payload: { clientId: string; timestamp: number } }
  | { type: 'WORKSPACE_DISCONNECTED'; payload: { reason: string } }
  | { type: 'CONNECTION_LOST'; payload: { error: string; willRetry: boolean } }
  | { type: 'CONNECTION_RESTORED'; payload: { timestamp: number } }
  | { type: 'PATCHES_RECEIVED'; payload: { filePath: string; patches: VDocPatch[]; version: number; timestamp: number } }
  | { type: 'MUTATION_APPLIED_OPTIMISTICALLY'; payload: { mutationId: string; mutation: Mutation; filePath: string } }
  | { type: 'MUTATION_ACKNOWLEDGED'; payload: { mutationId: string; newVersion: number; timestamp: number } }
  | { type: 'MUTATION_REBASED'; payload: { mutationId: string; transformedMutation: Mutation; reason: string; newVersion: number } }
  | { type: 'MUTATION_NOOP'; payload: { mutationId: string; reason: string } }
  | { type: 'OUTLINE_LOADED'; payload: { filePath: string; outline: DocumentOutline; version: number } }
  | { type: 'OUTLINE_NODE_UPDATED'; payload: { filePath: string; nodeId: string; changes: Partial<OutlineNode> } }
  | { type: 'ERROR_OCCURRED'; payload: { error: Error; context: string } };
```

#### Redux State Shape (with Immer)

```typescript
// src/state/reducer.ts
import { PCModule } from '@paperclip-ui/proto/lib/generated/virt/module';

export interface WorkspaceState {
  connection: {
    status: 'disconnected' | 'connecting' | 'connected' | 'reconnecting';
    clientId: string | null;
    error: string | null;
    reconnectAttempts: number;
  };

  // VDOM stored directly - Immer provides structural sharing!
  documents: {
    byPath: {
      [filePath: string]: {
        vdom: PCModule;           // ← Full VDOM, not patches!
        version: number;
        lastUpdate: number;
        error: string | null;
      };
    };
  };

  mutations: {
    pending: {
      [mutationId: string]: {
        mutation: Mutation;
        filePath: string;
        timestamp: number;
        expectedVersion: number;
        status: 'sending' | 'sent' | 'acknowledged' | 'rebased' | 'failed';
      };
    };
  };

  outlines: {
    byPath: {
      [filePath: string]: {
        version: number;
        nodes: {
          [nodeId: string]: OutlineNode;
        };
        rootNodeIds: string[];
      };
    };
  };
}

// Reducer uses Immer's produce for structural sharing
import { produce } from 'immer';

export const workspaceReducer = produce((draft, action) => {
  switch (action.type) {
    case 'PATCHES_RECEIVED': {
      const { filePath, patches, version } = action.payload;
      const doc = draft.documents.byPath[filePath];

      if (doc) {
        // Apply patches to VDOM
        // Immer tracks changes - only mutated nodes get new references!
        applyPatchesToVDOM(doc.vdom, patches);
        doc.version = version;
        doc.lastUpdate = Date.now();
      }
      break;
    }

    // ... other cases
  }
}, initialState);

// Helper: applies protocol patches to VDOM structure
function applyPatchesToVDOM(vdom: PCModule, patches: VDocPatch[]): void {
  for (const patch of patches) {
    switch (patch.patch_type) {
      case 'initialize':
        // Replace entire VDOM
        Object.assign(vdom, patch.initialize.vdom);
        break;

      case 'update_text': {
        const node = findNodeByPath(vdom, patch.update_text.path);
        if (node?.textNode) {
          node.textNode.value = patch.update_text.content;
        }
        break;
      }

      // ... other patch types
    }
  }
}
```

**Key Benefits of Immer:**
- Automatic structural sharing (unchanged parts keep same object references)
- Write "mutative" code, get immutable updates
- Perfect for `===` reference equality checks in renderer

#### Redux Middleware

```typescript
// src/middleware.ts

export const workspaceMiddleware = (client: WorkspaceClient) => {
  return (store: MiddlewareAPI) => (next: Dispatch) => (action: Action) => {
    // Handle command actions that trigger side effects
    if (action.type === 'workspace/applyMutation') {
      const { filePath, mutation, expectedVersion } = action.payload;

      // Dispatch optimistic event immediately
      store.dispatch({
        type: 'MUTATION_APPLIED_OPTIMISTICALLY',
        payload: {
          mutationId: generateMutationId(),
          mutation,
          filePath,
        },
      });

      // Send to server (async side effect)
      client.applyMutation(filePath, mutation, expectedVersion)
        .catch((error) => {
          store.dispatch({
            type: 'ERROR_OCCURRED',
            payload: { error, context: 'applyMutation' },
          });
        });
    }

    if (action.type === 'workspace/connect') {
      client.connect().catch((error) => {
        store.dispatch({
          type: 'ERROR_OCCURRED',
          payload: { error, context: 'connect' },
        });
      });
    }

    return next(action);
  };
};
```

#### Connection Management

```typescript
// src/connection.ts

export class ConnectionManager {
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;

  constructor(
    private transport: Transport,
    private config: ReconnectConfig,
    private dispatch: (event: WorkspaceEvent) => void
  ) {}

  async connect(): Promise<void> {
    try {
      await this.transport.connect();
      this.reconnectAttempts = 0;
      this.dispatch({
        type: 'WORKSPACE_CONNECTED',
        payload: { clientId: this.transport.clientId, timestamp: Date.now() },
      });
    } catch (error) {
      this.handleConnectionError(error);
    }
  }

  private handleConnectionError(error: Error): void {
    if (this.reconnectAttempts < this.config.maxAttempts) {
      this.reconnectAttempts++;
      this.dispatch({
        type: 'CONNECTION_LOST',
        payload: { error: error.message, willRetry: true },
      });

      this.reconnectTimer = setTimeout(() => {
        this.connect();
      }, this.config.delayMs * Math.pow(2, this.reconnectAttempts - 1)); // Exponential backoff
    } else {
      this.dispatch({
        type: 'CONNECTION_LOST',
        payload: { error: error.message, willRetry: false },
      });
    }
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }
    this.transport.disconnect();
    this.dispatch({
      type: 'WORKSPACE_DISCONNECTED',
      payload: { reason: 'client_disconnect' },
    });
  }
}
```

#### Transport Abstraction

```typescript
// src/transport/transport.ts

export interface Transport {
  readonly clientId: string;

  connect(): Promise<void>;
  disconnect(): void;

  applyMutation(request: MutationRequest): Promise<MutationResponse>;
  getOutline(request: OutlineRequest): Promise<OutlineResponse>;
  streamPreview(request: PreviewRequest): AsyncIterableIterator<PreviewUpdate>;
}

// src/transport/grpc-transport.ts (Node.js)
export class GrpcTransport implements Transport {
  private client: WorkspaceServiceClient;

  constructor(serverAddress: string) {
    this.client = new WorkspaceServiceClient(
      serverAddress,
      grpc.credentials.createInsecure()
    );
  }

  // Implementation...
}

// src/transport/grpc-web-transport.ts (Browser)
export class GrpcWebTransport implements Transport {
  private client: WorkspaceServiceClient;

  constructor(serverAddress: string) {
    this.client = new WorkspaceServiceClient(serverAddress, null, null);
  }

  // Implementation...
}
```

### Phase 3: Rendering Client Library

#### Package Structure

```
packages/rendering-client/
├── src/
│   ├── index.ts                # Public API
│   ├── PaperclipRenderer.tsx   # Main React component
│   ├── usePrevious.ts          # Hook for prev value tracking
│   ├── patcher.ts              # VDOM diffing and DOM patching
│   ├── iframe-manager.ts       # Iframe isolation per root node
│   ├── style-injector.ts       # Style injection via CSSOM
│   └── node-factory.ts         # DOM node creation utilities
├── tests/
├── package.json
├── tsconfig.json
└── README.md
```

#### Core API (Pure React Component)

```typescript
// src/index.ts

export interface PaperclipRendererProps {
  // VDOM to render (from Redux state)
  vdom: PCModule;

  // Frame index to render (multi-frame support)
  frameIndex?: number;

  // Isolation strategy
  isolation?: 'iframe' | 'none';

  // URL resolver for assets
  resolveUrl?: (url: string) => string;

  // Variant IDs to apply
  variantIds?: string[];

  // Show slot placeholders
  showSlotPlaceholders?: boolean;

  // Selection (for highlighting)
  selected?: string[];

  // Callbacks
  onNodeClick?: (nodeId: string) => void;
  onNodeHover?: (nodeId: string | null) => void;
}

// Pure React component - no class, no instance methods
export const PaperclipRenderer: React.FC<PaperclipRendererProps> = ({
  vdom,
  frameIndex = 0,
  isolation = 'iframe',
  ...options
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const prevVdom = usePrevious(vdom);

  // Initial mount
  useEffect(() => {
    if (!containerRef.current) return;

    const frame = renderFrame(vdom, frameIndex, {
      ...options,
      domFactory: createDOMFactory(),
    });

    containerRef.current.appendChild(frame);

    return () => {
      // Cleanup on unmount
      containerRef.current?.firstChild?.remove();
    };
  }, []);

  // Patch on updates
  useEffect(() => {
    if (!containerRef.current || !prevVdom) return;

    const frame = containerRef.current.firstChild as HTMLElement;
    if (!frame) return;

    // Efficient patching via reference equality checks
    patchFrame(frame, frameIndex, prevVdom, vdom, {
      ...options,
      domFactory: createDOMFactory(),
    });
  }, [vdom, frameIndex, prevVdom]);

  return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />;
};
```

#### VDOM Diffing and Patching (Reference Equality)

```typescript
// src/patcher.ts
// Inspired by libs/web-renderer - uses === checks for efficiency

import type * as html from '@paperclip-ui/proto/lib/generated/virt/html';
import type { PCModule } from '@paperclip-ui/proto/lib/generated/virt/module';

/**
 * Patch a frame by comparing prev and curr VDOM
 * Key insight: Immer gives structural sharing, so prevVdom === currVdom
 * means nothing changed - we can skip all work!
 */
export const patchFrame = (
  frame: HTMLElement,
  frameIndex: number,
  prevModule: PCModule,
  currModule: PCModule,
  options: RenderOptions
): void => {
  // Fast path: if modules are same reference, nothing changed
  if (prevModule === currModule) {
    return;
  }

  const prevFrame = prevModule.html.children[frameIndex];
  const currFrame = currModule.html.children[frameIndex];

  // Update styles if changed
  if (prevModule.css !== currModule.css) {
    patchDocumentStyles(frame, prevModule.css, currModule.css, options);
  }

  // Update imported styles if changed
  if (prevModule.imports !== currModule.imports) {
    patchImportedStyles(frame, prevModule, currModule, options);
  }

  // Patch the frame content
  const stage = frame.childNodes[STAGE_INDEX] as HTMLElement;
  patchNode(stage.childNodes[0] as HTMLElement, prevFrame, currFrame, options);
};

/**
 * Recursively patch a single node
 * Uses === to skip unchanged subtrees (thanks to Immer!)
 */
const patchNode = (
  domNode: HTMLElement | Text,
  prevVirt: html.Node,
  currVirt: html.Node,
  options: RenderOptions
): void => {
  // 🎯 KEY OPTIMIZATION: Reference equality check
  if (prevVirt === currVirt) {
    return;  // Nothing changed - skip all work!
  }

  // If node type changed, replace entirely
  if (!isNodeKindSame(prevVirt, currVirt)) {
    const replacement = createNativeNode(currVirt, options);
    domNode.parentNode.insertBefore(replacement, domNode);
    domNode.remove();
    return;
  }

  // Patch element
  if (prevVirt.element && currVirt.element) {
    patchElement(domNode as HTMLElement, prevVirt.element, currVirt.element, options);
  }

  // Patch text node
  else if (prevVirt.textNode && currVirt.textNode) {
    if (domNode.nodeType === Node.TEXT_NODE) {
      (domNode as Text).nodeValue = currVirt.textNode.value;
    } else {
      // Text node wrapped in span for styling
      (domNode.childNodes[0] as Text).nodeValue = currVirt.textNode.value;
    }
  }
};

/**
 * Patch element attributes and children
 */
const patchElement = (
  domElement: HTMLElement,
  prevVirt: html.Element,
  currVirt: html.Element,
  options: RenderOptions
): void => {
  // Update ID
  domElement.id = '_' + currVirt.id;

  // Patch attributes (only if changed)
  if (prevVirt.attributes !== currVirt.attributes) {
    patchAttributes(domElement, prevVirt, currVirt, options);
  }

  // Patch children (recursive)
  patchChildren(domElement, prevVirt.children, currVirt.children, options);
};

/**
 * Patch children array
 */
const patchChildren = (
  parent: HTMLElement,
  prevChildren: html.Node[],
  currChildren: html.Node[],
  options: RenderOptions
): void => {
  const low = Math.min(prevChildren.length, currChildren.length);

  // Update existing children
  for (let i = 0; i < low; i++) {
    patchNode(parent.childNodes[i] as any, prevChildren[i], currChildren[i], options);
  }

  // Insert new children
  if (prevChildren.length < currChildren.length) {
    for (let i = prevChildren.length; i < currChildren.length; i++) {
      parent.appendChild(createNativeNode(currChildren[i], options));
    }
  }

  // Remove old children
  else {
    for (let i = currChildren.length; i < prevChildren.length; i++) {
      parent.lastChild?.remove();
    }
  }
};

/**
 * Update CSS stylesheet via CSSOM (more efficient than replacing <style>)
 */
const patchCSSStyleSheet = (
  sheet: CSSStyleSheet,
  prevRules: css.Rule[],
  currRules: css.Rule[],
  options: RenderOptions
): void => {
  const low = Math.min(prevRules.length, currRules.length);

  // Update existing rules
  for (let i = 0; i < low; i++) {
    if (prevRules[i] !== currRules[i]) {  // ← Reference equality!
      sheet.deleteRule(i);
      sheet.insertRule(stringifyCSSRule(currRules[i], options), i);
    }
  }

  // Insert new rules
  if (prevRules.length < currRules.length) {
    for (let i = prevRules.length; i < currRules.length; i++) {
      sheet.insertRule(stringifyCSSRule(currRules[i], options));
    }
  }

  // Remove old rules
  else {
    for (let i = currRules.length; i < prevRules.length; i++) {
      sheet.deleteRule(sheet.cssRules.length - 1);
    }
  }
};
```

**Why this is fast:**
- `===` checks are O(1)
- Unchanged subtrees are skipped entirely (common case!)
- Only mutated parts get patched
- CSSOM updates are faster than string replacement
- No intermediate patch objects (diff and patch happen together)

#### Iframe Isolation

```typescript
// src/iframe-manager.ts

export class IframeManager {
  private iframes = new Map<string, HTMLIFrameElement>();

  constructor(private container: HTMLElement) {}

  createIframeForNode(nodeId: string, vnode: VNode): HTMLIFrameElement {
    const iframe = document.createElement('iframe');
    iframe.setAttribute('data-node-id', nodeId);
    iframe.style.border = 'none';
    iframe.style.width = '100%';
    iframe.style.height = '100%';

    this.container.appendChild(iframe);
    this.iframes.set(nodeId, iframe);

    // Wait for iframe to load, then render content
    iframe.addEventListener('load', () => {
      const doc = iframe.contentDocument;
      if (!doc) return;

      const rendered = this.renderVNode(vnode);
      doc.body.appendChild(rendered);
    });

    // Trigger load
    iframe.srcdoc = '<!DOCTYPE html><html><head></head><body></body></html>';

    return iframe;
  }

  getIframe(nodeId: string): HTMLIFrameElement | null {
    return this.iframes.get(nodeId) || null;
  }

  removeIframe(nodeId: string): void {
    const iframe = this.iframes.get(nodeId);
    if (iframe) {
      iframe.remove();
      this.iframes.delete(nodeId);
    }
  }

  private renderVNode(vnode: VNode): Element {
    // Render VNode to DOM element
    // (Reuse existing vdom.ts createElement logic)
    return createElement(vnode);
  }
}
```

#### Style Injection

```typescript
// src/style-injector.ts

export class StyleInjector {
  private styleElement: HTMLStyleElement;
  private rules = new Map<string, CSSRule>();

  constructor(
    private target: Document | ShadowRoot,
    private scoped: boolean
  ) {
    this.styleElement = target.createElement('style');
    target.head?.appendChild(this.styleElement) || target.appendChild(this.styleElement);
  }

  addRule(selector: string, properties: Record<string, string>): void {
    const cssText = this.buildCSSText(selector, properties);
    const sheet = this.styleElement.sheet;

    if (sheet) {
      const index = sheet.insertRule(cssText, sheet.cssRules.length);
      this.rules.set(selector, sheet.cssRules[index]);
    }
  }

  removeRule(selector: string): void {
    const rule = this.rules.get(selector);
    if (!rule) return;

    const sheet = this.styleElement.sheet;
    if (sheet) {
      const index = Array.from(sheet.cssRules).indexOf(rule);
      if (index >= 0) {
        sheet.deleteRule(index);
        this.rules.delete(selector);
      }
    }
  }

  private buildCSSText(selector: string, properties: Record<string, string>): string {
    const scopedSelector = this.scoped ? this.scopeSelector(selector) : selector;
    const props = Object.entries(properties)
      .map(([key, value]) => `${key}: ${value};`)
      .join(' ');
    return `${scopedSelector} { ${props} }`;
  }

  private scopeSelector(selector: string): string {
    // Add scoping attribute to prevent style leaking
    return `[data-paperclip-scope] ${selector}`;
  }
}
```

### Phase 4: Integration Example

```typescript
// Example: Designer application with Redux + Immer + React

import { configureStore } from '@reduxjs/toolkit';
import { Provider, useSelector } from 'react-redux';
import { createWorkspaceClient, workspaceReducer, workspaceMiddleware } from '@paperclip/workspace-client';
import { PaperclipRenderer } from '@paperclip/rendering-client';

// 1. Create workspace client
const workspaceClient = createWorkspaceClient({
  serverAddress: 'localhost:50051',
  transport: 'grpc-web',
  reconnect: {
    maxAttempts: 5,
    delayMs: 1000,
  },
});

// 2. Create Redux store (Redux Toolkit includes Immer!)
const store = configureStore({
  reducer: {
    workspace: workspaceReducer,  // Uses Immer internally
    // ... other reducers
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware().concat(workspaceMiddleware(workspaceClient)),
});

// 3. Connect workspace client to Redux
workspaceClient.on('*', (event) => {
  store.dispatch(event);
});

// 4. Designer Component (pure React)
function DesignerApp() {
  return (
    <Provider store={store}>
      <DesignerCanvas />
    </Provider>
  );
}

function DesignerCanvas() {
  // Get VDOM from Redux (with structural sharing via Immer!)
  const vdom = useSelector((state) =>
    state.workspace.documents.byPath['button.pc']?.vdom
  );

  const selected = useSelector((state) =>
    state.workspace.selection.nodeIds
  );

  const handleNodeClick = (nodeId: string) => {
    store.dispatch({
      type: 'workspace/selectNode',
      payload: { nodeId },
    });
  };

  if (!vdom) {
    return <div>Loading...</div>;
  }

  // Pure rendering component - just pass VDOM data!
  return (
    <PaperclipRenderer
      vdom={vdom}
      frameIndex={0}
      isolation="iframe"
      selected={selected}
      onNodeClick={handleNodeClick}
    />
  );
}

// 5. Connect to server and stream preview
async function init() {
  await store.dispatch({ type: 'workspace/connect' });

  // Start streaming preview
  const stream = workspaceClient.streamPreview('button.pc');
  for await (const update of stream) {
    // Events automatically dispatched to Redux
    // PaperclipRenderer re-renders with new VDOM (via useSelector)
  }
}

// 6. Apply a mutation (from toolbar, layers panel, etc.)
function applyTextEdit(nodeId: string, content: string) {
  const currentVersion = store.getState().workspace.documents.byPath['button.pc'].version;

  store.dispatch({
    type: 'workspace/applyMutation',
    payload: {
      filePath: 'button.pc',
      mutation: {
        mutation_type: 'update_text',
        update_text: {
          node_id: nodeId,
          content: content,
        },
      },
      expectedVersion: currentVersion,
    },
  });
}

// The elegant flow:
// 1. User edits → applyTextEdit() dispatches action
// 2. Middleware dispatches MUTATION_APPLIED_OPTIMISTICALLY
// 3. Reducer updates VDOM with Immer (only changed nodes get new refs)
// 4. PaperclipRenderer re-renders (useSelector triggers)
// 5. Renderer does === checks, only patches changed nodes
// 6. Meanwhile, mutation sent to server
// 7. Server broadcasts patches
// 8. Reducer applies patches with Immer (structural sharing!)
// 9. PaperclipRenderer patches only what changed

// Key insight: Immer + === checks mean we only re-render changed nodes!
```

## Acceptance Criteria

### Functional Requirements

#### Protocol Extension
- [ ] `ApplyMutation` RPC defined in proto with all mutation types
- [ ] `GetDocumentOutline` RPC defined with outline node structure
- [ ] `PreviewUpdate` extended with acknowledgment fields
- [ ] Proto types compile successfully for TypeScript and Rust
- [ ] Version tracking ensures optimistic concurrency control

#### Workspace Client
- [ ] Factory function creates client with config
- [ ] Supports both gRPC (Node.js) and gRPC-web (Browser) transports
- [ ] Connection management with exponential backoff reconnection
- [ ] Mutation API returns mutation ID for tracking
- [ ] Emits event-driven actions (past tense) for all state changes
- [ ] Redux middleware handles command actions and dispatches events
- [ ] Redux reducer updates state based on events
- [ ] Selectors provide efficient state queries
- [ ] Handles network errors gracefully with retry logic
- [ ] Supports streaming preview updates as async iterator

#### Rendering Client
- [ ] Factory function creates renderer with config
- [ ] Applies all patch types correctly to DOM
- [ ] Applies style patches to CSSOM efficiently
- [ ] Creates iframe per root node when isolation enabled
- [ ] Manages iframe lifecycle (create, update, destroy)
- [ ] Injects styles with optional scoping
- [ ] Listens to workspace client events automatically if provided
- [ ] Can be used standalone without workspace client
- [ ] Handles rapid patch updates without flickering

#### Integration
- [ ] Workspace client integrates with Redux via middleware
- [ ] Rendering client listens to Redux state changes
- [ ] Optimistic mutations show immediately in UI
- [ ] Server acknowledgments reconcile optimistic state
- [ ] Rebased mutations apply correctly without user intervention
- [ ] Connection loss pauses operations, reconnection resumes

### Non-Functional Requirements

#### Performance
- [ ] Mutation latency < 50ms (optimistic apply)
- [ ] Patch application < 16ms for 60fps rendering
- [ ] Redux state updates are batched to prevent thrashing
- [ ] Selectors use memoization for expensive computations
- [ ] Transport uses binary protobuf (not JSON) for efficiency

#### Type Safety
- [ ] Full TypeScript coverage with strict mode enabled
- [ ] All proto types have generated TypeScript definitions
- [ ] No `any` types in public API
- [ ] Discriminated unions for all variant types
- [ ] Exported types for all public interfaces

#### Testing
- [ ] Unit tests for all reducers (pure function testing)
- [ ] Unit tests for middleware side effects
- [ ] Integration tests for workspace client E2E flow
- [ ] Integration tests for rendering client patch application
- [ ] Mock transport for isolated client testing
- [ ] Latency tests validate < 50ms optimistic apply

#### Documentation
- [ ] Comprehensive README for each package
- [ ] API reference with all exported types and functions
- [ ] Integration guide with Redux example
- [ ] Migration guide from old client
- [ ] Architecture decision documentation

### Quality Gates
- [ ] All tests pass
- [ ] TypeScript compiles without errors
- [ ] No eslint warnings
- [ ] Code reviewed by at least one other developer
- [ ] Performance benchmarks meet targets
- [ ] Examples run successfully

## Implementation Phases

### Phase 1: Protocol Foundation (Week 1)
**Goal:** Extend gRPC protocol with editing capabilities

**Tasks:**
- [ ] Design `Mutation` protobuf message with all types
- [ ] Define `ApplyMutation` RPC with request/response types
- [ ] Define `GetDocumentOutline` RPC
- [ ] Extend `PreviewUpdate` with acknowledgment fields
- [ ] Generate TypeScript types from proto
- [ ] Implement Rust server handlers (stub responses initially)
- [ ] Write proto-level tests

**Deliverables:**
- Updated `proto/workspace.proto`
- Generated TypeScript types in `packages/proto-types/`
- Rust server stubs that compile

**Success Criteria:**
- Proto files compile for both TypeScript and Rust
- Can make ApplyMutation RPC call (even if it's a stub)

### Phase 2: Transport Layer (Week 1-2)
**Goal:** Abstract transport with gRPC and gRPC-web implementations

**Tasks:**
- [ ] Define `Transport` interface
- [ ] Implement `GrpcTransport` for Node.js
- [ ] Implement `GrpcWebTransport` for browser
- [ ] Add connection lifecycle management
- [ ] Add reconnection with exponential backoff
- [ ] Write transport unit tests with mocks

**Deliverables:**
- `packages/workspace-client/src/transport/`
- Unit tests with 90%+ coverage

**Success Criteria:**
- Can connect from Node.js using gRPC
- Can connect from browser using gRPC-web
- Reconnection works after connection loss

### Phase 3: Workspace Client Core (Week 2-3)
**Goal:** Implement workspace client with event-driven architecture

**Tasks:**
- [ ] Implement `WorkspaceClient` class
- [ ] Define all event types (past tense)
- [ ] Implement mutation API with ID generation
- [ ] Implement outline fetching
- [ ] Implement preview streaming
- [ ] Add event emission for all state changes
- [ ] Write client integration tests

**Deliverables:**
- `packages/workspace-client/src/client.ts`
- `packages/workspace-client/src/events.ts`
- Integration tests

**Success Criteria:**
- Can apply mutations and receive acknowledgments
- Can fetch document outline
- Can stream preview updates
- Events are emitted correctly

### Phase 4: Redux Integration (Week 3)
**Goal:** Integrate workspace client with Redux

**Tasks:**
- [ ] Define Redux state shape
- [ ] Implement Redux reducer for all events
- [ ] Implement Redux middleware for commands
- [ ] Create action creators for commands
- [ ] Create selectors for state queries
- [ ] Write reducer unit tests
- [ ] Write middleware tests

**Deliverables:**
- `packages/workspace-client/src/state/`
- `packages/workspace-client/src/middleware.ts`
- Comprehensive Redux tests

**Success Criteria:**
- Reducer handles all event types correctly
- Middleware dispatches events for side effects
- Selectors are memoized and efficient
- State shape is normalized

### Phase 5: Rendering Client Foundation (Week 4)
**Goal:** Implement rendering client with patch application

**Tasks:**
- [ ] Implement `RenderingClient` class
- [ ] Implement `PatchApplier` for all patch types
- [ ] Add DOM patch application logic
- [ ] Add CSSOM style application logic
- [ ] Reuse existing `vdom.ts` createElement logic
- [ ] Write patch applier unit tests

**Deliverables:**
- `packages/rendering-client/src/renderer.ts`
- `packages/rendering-client/src/patch-applier.ts`
- Unit tests

**Success Criteria:**
- All patch types apply correctly to DOM
- Style patches update CSSOM efficiently
- No flickering during rapid updates

### Phase 6: Iframe Isolation (Week 4-5)
**Goal:** Add iframe-per-node isolation

**Tasks:**
- [ ] Implement `IframeManager` class
- [ ] Create iframe creation and lifecycle management
- [ ] Render VNodes inside iframes
- [ ] Handle iframe document ready events
- [ ] Implement iframe cleanup on unmount
- [ ] Write iframe manager tests

**Deliverables:**
- `packages/rendering-client/src/iframe-manager.ts`
- Tests

**Success Criteria:**
- Each root node renders in isolated iframe
- Iframes clean up properly
- No memory leaks

### Phase 7: Style Injection (Week 5)
**Goal:** Implement style injection with scoping

**Tasks:**
- [ ] Implement `StyleInjector` class
- [ ] Add CSSOM rule insertion logic
- [ ] Add scoping support for isolated styles
- [ ] Handle style updates (add/remove rules)
- [ ] Write style injector tests

**Deliverables:**
- `packages/rendering-client/src/style-injector.ts`
- Tests

**Success Criteria:**
- Styles inject correctly
- Scoping prevents leaks between components
- Style updates are efficient

### Phase 8: Integration & Examples (Week 6)
**Goal:** Complete integration and provide examples

**Tasks:**
- [ ] Create example Redux application
- [ ] Create example VSCode extension stub
- [ ] Write integration guide
- [ ] Write migration guide from old client
- [ ] Performance benchmarking
- [ ] Documentation review

**Deliverables:**
- `examples/redux-designer/`
- `examples/vscode-extension-stub/`
- Complete documentation

**Success Criteria:**
- Examples run successfully
- Documentation is clear and complete
- Performance targets are met

## Alternative Approaches Considered

### 1. WebSocket Instead of gRPC
**Pros:**
- Simpler browser support (no gRPC-web needed)
- More flexible protocol

**Cons:**
- Have to design custom protocol
- Lose type safety from protobuf
- More maintenance burden

**Decision:** Stick with gRPC. Already invested in proto definitions, and gRPC-web works well in browsers.

### 2. Command-Style Redux Actions
**Pros:**
- More explicit about intent
- Traditional Redux pattern

**Cons:**
- Mixing commands and events creates confusion
- Hard to replay events or build event log
- Doesn't align with event-sourcing patterns

**Decision:** Use event-driven (past tense) actions. Clearer separation between "what happened" (events) and "what to do" (commands via middleware).

### 3. Full CRDT in Client
**Pros:**
- True peer-to-peer collaboration
- Offline editing support

**Cons:**
- Massive complexity
- Doesn't align with "server authority" model
- Not needed for current use cases

**Decision:** Keep CRDT server-side only. Client sends semantic mutations, server handles CRDT internally.

### 4. Direct AST Manipulation
**Pros:**
- More powerful editing
- No server round-trip for edits

**Cons:**
- Client would need parser
- Breaks "server authority" model
- Complex to keep in sync

**Decision:** Client sends semantic mutations, server applies to AST. Keeps client thin.

### 5. Single Unified Library
**Pros:**
- Simpler to use (one import)
- No integration needed

**Cons:**
- Tight coupling between concerns
- Harder to test
- Can't use one without the other

**Decision:** Separate libraries. Workspace client can be used without rendering (CLI, language server). Rendering client can be used without workspace (static rendering).

## Dependencies & Prerequisites

### Required
- Rust packages: `paperclip-workspace`, `paperclip-evaluator`, `paperclip-parser`
- Node.js 18+ (for development)
- TypeScript 5.3+
- Yarn 4.x (package manager)
- Protocol Buffers compiler

### New Dependencies

**Workspace Client:**
- `@grpc/grpc-js` - Node.js gRPC client
- `@grpc/proto-loader` - Dynamic proto loading
- `grpc-web` - Browser gRPC client
- `immer` - Structural sharing for efficient `===` checks
- `redux` or `@reduxjs/toolkit` (peer dependency)

**Rendering Client:**
- `react` (peer dependency)
- `@paperclip-ui/proto` - Protocol buffer types (from existing repo)

### Prerequisites
- Rust workspace server must be running
- Proto files must be up to date
- Existing `packages/client` for reference

## Risk Analysis & Mitigation

### High Risks

**1. gRPC-web Browser Compatibility**
- **Risk:** Some browsers may have issues with gRPC-web
- **Mitigation:** Provide WebSocket fallback transport in future. For MVP, target modern browsers only.
- **Likelihood:** Low
- **Impact:** High

**2. Redux State Bloat**
- **Risk:** Large documents cause Redux state to grow unbounded
- **Mitigation:** Implement state normalization. Consider using IndexedDB for large documents with Redux holding only references.
- **Likelihood:** Medium
- **Impact:** High

**3. Race Conditions in Optimistic Updates**
- **Risk:** Rapid user input creates conflicting optimistic mutations
- **Mitigation:** Use pending queue with strict ordering. Rebase logic must be deterministic. Extensive testing of edge cases.
- **Likelihood:** High
- **Impact:** Medium

**4. Iframe Performance**
- **Risk:** Too many iframes degrade performance
- **Mitigation:** Implement virtual scrolling for large component lists. Lazy-create iframes only when in viewport.
- **Likelihood:** Medium
- **Impact:** Medium

### Medium Risks

**5. Protocol Version Mismatch**
- **Risk:** Client and server protocol versions diverge
- **Mitigation:** Add protocol version to connection handshake. Reject incompatible versions with clear error.
- **Likelihood:** Medium
- **Impact:** Low

**6. Memory Leaks from Event Listeners**
- **Risk:** Forgotten event listener subscriptions cause leaks
- **Mitigation:** Return cleanup functions from `.on()`. Use WeakMap for listener storage. Add cleanup in unmount/disconnect.
- **Likelihood:** Low
- **Impact:** Medium

## Resource Requirements

### Team
- 1 TypeScript developer (full-time) - 6 weeks
- 1 Rust developer (part-time, 50%) - 2 weeks for protocol implementation
- 1 reviewer for code review

### Infrastructure
- Dev server running Rust workspace
- gRPC-web proxy for browser testing
- CI/CD for automated testing

### Time Estimate
- **Phase 1-2:** 2 weeks (protocol + transport)
- **Phase 3-4:** 2 weeks (workspace client + Redux)
- **Phase 5-7:** 3 weeks (rendering client)
- **Phase 8:** 1 week (integration + docs)
- **Total:** 8 weeks with buffer

## Future Considerations

### Extensibility

**1. Additional Transports**
- WebSocket transport for environments without gRPC
- HTTP long-polling fallback
- Message queue transport for offline support

**2. Offline Editing**
- Local IndexedDB persistence
- Offline mutation queue
- Background sync when reconnected

**3. Advanced Collaboration**
- Presence (show other users' cursors)
- Comments and annotations
- Conflict UI (show when rebasing happened)

**4. Performance Optimizations**
- Virtual DOM windowing for large documents
- Incremental evaluation (server-side)
- Compressed patch streaming

**5. Additional Features**
- Undo/Redo (needs event log)
- Time-travel debugging (replay events)
- Mutation batching (group rapid edits)

### Long-Term Vision

**Universal Client:**
- Same library works everywhere (browser, Node, Deno, Bun, mobile)
- Pluggable transports (gRPC, WebSocket, IPC)
- Pluggable renderers (DOM, Canvas, native)

**Editor Ecosystem:**
- VSCode extension uses workspace client
- CLI tools use workspace client
- Language server uses workspace client
- All tools interoperate seamlessly

**Collaboration-First:**
- Real-time multiplayer editing
- Conflict resolution UI
- Presence and awareness
- Activity feed

## Documentation Plan

### User-Facing Documentation

**1. Package READMEs**
- `packages/workspace-client/README.md`
  - Installation
  - Quick start
  - API reference
  - Redux integration guide
  - Examples

- `packages/rendering-client/README.md`
  - Installation
  - Quick start
  - API reference
  - Isolation strategies
  - Examples

**2. Integration Guides**
- "Integrating with Redux" - Step-by-step guide
- "Building a VSCode Extension" - Extension example
- "Rendering in Browser" - Browser-specific concerns
- "Migration from Old Client" - Breaking changes and migration path

**3. API Documentation**
- Generated TypeDoc for all public APIs
- Hosted on GitHub Pages or docs site

### Developer Documentation

**4. Architecture Decision Records**
- Why event-driven architecture?
- Why separate workspace and rendering clients?
- Why gRPC over WebSocket?
- State normalization strategy

**5. Protocol Documentation**
- Complete proto file documentation
- Message flow diagrams
- Sequence diagrams for key operations

**6. Contributing Guide**
- How to build and test
- How to add new mutation types
- How to add new patch types
- Code style and conventions

## References & Research

### Internal References

**Existing Packages:**
- `packages/client/src/vdom.ts:1` - Virtual DOM types and functions
- `packages/client/src/grpc-client.ts:1` - Existing gRPC client pattern
- `packages/client/DEVELOPMENT.md:1` - No globals, pure functions principle
- `packages/workspace/src/server.rs:1` - Workspace server implementation
- `packages/editor/src/lib.rs:1` - Editor architecture with EditSession

**Architecture Documents:**
- `docs/architecture/collaboration.md` - AST-level CRDT design
- `docs/architecture/editor-crate.md` - Editor abstractions (Document, EditSession, Pipeline)
- `packages/client/OT_REFACTOR.md` - Path-based patch architecture

**Proto Definitions:**
- `proto/workspace.proto:1` - Current workspace service
- `proto/vdom.proto:1` - Virtual DOM types
- `proto/patches.proto:1` - Patch types

### Key Architectural Decisions

**From `docs/architecture/collaboration.md`:**
1. **AST-level CRDT, not character-level** - Simpler, more predictable
2. **Server authority** - Client always defers to server
3. **Optimistic updates with rebase** - Apply locally, reconcile with server
4. **Deterministic conflict resolution** - Delete wins, last timestamp wins for moves
5. **source_id stability** - Essential for reliable diffing

**From `docs/architecture/editor-crate.md`:**
1. **EditSession pattern** - Per-client editing state with pending mutations
2. **Pipeline coordination** - Atomic: mutation → AST → evaluate → diff
3. **Document storage abstraction** - Memory, File, CRDT backends

**From `packages/client/DEVELOPMENT.md`:**
1. **No global state** - Explicit dependencies, pure functions
2. **Factory functions** - Configured instance creation
3. **Dependency injection** - Over singletons

### External Best Practices

**Redux:**
- Event-driven actions (past tense) for state changes
- Normalized state for efficient updates
- Middleware for side effects
- Selectors with memoization

**gRPC:**
- Streaming RPCs for real-time updates
- Binary protobuf for efficiency
- Version tracking in messages
- Graceful error handling

**Collaboration:**
- Optimistic UI with reconciliation
- Per-client state tracking
- Deterministic conflict resolution
- Patch-based updates (not full sync)

---

## Summary

This plan outlines the design and implementation of two complementary TypeScript libraries:

1. **Workspace Client** - Redux-integrated, event-driven client for managing connections, mutations, and document state
2. **Rendering Client** - Pure React component that efficiently patches DOM via reference equality checks

**Key Architectural Innovations:**

1. **Redux + Immer = Structural Sharing**
   - Protocol patches update Redux store
   - Immer gives automatic structural sharing (unchanged parts keep same refs)
   - Enables O(1) `===` checks to skip unchanged subtrees

2. **Reference Equality Diffing** (inspired by `libs/web-renderer`)
   ```typescript
   if (prevNode === currNode) {
     return;  // Nothing changed - skip all work!
   }
   ```

3. **Pure Rendering Component**
   - Receives VDOM data, not patch commands or client instances
   - Works like React's reconciliation but for VDOM → DOM
   - Testable, predictable, composable

4. **Event-Driven Architecture**
   - Workspace client emits events (past tense) describing what happened
   - Redux middleware handles command actions
   - Clean separation: commands vs events

5. **Server Authority + Optimistic Updates**
   - Apply mutations immediately for responsive UI
   - Server rebases on conflict (client never partially merges)
   - AST-level semantic mutations (not character-level CRDT)

**Timeline:** 8 weeks, phased implementation with clear deliverables and success criteria at each phase.

**Why This Architecture is Elegant:**
- Leverages existing patterns from `libs/web-renderer` (proven in production)
- Immer + `===` checks make patching extremely efficient
- Pure React component is easy to test and reason about
- Protocol patches are just for Redux updates, not DOM manipulation
- Separation of concerns: workspace client (data) vs renderer (view)
