"use client";

import React, { memo, useCallback, useMemo } from "react";
import { useDispatch } from "@paperclip/common";
import { DesignerMachine, DesignerEvent, Frame, Transform, ResizeHandle, Point, FrameBounds } from "../machine";
import { canvasToScreen, screenToCanvas } from "../machine/geometry";
import { useDrag } from "../hooks/useDrag";

// ============================================================================
// Constants
// ============================================================================

const HANDLE_SIZE = 8;
const HANDLE_HALF = HANDLE_SIZE / 2;

const HANDLES: ResizeHandle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];

const CURSOR_MAP: Record<ResizeHandle, string> = {
  nw: "nwse-resize",
  n: "ns-resize",
  ne: "nesw-resize",
  e: "ew-resize",
  se: "nwse-resize",
  s: "ns-resize",
  sw: "nesw-resize",
  w: "ew-resize",
};

// ============================================================================
// Types
// ============================================================================

interface ResizeHandlesProps {
  frame: Frame;
  transform: Transform;
}

// ============================================================================
// Helper Functions
// ============================================================================

function getHandleScreenPosition(
  handle: ResizeHandle,
  bounds: FrameBounds,
  transform: Transform
): Point {
  let canvasX = bounds.x;
  let canvasY = bounds.y;

  // Horizontal position
  if (handle.includes("e")) {
    canvasX = bounds.x + bounds.width;
  } else if (handle === "n" || handle === "s") {
    canvasX = bounds.x + bounds.width / 2;
  }

  // Vertical position
  if (handle.includes("s")) {
    canvasY = bounds.y + bounds.height;
  } else if (handle === "e" || handle === "w") {
    canvasY = bounds.y + bounds.height / 2;
  }

  return canvasToScreen({ x: canvasX, y: canvasY }, transform);
}

function calculatePreviewBounds(
  startBounds: FrameBounds,
  handle: ResizeHandle,
  delta: Point
): FrameBounds {
  const MIN_SIZE = 50;
  let { x, y, width, height } = startBounds;

  if (handle.includes("w")) {
    const newWidth = Math.max(MIN_SIZE, width - delta.x);
    if (newWidth !== width) {
      x = x + (width - newWidth);
      width = newWidth;
    }
  } else if (handle.includes("e")) {
    width = Math.max(MIN_SIZE, width + delta.x);
  }

  if (handle.includes("n")) {
    const newHeight = Math.max(MIN_SIZE, height - delta.y);
    if (newHeight !== height) {
      y = y + (height - newHeight);
      height = newHeight;
    }
  } else if (handle.includes("s")) {
    height = Math.max(MIN_SIZE, height + delta.y);
  }

  return { x, y, width, height };
}

// ============================================================================
// Main Component
// ============================================================================

export const ResizeHandles = memo(function ResizeHandles({
  frame,
  transform,
}: ResizeHandlesProps) {
  const dispatch = useDispatch<DesignerEvent>();
  const drag = DesignerMachine.useSelector((s) => s.tool.drag);

  // Calculate screen bounds for the selection box
  const screenBounds = useMemo(() => {
    const topLeft = canvasToScreen({ x: frame.bounds.x, y: frame.bounds.y }, transform);
    const bottomRight = canvasToScreen(
      { x: frame.bounds.x + frame.bounds.width, y: frame.bounds.y + frame.bounds.height },
      transform
    );
    return {
      left: topLeft.x,
      top: topLeft.y,
      width: bottomRight.x - topLeft.x,
      height: bottomRight.y - topLeft.y,
    };
  }, [frame.bounds, transform]);

  // Calculate preview bounds during drag
  const previewBounds = useMemo(() => {
    if (!drag) return null;

    const startCanvas = screenToCanvas(drag.startMouse, transform);
    const currentCanvas = screenToCanvas(drag.currentMouse, transform);
    const delta = {
      x: currentCanvas.x - startCanvas.x,
      y: currentCanvas.y - startCanvas.y,
    };

    return calculatePreviewBounds(drag.startBounds, drag.handle, delta);
  }, [drag, transform]);

  // Drag hook
  const { startDrag } = useDrag({
    onDragMove: (event, info) => {
      dispatch({
        type: "tool/resizeMove",
        payload: info.currentMouse,
      });
    },
    onDragEnd: () => {
      dispatch({ type: "tool/resizeEnd" });
    },
  });

  const handleMouseDown = useCallback(
    (e: React.MouseEvent, handle: ResizeHandle) => {
      console.log("[ResizeHandles] mousedown on handle:", handle, "at", e.clientX, e.clientY);
      e.stopPropagation();
      dispatch({
        type: "tool/resizeStart",
        payload: {
          handle,
          mouse: { x: e.clientX, y: e.clientY },
        },
      });
      startDrag(e);
    },
    [dispatch, startDrag]
  );

  // Calculate preview screen bounds if dragging
  const previewScreenBounds = useMemo(() => {
    if (!previewBounds) return null;
    const topLeft = canvasToScreen({ x: previewBounds.x, y: previewBounds.y }, transform);
    const bottomRight = canvasToScreen(
      { x: previewBounds.x + previewBounds.width, y: previewBounds.y + previewBounds.height },
      transform
    );
    return {
      left: topLeft.x,
      top: topLeft.y,
      width: bottomRight.x - topLeft.x,
      height: bottomRight.y - topLeft.y,
    };
  }, [previewBounds, transform]);

  return (
    <>
      {/* Selection box outline */}
      <div
        style={{
          position: "absolute",
          left: screenBounds.left,
          top: screenBounds.top,
          width: screenBounds.width,
          height: screenBounds.height,
          boxShadow: "inset 0 0 0 1px #0066ff",
          pointerEvents: "none",
        }}
      />

      {/* Ghost preview during drag */}
      {previewScreenBounds && (
        <div
          style={{
            position: "absolute",
            left: previewScreenBounds.left,
            top: previewScreenBounds.top,
            width: previewScreenBounds.width,
            height: previewScreenBounds.height,
            border: "2px dashed #0066ff",
            backgroundColor: "rgba(0, 102, 255, 0.1)",
            pointerEvents: "none",
          }}
        />
      )}

      {/* Resize handles */}
      {HANDLES.map((handle) => {
        const pos = getHandleScreenPosition(handle, frame.bounds, transform);
        return (
          <div
            key={handle}
            onMouseDown={(e) => handleMouseDown(e, handle)}
            style={{
              position: "absolute",
              left: pos.x - HANDLE_HALF,
              top: pos.y - HANDLE_HALF,
              width: HANDLE_SIZE,
              height: HANDLE_SIZE,
              backgroundColor: "#ffffff",
              border: "1px solid #0066ff",
              cursor: CURSOR_MAP[handle],
              zIndex: 10,
            }}
          />
        );
      })}
    </>
  );
});
