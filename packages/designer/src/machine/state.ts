/**
 * Designer state types and initial values
 *
 * Uses proto types directly from @paperclip/proto - no transformation layer!
 */

import { BaseEvent } from "@paperclip/common";
import {
  VNode as ProtoVNode,
  VDocument as ProtoVDocument,
  CssRule as ProtoCssRule,
} from "@paperclip/proto";

// Re-export proto types as canonical VDOM types
export type VNode = ProtoVNode;
export type VDocument = ProtoVDocument;
export type CssRule = ProtoCssRule;

// ============================================================================
// Geometry Types
// ============================================================================

export interface Point {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Transform {
  x: number;
  y: number;
  z: number;
}

// ============================================================================
// Frame Types
// ============================================================================

export interface FrameBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Frame {
  id: string;
  bounds: FrameBounds;
}

export const DEFAULT_FRAME_BOUNDS: FrameBounds = {
  x: 0,
  y: 0,
  width: 1024,
  height: 768,
};

// ============================================================================
// Tool Types
// ============================================================================

export type ResizeHandle = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w";

export interface DragState {
  handle: ResizeHandle;
  frameIndex: number;
  startBounds: FrameBounds;
  startMouse: Point;
  currentMouse: Point;
}

export interface ToolState {
  drag?: DragState;
}

export const DEFAULT_TOOL_STATE: ToolState = {};

// ============================================================================
// Element Selection Types
// ============================================================================

/**
 * Represents a selected element within a frame
 */
export interface ElementSelection {
  /** Index of the frame containing this element */
  frameIndex: number;
  /** Semantic ID from VDOM - stable identifier for the element */
  nodeId: string;
  /** Source ID for mutations - maps to AST span.id */
  sourceId: string;
}

/**
 * Style value with origin tracking
 */
export interface ComputedStyleValue {
  /** CSS value */
  value: string;
  /** Where this style comes from */
  origin: "inline" | "mixin" | "inherited" | "default";
  /** Source ID of where the style is defined (for mixin/inherited) */
  sourceId?: string;
}

/**
 * Map of CSS property name to computed value with origin
 */
export type ComputedStyles = Record<string, ComputedStyleValue>;

// ============================================================================
// Layer Panel Types
// ============================================================================

/**
 * State for the layer panel tree expansion
 */
export interface LayerPanelState {
  /** Set of node IDs that are expanded in the tree */
  expandedNodes: Set<string>;
}

// ============================================================================
// Pending Mutations (for optimistic updates)
// ============================================================================

export interface PendingFrameMutation {
  mutationId: string;
  type: "setFrameBounds";
  frameId: string;
  optimisticBounds: FrameBounds;
  createdAt: number;
}

export interface PendingStyleMutation {
  mutationId: string;
  type: "setStyleProperty" | "deleteStyleProperty";
  nodeId: string;
  property: string;
  value?: string; // undefined for delete
  createdAt: number;
}

export type PendingMutation = PendingFrameMutation | PendingStyleMutation;

export interface PendingMutationsState {
  mutations: Map<string, PendingMutation>;
}

// ============================================================================
// Canvas State
// ============================================================================

export interface CanvasState {
  size: Size;
  transform: Transform;
  mousePosition?: Point;
  isExpanded?: boolean;
  activeFrameIndex?: number;
}

export const DEFAULT_CANVAS_STATE: CanvasState = {
  size: { width: 0, height: 0 },
  transform: { x: 0, y: 0, z: 1 },
};

// ============================================================================
// Designer State
// ============================================================================

export interface DesignerState {
  // Canvas state
  canvas: CanvasState;

  // Document and frames
  document?: VDocument;
  frames: Frame[];
  selectedFrameIndex?: number;
  rects: Record<string, Box>;
  centeredInitial: boolean;

  // Tool state
  tool: ToolState;

  // Element selection (within frames)
  selectedElement?: ElementSelection;

  // Computed styles for selected element
  computedStyles: ComputedStyles;

  // Pending style edits (for optimistic updates, keyed by property name)
  pendingStyleChanges: Record<string, string>;

  // Layer panel state
  layerPanel: LayerPanelState;

  // Pending mutations (frame + style)
  pendingMutations: Map<string, PendingMutation>;
}

export const DEFAULT_DESIGNER_STATE: DesignerState = {
  canvas: DEFAULT_CANVAS_STATE,
  frames: [],
  rects: {},
  centeredInitial: false,
  tool: DEFAULT_TOOL_STATE,
  selectedElement: undefined,
  computedStyles: {},
  pendingStyleChanges: {},
  layerPanel: { expandedNodes: new Set() },
  pendingMutations: new Map(),
};

// ============================================================================
// Events
// ============================================================================

export type DesignerEvent =
  // Canvas events
  | BaseEvent<"canvas/resized", Size>
  | BaseEvent<"canvas/panned", { delta: Point; metaKey: boolean; ctrlKey: boolean }>
  | BaseEvent<"canvas/zoomed", { delta: number; center: Point }>
  | BaseEvent<"canvas/mouseMove", Point>
  | BaseEvent<"canvas/centerOnFrames">

  // Frame events
  | BaseEvent<"frame/selected", { index: number }>
  | BaseEvent<"frame/resized", { index: number; bounds: FrameBounds }>
  | BaseEvent<"frame/moved", { index: number; position: Point }>
  | BaseEvent<"frame/moveEnd", { index: number }>

  // Document events
  | BaseEvent<"document/loaded", { document: VDocument; frames: Frame[] }>

  // Tool events
  | BaseEvent<"tool/resizeStart", { handle: ResizeHandle; mouse: Point }>
  | BaseEvent<"tool/resizeMove", Point>
  | BaseEvent<"tool/resizeEnd">

  // Element selection events
  | BaseEvent<"element/selected", ElementSelection>
  | BaseEvent<"element/deselected">
  | BaseEvent<"element/hovered", { nodeId: string } | null>

  // Style editing events
  | BaseEvent<"style/propertyFocused", { property: string }>
  | BaseEvent<"style/propertyBlurred">
  | BaseEvent<"style/changed", { property: string; value: string }>
  | BaseEvent<"style/removed", { property: string }>

  // Layer panel events
  | BaseEvent<"layer/nodeExpanded", { nodeId: string }>
  | BaseEvent<"layer/nodeCollapsed", { nodeId: string }>
  | BaseEvent<"layer/nodeClicked", { nodeId: string; sourceId: string; frameIndex: number }>

  // Mutation lifecycle events
  | BaseEvent<"mutation/started", { mutation: PendingMutation }>
  | BaseEvent<"mutation/acknowledged", { mutationId: string; version: number }>
  | BaseEvent<"mutation/failed", { mutationId: string; error: string }>;
