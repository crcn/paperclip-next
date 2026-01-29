/**
 * Paperclip Workspace Client
 * Universal TypeScript client for real-time preview and editing
 */

export { WorkspaceClient, createWorkspaceClient } from './client.js';
export type { WorkspaceClientConfig } from './client.js';

export { EventEmitter } from './events.js';
export type {
  WorkspaceEvent,
  WorkspaceEventUnion,
  EventListener,
  Connected,
  Disconnected,
  PreviewUpdated,
  FileChanged,
  MutationAcknowledged,
  OutlineReceived,
  ConnectionFailed,
  RpcFailed,
} from './events.js';

// Re-export event types that have the same name as response types
export type {
  MutationRebased as MutationRebasedEvent,
  MutationNoop as MutationNoopEvent,
} from './events.js';

export type {
  Transport,
  TransportError,
  ConnectionError,
  RpcError,
} from './transport/interface.js';

export type {
  // VDOM types
  VNode,
  ElementNode,
  TextNode,
  CommentNode,
  ComponentNode,
  VDocument,
  CssRule,
  // Patch types
  VDocPatch,
  InitializePatch,
  CreateNodePatch,
  RemoveNodePatch,
  ReplaceNodePatch,
  UpdateAttributesPatch,
  UpdateStylesPatch,
  UpdateTextPatch,
  AddStyleRulePatch,
  RemoveStyleRulePatch,
  // Workspace service types
  PreviewRequest,
  PreviewUpdate,
  WatchRequest,
  FileEvent,
  FileEventType,
  // Mutation types
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
  // Outline types
  OutlineRequest,
  OutlineResponse,
  OutlineNode,
  NodeType,
  SourceSpan,
} from './types.js';
