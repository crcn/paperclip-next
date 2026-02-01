---
title: Tool Overlay Layer with Frame Resizer
type: feat
date: 2026-02-01
---

# Tool Overlay Layer with Frame Resizer

## Overview

Add a tool overlay layer to the designer that sits above each rendered canvas, starting with a frame resizer tool. When frames are resized or moved, the tool persists bounds back to the source file by updating (or creating) the `/** @frame */` doc comment on each root node.

## Problem Statement / Motivation

Currently, the paperclip-next designer renders frames but lacks interactive tools for manipulating them. Users need visual handles to resize and reposition frames, with changes persisted to the source code. This is essential for the designer to be a functional visual editing tool.

The original paperclip codebase has this feature implemented, providing a proven reference for the interaction patterns and persistence mechanism.

## Proposed Solution

Build a tool overlay system following the architecture patterns established in both the paperclip-next codebase and the original paperclip:

1. **Tool Overlay Component** - A layer rendered above the canvas that displays resize handles for the selected frame
2. **Resize Handles** - 8 handles (4 corners + 4 edges) with drag interactions
3. **Preview Ghost** - Show a ghost/outline during drag rather than live resize
4. **SetFrameBounds Mutation** - Add a mutation type to persist frame bounds to `@frame` doc comments

### Key Interaction Flow

```
User selects frame → Handles appear around frame
                          ↓
User drags handle → Ghost preview shows new bounds
                          ↓
User releases mouse → Dispatch frame/resized event
                          ↓
Engine sends SetFrameBounds mutation → Server updates @frame comment
                          ↓
Server sends PreviewUpdate → Frame renders at new position
```

## Technical Considerations

### Architecture Alignment

Follow the **Machine + Engines pattern** from the codebase:
- **Reducer**: Pure state transitions for drag state and frame bounds
- **Engine**: Side effects for persisting mutations to server

Reference: `/docs/plans/2026-01-27-feat-paperclip-next-full-rewrite-plan.md` lines 1814-1877

### Coordinate Systems

The overlay needs careful coordinate handling:
- **Screen space**: Where mouse events occur, where handles are rendered
- **Canvas space**: Where frames exist, affected by pan/zoom transform

Use existing utilities from `geometry.ts`:
- `screenToCanvas()` - Convert mouse position to canvas coordinates
- `canvasToScreen()` - Position handles in screen space based on frame bounds

### Drag Pattern

Use the proven drag pattern with `_started` flag to prevent false triggers on double-click:

```typescript
const startDOMDrag = (startEvent, onStart, update, stop) => {
  let _started = false;
  const drag = throttle((event) => {
    if (!_started) {
      _started = true;
      onStart?.(event);
    }
    update(event, { delta: { x: event.clientX - sx, y: event.clientY - sy } });
  }, 10);
};
```

Reference: `/docs/plans/2026-01-27-feat-paperclip-next-full-rewrite-plan.md` lines 1634-1660

### @frame Comment Format

Based on original paperclip, the format is:
```
/** @frame(x: 100, y: 200, width: 1024, height: 768) */
public component Card { ... }
```

### Mutation API

A new `SetFrameBounds` mutation type is needed:

```protobuf
message SetFrameBounds {
  string node_id = 1;      // semanticId of the frame's root element
  int32 x = 2;
  int32 y = 3;
  int32 width = 4;
  int32 height = 5;
}
```

The server-side implementation will:
1. Find the component by node_id
2. Parse or create the doc comment
3. Update the @frame parameters
4. Serialize back to source

Reference: Original paperclip at `/libs/core/src/proto/ast_mutate/set_frame_bounds.rs`

### Overlay Pointer Events

The overlay div currently has `pointerEvents: "none"`. Strategy:
- Keep overlay background as `pointerEvents: "none"` for click-through
- Set handles to `pointerEvents: "auto"` to capture mouse events

## Acceptance Criteria

### Core Functionality
- [x] Tool overlay layer renders above canvas content
- [x] Selected frame shows 8 resize handles (4 corners, 4 edges)
- [x] Dragging corner handles resizes frame in both dimensions
- [x] Dragging edge handles resizes frame in one dimension
- [x] Ghost preview shows new bounds during drag
- [x] Frame bounds update on drag completion (mouseup)
- [ ] Changes persist to `@frame` doc comment via SetFrameBounds mutation

### Handle Behavior
- [x] Handles maintain constant screen-space size (8x8px) regardless of zoom
- [x] Handles positioned correctly accounting for canvas transform
- [x] Cursor changes appropriately for each handle (nwse-resize, nesw-resize, ns-resize, ew-resize)
- [x] Minimum frame size enforced (50x50px) to prevent negative dimensions

### @frame Persistence
- [ ] Existing @frame comment is updated with new bounds
- [ ] Missing @frame comment is created with full bounds
- [x] Numeric values are integers (no decimals)

### State Management
- [x] New events: `tool/resizeStart`, `tool/resizeMove`, `tool/resizeEnd`
- [x] Drag state tracked in DesignerState
- [x] Reducer handles all resize events
- [ ] Engine persists changes on `tool/resizeEnd`

## Success Metrics

- Frame resizing feels responsive and intuitive
- Changes persist correctly to source files
- No regressions in existing canvas interactions (pan, zoom, frame selection)

## Dependencies & Risks

### Dependencies
- **Proto schema changes**: New SetFrameBounds mutation type
- **Server-side mutation handler**: Rust implementation for @frame comment updates
- **workspace-client**: Method to send mutations

### Risks
- **Coordinate transform complexity**: Handle positioning requires careful math with zoom/pan
  - *Mitigation*: Use existing geometry utilities, add comprehensive tests
- **Mutation API changes**: Requires coordination with server-side implementation
  - *Mitigation*: Can mock server response initially for UI development

## Files to Create/Modify

### New Files
| File | Purpose |
|------|---------|
| `packages/designer/src/components/ToolOverlay.tsx` | Container for tool overlay layer |
| `packages/designer/src/components/ResizeHandles.tsx` | The 8 resize handles component |
| `packages/designer/src/hooks/useDrag.ts` | Reusable drag interaction hook |

### Modified Files
| File | Changes |
|------|---------|
| `packages/designer/src/machine/state.ts` | Add ToolState, drag state, new events |
| `packages/designer/src/machine/reducers.ts` | Handle resize events |
| `packages/designer/src/machine/engines.ts` | Add persistence engine for mutations |
| `packages/designer/src/components/Canvas.tsx` | Render ToolOverlay in overlay div |
| `packages/proto/src/workspace.proto` | Add SetFrameBounds mutation |

### Server-Side (Rust)
| File | Changes |
|------|---------|
| `packages/evaluator/src/mutations.rs` | Handle SetFrameBounds mutation |
| Parse/serialize @frame doc comments | Similar to original paperclip implementation |

## References & Research

### Internal References
- Original paperclip resizer: `~/Developer/crcn/paperclip/libs/designer/src/ui/logic/Editor/Canvas/Tools/Selectable/index.tsx`
- Original SetFrameBounds mutation: `~/Developer/crcn/paperclip/libs/core/src/proto/ast_mutate/set_frame_bounds.rs`
- Existing geometry utilities: `packages/designer/src/machine/geometry.ts`
- State machine pattern: `packages/designer/src/machine/index.ts`
- Canvas overlay placeholder: `packages/designer/src/components/Canvas.tsx:47-56`

### Architecture Documents
- Machine + Engines pattern: `/docs/plans/2026-01-27-feat-paperclip-next-full-rewrite-plan.md`
- Architecture constitution: `/docs/ARCHITECTURE_CONSTITUTION.md`

## MVP Implementation Order

### Phase 1: UI Foundation
```typescript
// packages/designer/src/components/ToolOverlay.tsx
export const ToolOverlay = () => {
  const selectedFrameIndex = DesignerMachine.useSelector(s => s.selectedFrameIndex);
  const frames = DesignerMachine.useSelector(s => s.frames);
  const transform = DesignerMachine.useSelector(s => s.canvas.transform);

  if (selectedFrameIndex === undefined) return null;

  const frame = frames[selectedFrameIndex];
  return <ResizeHandles frame={frame} transform={transform} />;
};
```

### Phase 2: Resize Handles Component
```typescript
// packages/designer/src/components/ResizeHandles.tsx
const HANDLES = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'] as const;

export const ResizeHandles = ({ frame, transform }: Props) => {
  const dispatch = useDispatch<DesignerEvent>();

  // Convert frame bounds to screen coordinates
  const screenBounds = boundsToScreen(frame.bounds, transform);

  return (
    <>
      {/* Ghost preview during drag */}
      {dragState && <GhostPreview bounds={dragState.previewBounds} />}

      {/* Resize handles */}
      {HANDLES.map(handle => (
        <Handle
          key={handle}
          position={handle}
          bounds={screenBounds}
          onDragStart={(e) => dispatch({ type: 'tool/resizeStart', payload: { handle, mouse: getMousePos(e) }})}
        />
      ))}
    </>
  );
};
```

### Phase 3: State & Events
```typescript
// packages/designer/src/machine/state.ts
export type ResizeHandle = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';

export interface DragState {
  handle: ResizeHandle;
  frameIndex: number;
  startBounds: FrameBounds;
  startMouse: Point;
  previewBounds: FrameBounds;
}

export interface ToolState {
  drag?: DragState;
}

export type DesignerEvent =
  // ... existing events
  | BaseEvent<'tool/resizeStart', { handle: ResizeHandle; mouse: Point }>
  | BaseEvent<'tool/resizeMove', Point>
  | BaseEvent<'tool/resizeEnd'>;
```

### Phase 4: Persistence Engine
```typescript
// In engines.ts - add to handleEvent
if (event.type === 'tool/resizeEnd' && prevState.tool.drag) {
  const { frameIndex, previewBounds } = prevState.tool.drag;
  const frame = prevState.frames[frameIndex];

  // Send mutation to server
  sendMutation({
    setFrameBounds: {
      nodeId: frame.id,
      x: Math.round(previewBounds.x),
      y: Math.round(previewBounds.y),
      width: Math.round(previewBounds.width),
      height: Math.round(previewBounds.height),
    }
  });
}
```
