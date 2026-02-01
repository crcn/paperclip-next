"use client";

import React from "react";
import { DesignerMachine, Frame as FrameType, VDocument } from "../machine";
import { Frame } from "./Frame";

// ============================================================================
// Component
// ============================================================================

export function Frames() {
  const { frames, document, selectedFrameIndex } = useFrames();

  console.log("[Frames] Rendering - frames:", frames.length, "document:", document ? "yes" : "no");

  if (frames.length === 0) {
    console.log("[Frames] No frames to render");
  }

  return (
    <>
      {frames.map((frame, index) => {
        const node = document?.nodes[index];
        // Proto oneof: check which field is set
        const nodeType = node?.element ? "element" : node?.text ? "text" : node?.comment ? "comment" : "unknown";
        console.log("[Frames] Mapping frame", index, "node:", nodeType);
        return (
          <Frame
            key={frame.id}
            frame={frame}
            index={index}
            node={node}
            isSelected={selectedFrameIndex === index}
          />
        );
      })}
    </>
  );
}

// ============================================================================
// Hook
// ============================================================================

interface UseFramesResult {
  frames: FrameType[];
  document: VDocument | undefined;
  selectedFrameIndex: number | undefined;
}

function useFrames(): UseFramesResult {
  const frames = DesignerMachine.useSelector((s) => s.frames);
  const document = DesignerMachine.useSelector((s) => s.document);
  const selectedFrameIndex = DesignerMachine.useSelector((s) => s.selectedFrameIndex);

  console.log("[Frames] useFrames hook - frames:", frames.length, "document nodes:", document?.nodes?.length);

  return { frames, document, selectedFrameIndex };
}
