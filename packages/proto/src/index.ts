/**
 * Paperclip Proto - Generated TypeScript types from protobuf definitions
 */

// VDOM types
export {
  VNode,
  ElementNode,
  TextNode,
  CommentNode,
  ComponentNode,
  VDocument,
  CssRule,
} from './generated/vdom.js';

// Patch types
export {
  VDocPatch,
  PatchPath,
  PositionalPath,
  InitializePatch,
  CreateNodePatch,
  RemoveNodePatch,
  ReplaceNodePatch,
  UpdateAttributesPatch,
  UpdateStylesPatch,
  UpdateTextPatch,
  AddStyleRulePatch,
  RemoveStyleRulePatch,
  MoveChildPatch,
} from './generated/patches.js';

// Workspace service types
export {
  PreviewRequest,
  PreviewUpdate,
  WatchRequest,
  FileEvent,
  FileEvent_EventType,
  MutationRequest,
  MutationResponse,
  MutationAck,
  MutationRebased,
  MutationNoop,
  Mutation,
  MoveElement,
  UpdateText,
  SetInlineStyle,
  SetAttribute,
  RemoveNode,
  InsertElement,
  OutlineRequest,
  OutlineResponse,
  OutlineNode,
  NodeType,
  SourceSpan,
  BufferRequest,
  ClosePreviewRequest,
  ClosePreviewResponse,
  HeartbeatRequest,
  HeartbeatResponse,
} from './generated/workspace.js';
