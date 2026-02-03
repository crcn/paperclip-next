import * as _m0 from "protobufjs/minimal";
import { VDocPatch } from "./patches";
export declare const protobufPackage = "paperclip.workspace";
/** Node type enum */
export declare enum NodeType {
    COMPONENT = 0,
    ELEMENT = 1,
    TEXT = 2,
    CONDITIONAL = 3,
    REPEAT = 4,
    INSERT = 5,
    UNRECOGNIZED = -1
}
export declare function nodeTypeFromJSON(object: any): NodeType;
export declare function nodeTypeToJSON(object: NodeType): string;
/** Request to start preview streaming */
export interface PreviewRequest {
    rootPath: string;
}
/** Preview update containing VDocument patches */
export interface PreviewUpdate {
    filePath: string;
    patches: VDocPatch[];
    error?: string | undefined;
    timestamp: number;
    version: number;
    /** NEW: Include mutation acknowledgments */
    acknowledgedMutationIds: string[];
    changedByClientId?: string | undefined;
}
/** Request to watch files */
export interface WatchRequest {
    directory: string;
    /** e.g., ["*.pc"] */
    patterns: string[];
}
/** File change event */
export interface FileEvent {
    eventType: FileEvent_EventType;
    filePath: string;
    timestamp: number;
}
export declare enum FileEvent_EventType {
    CREATED = 0,
    MODIFIED = 1,
    DELETED = 2,
    UNRECOGNIZED = -1
}
export declare function fileEvent_EventTypeFromJSON(object: any): FileEvent_EventType;
export declare function fileEvent_EventTypeToJSON(object: FileEvent_EventType): string;
/** Request to apply a semantic mutation */
export interface MutationRequest {
    clientId: string;
    filePath: string;
    mutation?: Mutation | undefined;
    /** Optimistic concurrency control */
    expectedVersion: number;
}
/** Response to mutation application */
export interface MutationResponse {
    ack?: MutationAck | undefined;
    rebased?: MutationRebased | undefined;
    noop?: MutationNoop | undefined;
}
/** Mutation was accepted and applied */
export interface MutationAck {
    mutationId: string;
    newVersion: number;
    timestamp: number;
}
/** Mutation was transformed (rebased) due to concurrent changes */
export interface MutationRebased {
    originalMutationId: string;
    transformedMutation?: Mutation | undefined;
    newVersion: number;
    reason: string;
}
/** Mutation had no effect (e.g., node already deleted) */
export interface MutationNoop {
    mutationId: string;
    reason: string;
}
/** Semantic AST operation */
export interface Mutation {
    mutationId: string;
    timestamp: number;
    moveElement?: MoveElement | undefined;
    updateText?: UpdateText | undefined;
    setInlineStyle?: SetInlineStyle | undefined;
    setAttribute?: SetAttribute | undefined;
    removeNode?: RemoveNode | undefined;
    insertElement?: InsertElement | undefined;
    setFrameBounds?: SetFrameBounds | undefined;
}
/** Set frame bounds (updates @frame doc comment) */
export interface SetFrameBounds {
    frameId: string;
    bounds?: Bounds | undefined;
}
/** Frame bounds */
export interface Bounds {
    x: number;
    y: number;
    width: number;
    height: number;
}
/** Move an element to a new parent at index */
export interface MoveElement {
    nodeId: string;
    newParentId: string;
    index: number;
}
/** Update text content (atomic replacement) */
export interface UpdateText {
    nodeId: string;
    content: string;
}
/** Set an inline style property */
export interface SetInlineStyle {
    nodeId: string;
    property: string;
    value: string;
}
/** Set an attribute value */
export interface SetAttribute {
    nodeId: string;
    name: string;
    value: string;
}
/** Remove a node from the tree */
export interface RemoveNode {
    nodeId: string;
}
/** Insert a new element (rare - most creation via templates) */
export interface InsertElement {
    parentId: string;
    index: number;
    /** Serialized AST element */
    elementJson: string;
}
/** Request document outline */
export interface OutlineRequest {
    filePath: string;
}
/** Document outline response */
export interface OutlineResponse {
    nodes: OutlineNode[];
    version: number;
}
/** Single node in document outline */
export interface OutlineNode {
    nodeId: string;
    type: NodeType;
    parentId?: string | undefined;
    childIds: string[];
    span?: SourceSpan | undefined;
    /** e.g., component name, tag name */
    label?: string | undefined;
}
/** Source code location */
export interface SourceSpan {
    startLine: number;
    startCol: number;
    endLine: number;
    endCol: number;
}
/**
 * Bidirectional CRDT sync stream
 * Client sends updates, server broadcasts to other clients and sends VDOM patches
 */
export interface CrdtSyncRequest {
    clientId: string;
    filePath: string;
    /** Initial sync - join session and get current state */
    join?: CrdtJoin | undefined;
    /** Send local CRDT update */
    update?: CrdtUpdate | undefined;
    /** Acknowledge received update (for flow control) */
    ack?: CrdtAck | undefined;
}
/** Join a CRDT editing session */
export interface CrdtJoin {
    /** Client's current state vector (empty for new session) */
    stateVector: Uint8Array;
}
/** CRDT update (Yjs-compatible binary format) */
export interface CrdtUpdate {
    /** Delta update (encoded via Y.encodeStateAsUpdate) */
    update: Uint8Array;
    /** Client's state vector after this update */
    stateVector: Uint8Array;
    /** Origin of the change (for filtering) */
    origin: string;
}
/** Acknowledge receipt of an update */
export interface CrdtAck {
    sequence: number;
}
/** Server response in CRDT sync stream */
export interface CrdtSyncResponse {
    /** Welcome message with current document state */
    welcome?: CrdtWelcome | undefined;
    /** Remote CRDT update from another client */
    remoteUpdate?: CrdtUpdate | undefined;
    /** VDOM patch after successful parse */
    vdomPatch?: CrdtVdomPatch | undefined;
    /** CSSOM update after successful parse */
    cssomPatch?: CrdtCssomPatch | undefined;
    /** Parse error (document still synced, just invalid) */
    parseError?: CrdtParseError | undefined;
}
/** Welcome message when joining a session */
export interface CrdtWelcome {
    /** Full document state (for new clients) */
    documentState: Uint8Array;
    /** Current state vector */
    stateVector: Uint8Array;
    /** Current VDOM (if document is valid) */
    initialVdom?: VDocPatch | undefined;
    /** Session info */
    version: number;
    clientCount: number;
}
/** VDOM patch from server after parse */
export interface CrdtVdomPatch {
    patches: VDocPatch[];
    version: number;
    /** Which client triggered this update (for origin filtering) */
    originClientId: string;
}
/** CSSOM update from server */
export interface CrdtCssomPatch {
    rules: CssRule[];
    version: number;
}
/** CSS rule for CSSOM */
export interface CssRule {
    selector: string;
    properties: {
        [key: string]: string;
    };
}
export interface CssRule_PropertiesEntry {
    key: string;
    value: string;
}
/** Parse error (document synced but invalid) */
export interface CrdtParseError {
    error: string;
    line: number;
    column: number;
}
/** Request to stream buffer content directly (no file I/O) */
export interface BufferRequest {
    clientId: string;
    filePath: string;
    content: string;
    /** Version-based sync for reconnection handling */
    expectedStateVersion?: number | undefined;
}
/** Request to close preview and cleanup state */
export interface ClosePreviewRequest {
    clientId: string;
}
/** Response confirming state cleanup */
export interface ClosePreviewResponse {
    success: boolean;
    message?: string | undefined;
}
/** Heartbeat request for liveness tracking */
export interface HeartbeatRequest {
    clientId: string;
}
/** Heartbeat response */
export interface HeartbeatResponse {
    acknowledged: boolean;
    serverTime: number;
}
export declare const PreviewRequest: {
    encode(message: PreviewRequest, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): PreviewRequest;
    fromJSON(object: any): PreviewRequest;
    toJSON(message: PreviewRequest): unknown;
    create<I extends Exact<DeepPartial<PreviewRequest>, I>>(base?: I): PreviewRequest;
    fromPartial<I extends Exact<DeepPartial<PreviewRequest>, I>>(object: I): PreviewRequest;
};
export declare const PreviewUpdate: {
    encode(message: PreviewUpdate, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): PreviewUpdate;
    fromJSON(object: any): PreviewUpdate;
    toJSON(message: PreviewUpdate): unknown;
    create<I extends Exact<DeepPartial<PreviewUpdate>, I>>(base?: I): PreviewUpdate;
    fromPartial<I extends Exact<DeepPartial<PreviewUpdate>, I>>(object: I): PreviewUpdate;
};
export declare const WatchRequest: {
    encode(message: WatchRequest, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): WatchRequest;
    fromJSON(object: any): WatchRequest;
    toJSON(message: WatchRequest): unknown;
    create<I extends Exact<DeepPartial<WatchRequest>, I>>(base?: I): WatchRequest;
    fromPartial<I extends Exact<DeepPartial<WatchRequest>, I>>(object: I): WatchRequest;
};
export declare const FileEvent: {
    encode(message: FileEvent, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): FileEvent;
    fromJSON(object: any): FileEvent;
    toJSON(message: FileEvent): unknown;
    create<I extends Exact<DeepPartial<FileEvent>, I>>(base?: I): FileEvent;
    fromPartial<I extends Exact<DeepPartial<FileEvent>, I>>(object: I): FileEvent;
};
export declare const MutationRequest: {
    encode(message: MutationRequest, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): MutationRequest;
    fromJSON(object: any): MutationRequest;
    toJSON(message: MutationRequest): unknown;
    create<I extends Exact<DeepPartial<MutationRequest>, I>>(base?: I): MutationRequest;
    fromPartial<I extends Exact<DeepPartial<MutationRequest>, I>>(object: I): MutationRequest;
};
export declare const MutationResponse: {
    encode(message: MutationResponse, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): MutationResponse;
    fromJSON(object: any): MutationResponse;
    toJSON(message: MutationResponse): unknown;
    create<I extends Exact<DeepPartial<MutationResponse>, I>>(base?: I): MutationResponse;
    fromPartial<I extends Exact<DeepPartial<MutationResponse>, I>>(object: I): MutationResponse;
};
export declare const MutationAck: {
    encode(message: MutationAck, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): MutationAck;
    fromJSON(object: any): MutationAck;
    toJSON(message: MutationAck): unknown;
    create<I extends Exact<DeepPartial<MutationAck>, I>>(base?: I): MutationAck;
    fromPartial<I extends Exact<DeepPartial<MutationAck>, I>>(object: I): MutationAck;
};
export declare const MutationRebased: {
    encode(message: MutationRebased, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): MutationRebased;
    fromJSON(object: any): MutationRebased;
    toJSON(message: MutationRebased): unknown;
    create<I extends Exact<DeepPartial<MutationRebased>, I>>(base?: I): MutationRebased;
    fromPartial<I extends Exact<DeepPartial<MutationRebased>, I>>(object: I): MutationRebased;
};
export declare const MutationNoop: {
    encode(message: MutationNoop, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): MutationNoop;
    fromJSON(object: any): MutationNoop;
    toJSON(message: MutationNoop): unknown;
    create<I extends Exact<DeepPartial<MutationNoop>, I>>(base?: I): MutationNoop;
    fromPartial<I extends Exact<DeepPartial<MutationNoop>, I>>(object: I): MutationNoop;
};
export declare const Mutation: {
    encode(message: Mutation, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): Mutation;
    fromJSON(object: any): Mutation;
    toJSON(message: Mutation): unknown;
    create<I extends Exact<DeepPartial<Mutation>, I>>(base?: I): Mutation;
    fromPartial<I extends Exact<DeepPartial<Mutation>, I>>(object: I): Mutation;
};
export declare const SetFrameBounds: {
    encode(message: SetFrameBounds, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): SetFrameBounds;
    fromJSON(object: any): SetFrameBounds;
    toJSON(message: SetFrameBounds): unknown;
    create<I extends Exact<DeepPartial<SetFrameBounds>, I>>(base?: I): SetFrameBounds;
    fromPartial<I extends Exact<DeepPartial<SetFrameBounds>, I>>(object: I): SetFrameBounds;
};
export declare const Bounds: {
    encode(message: Bounds, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): Bounds;
    fromJSON(object: any): Bounds;
    toJSON(message: Bounds): unknown;
    create<I extends Exact<DeepPartial<Bounds>, I>>(base?: I): Bounds;
    fromPartial<I extends Exact<DeepPartial<Bounds>, I>>(object: I): Bounds;
};
export declare const MoveElement: {
    encode(message: MoveElement, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): MoveElement;
    fromJSON(object: any): MoveElement;
    toJSON(message: MoveElement): unknown;
    create<I extends Exact<DeepPartial<MoveElement>, I>>(base?: I): MoveElement;
    fromPartial<I extends Exact<DeepPartial<MoveElement>, I>>(object: I): MoveElement;
};
export declare const UpdateText: {
    encode(message: UpdateText, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): UpdateText;
    fromJSON(object: any): UpdateText;
    toJSON(message: UpdateText): unknown;
    create<I extends Exact<DeepPartial<UpdateText>, I>>(base?: I): UpdateText;
    fromPartial<I extends Exact<DeepPartial<UpdateText>, I>>(object: I): UpdateText;
};
export declare const SetInlineStyle: {
    encode(message: SetInlineStyle, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): SetInlineStyle;
    fromJSON(object: any): SetInlineStyle;
    toJSON(message: SetInlineStyle): unknown;
    create<I extends Exact<DeepPartial<SetInlineStyle>, I>>(base?: I): SetInlineStyle;
    fromPartial<I extends Exact<DeepPartial<SetInlineStyle>, I>>(object: I): SetInlineStyle;
};
export declare const SetAttribute: {
    encode(message: SetAttribute, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): SetAttribute;
    fromJSON(object: any): SetAttribute;
    toJSON(message: SetAttribute): unknown;
    create<I extends Exact<DeepPartial<SetAttribute>, I>>(base?: I): SetAttribute;
    fromPartial<I extends Exact<DeepPartial<SetAttribute>, I>>(object: I): SetAttribute;
};
export declare const RemoveNode: {
    encode(message: RemoveNode, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): RemoveNode;
    fromJSON(object: any): RemoveNode;
    toJSON(message: RemoveNode): unknown;
    create<I extends Exact<DeepPartial<RemoveNode>, I>>(base?: I): RemoveNode;
    fromPartial<I extends Exact<DeepPartial<RemoveNode>, I>>(object: I): RemoveNode;
};
export declare const InsertElement: {
    encode(message: InsertElement, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): InsertElement;
    fromJSON(object: any): InsertElement;
    toJSON(message: InsertElement): unknown;
    create<I extends Exact<DeepPartial<InsertElement>, I>>(base?: I): InsertElement;
    fromPartial<I extends Exact<DeepPartial<InsertElement>, I>>(object: I): InsertElement;
};
export declare const OutlineRequest: {
    encode(message: OutlineRequest, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): OutlineRequest;
    fromJSON(object: any): OutlineRequest;
    toJSON(message: OutlineRequest): unknown;
    create<I extends Exact<DeepPartial<OutlineRequest>, I>>(base?: I): OutlineRequest;
    fromPartial<I extends Exact<DeepPartial<OutlineRequest>, I>>(object: I): OutlineRequest;
};
export declare const OutlineResponse: {
    encode(message: OutlineResponse, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): OutlineResponse;
    fromJSON(object: any): OutlineResponse;
    toJSON(message: OutlineResponse): unknown;
    create<I extends Exact<DeepPartial<OutlineResponse>, I>>(base?: I): OutlineResponse;
    fromPartial<I extends Exact<DeepPartial<OutlineResponse>, I>>(object: I): OutlineResponse;
};
export declare const OutlineNode: {
    encode(message: OutlineNode, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): OutlineNode;
    fromJSON(object: any): OutlineNode;
    toJSON(message: OutlineNode): unknown;
    create<I extends Exact<DeepPartial<OutlineNode>, I>>(base?: I): OutlineNode;
    fromPartial<I extends Exact<DeepPartial<OutlineNode>, I>>(object: I): OutlineNode;
};
export declare const SourceSpan: {
    encode(message: SourceSpan, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): SourceSpan;
    fromJSON(object: any): SourceSpan;
    toJSON(message: SourceSpan): unknown;
    create<I extends Exact<DeepPartial<SourceSpan>, I>>(base?: I): SourceSpan;
    fromPartial<I extends Exact<DeepPartial<SourceSpan>, I>>(object: I): SourceSpan;
};
export declare const CrdtSyncRequest: {
    encode(message: CrdtSyncRequest, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtSyncRequest;
    fromJSON(object: any): CrdtSyncRequest;
    toJSON(message: CrdtSyncRequest): unknown;
    create<I extends Exact<DeepPartial<CrdtSyncRequest>, I>>(base?: I): CrdtSyncRequest;
    fromPartial<I extends Exact<DeepPartial<CrdtSyncRequest>, I>>(object: I): CrdtSyncRequest;
};
export declare const CrdtJoin: {
    encode(message: CrdtJoin, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtJoin;
    fromJSON(object: any): CrdtJoin;
    toJSON(message: CrdtJoin): unknown;
    create<I extends Exact<DeepPartial<CrdtJoin>, I>>(base?: I): CrdtJoin;
    fromPartial<I extends Exact<DeepPartial<CrdtJoin>, I>>(object: I): CrdtJoin;
};
export declare const CrdtUpdate: {
    encode(message: CrdtUpdate, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtUpdate;
    fromJSON(object: any): CrdtUpdate;
    toJSON(message: CrdtUpdate): unknown;
    create<I extends Exact<DeepPartial<CrdtUpdate>, I>>(base?: I): CrdtUpdate;
    fromPartial<I extends Exact<DeepPartial<CrdtUpdate>, I>>(object: I): CrdtUpdate;
};
export declare const CrdtAck: {
    encode(message: CrdtAck, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtAck;
    fromJSON(object: any): CrdtAck;
    toJSON(message: CrdtAck): unknown;
    create<I extends Exact<DeepPartial<CrdtAck>, I>>(base?: I): CrdtAck;
    fromPartial<I extends Exact<DeepPartial<CrdtAck>, I>>(object: I): CrdtAck;
};
export declare const CrdtSyncResponse: {
    encode(message: CrdtSyncResponse, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtSyncResponse;
    fromJSON(object: any): CrdtSyncResponse;
    toJSON(message: CrdtSyncResponse): unknown;
    create<I extends Exact<DeepPartial<CrdtSyncResponse>, I>>(base?: I): CrdtSyncResponse;
    fromPartial<I extends Exact<DeepPartial<CrdtSyncResponse>, I>>(object: I): CrdtSyncResponse;
};
export declare const CrdtWelcome: {
    encode(message: CrdtWelcome, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtWelcome;
    fromJSON(object: any): CrdtWelcome;
    toJSON(message: CrdtWelcome): unknown;
    create<I extends Exact<DeepPartial<CrdtWelcome>, I>>(base?: I): CrdtWelcome;
    fromPartial<I extends Exact<DeepPartial<CrdtWelcome>, I>>(object: I): CrdtWelcome;
};
export declare const CrdtVdomPatch: {
    encode(message: CrdtVdomPatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtVdomPatch;
    fromJSON(object: any): CrdtVdomPatch;
    toJSON(message: CrdtVdomPatch): unknown;
    create<I extends Exact<DeepPartial<CrdtVdomPatch>, I>>(base?: I): CrdtVdomPatch;
    fromPartial<I extends Exact<DeepPartial<CrdtVdomPatch>, I>>(object: I): CrdtVdomPatch;
};
export declare const CrdtCssomPatch: {
    encode(message: CrdtCssomPatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtCssomPatch;
    fromJSON(object: any): CrdtCssomPatch;
    toJSON(message: CrdtCssomPatch): unknown;
    create<I extends Exact<DeepPartial<CrdtCssomPatch>, I>>(base?: I): CrdtCssomPatch;
    fromPartial<I extends Exact<DeepPartial<CrdtCssomPatch>, I>>(object: I): CrdtCssomPatch;
};
export declare const CssRule: {
    encode(message: CssRule, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CssRule;
    fromJSON(object: any): CssRule;
    toJSON(message: CssRule): unknown;
    create<I extends Exact<DeepPartial<CssRule>, I>>(base?: I): CssRule;
    fromPartial<I extends Exact<DeepPartial<CssRule>, I>>(object: I): CssRule;
};
export declare const CssRule_PropertiesEntry: {
    encode(message: CssRule_PropertiesEntry, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CssRule_PropertiesEntry;
    fromJSON(object: any): CssRule_PropertiesEntry;
    toJSON(message: CssRule_PropertiesEntry): unknown;
    create<I extends Exact<DeepPartial<CssRule_PropertiesEntry>, I>>(base?: I): CssRule_PropertiesEntry;
    fromPartial<I extends Exact<DeepPartial<CssRule_PropertiesEntry>, I>>(object: I): CssRule_PropertiesEntry;
};
export declare const CrdtParseError: {
    encode(message: CrdtParseError, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CrdtParseError;
    fromJSON(object: any): CrdtParseError;
    toJSON(message: CrdtParseError): unknown;
    create<I extends Exact<DeepPartial<CrdtParseError>, I>>(base?: I): CrdtParseError;
    fromPartial<I extends Exact<DeepPartial<CrdtParseError>, I>>(object: I): CrdtParseError;
};
export declare const BufferRequest: {
    encode(message: BufferRequest, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): BufferRequest;
    fromJSON(object: any): BufferRequest;
    toJSON(message: BufferRequest): unknown;
    create<I extends Exact<DeepPartial<BufferRequest>, I>>(base?: I): BufferRequest;
    fromPartial<I extends Exact<DeepPartial<BufferRequest>, I>>(object: I): BufferRequest;
};
export declare const ClosePreviewRequest: {
    encode(message: ClosePreviewRequest, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ClosePreviewRequest;
    fromJSON(object: any): ClosePreviewRequest;
    toJSON(message: ClosePreviewRequest): unknown;
    create<I extends Exact<DeepPartial<ClosePreviewRequest>, I>>(base?: I): ClosePreviewRequest;
    fromPartial<I extends Exact<DeepPartial<ClosePreviewRequest>, I>>(object: I): ClosePreviewRequest;
};
export declare const ClosePreviewResponse: {
    encode(message: ClosePreviewResponse, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ClosePreviewResponse;
    fromJSON(object: any): ClosePreviewResponse;
    toJSON(message: ClosePreviewResponse): unknown;
    create<I extends Exact<DeepPartial<ClosePreviewResponse>, I>>(base?: I): ClosePreviewResponse;
    fromPartial<I extends Exact<DeepPartial<ClosePreviewResponse>, I>>(object: I): ClosePreviewResponse;
};
export declare const HeartbeatRequest: {
    encode(message: HeartbeatRequest, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): HeartbeatRequest;
    fromJSON(object: any): HeartbeatRequest;
    toJSON(message: HeartbeatRequest): unknown;
    create<I extends Exact<DeepPartial<HeartbeatRequest>, I>>(base?: I): HeartbeatRequest;
    fromPartial<I extends Exact<DeepPartial<HeartbeatRequest>, I>>(object: I): HeartbeatRequest;
};
export declare const HeartbeatResponse: {
    encode(message: HeartbeatResponse, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): HeartbeatResponse;
    fromJSON(object: any): HeartbeatResponse;
    toJSON(message: HeartbeatResponse): unknown;
    create<I extends Exact<DeepPartial<HeartbeatResponse>, I>>(base?: I): HeartbeatResponse;
    fromPartial<I extends Exact<DeepPartial<HeartbeatResponse>, I>>(object: I): HeartbeatResponse;
};
type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;
export type DeepPartial<T> = T extends Builtin ? T : T extends globalThis.Array<infer U> ? globalThis.Array<DeepPartial<U>> : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>> : T extends {} ? {
    [K in keyof T]?: DeepPartial<T[K]>;
} : Partial<T>;
type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin ? P : P & {
    [K in keyof P]: Exact<P[K], I[K]>;
} & {
    [K in Exclude<keyof I, KeysOfUnion<P>>]: never;
};
export {};
//# sourceMappingURL=workspace.d.ts.map