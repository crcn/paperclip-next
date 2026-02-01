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
  patches: RawPatch[];
  error?: string;
  timestamp: number;
  version: number;
}

// Raw patch from server (prost serde format)
interface RawPatch {
  patch_type?: Record<string, unknown>;
  patchType?: string;
}

// Raw node from server (prost serde format)
interface RawNode {
  node_type?: Record<string, unknown>;
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
function transformNode(raw: RawNode & Record<string, unknown>): VNode {
  console.log("[transformNode] Input keys:", Object.keys(raw));

  // Format 1: Already in proto format (direct field names)
  if ("element" in raw && raw.element) {
    console.log("[transformNode] Direct 'element' field found");
    const elemData = convertKeysToCamelCase(raw.element as Record<string, unknown>);
    if (elemData.children && Array.isArray(elemData.children)) {
      elemData.children = (elemData.children as unknown[]).map((c) => transformNode(c as RawNode));
    }
    return { element: elemData } as VNode;
  }
  if ("text" in raw && raw.text) {
    return { text: convertKeysToCamelCase(raw.text as Record<string, unknown>) } as VNode;
  }
  if ("comment" in raw && raw.comment) {
    return { comment: convertKeysToCamelCase(raw.comment as Record<string, unknown>) } as VNode;
  }
  if ("error" in raw && raw.error) {
    return { error: convertKeysToCamelCase(raw.error as Record<string, unknown>) } as VNode;
  }
  if ("component" in raw && raw.component) {
    const compData = convertKeysToCamelCase(raw.component as Record<string, unknown>);
    if (compData.children && Array.isArray(compData.children)) {
      compData.children = (compData.children as unknown[]).map((c) => transformNode(c as RawNode));
    }
    return { component: compData } as VNode;
  }

  // Format 2: Prost serde format with node_type wrapper
  if (raw.node_type) {
    const keys = Object.keys(raw.node_type);
    if (keys.length === 0) {
      console.warn("[transformNode] Empty node_type:", raw);
      return {};
    }

    const variantName = keys[0];
    const variantData = convertKeysToCamelCase(raw.node_type[variantName] as Record<string, unknown>);

    console.log("[transformNode] node_type variant:", variantName);

    // Convert variant name to lowercase field name (Element -> element)
    const fieldName = variantName.charAt(0).toLowerCase() + variantName.slice(1);

    // Recursively transform children if present
    if (variantData.children && Array.isArray(variantData.children)) {
      variantData.children = (variantData.children as unknown[]).map((c) => transformNode(c as RawNode));
    }

    // Build the proto-format node
    const result: Record<string, unknown> = {};
    result[fieldName] = variantData;

    console.log("[transformNode] Output field:", fieldName);

    return result as VNode;
  }

  console.warn("[transformNode] Unknown format, keys:", Object.keys(raw));
  return {};
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
  };
}

/**
 * Transform a raw patch from server format to proto format.
 *
 * Two formats are possible:
 * 1. Initial load (manual JSON): { initialize: { vdom: {...} } }
 * 2. Incremental (prost serde): { patch_type: { UpdateText: {...} } }
 */
function transformPatch(raw: RawPatch & Record<string, unknown>): VDocPatch {
  console.log("[transformPatch] Raw keys:", Object.keys(raw));

  // Format 1: Direct field names (from process_file_to_json)
  // Check for direct patch fields first
  if ("initialize" in raw && raw.initialize) {
    console.log("[transformPatch] Found direct 'initialize' field");
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

    console.log("[transformPatch] patch_type variant:", variantName);

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
        for (const rawPatch of update.patches) {
          // Transform from prost serde format to proto format
          const patch = transformPatch(rawPatch);
          console.log("[SSE] Transformed patch:", Object.keys(patch).filter(k => (patch as any)[k] !== undefined));

          // Simple oneof checking - check which field is set
          if (patch.initialize?.vdom) {
            // Initialize patch - use vdom directly (already transformed!)
            vdom = patch.initialize.vdom;
            console.log("[SSE] Initialized VDOM, nodes:", vdom.nodes.length);
            if (vdom.nodes[0]) {
              console.log("[SSE] First node keys:", Object.keys(vdom.nodes[0]));
            }
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
