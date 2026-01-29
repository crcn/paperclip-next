# @paperclip/workspace-client

Universal TypeScript client for Paperclip workspace service. Provides real-time preview streaming, document editing via mutations, and AST access for building design tools.

## Features

- **Event-Driven Architecture**: Past-tense events describe what happened, perfect for Redux integration
- **Universal Transport**: Works in Node.js (gRPC) and browsers (gRPC-web)
- **Real-Time Preview**: Streaming VDOM patches with efficient diffing
- **Document Editing**: Semantic mutations at the AST level (not character-level)
- **Type-Safe**: Full TypeScript definitions for all protocol messages
- **Automatic Reconnection**: Exponential backoff with configurable retry limits

## Installation

```bash
yarn add @paperclip/workspace-client @grpc/grpc-js @grpc/proto-loader
```

## Quick Start

```typescript
import { createWorkspaceClient, GrpcTransport } from '@paperclip/workspace-client';

// Create transport (Node.js)
const transport = new GrpcTransport();

// Create client
const client = createWorkspaceClient(transport, {
  clientId: 'my-editor',
  autoReconnect: true,
});

// Connect to workspace server
await client.connect('localhost:50051');

// Listen to events
client.on('preview-updated', (event) => {
  console.log('Preview updated:', event.update.version);
});

client.on('mutation-acknowledged', (event) => {
  console.log('Mutation applied:', event.mutation_id);
});

// Stream preview updates
for await (const update of client.streamPreview('button.pc')) {
  console.log('Patches:', update.patches.length);
}

// Apply a mutation
await client.applyMutation('button.pc', {
  update_text: {
    node_id: 'text-1',
    content: 'Hello World',
  },
}, expectedVersion);

// Get document outline
const outline = await client.getOutline('button.pc');
```

## Architecture

### Event-Driven Design

All state changes emit past-tense events:

```typescript
// ✓ Good: Past tense (what happened)
client.on('preview-updated', handleUpdate);
client.on('mutation-acknowledged', handleAck);
client.on('file-changed', handleChange);

// ✗ Bad: Commands (what to do)
// Not used in this architecture
```

### Redux Integration

The client is designed to integrate seamlessly with Redux:

```typescript
import { createWorkspaceClient } from '@paperclip/workspace-client';
import { store } from './store';

const client = createWorkspaceClient(transport);

// Forward all events to Redux
client.on('preview-updated', (event) => {
  store.dispatch({ type: 'workspace/preview-updated', payload: event.update });
});

client.on('mutation-acknowledged', (event) => {
  store.dispatch({ type: 'workspace/mutation-acknowledged', payload: event });
});
```

### Transport Abstraction

The client works with any transport implementation:

```typescript
// Node.js with gRPC
import { GrpcTransport } from '@paperclip/workspace-client/grpc';
const transport = new GrpcTransport();

// Browser with gRPC-web (coming soon)
import { GrpcWebTransport } from '@paperclip/workspace-client/grpc-web';
const transport = new GrpcWebTransport();
```

## API Reference

### WorkspaceClient

#### `connect(address: string): Promise<void>`

Connect to the workspace server.

```typescript
await client.connect('localhost:50051');
```

#### `disconnect(): Promise<void>`

Disconnect from the workspace server.

```typescript
await client.disconnect();
```

#### `streamPreview(filePath: string): AsyncIterableIterator<PreviewUpdate>`

Stream real-time preview updates for a file.

```typescript
for await (const update of client.streamPreview('button.pc')) {
  // Handle VDOM patches
  applyPatches(update.patches);
}
```

#### `applyMutation(filePath: string, mutation: Mutation, expectedVersion: number): Promise<MutationResponse>`

Apply a semantic mutation to a document. Mutations are automatically rebased if concurrent changes occurred.

```typescript
const response = await client.applyMutation('button.pc', {
  set_inline_style: {
    node_id: 'element-1',
    property: 'color',
    value: 'red',
  },
}, currentVersion);

if (response.ack) {
  // Mutation applied successfully
} else if (response.rebased) {
  // Mutation was transformed due to concurrent changes
} else if (response.noop) {
  // Mutation had no effect
}
```

#### `getOutline(filePath: string): Promise<OutlineResponse>`

Get the document outline (component list, AST structure).

```typescript
const outline = await client.getOutline('button.pc');
console.log('Components:', outline.nodes);
```

#### `on<T>(eventType: string, listener: (event: T) => void): () => void`

Register an event listener. Returns an unsubscribe function.

```typescript
const unsubscribe = client.on('preview-updated', (event) => {
  console.log('Update:', event.update);
});

// Later, unsubscribe
unsubscribe();
```

### Event Types

All events follow a consistent structure:

```typescript
interface WorkspaceEvent {
  type: string;        // Event type (past tense)
  timestamp: number;   // When the event occurred
  // ... event-specific fields
}
```

Available events:

- `connected` - Connection established
- `disconnected` - Connection lost
- `preview-updated` - Preview update received
- `file-changed` - File changed on disk
- `mutation-acknowledged` - Mutation was applied
- `mutation-rebased` - Mutation was transformed
- `mutation-noop` - Mutation had no effect
- `outline-received` - Document outline received
- `connection-failed` - Connection attempt failed
- `rpc-failed` - RPC call failed

### Mutation Types

Supported mutation operations:

```typescript
// Move element to new parent
{
  move_element: {
    node_id: 'element-1',
    new_parent_id: 'parent-2',
    index: 0,
  }
}

// Update text content
{
  update_text: {
    node_id: 'text-1',
    content: 'New text',
  }
}

// Set inline style
{
  set_inline_style: {
    node_id: 'element-1',
    property: 'background',
    value: 'blue',
  }
}

// Set attribute
{
  set_attribute: {
    node_id: 'element-1',
    name: 'class',
    value: 'active',
  }
}

// Remove node
{
  remove_node: {
    node_id: 'element-1',
  }
}

// Insert element
{
  insert_element: {
    parent_id: 'parent-1',
    index: 2,
    element_json: '{"tag":"div"}',
  }
}
```

## Configuration

### WorkspaceClientConfig

```typescript
{
  // Client ID for tracking mutations (default: auto-generated)
  clientId?: string;

  // Enable automatic reconnection (default: true)
  autoReconnect?: boolean;

  // Maximum reconnection attempts (default: 10)
  maxReconnectAttempts?: number;

  // Initial reconnection delay in ms (default: 1000)
  reconnectDelayMs?: number;

  // Maximum reconnection delay in ms (default: 30000)
  maxReconnectDelayMs?: number;
}
```

## Examples

See the [examples directory](../../examples) for complete applications:

- `redux-designer` - Full Redux integration with live preview
- `vscode-extension-stub` - VSCode extension integration

## License

MIT
