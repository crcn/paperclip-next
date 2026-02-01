"use client";

import React, { memo, useCallback, useEffect, useRef } from "react";
import { useDispatch } from "@paperclip/common";
import { VNode } from "@paperclip/proto";
import { DesignerEvent, Frame as FrameType } from "../machine";

// ============================================================================
// Component
// ============================================================================

export interface FrameProps {
  frame: FrameType;
  index: number;
  node?: VNode;
  isSelected?: boolean;
}

export const Frame = memo(function Frame({
  frame,
  index,
  node,
  isSelected,
}: FrameProps) {
  const { iframeRef, onClick } = useFrame({ index, node });
  const { bounds } = frame;

  return (
    <div
      onClick={onClick}
      style={{
        position: "absolute",
        left: bounds.x,
        top: bounds.y,
        width: bounds.width,
        height: bounds.height,
        backgroundColor: "#ffffff",
        boxShadow: isSelected
          ? "0 0 0 2px #0066ff, 0 4px 12px rgba(0,0,0,0.15)"
          : "0 2px 8px rgba(0,0,0,0.1)",
        borderRadius: 4,
        overflow: "hidden",
        cursor: "pointer",
      }}
    >
      <div
        style={{
          position: "absolute",
          top: -24,
          left: 0,
          fontSize: 12,
          color: isSelected ? "#0066ff" : "#666",
          fontFamily: "system-ui, sans-serif",
          whiteSpace: "nowrap",
        }}
      >
        Frame {index + 1}
      </div>

      <iframe
        ref={iframeRef}
        style={{
          width: "100%",
          height: "100%",
          border: "none",
          pointerEvents: "none",
        }}
        title={`Frame ${index + 1}`}
      />
    </div>
  );
});

// ============================================================================
// Hook
// ============================================================================

interface UseFrameOptions {
  index: number;
  node?: VNode;
}

interface UseFrameResult {
  iframeRef: React.RefObject<HTMLIFrameElement>;
  onClick: () => void;
}

function useFrame({ index, node }: UseFrameOptions): UseFrameResult {
  const dispatch = useDispatch<DesignerEvent>();
  const iframeRef = useRef<HTMLIFrameElement>(null);

  console.log("[Frame] useFrame hook - index:", index, "node:", node);

  useEffect(() => {
    console.log("[Frame] useEffect running - index:", index, "node:", node);
    const iframe = iframeRef.current;
    if (!iframe || !node) {
      console.log("[Frame] Early return - iframe:", !!iframe, "node:", !!node);
      return;
    }

    const doc = iframe.contentDocument;
    if (!doc) {
      console.log("[Frame] No contentDocument");
      return;
    }

    doc.open();
    doc.write(`
      <!DOCTYPE html>
      <html>
        <head>
          <style>
            html, body {
              margin: 0;
              padding: 0;
              width: 100%;
              height: 100%;
              overflow: hidden;
            }
          </style>
        </head>
        <body></body>
      </html>
    `);
    doc.close();

    const rendered = renderNode(node, doc);
    console.log("[Frame] Rendered node:", rendered?.nodeName, "children:", rendered?.childNodes?.length);
    if (rendered) {
      doc.body.appendChild(rendered);
      console.log("[Frame] Appended to body, innerHTML:", doc.body.innerHTML.substring(0, 200));
    }
  }, [node, index]);

  const onClick = useCallback(() => {
    dispatch({ type: "frame/selected", payload: { index } });
  }, [dispatch, index]);

  return { iframeRef, onClick };
}

// ============================================================================
// Utilities - Proto oneof handling (like old codebase)
// ============================================================================

/**
 * Render a proto VNode to native DOM.
 * Uses simple oneof checking - just check which field is truthy.
 */
function renderNode(node: VNode, doc: Document): Node | null {
  // Proto oneof: check each field for truthiness
  if (node.element) {
    const el = doc.createElement(node.element.tag);

    for (const [key, value] of Object.entries(node.element.attributes)) {
      el.setAttribute(key, value);
    }

    for (const [key, value] of Object.entries(node.element.styles)) {
      (el.style as any)[key] = value;
    }

    for (const child of node.element.children) {
      const childEl = renderNode(child, doc);
      if (childEl) {
        el.appendChild(childEl);
      }
    }

    return el;
  }

  if (node.text) {
    return doc.createTextNode(node.text.content);
  }

  if (node.comment) {
    return doc.createComment(node.comment.content);
  }

  if (node.error) {
    // Render error as a visible element
    const errorEl = doc.createElement("span");
    errorEl.style.color = "red";
    errorEl.style.fontWeight = "bold";
    errorEl.style.background = "#fee";
    errorEl.style.padding = "2px 4px";
    errorEl.style.borderRadius = "2px";
    errorEl.style.border = "1px solid red";
    errorEl.textContent = `⚠ ${node.error.message}`;
    return errorEl;
  }

  if (node.component) {
    // For now, render component as a placeholder div
    const el = doc.createElement("div");
    el.setAttribute("data-component-id", node.component.componentId);
    for (const child of node.component.children) {
      const childEl = renderNode(child, doc);
      if (childEl) {
        el.appendChild(childEl);
      }
    }
    return el;
  }

  return null;
}
