"use client";

import React, { memo, useCallback, useRef, useState, useEffect } from "react";
import { useDispatch } from "@paperclip/common";
import { DesignerMachine, DesignerEvent, Frame, Transform, Point } from "../machine";
import { screenToCanvas, canvasToScreen } from "../machine/geometry";
import { ResizeHandles } from "./ResizeHandles";

// ============================================================================
// Component
// ============================================================================

interface DragState {
  frameIndex: number;
  startMouse: Point;
  startPosition: Point;
}

export const ToolOverlay = memo(function ToolOverlay() {
  const dispatch = useDispatch<DesignerEvent>();
  const ref = useRef<HTMLDivElement>(null);
  const [dragState, setDragState] = useState<DragState | null>(null);

  const selectedFrameIndex = DesignerMachine.useSelector((s) => s.selectedFrameIndex);
  const frames = DesignerMachine.useSelector((s) => s.frames);
  const transform = DesignerMachine.useSelector((s) => s.canvas.transform);

  // Handle drag move and end globally
  useEffect(() => {
    if (!dragState) {
      console.log("[ToolOverlay] no dragState, skipping drag handlers");
      return;
    }

    console.log("[ToolOverlay] setting up drag handlers for frame", dragState.frameIndex);

    const handleMouseMove = (e: MouseEvent) => {
      const rect = ref.current?.getBoundingClientRect();
      if (!rect) return;

      const screenPos = { x: e.clientX - rect.left, y: e.clientY - rect.top };
      const canvasPos = screenToCanvas(screenPos, transform);
      const startCanvasPos = screenToCanvas(dragState.startMouse, transform);

      const deltaX = canvasPos.x - startCanvasPos.x;
      const deltaY = canvasPos.y - startCanvasPos.y;

      const newPosition = {
        x: dragState.startPosition.x + deltaX,
        y: dragState.startPosition.y + deltaY,
      };

      console.log("[ToolOverlay] mousemove delta:", { deltaX, deltaY }, "newPosition:", newPosition);

      dispatch({
        type: "frame/moved",
        payload: { index: dragState.frameIndex, position: newPosition },
      });
    };

    const handleMouseUp = () => {
      console.log("[ToolOverlay] mouseup, ending drag for frame", dragState.frameIndex);
      // Trigger mutation to save the new position
      dispatch({ type: "frame/moveEnd", payload: { index: dragState.frameIndex } });
      setDragState(null);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [dragState, dispatch, frames, transform]);

  // Handle mousedown to start drag or select
  const onMouseDown = useCallback(
    (event: React.MouseEvent) => {
      console.log("[ToolOverlay] mousedown", event.clientX, event.clientY);
      const rect = ref.current?.getBoundingClientRect();
      if (!rect) {
        console.log("[ToolOverlay] no rect");
        return;
      }

      const screenPos = {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
      };
      const canvasPos = screenToCanvas(screenPos, transform);
      console.log("[ToolOverlay] screenPos:", screenPos, "canvasPos:", canvasPos, "transform:", transform);
      console.log("[ToolOverlay] frames:", frames.map(f => ({ id: f.id, bounds: f.bounds })));

      // Find which frame was clicked
      const clickedFrameIndex = frames.findIndex((frame) => {
        const { x, y, width, height } = frame.bounds;
        const hit = canvasPos.x >= x &&
          canvasPos.x <= x + width &&
          canvasPos.y >= y &&
          canvasPos.y <= y + height;
        console.log("[ToolOverlay] checking frame", frame.id, "bounds:", { x, y, width, height }, "hit:", hit);
        return hit;
      });

      console.log("[ToolOverlay] clickedFrameIndex:", clickedFrameIndex);

      if (clickedFrameIndex >= 0) {
        event.preventDefault();

        // Select the frame
        dispatch({ type: "frame/selected", payload: { index: clickedFrameIndex } });

        // Start dragging
        const frame = frames[clickedFrameIndex];
        console.log("[ToolOverlay] starting drag for frame", frame.id, "at", frame.bounds);
        setDragState({
          frameIndex: clickedFrameIndex,
          startMouse: screenPos,
          startPosition: { x: frame.bounds.x, y: frame.bounds.y },
        });
      }
    },
    [dispatch, frames, transform]
  );

  const selectedFrame = selectedFrameIndex !== undefined ? frames[selectedFrameIndex] : undefined;

  // Calculate screen position of frames for debug overlay
  const frameScreenBounds = frames.map(frame => {
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
  });

  return (
    <div
      ref={ref}
      onMouseDown={onMouseDown}
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        background: "transparent",
        zIndex: 1000,
        cursor: dragState ? "grabbing" : "default",
      }}
    >
      {/* Debug: show frame hit areas */}
      {frameScreenBounds.map((bounds, i) => (
        <div
          key={i}
          style={{
            position: "absolute",
            left: bounds.left,
            top: bounds.top,
            width: bounds.width,
            height: bounds.height,
            border: "2px dashed rgba(255,0,0,0.5)",
            pointerEvents: "none",
          }}
        />
      ))}

      {/* Debug: show frame count */}
      <div style={{
        position: "absolute",
        top: 10,
        right: 10,
        background: frames.length > 0 ? "#00ff00" : "#ff0000",
        color: "#000",
        padding: "4px 8px",
        fontFamily: "monospace",
        fontSize: 12,
        zIndex: 99999,
        pointerEvents: "none",
      }}>
        Frames: {frames.length} | Selected: {selectedFrameIndex ?? "none"} | Drag: {dragState ? "yes" : "no"}
      </div>

      {selectedFrame && !dragState && (
        <ResizeHandles frame={selectedFrame} transform={transform} />
      )}
    </div>
  );
});
