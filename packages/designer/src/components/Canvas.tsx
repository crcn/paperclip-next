"use client";

import React, { useCallback, useEffect, useRef, useState } from "react";
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
  const { ref, transform, isPanning, isSpaceHeld, onMouseMove, onMouseDown } = useCanvas();

  const innerStyle: React.CSSProperties = {
    transform: `translateX(${transform.x}px) translateY(${transform.y}px) scale(${transform.z}) translateZ(0)`,
    transformOrigin: "top left",
    position: "absolute",
    top: 0,
    left: 0,
    willChange: "transform",
  };

  // Show grab cursor when space held, grabbing when actively panning
  const cursor = isPanning ? "grabbing" : isSpaceHeld ? "grab" : undefined;

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
        cursor,
        ...style,
      }}
      onMouseMove={onMouseMove}
      onMouseDown={onMouseDown}
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
  isPanning: boolean;
  isSpaceHeld: boolean;
  onMouseMove: (event: React.MouseEvent) => void;
  onMouseDown: (event: React.MouseEvent) => void;
}

function useCanvas(): UseCanvasResult {
  const dispatch = useDispatch<DesignerEvent>();
  const ref = useRef<HTMLDivElement>(null);
  const transform = DesignerMachine.useSelector((s) => s.canvas.transform);

  // Pan drag state
  const [isPanning, setIsPanning] = useState(false);
  const [isSpaceHeld, setIsSpaceHeld] = useState(false);
  const panStartRef = useRef<{ x: number; y: number } | null>(null);

  // Track space key for pan mode
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.code === "Space" && !e.repeat) {
        e.preventDefault();
        setIsSpaceHeld(true);
      }
    };
    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.code === "Space") {
        setIsSpaceHeld(false);
        setIsPanning(false);
        panStartRef.current = null;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, []);

  // Handle pan drag (space+drag or middle mouse)
  useEffect(() => {
    if (!isPanning) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!panStartRef.current) return;

      const deltaX = panStartRef.current.x - e.clientX;
      const deltaY = panStartRef.current.y - e.clientY;

      dispatch({
        type: "canvas/panned",
        payload: {
          delta: { x: deltaX, y: deltaY },
          metaKey: false,
          ctrlKey: false,
        },
      });

      panStartRef.current = { x: e.clientX, y: e.clientY };
    };

    const handleMouseUp = () => {
      setIsPanning(false);
      panStartRef.current = null;
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isPanning, dispatch]);

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

  const onMouseDown = useCallback(
    (event: React.MouseEvent) => {
      // Start pan on space+click or middle mouse button
      if (isSpaceHeld || event.button === 1) {
        event.preventDefault();
        setIsPanning(true);
        panStartRef.current = { x: event.clientX, y: event.clientY };
      }
    },
    [isSpaceHeld]
  );

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

  return { ref, transform, isPanning, isSpaceHeld, onMouseMove, onMouseDown };
}
