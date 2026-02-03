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
  FrameBounds,
  PendingMutation,
  Point,
  ResizeHandle,
} from "./state";
import { screenToCanvas } from "./geometry";

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
  patches: RawPatch[];
  error?: string;
  timestamp: number;
  version: number;
}

// Raw patch from server (prost serde format)
interface RawPatch {
  patch_type?: Record<string, unknown>;
  patchType?: string;
  initialize?: unknown;
  replaceNode?: unknown;
  addNode?: unknown;
  removeNode?: unknown;
  setAttribute?: unknown;
  setStyle?: unknown;
  [key: string]: unknown;  // Allow additional properties
}

// Raw node from server (prost serde format)
// Can be either:
// 1. Prost serde format: { node_type: { Element: {...} } }
// 2. Proto JSON format: { element: {...} } or { text: {...} } etc.
interface RawNode {
  node_type?: Record<string, unknown>;
  element?: Record<string, unknown>;
  text?: Record<string, unknown>;
  comment?: Record<string, unknown>;
  error?: Record<string, unknown>;
  component?: Record<string, unknown>;
  [key: string]: unknown;  // Allow additional properties
}

// Raw document from server
interface RawDocument {
  nodes?: RawNode[];
  styles?: Array<{ selector: string; properties: Record<string, string> }>;
}

// ============================================================================
// Transform Functions: Prost Serde Format -> Proto Canonical JSON
// ============================================================================

/**
 * Convert snake_case keys to camelCase
 */
function snakeToCamel(str: string): string {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

/**
 * Convert all keys in an object from snake_case to camelCase (recursive for nested objects)
 */
function convertKeysToCamelCase(obj: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(obj)) {
    const camelKey = snakeToCamel(key);
    if (value && typeof value === "object" && !Array.isArray(value)) {
      result[camelKey] = convertKeysToCamelCase(value as Record<string, unknown>);
    } else {
      result[camelKey] = value;
    }
  }
  return result;
}

/**
 * Transform a raw node from prost serde format to proto format.
 *
 * Two formats are possible:
 * 1. Direct format (already correct): { element: { tag: "div", ... } }
 * 2. Prost serde format: { node_type: { Element: { tag: "div", ... } } }
 *
 * Also converts snake_case field names to camelCase (semantic_id -> semanticId)
 */
function transformNode(raw: RawNode): VNode {
  // Format 1: Already in proto format (direct field names)
  if ("element" in raw && raw.element) {
    const elemData = convertKeysToCamelCase(raw.element as Record<string, unknown>);
    if (elemData.children && Array.isArray(elemData.children)) {
      elemData.children = (elemData.children as unknown[]).map((c) => transformNode(c as RawNode));
    }
    return { element: elemData } as unknown as VNode;
  }
  if ("text" in raw && raw.text) {
    return { text: convertKeysToCamelCase(raw.text as Record<string, unknown>) } as unknown as VNode;
  }
  if ("comment" in raw && raw.comment) {
    return { comment: convertKeysToCamelCase(raw.comment as Record<string, unknown>) } as unknown as VNode;
  }
  if ("error" in raw && raw.error) {
    return { error: convertKeysToCamelCase(raw.error as Record<string, unknown>) } as unknown as VNode;
  }
  if ("component" in raw && raw.component) {
    const compData = convertKeysToCamelCase(raw.component as Record<string, unknown>);
    if (compData.children && Array.isArray(compData.children)) {
      compData.children = (compData.children as unknown[]).map((c) => transformNode(c as RawNode));
    }
    return { component: compData } as unknown as VNode;
  }

  // Format 2: Prost serde format with node_type wrapper
  if (raw.node_type) {
    const keys = Object.keys(raw.node_type);
    if (keys.length === 0) {
      console.warn("[transformNode] Empty node_type:", raw);
      return {} as VNode;
    }

    const variantName = keys[0];
    const variantData = convertKeysToCamelCase(raw.node_type[variantName] as Record<string, unknown>);

    // Convert variant name to lowercase field name (Element -> element)
    const fieldName = variantName.charAt(0).toLowerCase() + variantName.slice(1);

    // Recursively transform children if present
    if (variantData.children && Array.isArray(variantData.children)) {
      variantData.children = (variantData.children as unknown[]).map((c) => transformNode(c as RawNode));
    }

    // Build the proto-format node
    const result: Record<string, unknown> = {};
    result[fieldName] = variantData;

    return result as unknown as VNode;
  }

  console.warn("[transformNode] Unknown format, keys:", Object.keys(raw));
  return {} as VNode;
}

/**
 * Transform a raw document from prost serde format to proto format.
 */
function transformDocument(raw: RawDocument): VDocument {
  return {
    nodes: (raw.nodes || []).map(transformNode),
    styles: (raw.styles || []).map(s => ({
      selector: s.selector,
      properties: s.properties || {},
    })),
    components: [],  // Component metadata is not included in raw server responses
  };
}

/**
 * Transform a raw patch from server format to proto format.
 *
 * Two formats are possible:
 * 1. Initial load (manual JSON): { initialize: { vdom: {...} } }
 * 2. Incremental (prost serde): { patch_type: { UpdateText: {...} } }
 */
function transformPatch(raw: RawPatch): VDocPatch {
  // Format 1: Direct field names (from process_file_to_json)
  // Check for direct patch fields first
  if ("initialize" in raw && raw.initialize) {
    const initData = raw.initialize as Record<string, unknown>;
    if (initData.vdom) {
      initData.vdom = transformDocument(initData.vdom as RawDocument);
    }
    return { initialize: initData } as VDocPatch;
  }

  // Format 2: patch_type wrapper (from prost serde)
  if (raw.patch_type) {
    const keys = Object.keys(raw.patch_type);
    if (keys.length === 0) {
      console.warn("[transformPatch] Empty patch_type:", raw);
      return {};
    }

    const variantName = keys[0];
    const variantData = raw.patch_type[variantName] as Record<string, unknown>;

    // Convert variant name to camelCase field name (UpdateText -> updateText)
    const fieldName = variantName.charAt(0).toLowerCase() + variantName.slice(1);

    // Transform nested vdom if this is an Initialize patch
    if (fieldName === "initialize" && variantData && "vdom" in variantData) {
      variantData.vdom = transformDocument(variantData.vdom as RawDocument);
    }

    // Transform nested node if present (ReplaceNode, CreateNode)
    if (variantData && "newNode" in variantData) {
      variantData.newNode = transformNode(variantData.newNode as RawNode);
    }
    if (variantData && "new_node" in variantData) {
      variantData.newNode = transformNode(variantData.new_node as RawNode);
      delete variantData.new_node;
    }
    if (variantData && "node" in variantData) {
      variantData.node = transformNode(variantData.node as RawNode);
    }

    const result: Record<string, unknown> = {};
    result[fieldName] = variantData;

    return result as VDocPatch;
  }

  console.warn("[transformPatch] Unknown format:", raw);
  return {};
}

// ============================================================================
// Patch Application
// ============================================================================

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

/**
 * Extract frame bounds from element metadata or fallback to data-frame-* attributes
 */
function extractFrameFromMetadata(
  metadata: { objectValue?: { fields?: Record<string, { objectValue?: { fields?: Record<string, { numberValue?: number }> } }> } } | undefined,
  attrs: Record<string, string>,
  defaultX: number
): FrameBounds {
  // Try to extract from metadata.objectValue.fields.frame.objectValue.fields
  // Proto Value format: { objectValue: { fields: { x: { numberValue: 100 }, ... } } }
  if (metadata?.objectValue?.fields) {
    const frame = metadata.objectValue.fields.frame?.objectValue?.fields;
    if (frame) {
      return {
        x: frame.x?.numberValue ?? defaultX,
        y: frame.y?.numberValue ?? 0,
        width: frame.width?.numberValue ?? 1024,
        height: frame.height?.numberValue ?? 768,
      };
    }
  }

  // Fallback to data-frame-* attributes for backwards compatibility
  return {
    x: parseFloat(attrs["data-frame-x"] ?? "0") || defaultX,
    y: parseFloat(attrs["data-frame-y"] ?? "0") || 0,
    width: parseFloat(attrs["data-frame-width"] ?? "1024") || 1024,
    height: parseFloat(attrs["data-frame-height"] ?? "768") || 768,
  };
}

function extractFramesFromDocument(doc: VDocument): Frame[] {
  return doc.nodes.map((node, index) => {
    // Oneof check: if it's an element, extract frame data
    if (node.element) {
      const attrs = node.element.attributes ?? {};
      const metadata = node.element.metadata as Record<string, unknown> | undefined;
      // Use sourceId for mutations (maps to AST span.id), fall back to semanticId
      const frameId = node.element.sourceId ?? node.element.semanticId ?? `frame-${index}`;
      return {
        id: frameId,
        bounds: extractFrameFromMetadata(metadata, attrs, index * 1100),
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

// ============================================================================
// Mutation API
// ============================================================================

const MIN_FRAME_SIZE = 50;

/**
 * Calculate new frame bounds based on resize handle drag
 */
function calculateResizedBounds(
  startBounds: FrameBounds,
  handle: ResizeHandle,
  delta: Point
): FrameBounds {
  let { x, y, width, height } = startBounds;

  // Handle horizontal resize
  if (handle.includes("w")) {
    const newWidth = Math.max(MIN_FRAME_SIZE, width - delta.x);
    if (newWidth !== width) {
      x = x + (width - newWidth);
      width = newWidth;
    }
  } else if (handle.includes("e")) {
    width = Math.max(MIN_FRAME_SIZE, width + delta.x);
  }

  // Handle vertical resize
  if (handle.includes("n")) {
    const newHeight = Math.max(MIN_FRAME_SIZE, height - delta.y);
    if (newHeight !== height) {
      y = y + (height - newHeight);
      height = newHeight;
    }
  } else if (handle.includes("s")) {
    height = Math.max(MIN_FRAME_SIZE, height + delta.y);
  }

  return {
    x: Math.round(x),
    y: Math.round(y),
    width: Math.round(width),
    height: Math.round(height),
  };
}

/**
 * Server mutation response
 */
interface MutationResponse {
  success: boolean;
  mutation_id: string;
  version: number;
  error?: string;
}

/**
 * Send a mutation to the server
 * Returns the server response with mutation_id and version
 */
async function sendMutation(
  serverUrl: string,
  filePath: string,
  mutation: { type: string; [key: string]: unknown }
): Promise<MutationResponse> {
  const url = `${serverUrl}/api/mutation`;

  try {
    const response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        file_path: filePath,
        mutation,
      }),
    });

    const result: MutationResponse = await response.json();

    if (!response.ok || !result.success) {
      console.error("[API] Mutation failed:", response.status, result.error);
      return result;
    }

    return result;
  } catch (err) {
    console.error("[API] Mutation error:", err);
    return {
      success: false,
      mutation_id: "",
      version: 0,
      error: err instanceof Error ? err.message : "Unknown error",
    };
  }
}

// ============================================================================
// SSE Engine
// ============================================================================

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
      // Connection established
    };

    eventSource.onmessage = (event) => {
      try {
        const update: PreviewUpdate = JSON.parse(event.data);

        if (update.error) {
          console.error("[SSE] Server error:", update.error);
          return;
        }

        let changed = false;
        for (const rawPatch of update.patches) {
          // Transform from prost serde format to proto format
          const patch = transformPatch(rawPatch);

          // Simple oneof checking - check which field is set
          if (patch.initialize?.vdom) {
            // Initialize patch - use vdom directly (already transformed!)
            vdom = patch.initialize.vdom;
            changed = true;
          } else if (vdom) {
            // Apply incremental patch
            vdom = applyPatch(vdom, patch);
            changed = true;
          }
        }

        if (vdom && changed) {
          const frames = extractFramesFromDocument(vdom);
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
      // Connect on start if props are available
      if (propsRef.current?.filePath) {
        connect(propsRef.current);
      }
    },

    handleEvent(event, prevState) {
      // Handle tool/resizeEnd - send mutation to server
      if (event.type === "tool/resizeEnd") {
        const drag = prevState.tool.drag;
        if (!drag) return;

        // Get frame info
        const frame = prevState.frames[drag.frameIndex];
        if (!frame) return;

        // Calculate the delta in canvas space (same logic as reducer)
        const { transform } = prevState.canvas;
        const startCanvas = screenToCanvas(drag.startMouse, transform);
        const endCanvas = screenToCanvas(drag.currentMouse, transform);
        const delta = {
          x: endCanvas.x - startCanvas.x,
          y: endCanvas.y - startCanvas.y,
        };

        // Calculate new bounds
        const newBounds = calculateResizedBounds(drag.startBounds, drag.handle, delta);

        // Generate client-side mutation ID for tracking
        const mutationId = `mut-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

        // Create pending mutation for optimistic update tracking
        const pendingMutation: PendingMutation = {
          mutationId,
          type: "setFrameBounds",
          frameId: frame.id,
          optimisticBounds: newBounds,
          createdAt: Date.now(),
        };

        // Dispatch mutation/started to track the pending mutation
        machine.dispatch({
          type: "mutation/started",
          payload: { mutation: pendingMutation },
        });

        // Send mutation to server
        const serverUrl = propsRef.current?.serverUrl || "";
        const filePath = propsRef.current?.filePath;

        if (filePath) {
          sendMutation(serverUrl, filePath, {
            type: "setFrameBounds",
            frame_id: frame.id,
            bounds: newBounds,
          }).then((response) => {
            if (response.success) {
              // Mutation acknowledged by server
              machine.dispatch({
                type: "mutation/acknowledged",
                payload: {
                  mutationId: pendingMutation.mutationId,
                  version: response.version,
                },
              });
            } else {
              // Mutation failed - dispatch failure to trigger revert
              machine.dispatch({
                type: "mutation/failed",
                payload: {
                  mutationId: pendingMutation.mutationId,
                  error: response.error || "Unknown error",
                },
              });
            }
          });
        } else {
          console.error("[API] No filePath available for mutation");
          // Immediately fail if no file path
          machine.dispatch({
            type: "mutation/failed",
            payload: {
              mutationId: pendingMutation.mutationId,
              error: "No file path available",
            },
          });
        }
      }

      // Handle frame/moveEnd - send mutation to server for frame move
      if (event.type === "frame/moveEnd") {
        const { index } = event.payload;
        const frame = prevState.frames[index];
        if (!frame) return;

        // Generate client-side mutation ID for tracking
        const mutationId = `mut-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

        // Create pending mutation for optimistic update tracking
        const pendingMutation: PendingMutation = {
          mutationId,
          type: "setFrameBounds",
          frameId: frame.id,
          optimisticBounds: frame.bounds,
          createdAt: Date.now(),
        };

        // Dispatch mutation/started to track the pending mutation
        machine.dispatch({
          type: "mutation/started",
          payload: { mutation: pendingMutation },
        });

        // Send mutation to server
        const serverUrl = propsRef.current?.serverUrl || "";
        const filePath = propsRef.current?.filePath;

        if (filePath) {
          sendMutation(serverUrl, filePath, {
            type: "setFrameBounds",
            frame_id: frame.id,
            bounds: frame.bounds,
          }).then((response) => {
            if (response.success) {
              machine.dispatch({
                type: "mutation/acknowledged",
                payload: {
                  mutationId: pendingMutation.mutationId,
                  version: response.version,
                },
              });
            } else {
              machine.dispatch({
                type: "mutation/failed",
                payload: {
                  mutationId: pendingMutation.mutationId,
                  error: response.error || "Unknown error",
                },
              });
            }
          });
        } else {
          console.error("[API] No filePath available for mutation");
          machine.dispatch({
            type: "mutation/failed",
            payload: {
              mutationId: pendingMutation.mutationId,
              error: "No file path available",
            },
          });
        }
      }

      // Handle style/changed - send SetInlineStyle mutation via postMessage to VSCode
      if (event.type === "style/changed") {
        const { property, value } = event.payload;
        const selectedElement = prevState.selectedElement;
        if (!selectedElement) {
          console.warn("[API] style/changed but no element selected");
          return;
        }

        const mutationId = `mut-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

        // Send mutation to VSCode extension via postMessage
        console.log("[API] Sending SetInlineStyle mutation via postMessage");
        window.parent.postMessage({
          type: "mutation",
          mutationId,
          mutationType: "SetInlineStyle",
          payload: {
            node_id: selectedElement.sourceId,
            property,
            value,
          },
        }, "*");
      }

      // Handle style/removed - send DeleteInlineStyle mutation via postMessage to VSCode
      if (event.type === "style/removed") {
        const { property } = event.payload;
        const selectedElement = prevState.selectedElement;
        if (!selectedElement) {
          console.warn("[API] style/removed but no element selected");
          return;
        }

        const mutationId = `mut-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

        // Send mutation to VSCode extension via postMessage
        console.log("[API] Sending DeleteInlineStyle mutation via postMessage");
        window.parent.postMessage({
          type: "mutation",
          mutationId,
          mutationType: "DeleteInlineStyle",
          payload: {
            node_id: selectedElement.sourceId,
            property,
          },
        }, "*");
      }
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
