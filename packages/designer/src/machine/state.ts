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
// Pending Mutations (for optimistic updates)
// ============================================================================

export interface PendingMutation {
  mutationId: string;
  type: "setFrameBounds";
  frameId: string;
  optimisticBounds: FrameBounds;
  createdAt: number;
}

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
  canvas: CanvasState;
  document?: VDocument;
  frames: Frame[];
  selectedFrameIndex?: number;
  rects: Record<string, Box>;
  centeredInitial: boolean;
  tool: ToolState;
  pendingMutations: Map<string, PendingMutation>;
}

export const DEFAULT_DESIGNER_STATE: DesignerState = {
  canvas: DEFAULT_CANVAS_STATE,
  frames: [],
  rects: {},
  centeredInitial: false,
  tool: DEFAULT_TOOL_STATE,
  pendingMutations: new Map(),
};

// ============================================================================
// Events
// ============================================================================

export type DesignerEvent =
  | BaseEvent<"canvas/resized", Size>
  | BaseEvent<"canvas/panned", { delta: Point; metaKey: boolean; ctrlKey: boolean }>
  | BaseEvent<"canvas/zoomed", { delta: number; center: Point }>
  | BaseEvent<"canvas/mouseMove", Point>
  | BaseEvent<"canvas/centerOnFrames">
  | BaseEvent<"frame/selected", { index: number }>
  | BaseEvent<"frame/resized", { index: number; bounds: FrameBounds }>
  | BaseEvent<"frame/moved", { index: number; position: Point }>
  | BaseEvent<"document/loaded", { document: VDocument; frames: Frame[] }>
  | BaseEvent<"tool/resizeStart", { handle: ResizeHandle; mouse: Point }>
  | BaseEvent<"tool/resizeMove", Point>
  | BaseEvent<"tool/resizeEnd">
  // Mutation lifecycle events
  | BaseEvent<"mutation/started", { mutation: PendingMutation }>
  | BaseEvent<"mutation/acknowledged", { mutationId: string; version: number }>
  | BaseEvent<"mutation/failed", { mutationId: string; error: string }>;
