# Workspace & Rendering Client Libraries - Progress

## ✅ Phase 1: Protocol Foundation (COMPLETE)

- [x] Extended `proto/workspace.proto` with full mutation support
  - ApplyMutation RPC with MutationRequest/MutationResponse
  - All mutation types: MoveElement, UpdateText, SetInlineStyle, SetAttribute, RemoveNode, InsertElement
  - GetDocumentOutline RPC with complete AST extraction
  - PreviewUpdate extended with acknowledged_mutation_ids and changed_by_client_id

- [x] Rust server implementation
  - Complete GetDocumentOutline with line/col position conversion from byte offsets
  - Recursive AST traversal extracting all node types (Tag, Text, Instance, Conditional, Repeat, Insert)
  - ApplyMutation stub (needs full CRDT integration)

- [x] TypeScript types
  - Manually created comprehensive types matching all proto messages
  - VDOM types (VNode, ElementNode, TextNode, ComponentNode)
  - Patch types (all 9 patch operations)
  - Mutation types (all 6 mutation operations)
  - Outline types (OutlineNode, NodeType, SourceSpan)

**Files Modified:**
- `proto/workspace.proto` - Extended with mutations and outline
- `packages/workspace/src/server.rs` - Implemented outline extraction
- `packages/workspace-client/src/types.ts` - Complete TypeScript types

## ✅ Phase 2: Transport Layer (COMPLETE)

- [x] Transport interface abstraction
  - Defines contract for gRPC and gRPC-web implementations
  - Streaming support (AsyncIterableIterator)
  - Error types (ConnectionError, RpcError)

- [x] GrpcTransport for Node.js
  - Full implementation using @grpc/grpc-js
  - Streaming support for StreamPreview and WatchFiles
  - Unary RPCs for ApplyMutation and GetDocumentOutline
  - Connection lifecycle with health checks
  - Exponential backoff reconnection (configurable attempts and delays)

- [x] GrpcWebTransport stub for browsers
  - Interface defined
  - Implementation pending (requires @grpc/grpc-web + proxy)

**Files Created:**
- `packages/workspace-client/src/transport/interface.ts`
- `packages/workspace-client/src/transport/grpc.ts`
- `packages/workspace-client/src/transport/grpc-web.ts`
- `packages/workspace-client/src/transport/index.ts`

## ✅ Phase 3: Workspace Client Core (COMPLETE)

- [x] Package structure
  - Created `@paperclip/workspace-client` package
  - TypeScript configuration with ESM support
  - Yarn 4 standalone package

- [x] Event-driven architecture
  - Past-tense events (connected, preview-updated, mutation-acknowledged, etc.)
  - EventEmitter with subscription management
  - Type-safe event system with discriminated unions

- [x] WorkspaceClient class
  - Factory function `createWorkspaceClient(transport, config)`
  - Mutation API with automatic ID generation
  - Methods: connect, disconnect, streamPreview, applyMutation, getOutline, watchFiles
  - Event emission for all state changes
  - Automatic reconnection with exponential backoff

- [x] Testing
  - 11 passing integration tests
  - Mock transport for testing
  - Tests cover: connection, streaming, mutations, outline, events, subscriptions

- [x] Documentation
  - Complete README with examples
  - API reference
  - Event types documentation
  - Mutation types examples

**Files Created:**
- `packages/workspace-client/src/client.ts` - Main client implementation
- `packages/workspace-client/src/events.ts` - Event system
- `packages/workspace-client/src/client.spec.ts` - Integration tests
- `packages/workspace-client/README.md` - Documentation
- `packages/workspace-client/package.json`, `tsconfig.json`, `yarn.lock`

**Build Status:**
- ✅ TypeScript compiles successfully
- ✅ All 11 tests passing
- ✅ Package exports configured

## ✅ Machine/Engine Pattern (PORTED)

- [x] Core machine pattern ported from `~/Developer/fourthplaces/shay`
  - Located in `packages/common-js/src/machine/`
  - Clean separation: Reducer (pure) + Engine (side effects)
  - No Redux middleware needed

- [x] Core types
  - `Reducer<Event, State>` - Pure state transitions
  - `Engine<Event, State, Props>` - Side effects handler
  - `MachineHandle` - Minimal interface for engines
  - `PropsRef` - Stable reference to dependencies
  - `MachineInstance` - Running machine with state

- [x] React integration
  - `defineMachine()` - Creates Provider + useSelector
  - `DispatchProvider` - Global event dispatch registry
  - `useDispatcher()` - Access dispatcher at any level
  - Event bubbling through all machines

**Files Created:**
- `packages/common-js/src/machine/types.ts` - Core types
- `packages/common-js/src/machine/instance.ts` - Machine instance creation
- `packages/common-js/src/machine/index.ts` - Exports
- `packages/common-js/src/machine/react/context.ts` - React contexts
- `packages/common-js/src/machine/react/dispatchProvider.ts` - Provider component
- `packages/common-js/src/machine/react/defineMachine.ts` - Machine factory
- `packages/common-js/src/machine/react/index.ts` - React exports
- `packages/common-js/src/disposable.ts` - Disposable type
- `packages/common-js/src/index.ts` - Main exports
- `packages/common-js/package.json`, `tsconfig.json`

## 🔄 Phase 4: Redux Integration → Machine Integration (IN PROGRESS)

**Original Plan:** Redux reducers + middleware

**New Approach:** Machine/Engine pattern (cleaner, no Redux needed)

**Next Steps:**
1. Define workspace events (past tense)
2. Create workspace reducer (pure state transitions)
3. Create workspace engine (handles workspace client side effects)
4. Build WorkspaceMachine using defineMachine()

## ⏳ Phase 5: Rendering Client Foundation (PENDING)

Port rendering code from `~/Developer/crcn/paperclip/libs/web-renderer`:
- Pure React component receiving VDOM data
- Reference equality checks (prevNode === currNode)
- patchFrame, patchNode, patchElement, patchChildren
- CSSOM updates via patchCSSStyleSheet

## ⏳ Phase 6-8: Remaining Work

- Phase 6: Iframe isolation
- Phase 7: Style injection
- Phase 8: Examples and integration

## Key Architectural Decisions

1. **Machine/Engine over Redux**
   - Cleaner separation of concerns
   - No middleware complexity
   - Direct side effect handling in engines

2. **Event-Driven Architecture**
   - Past tense events (what happened, not commands)
   - Perfect for Redux/Machine integration
   - Easy to debug and trace

3. **Transport Abstraction**
   - Works in Node.js (gRPC) and browsers (gRPC-web)
   - Easy to add new transports
   - Mock-friendly for testing

4. **Reference Equality with Immer**
   - Structural sharing enables O(1) === checks
   - Skip unchanged subtrees in rendering
   - Proven pattern from existing web-renderer

## Next Immediate Tasks

1. Build WorkspaceMachine:
   - Define workspace events
   - Create workspace reducer
   - Create workspace engine (wraps WorkspaceClient)

2. Create example app demonstrating:
   - DispatchProvider setup
   - WorkspaceMachine.Provider
   - Real-time preview with mutations
   - Document outline display

3. Port rendering client from libs/web-renderer

## Files to Review

**Completed:**
- `packages/workspace-client/` - Full client implementation
- `packages/common-js/src/machine/` - Machine pattern
- `proto/workspace.proto` - Extended protocol
- `packages/workspace/src/server.rs` - Server implementation

**Next:**
- Create `examples/workspace-machine-demo/`
- Port `~/Developer/crcn/paperclip/libs/web-renderer`
