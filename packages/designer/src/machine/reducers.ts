/**
 * Designer reducers - pure state transitions
 */

import { Reducer } from "@paperclip/common";
import {
  centerTransformOnBounds,
  centerTransformZoom,
  getFramesBounds,
  ZOOM_SENSITIVITY,
} from "./geometry";
import { DesignerEvent, DesignerState } from "./state";

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
      console.log("[reducer] document/loaded frames:", event.payload.frames.length, "canvasSize:", state.canvas.size, "centeredInitial:", state.centeredInitial);

      let newState = {
        ...state,
        document: event.payload.document,
        frames: event.payload.frames,
      };

      // Auto-center on first load if canvas has size
      if (!state.centeredInitial && state.canvas.size.width > 0) {
        const bounds = getFramesBounds(event.payload.frames.map((f) => f.bounds));
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

    default:
      return state;
  }
};
