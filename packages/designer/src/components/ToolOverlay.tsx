"use client";

import React, { memo, useCallback, useRef } from "react";
import { useDispatch } from "@paperclip/common";
import { DesignerMachine, DesignerEvent, Frame, Transform } from "../machine";
import { screenToCanvas } from "../machine/geometry";
import { ResizeHandles } from "./ResizeHandles";

// ============================================================================
// Component
// ============================================================================

export const ToolOverlay = memo(function ToolOverlay() {
  const dispatch = useDispatch<DesignerEvent>();
  const ref = useRef<HTMLDivElement>(null);

  const selectedFrameIndex = DesignerMachine.useSelector((s) => s.selectedFrameIndex);
  const frames = DesignerMachine.useSelector((s) => s.frames);
  const transform = DesignerMachine.useSelector((s) => s.canvas.transform);

  // Handle click to select frames
  const onClick = useCallback(
    (event: React.MouseEvent) => {
      const rect = ref.current?.getBoundingClientRect();
      if (!rect) return;

      // Get mouse position in screen space (relative to canvas container)
      const screenPos = {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
      };

      // Convert to canvas space
      const canvasPos = screenToCanvas(screenPos, transform);

      // Find which frame was clicked
      const clickedFrameIndex = frames.findIndex((frame) => {
        const { x, y, width, height } = frame.bounds;
        return (
          canvasPos.x >= x &&
          canvasPos.x <= x + width &&
          canvasPos.y >= y &&
          canvasPos.y <= y + height
        );
      });

      if (clickedFrameIndex >= 0) {
        dispatch({ type: "frame/selected", payload: { index: clickedFrameIndex } });
      }
    },
    [dispatch, frames, transform]
  );

  const selectedFrame = selectedFrameIndex !== undefined ? frames[selectedFrameIndex] : undefined;

  console.log("[ToolOverlay] frames:", frames.length, "selectedFrameIndex:", selectedFrameIndex, "selectedFrame:", selectedFrame);

  return (
    <div
      ref={ref}
      onClick={onClick}
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        background: "transparent",
        zIndex: 1000,
      }}
    >
      {/* Debug: show frame count */}
      <div style={{
        position: "fixed",
        top: 10,
        right: 10,
        background: frames.length > 0 ? "#00ff00" : "#ff0000",
        color: "#000",
        padding: "4px 8px",
        fontFamily: "monospace",
        fontSize: 12,
        zIndex: 99999,
      }}>
        Frames: {frames.length} | Selected: {selectedFrameIndex ?? "none"}
      </div>

      {selectedFrame && (
        <ResizeHandles frame={selectedFrame} transform={transform} />
      )}
    </div>
  );
});
