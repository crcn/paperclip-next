/**
 * Designer reducers - pure state transitions
 */

import { Reducer } from "@paperclip/common";
import {
  centerTransformOnBounds,
  centerTransformZoom,
  getFramesBounds,
  screenToCanvas,
  ZOOM_SENSITIVITY,
} from "./geometry";
import {
  DesignerEvent,
  DesignerState,
  Frame,
  FrameBounds,
  PendingMutation,
  Point,
  ResizeHandle,
} from "./state";

const MIN_FRAME_SIZE = 50;

/**
 * Calculate new frame bounds based on resize handle drag
 */
function calculateResizedBounds(
  startBounds: FrameBounds,
  handle: ResizeHandle,
  delta: Point
): FrameBounds {
  let { x, y, width, height } = startBounds;

  // Handle horizontal resize
  if (handle.includes("w")) {
    const newWidth = Math.max(MIN_FRAME_SIZE, width - delta.x);
    if (newWidth !== width) {
      x = x + (width - newWidth);
      width = newWidth;
    }
  } else if (handle.includes("e")) {
    width = Math.max(MIN_FRAME_SIZE, width + delta.x);
  }

  // Handle vertical resize
  if (handle.includes("n")) {
    const newHeight = Math.max(MIN_FRAME_SIZE, height - delta.y);
    if (newHeight !== height) {
      y = y + (height - newHeight);
      height = newHeight;
    }
  } else if (handle.includes("s")) {
    height = Math.max(MIN_FRAME_SIZE, height + delta.y);
  }

  return {
    x: Math.round(x),
    y: Math.round(y),
    width: Math.round(width),
    height: Math.round(height),
  };
}

export const reducer: Reducer<DesignerEvent, DesignerState> = (event, state) => {
  switch (event.type) {
    case "canvas/resized": {
      console.log("[reducer] canvas/resized", event.payload, "frames:", state.frames.length, "centeredInitial:", state.centeredInitial);

      let newState = {
        ...state,
        canvas: {
          ...state.canvas,
          size: event.payload,
        },
      };

      // Auto-center if we have frames but haven't centered yet
      if (!state.centeredInitial && state.frames.length > 0 && event.payload.width > 0) {
        const bounds = getFramesBounds(state.frames.map((f) => f.bounds));
        console.log("[reducer] centering on bounds:", bounds);
        if (bounds) {
          const transform = centerTransformOnBounds(event.payload, bounds, true);
          console.log("[reducer] new transform:", transform);
          newState = {
            ...newState,
            canvas: {
              ...newState.canvas,
              transform,
            },
            centeredInitial: true,
          };
        }
      }

      return newState;
    }

    case "canvas/panned": {
      const { delta, metaKey, ctrlKey } = event.payload;

      if (metaKey || ctrlKey) {
        const zoomDelta = delta.y / ZOOM_SENSITIVITY;
        const newZoom = state.canvas.transform.z * (1 - zoomDelta);

        return {
          ...state,
          canvas: {
            ...state.canvas,
            transform: centerTransformZoom(
              state.canvas.transform,
              state.canvas.size,
              newZoom,
              state.canvas.mousePosition
            ),
          },
        };
      }

      return {
        ...state,
        canvas: {
          ...state.canvas,
          transform: {
            ...state.canvas.transform,
            x: state.canvas.transform.x - delta.x,
            y: state.canvas.transform.y - delta.y,
          },
        },
      };
    }

    case "canvas/zoomed": {
      const { delta, center } = event.payload;
      const zoomDelta = delta / ZOOM_SENSITIVITY;
      const newZoom = state.canvas.transform.z * (1 - zoomDelta);

      return {
        ...state,
        canvas: {
          ...state.canvas,
          transform: centerTransformZoom(
            state.canvas.transform,
            state.canvas.size,
            newZoom,
            center
          ),
        },
      };
    }

    case "canvas/mouseMove": {
      return {
        ...state,
        canvas: {
          ...state.canvas,
          mousePosition: event.payload,
        },
      };
    }

    case "canvas/centerOnFrames": {
      const bounds = getFramesBounds(state.frames.map((f) => f.bounds));
      if (!bounds || state.canvas.size.width === 0) {
        return state;
      }

      return {
        ...state,
        canvas: {
          ...state.canvas,
          transform: centerTransformOnBounds(state.canvas.size, bounds, true),
        },
        centeredInitial: true,
      };
    }

    case "frame/selected": {
      return {
        ...state,
        selectedFrameIndex: event.payload.index,
      };
    }

    case "frame/resized": {
      const { index, bounds } = event.payload;
      const frames = [...state.frames];
      if (frames[index]) {
        frames[index] = { ...frames[index], bounds };
      }
      return { ...state, frames };
    }

    case "frame/moved": {
      const { index, position } = event.payload;
      const frames = [...state.frames];
      if (frames[index]) {
        frames[index] = {
          ...frames[index],
          bounds: {
            ...frames[index].bounds,
            x: position.x,
            y: position.y,
          },
        };
      }
      return { ...state, frames };
    }

    case "document/loaded": {
      console.log("[reducer] document/loaded frames:", event.payload.frames.length, "canvasSize:", state.canvas.size, "centeredInitial:", state.centeredInitial, "pending:", state.pendingMutations.size);

      // Merge server frames with pending mutations to preserve optimistic bounds
      const mergedFrames = mergeFramesWithPending(event.payload.frames, state.pendingMutations);

      let newState = {
        ...state,
        document: event.payload.document,
        frames: mergedFrames,
      };

      // Auto-center on first load if canvas has size
      if (!state.centeredInitial && state.canvas.size.width > 0) {
        const bounds = getFramesBounds(mergedFrames.map((f) => f.bounds));
        console.log("[reducer] centering on bounds:", bounds);
        if (bounds) {
          const transform = centerTransformOnBounds(state.canvas.size, bounds, true);
          console.log("[reducer] new transform:", transform);
          newState = {
            ...newState,
            canvas: {
              ...newState.canvas,
              transform,
            },
            centeredInitial: true,
          };
        }
      }

      return newState;
    }

    case "tool/resizeStart": {
      const { handle, mouse } = event.payload;
      const frameIndex = state.selectedFrameIndex;
      console.log("[reducer] tool/resizeStart handle:", handle, "mouse:", mouse, "frameIndex:", frameIndex);

      if (frameIndex === undefined || !state.frames[frameIndex]) {
        console.log("[reducer] tool/resizeStart - no frame selected");
        return state;
      }

      const startBounds = state.frames[frameIndex].bounds;
      console.log("[reducer] tool/resizeStart startBounds:", startBounds);

      return {
        ...state,
        tool: {
          ...state.tool,
          drag: {
            handle,
            frameIndex,
            startBounds,
            startMouse: mouse,
            currentMouse: mouse,
          },
        },
      };
    }

    case "tool/resizeMove": {
      const drag = state.tool.drag;
      if (!drag) return state;

      console.log("[reducer] tool/resizeMove currentMouse:", event.payload);

      return {
        ...state,
        tool: {
          ...state.tool,
          drag: {
            ...drag,
            currentMouse: event.payload,
          },
        },
      };
    }

    case "tool/resizeEnd": {
      const drag = state.tool.drag;
      if (!drag) {
        console.log("[reducer] tool/resizeEnd - no drag state");
        return state;
      }

      // Calculate the delta in canvas space
      const { transform } = state.canvas;
      const startCanvas = screenToCanvas(drag.startMouse, transform);
      const endCanvas = screenToCanvas(drag.currentMouse, transform);
      const delta = {
        x: endCanvas.x - startCanvas.x,
        y: endCanvas.y - startCanvas.y,
      };

      console.log("[reducer] tool/resizeEnd delta:", delta, "handle:", drag.handle);

      // Calculate final bounds
      const newBounds = calculateResizedBounds(drag.startBounds, drag.handle, delta);
      console.log("[reducer] tool/resizeEnd newBounds:", newBounds, "oldBounds:", drag.startBounds);

      // Update the frame
      const frames = [...state.frames];
      if (frames[drag.frameIndex]) {
        frames[drag.frameIndex] = { ...frames[drag.frameIndex], bounds: newBounds };
      }

      return {
        ...state,
        frames,
        tool: {
          ...state.tool,
          drag: undefined,
        },
      };
    }

    // =========================================================================
    // Mutation Lifecycle Events
    // =========================================================================

    case "mutation/started": {
      const { mutation } = event.payload;
      const newPending = new Map(state.pendingMutations);
      newPending.set(mutation.mutationId, mutation);
      console.log("[reducer] mutation/started:", mutation.mutationId, "pending count:", newPending.size);
      return {
        ...state,
        pendingMutations: newPending,
      };
    }

    case "mutation/acknowledged": {
      const { mutationId, version } = event.payload;
      const newPending = new Map(state.pendingMutations);
      newPending.delete(mutationId);
      console.log("[reducer] mutation/acknowledged:", mutationId, "version:", version, "pending remaining:", newPending.size);
      return {
        ...state,
        pendingMutations: newPending,
      };
    }

    case "mutation/failed": {
      const { mutationId, error } = event.payload;
      console.error("[reducer] mutation/failed:", mutationId, error);

      // Remove the pending mutation and revert the optimistic update
      const failedMutation = state.pendingMutations.get(mutationId);
      const newPending = new Map(state.pendingMutations);
      newPending.delete(mutationId);

      // If we have the original mutation, we could revert here
      // For now, we just remove from pending and let the next SSE update fix it
      // TODO: Implement proper revert to pre-mutation state if needed

      return {
        ...state,
        pendingMutations: newPending,
      };
    }

    default:
      return state;
  }
};

/**
 * Merge server frames with pending mutations.
 * Preserves optimistic bounds for frames that have in-flight mutations.
 */
export function mergeFramesWithPending(
  serverFrames: Frame[],
  pendingMutations: Map<string, PendingMutation>
): Frame[] {
  if (pendingMutations.size === 0) {
    return serverFrames;
  }

  return serverFrames.map((frame) => {
    // Check if this frame has a pending mutation
    for (const [, mutation] of pendingMutations) {
      if (mutation.frameId === frame.id) {
        console.log("[mergeFramesWithPending] Preserving optimistic bounds for frame:", frame.id);
        return {
          ...frame,
          bounds: mutation.optimisticBounds,
        };
      }
    }
    return frame;
  });
}
