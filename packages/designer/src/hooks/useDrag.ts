/**
 * Drag interaction hook
 *
 * Based on the original paperclip's startDOMDrag pattern with:
 * - _started flag to prevent false triggers on double-click
 * - Throttled mouse move for performance
 * - Delta tracking from start position
 */

import { useCallback, useRef } from "react";
import { Point } from "../machine/state";

export interface DragInfo {
  delta: Point;
  startMouse: Point;
  currentMouse: Point;
}

export interface UseDragOptions {
  onDragStart?: (event: MouseEvent, info: DragInfo) => void;
  onDragMove?: (event: MouseEvent, info: DragInfo) => void;
  onDragEnd?: (event: MouseEvent, info: DragInfo) => void;
  throttleMs?: number;
}

export function useDrag(options: UseDragOptions) {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const startDrag = useCallback((startEvent: React.MouseEvent) => {
    const sx = startEvent.clientX;
    const sy = startEvent.clientY;
    const doc = (startEvent.target as HTMLElement).ownerDocument;

    let _started = false;
    let lastMoveTime = 0;
    const throttleMs = optionsRef.current.throttleMs ?? 10;

    const createInfo = (event: MouseEvent): DragInfo => ({
      delta: {
        x: event.clientX - sx,
        y: event.clientY - sy,
      },
      startMouse: { x: sx, y: sy },
      currentMouse: { x: event.clientX, y: event.clientY },
    });

    const onMouseMove = (event: MouseEvent) => {
      const now = Date.now();
      if (now - lastMoveTime < throttleMs) return;
      lastMoveTime = now;

      event.preventDefault();

      if (!_started) {
        _started = true;
        optionsRef.current.onDragStart?.(event, createInfo(event));
      }

      optionsRef.current.onDragMove?.(event, createInfo(event));
    };

    const onMouseUp = (event: MouseEvent) => {
      doc.removeEventListener("mousemove", onMouseMove);
      doc.removeEventListener("mouseup", onMouseUp);

      if (_started) {
        optionsRef.current.onDragEnd?.(event, createInfo(event));
      }
    };

    doc.addEventListener("mousemove", onMouseMove);
    doc.addEventListener("mouseup", onMouseUp);
  }, []);

  return { startDrag };
}
