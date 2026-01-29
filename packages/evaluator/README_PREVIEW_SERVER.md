# Paperclip Preview Server

A lightweight HTTP server with WebSocket support for live preview of Paperclip components.

## Features

- 🚀 **Live Hot Reload**: Automatically recompiles and updates browser when `.pc` files change
- 🔌 **WebSocket Streaming**: Real-time updates without page reload
- 🎨 **Simple VDOM Rendering**: Client-side JavaScript renders Virtual DOM to actual DOM
- ⚡ **Fast Feedback Loop**: See changes instantly in browser

## Usage

### Start the Server

```bash
cargo run --bin preview_server --features preview -- path/to/component.pc
```

Example:
```bash
cargo run --bin preview_server --features preview -- examples/test.pc
```

### Open Preview

Once the server starts, open your browser to:
```
http://localhost:3030
```

The preview will automatically update whenever you save changes to the `.pc` file.

## How It Works

### Server-Side

1. **File Watching**: Uses `notify` crate to watch for file system changes
2. **Parse → Evaluate**: On file change, re-parses and evaluates to VDOM
3. **WebSocket Broadcast**: Sends updated VDOM to all connected clients

### Client-Side

1. **WebSocket Connection**: Browser connects to `ws://localhost:3030/ws`
2. **VDOM Rendering**: JavaScript receives VDOM JSON and renders to real DOM
3. **Auto-Reconnect**: Automatically reconnects if connection drops

## Architecture

```
┌─────────────────┐
│  .pc File       │
│  (on disk)      │
└────────┬────────┘
         │ (file change event)
         ▼
┌─────────────────┐
│  File Watcher   │
│  (notify crate) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Parser         │
│  → AST          │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Evaluator      │
│  → VDOM         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  WebSocket      │
│  Broadcast      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Browser        │
│  (Live Update)  │
└─────────────────┘
```

## Message Format

### Initial State

Sent when client first connects:

```json
{
  "type": "initial",
  "version": 1,
  "vdom": {
    "nodes": [...],
    "styles": [...]
  }
}
```

### Updates

Sent when file changes:

```json
{
  "type": "update",
  "version": 2,
  "vdom": {
    "nodes": [...],
    "styles": [...]
  }
}
```

### Errors

Sent when parsing/evaluation fails:

```json
{
  "type": "error",
  "message": "Parse error: ..."
}
```

## VDOM Structure

The Virtual DOM is sent as JSON with this structure:

```typescript
interface VNode {
  type: "Element" | "Text" | "Comment" | "Error";

  // Element nodes
  tag?: string;
  attributes?: Record<string, string>;
  children?: VNode[];

  // Text/Comment nodes
  content?: string;

  // Error nodes
  message?: string;
}

interface VDocument {
  nodes: VNode[];
  styles: CssRule[];
}
```

## Implementation Notes

### Current Approach: Full VDOM Re-rendering

For simplicity, the current implementation sends the full VDOM on each update and the browser re-renders everything. This is:

**Pros:**
- ✅ Simple to implement and understand
- ✅ No complex diffing logic in browser
- ✅ Reliable - no sync issues

**Cons:**
- ❌ Less efficient for large documents
- ❌ Doesn't preserve DOM state (scroll, focus, etc.)

### Future: Incremental Patching

For production use, we could send incremental patches instead:

1. Compute diff on server using `diff_vdocument()`
2. Send only the patches (add/remove/update operations)
3. Apply patches surgically in browser
4. Preserve DOM state between updates

This would require:
- Adding serde support to protobuf types
- Implementing DOM patching algorithm in JavaScript
- Handling edge cases (concurrent updates, etc.)

## Related

- **gRPC Server**: `packages/workspace/src/server.rs` - More sophisticated server with full gRPC support
- **Evaluator**: `packages/evaluator/src/evaluator.rs` - VDOM evaluation logic
- **Differ**: `packages/evaluator/src/vdom_differ.rs` - VDOM diffing algorithm

## Example Component

```paperclip
public component HelloWorld {
    render div {
        h1 {
            text "Hello, Paperclip!"
        }
        p {
            text "This is a live preview."
        }
    }
}
```

Edit this file and watch it update live in the browser!
