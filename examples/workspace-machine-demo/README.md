# Workspace Machine Demo

Demo application showing the WorkspaceMachine in action with real-time preview.

## Features

- **Connection Management**: Connect/disconnect from workspace server
- **Real-time Preview**: Stream VDOM updates from `.pc` files
- **Document Outline**: View AST structure with component/element tree
- **State Inspection**: See all loaded documents and their versions
- **Machine Pattern**: Uses clean machine/engine architecture (no Redux)

## Architecture

```
App
├─ DispatchProvider (machine registry)
└─ WorkspaceMachine.Provider (workspace state)
   └─ WorkspaceDemo (UI)
```

**State Flow:**
```
UI Dispatch Event → Reducer (pure) → Engine (side effects) → WorkspaceClient
                         ↓                                         ↓
                    State Update ←─────── Client Events ←─────────┘
```

## Setup

1. **Start the workspace server:**
   ```bash
   cd packages/workspace
   cargo run
   ```
   Server will listen on `localhost:50051`

2. **Create a test file:**
   ```bash
   echo 'component Button { <button>Click me</button> }' > button.pc
   ```

3. **Run the demo:**
   ```bash
   cd examples/workspace-machine-demo
   yarn install
   yarn dev
   ```
   Open http://localhost:3000

## Usage

1. Click **Connect** to establish connection to `localhost:50051`
2. Enter a file path (e.g., `button.pc`)
3. Click **Load Preview** to stream VDOM updates
4. Click **Load Outline** to see document structure
5. Edit the `.pc` file - watch real-time updates!

## Code Tour

**`App.tsx`** - Root component
- Creates WorkspaceClient once with GrpcTransport
- Sets up DispatchProvider + WorkspaceMachine.Provider

**`WorkspaceDemo.tsx`** - Main UI
- Uses `WorkspaceMachine.useSelector()` to read state
- Uses `dispatch.dispatch()` to trigger events
- Shows connection status, documents, VDOM, outline

**Machine Events:**
- `connection-requested` - User wants to connect
- `preview-requested` - User wants to load file
- `connected` - Connection established (from engine)
- `preview-updated` - VDOM received (from engine)
- `outline-received` - AST received (from engine)

## Key Concepts

### 1. Event-Driven (Past Tense)

Events describe **what happened**, not commands:
```tsx
// ✓ Good
dispatch({ type: 'connection-requested', payload: { address } });
dispatch({ type: 'preview-updated', payload: { update } });

// ✗ Bad (imperative)
dispatch({ type: 'connect', payload: { address } });
dispatch({ type: 'update-preview', payload: { update } });
```

### 2. Reducer = Pure State

```tsx
case 'preview-updated':
  return {
    ...state,
    documents: {
      ...state.documents,
      [update.file_path]: {
        vdom: update.patches[0].initialize.vdom,
        version: update.version,
      },
    },
  };
```

### 3. Engine = Side Effects

```tsx
async handleEvent(event) {
  if (event.type === 'preview-requested') {
    // Call WorkspaceClient (side effect)
    for await (const update of client.streamPreview(filePath)) {
      machine.dispatch({ type: 'preview-updated', payload: { update } });
    }
  }
}
```

### 4. No Redux Middleware

Traditional Redux:
```tsx
// Need thunk/saga middleware
const loadPreview = (path) => async (dispatch) => {
  try {
    const updates = await client.streamPreview(path);
    dispatch({ type: 'PREVIEW_LOADED', payload: updates });
  } catch (error) {
    dispatch({ type: 'PREVIEW_ERROR', payload: error });
  }
};
```

Machine pattern:
```tsx
// Engine handles it directly
dispatch({ type: 'preview-requested', payload: { filePath } });
// Engine sees event, calls client, dispatches results
```

## Next Steps

- Add mutation support (edit VDOM, see updates)
- Add optimistic updates visualization
- Add multiple file tabs
- Add error boundaries
- Add WebSocket fallback for browsers

## Troubleshooting

**"Cannot connect to server"**
- Ensure workspace server is running (`cargo run` in packages/workspace)
- Check server is on port 50051
- Verify gRPC server is accessible

**"No preview updates"**
- Make sure file exists in workspace directory
- Check server logs for parse errors
- Try a simple file: `component Test { <div>Hello</div> }`

**"Module not found" errors**
- Run `yarn install` in workspace-client and common-js packages
- Rebuild packages: `cd packages/workspace-client && yarn build`
