"use client";

import React, { useCallback, useEffect, useRef } from "react";
import { useDispatch } from "@paperclip/common";
import { DesignerMachine, DesignerEvent, Transform } from "../machine";
import { Frames } from "./Frames";
import { ToolOverlay } from "./ToolOverlay";

// ============================================================================
// Component
// ============================================================================

export interface CanvasProps {
  className?: string;
  style?: React.CSSProperties;
}

export function Canvas({ className, style }: CanvasProps) {
  const { ref, transform, onMouseMove } = useCanvas();

  const innerStyle: React.CSSProperties = {
    transform: `translateX(${transform.x}px) translateY(${transform.y}px) scale(${transform.z}) translateZ(0)`,
    transformOrigin: "top left",
    position: "absolute",
    top: 0,
    left: 0,
    willChange: "transform",
  };

  return (
    <div
      ref={ref}
      className={className}
      style={{
        position: "relative",
        overflow: "hidden",
        width: "100%",
        height: "100%",
        backgroundColor: "#1a1a1a",
        ...style,
      }}
      onMouseMove={onMouseMove}
    >
      <div style={innerStyle}>
        <Frames />
      </div>

      <ToolOverlay />
    </div>
  );
}

// ============================================================================
// Hook
// ============================================================================

function normalizeWheel(event: WheelEvent): { pixelX: number; pixelY: number } {
  let pixelX = event.deltaX;
  let pixelY = event.deltaY;

  if (event.deltaMode === 1) {
    pixelX *= 40;
    pixelY *= 40;
  } else if (event.deltaMode === 2) {
    pixelX *= 800;
    pixelY *= 800;
  }

  return { pixelX, pixelY };
}

interface UseCanvasResult {
  ref: React.RefObject<HTMLDivElement>;
  transform: Transform;
  onMouseMove: (event: React.MouseEvent) => void;
}

function useCanvas(): UseCanvasResult {
  const dispatch = useDispatch<DesignerEvent>();
  const ref = useRef<HTMLDivElement>(null);
  const transform = DesignerMachine.useSelector((s) => s.canvas.transform);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        dispatch({
          type: "canvas/resized",
          payload: {
            width: entry.contentRect.width,
            height: entry.contentRect.height,
          },
        });
      }
    });

    observer.observe(canvas);
    return () => observer.disconnect();
  }, [dispatch]);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;

    const handleWheel = (event: WheelEvent) => {
      event.preventDefault();

      const { pixelX, pixelY } = normalizeWheel(event);
      const rect = canvas.getBoundingClientRect();
      const mouseX = event.clientX - rect.left;
      const mouseY = event.clientY - rect.top;

      dispatch({
        type: "canvas/mouseMove",
        payload: { x: mouseX, y: mouseY },
      });

      dispatch({
        type: "canvas/panned",
        payload: {
          delta: { x: pixelX, y: pixelY },
          metaKey: event.metaKey,
          ctrlKey: event.ctrlKey,
        },
      });
    };

    canvas.addEventListener("wheel", handleWheel, { passive: false });
    return () => canvas.removeEventListener("wheel", handleWheel);
  }, [dispatch]);

  const onMouseMove = useCallback(
    (event: React.MouseEvent) => {
      const rect = ref.current?.getBoundingClientRect();
      if (!rect) return;

      dispatch({
        type: "canvas/mouseMove",
        payload: {
          x: event.clientX - rect.left,
          y: event.clientY - rect.top,
        },
      });
    },
    [dispatch]
  );

  return { ref, transform, onMouseMove };
}
