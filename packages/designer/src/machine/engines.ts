/**
 * Designer engines - side effects
 *
 * Uses proto types directly from @paperclip/proto with simple oneof checking.
 * Pattern follows the old codebase: direct truthiness checks on oneof fields.
 */

import { Engine, EngineFactory, MachineHandle, PropsRef } from "@paperclip/common";
import {
  VNode,
  VDocument,
  VDocPatch,
} from "@paperclip/proto";
import {
  DesignerEvent,
  DesignerState,
  Frame,
} from "./state";

// ============================================================================
// SSE Engine
// ============================================================================

export interface SSEEngineProps {
  filePath: string;
  serverUrl?: string;
}

// SSE response format (from Rust server)
interface PreviewUpdate {
  file_path: string;
  patches: VDocPatch[];
  error?: string;
  timestamp: number;
  version: number;
}

// Get node at path in proto VDocument
function getNodeAtPath(doc: VDocument, path: number[]): VNode | null {
  if (path.length === 0) return null;

  let current: VNode | undefined = doc.nodes[path[0]];
  for (let i = 1; i < path.length && current; i++) {
    // Oneof check: if it's an element, traverse children
    if (current.element) {
      current = current.element.children[path[i]];
    } else {
      return null;
    }
  }
  return current || null;
}

// Apply a patch to the document, returns new document (immutable update)
function applyPatch(doc: VDocument, patch: VDocPatch): VDocument {
  // Deep clone for immutability
  const newDoc: VDocument = JSON.parse(JSON.stringify(doc));

  // Simple oneof checking - no getPatchType() function!
  if (patch.replaceNode) {
    const { path, newNode } = patch.replaceNode;
    if (newNode && path.length === 1) {
      newDoc.nodes[path[0]] = newNode;
    } else if (newNode && path.length > 1) {
      const parentPath = path.slice(0, -1);
      const index = path[path.length - 1];
      const parent = getNodeAtPath(newDoc, parentPath);
      if (parent?.element) {
        parent.element.children[index] = newNode;
      }
    }
  } else if (patch.updateText) {
    const { path, content } = patch.updateText;
    const node = getNodeAtPath(newDoc, path);
    if (node?.text) {
      node.text.content = content;
    }
  } else if (patch.updateStyles) {
    const { path, styles } = patch.updateStyles;
    const node = getNodeAtPath(newDoc, path);
    if (node?.element) {
      node.element.styles = styles;
    }
  } else if (patch.updateAttributes) {
    const { path, attributes } = patch.updateAttributes;
    const node = getNodeAtPath(newDoc, path);
    if (node?.element) {
      node.element.attributes = attributes;
    }
  } else if (patch.createNode) {
    const { path, node, index } = patch.createNode;
    if (node && path.length === 0) {
      newDoc.nodes.splice(index, 0, node);
    } else if (node) {
      const parent = getNodeAtPath(newDoc, path);
      if (parent?.element) {
        parent.element.children.splice(index, 0, node);
      }
    }
  } else if (patch.removeNode) {
    const { path } = patch.removeNode;
    if (path.length === 1) {
      newDoc.nodes.splice(path[0], 1);
    } else if (path.length > 1) {
      const parentPath = path.slice(0, -1);
      const index = path[path.length - 1];
      const parent = getNodeAtPath(newDoc, parentPath);
      if (parent?.element) {
        parent.element.children.splice(index, 1);
      }
    }
  }

  return newDoc;
}

function extractFramesFromDocument(doc: VDocument): Frame[] {
  return doc.nodes.map((node, index) => {
    // Oneof check: if it's an element, extract frame data
    if (node.element) {
      const attrs = node.element.attributes;
      return {
        id: node.element.semanticId ?? `frame-${index}`,
        bounds: {
          x: parseFloat(attrs["data-frame-x"] ?? "0") || index * 1100,
          y: parseFloat(attrs["data-frame-y"] ?? "0") || 0,
          width: parseFloat(attrs["data-frame-width"] ?? "1024") || 1024,
          height: parseFloat(attrs["data-frame-height"] ?? "768") || 768,
        },
      };
    }

    return {
      id: `frame-${index}`,
      bounds: {
        x: index * 1100,
        y: 0,
        width: 1024,
        height: 768,
      },
    };
  });
}

export const createSSEEngine: EngineFactory<DesignerEvent, DesignerState, SSEEngineProps> = (
  propsRef: PropsRef<SSEEngineProps>,
  machine: MachineHandle<DesignerEvent, DesignerState>
): Engine<DesignerEvent, DesignerState, SSEEngineProps> => {
  let eventSource: EventSource | null = null;
  let vdom: VDocument | null = null;

  function connect(props: SSEEngineProps) {
    const { filePath, serverUrl = "" } = props;
    const sseUrl = `${serverUrl}/api/preview?file=${encodeURIComponent(filePath)}`;

    eventSource = new EventSource(sseUrl);

    eventSource.onopen = () => {
      console.log("[SSE] Connection opened");
    };

    eventSource.onmessage = (event) => {
      try {
        console.log("[SSE] Received message");
        const update: PreviewUpdate = JSON.parse(event.data);
        console.log("[SSE] Parsed update, patches:", update.patches?.length);

        if (update.error) {
          console.error("[SSE] Server error:", update.error);
          return;
        }

        let changed = false;
        for (const patch of update.patches) {
          console.log("[SSE] Processing patch:", Object.keys(patch).filter(k => (patch as any)[k] !== undefined));

          // Simple oneof checking - check which field is set
          if (patch.initialize?.vdom) {
            // Initialize patch - use vdom directly (no transformation!)
            vdom = patch.initialize.vdom;
            console.log("[SSE] Initialized VDOM, nodes:", vdom.nodes.length);
            changed = true;
          } else if (vdom) {
            // Apply incremental patch
            vdom = applyPatch(vdom, patch);
            console.log("[SSE] Applied incremental patch");
            changed = true;
          }
        }

        if (vdom && changed) {
          const frames = extractFramesFromDocument(vdom);
          console.log("[SSE] Extracted frames:", frames);
          machine.dispatch({
            type: "document/loaded",
            payload: { document: vdom, frames },
          });
        }
      } catch (err) {
        console.error("[SSE] Failed to parse message:", err);
      }
    };

    eventSource.onerror = (err) => {
      console.error("[SSE] Connection error:", err);
    };
  }

  function disconnect() {
    if (eventSource) {
      eventSource.close();
      eventSource = null;
    }
  }

  return {
    start() {
      console.log("[SSE Engine] start() called, props:", propsRef.current);
      // Connect on start if props are available
      if (propsRef.current?.filePath) {
        connect(propsRef.current);
      }
    },

    handleEvent(_event, _prevState) {
      // No events to handle currently
    },

    handlePropsChange(prevProps: SSEEngineProps, nextProps: SSEEngineProps) {
      // Reconnect if props changed (but not on initial mount - start() handles that)
      if (
        prevProps !== nextProps &&
        (prevProps?.filePath !== nextProps?.filePath ||
          prevProps?.serverUrl !== nextProps?.serverUrl)
      ) {
        disconnect();
        if (nextProps?.filePath) {
          connect(nextProps);
        }
      }
    },

    dispose() {
      disconnect();
    },
  };
};
