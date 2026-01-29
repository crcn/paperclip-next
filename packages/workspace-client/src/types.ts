/**
 * TypeScript types for Paperclip workspace protocol
 * Generated from proto/workspace.proto, proto/patches.proto, proto/vdom.proto
 */

// ============================================================================
// VDOM Types (from vdom.proto)
// ============================================================================

export interface VNode {
  element?: ElementNode;
  text?: TextNode;
  comment?: CommentNode;
  component?: ComponentNode;
}

export interface ElementNode {
  tag: string;
  attributes: Record<string, string>;
  styles: Record<string, string>;
  children: VNode[];
  id?: string;
}

export interface TextNode {
  content: string;
}

export interface CommentNode {
  content: string;
}

export interface ComponentNode {
  component_id: string;
  props: Record<string, string>;
  children: VNode[];
  id?: string;
}

export interface VDocument {
  nodes: VNode[];
  styles: CssRule[];
}

export interface CssRule {
  selector: string;
  properties: Record<string, string>;
}

// ============================================================================
// Patch Types (from patches.proto)
// ============================================================================

export interface VDocPatch {
  initialize?: InitializePatch;
  create_node?: CreateNodePatch;
  remove_node?: RemoveNodePatch;
  replace_node?: ReplaceNodePatch;
  update_attributes?: UpdateAttributesPatch;
  update_styles?: UpdateStylesPatch;
  update_text?: UpdateTextPatch;
  add_style_rule?: AddStyleRulePatch;
  remove_style_rule?: RemoveStyleRulePatch;
}

export interface InitializePatch {
  vdom: VDocument;
}

export interface CreateNodePatch {
  path: number[];
  node: VNode;
  index: number;
}

export interface RemoveNodePatch {
  path: number[];
}

export interface ReplaceNodePatch {
  path: number[];
  new_node: VNode;
}

export interface UpdateAttributesPatch {
  path: number[];
  attributes: Record<string, string>;
}

export interface UpdateStylesPatch {
  path: number[];
  styles: Record<string, string>;
}

export interface UpdateTextPatch {
  path: number[];
  content: string;
}

export interface AddStyleRulePatch {
  rule: CssRule;
}

export interface RemoveStyleRulePatch {
  index: number;
}

// ============================================================================
// Workspace Service Types (from workspace.proto)
// ============================================================================

export interface PreviewRequest {
  root_path: string;
}

export interface PreviewUpdate {
  file_path: string;
  patches: VDocPatch[];
  error?: string;
  timestamp: number;
  version: number;
  acknowledged_mutation_ids: string[];
  changed_by_client_id?: string;
}

export interface WatchRequest {
  directory: string;
  patterns: string[];
}

export enum FileEventType {
  CREATED = 0,
  MODIFIED = 1,
  DELETED = 2,
}

export interface FileEvent {
  event_type: FileEventType;
  file_path: string;
  timestamp: number;
}

// ============================================================================
// Mutation Types (Phase 1)
// ============================================================================

export interface MutationRequest {
  client_id: string;
  file_path: string;
  mutation: Mutation;
  expected_version: number;
}

export interface MutationResponse {
  ack?: MutationAck;
  rebased?: MutationRebased;
  noop?: MutationNoop;
}

export interface MutationAck {
  mutation_id: string;
  new_version: number;
  timestamp: number;
}

export interface MutationRebased {
  original_mutation_id: string;
  transformed_mutation: Mutation;
  new_version: number;
  reason: string;
}

export interface MutationNoop {
  mutation_id: string;
  reason: string;
}

export interface Mutation {
  mutation_id: string;
  timestamp: number;
  move_element?: MoveElement;
  update_text?: UpdateText;
  set_inline_style?: SetInlineStyle;
  set_attribute?: SetAttribute;
  remove_node?: RemoveNode;
  insert_element?: InsertElement;
}

export interface MoveElement {
  node_id: string;
  new_parent_id: string;
  index: number;
}

export interface UpdateText {
  node_id: string;
  content: string;
}

export interface SetInlineStyle {
  node_id: string;
  property: string;
  value: string;
}

export interface SetAttribute {
  node_id: string;
  name: string;
  value: string;
}

export interface RemoveNode {
  node_id: string;
}

export interface InsertElement {
  parent_id: string;
  index: number;
  element_json: string;
}

// ============================================================================
// Document Outline Types (Phase 1)
// ============================================================================

export interface OutlineRequest {
  file_path: string;
}

export interface OutlineResponse {
  nodes: OutlineNode[];
  version: number;
}

export interface OutlineNode {
  node_id: string;
  type: NodeType;
  parent_id?: string;
  child_ids: string[];
  span: SourceSpan;
  label?: string;
}

export enum NodeType {
  COMPONENT = 0,
  ELEMENT = 1,
  TEXT = 2,
  CONDITIONAL = 3,
  REPEAT = 4,
  INSERT = 5,
}

export interface SourceSpan {
  start_line: number;
  start_col: number;
  end_line: number;
  end_col: number;
}
