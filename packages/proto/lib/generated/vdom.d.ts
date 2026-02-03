import * as _m0 from "protobufjs/minimal";
export declare const protobufPackage = "paperclip.vdom";
export declare enum NullValue {
    NULL_VALUE = 0,
    UNRECOGNIZED = -1
}
export declare function nullValueFromJSON(object: any): NullValue;
export declare function nullValueToJSON(object: NullValue): string;
export interface Value {
    nullValue?: NullValue | undefined;
    numberValue?: number | undefined;
    stringValue?: string | undefined;
    boolValue?: boolean | undefined;
    objectValue?: ObjectValue | undefined;
    listValue?: ListValue | undefined;
}
export interface ObjectValue {
    fields: {
        [key: string]: Value;
    };
}
export interface ObjectValue_FieldsEntry {
    key: string;
    value?: Value | undefined;
}
export interface ListValue {
    values: Value[];
}
export interface Span {
    start: number;
    end: number;
    id: string;
}
/** Virtual DOM node (discriminated union) */
export interface VNode {
    element?: ElementNode | undefined;
    text?: TextNode | undefined;
    comment?: CommentNode | undefined;
    component?: ComponentNode | undefined;
    /** Inline error display */
    error?: ErrorNode | undefined;
}
export interface ElementNode {
    tag: string;
    attributes: {
        [key: string]: string;
    };
    styles: {
        [key: string]: string;
    };
    children: VNode[];
    /** Stable identity for diffing */
    semanticId: string;
    /** Explicit key for repeat items */
    key?: string | undefined;
    /** Maps back to AST span.id for mutations */
    sourceId?: string | undefined;
    /** Flexible metadata (frame info, annotations, etc.) */
    metadata?: Value | undefined;
}
export interface ElementNode_AttributesEntry {
    key: string;
    value: string;
}
export interface ElementNode_StylesEntry {
    key: string;
    value: string;
}
export interface TextNode {
    content: string;
}
export interface CommentNode {
    content: string;
}
export interface ComponentNode {
    componentId: string;
    props: {
        [key: string]: string;
    };
    children: VNode[];
    /** Stable identity for diffing */
    semanticId: string;
}
export interface ComponentNode_PropsEntry {
    key: string;
    value: string;
}
export interface ErrorNode {
    message: string;
    semanticId: string;
    /** Source location for debugging */
    span?: Span | undefined;
}
export interface CssRule {
    selector: string;
    properties: {
        [key: string]: string;
    };
    mediaQuery?: string | undefined;
    /** Flexible metadata (source info, annotations, etc.) */
    metadata?: Value | undefined;
}
export interface CssRule_PropertiesEntry {
    key: string;
    value: string;
}
/** Virtual CSSOM for CSS-specific operations */
export interface CssDocument {
    rules: CssRule[];
    /** Document-level CSS metadata (variables, tokens, etc.) */
    metadata?: Value | undefined;
}
export interface ComponentMetadata {
    name: string;
    description?: string | undefined;
    frame?: FrameMetadata | undefined;
    annotations: AnnotationMetadata[];
    sourceId?: string | undefined;
}
export interface FrameMetadata {
    x: number;
    y: number;
    width?: number | undefined;
    height?: number | undefined;
}
export interface AnnotationMetadata {
    name: string;
    params: {
        [key: string]: Value;
    };
}
export interface AnnotationMetadata_ParamsEntry {
    key: string;
    value?: Value | undefined;
}
export interface VDocument {
    nodes: VNode[];
    styles: CssRule[];
    /** Component metadata for designer */
    components: ComponentMetadata[];
    /** Document-level metadata */
    metadata?: Value | undefined;
}
export declare const Value: {
    encode(message: Value, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): Value;
    fromJSON(object: any): Value;
    toJSON(message: Value): unknown;
    create<I extends Exact<DeepPartial<Value>, I>>(base?: I): Value;
    fromPartial<I extends Exact<DeepPartial<Value>, I>>(object: I): Value;
};
export declare const ObjectValue: {
    encode(message: ObjectValue, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ObjectValue;
    fromJSON(object: any): ObjectValue;
    toJSON(message: ObjectValue): unknown;
    create<I extends Exact<DeepPartial<ObjectValue>, I>>(base?: I): ObjectValue;
    fromPartial<I extends Exact<DeepPartial<ObjectValue>, I>>(object: I): ObjectValue;
};
export declare const ObjectValue_FieldsEntry: {
    encode(message: ObjectValue_FieldsEntry, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ObjectValue_FieldsEntry;
    fromJSON(object: any): ObjectValue_FieldsEntry;
    toJSON(message: ObjectValue_FieldsEntry): unknown;
    create<I extends Exact<DeepPartial<ObjectValue_FieldsEntry>, I>>(base?: I): ObjectValue_FieldsEntry;
    fromPartial<I extends Exact<DeepPartial<ObjectValue_FieldsEntry>, I>>(object: I): ObjectValue_FieldsEntry;
};
export declare const ListValue: {
    encode(message: ListValue, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ListValue;
    fromJSON(object: any): ListValue;
    toJSON(message: ListValue): unknown;
    create<I extends Exact<DeepPartial<ListValue>, I>>(base?: I): ListValue;
    fromPartial<I extends Exact<DeepPartial<ListValue>, I>>(object: I): ListValue;
};
export declare const Span: {
    encode(message: Span, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): Span;
    fromJSON(object: any): Span;
    toJSON(message: Span): unknown;
    create<I extends Exact<DeepPartial<Span>, I>>(base?: I): Span;
    fromPartial<I extends Exact<DeepPartial<Span>, I>>(object: I): Span;
};
export declare const VNode: {
    encode(message: VNode, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): VNode;
    fromJSON(object: any): VNode;
    toJSON(message: VNode): unknown;
    create<I extends Exact<DeepPartial<VNode>, I>>(base?: I): VNode;
    fromPartial<I extends Exact<DeepPartial<VNode>, I>>(object: I): VNode;
};
export declare const ElementNode: {
    encode(message: ElementNode, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ElementNode;
    fromJSON(object: any): ElementNode;
    toJSON(message: ElementNode): unknown;
    create<I extends Exact<DeepPartial<ElementNode>, I>>(base?: I): ElementNode;
    fromPartial<I extends Exact<DeepPartial<ElementNode>, I>>(object: I): ElementNode;
};
export declare const ElementNode_AttributesEntry: {
    encode(message: ElementNode_AttributesEntry, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ElementNode_AttributesEntry;
    fromJSON(object: any): ElementNode_AttributesEntry;
    toJSON(message: ElementNode_AttributesEntry): unknown;
    create<I extends Exact<DeepPartial<ElementNode_AttributesEntry>, I>>(base?: I): ElementNode_AttributesEntry;
    fromPartial<I extends Exact<DeepPartial<ElementNode_AttributesEntry>, I>>(object: I): ElementNode_AttributesEntry;
};
export declare const ElementNode_StylesEntry: {
    encode(message: ElementNode_StylesEntry, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ElementNode_StylesEntry;
    fromJSON(object: any): ElementNode_StylesEntry;
    toJSON(message: ElementNode_StylesEntry): unknown;
    create<I extends Exact<DeepPartial<ElementNode_StylesEntry>, I>>(base?: I): ElementNode_StylesEntry;
    fromPartial<I extends Exact<DeepPartial<ElementNode_StylesEntry>, I>>(object: I): ElementNode_StylesEntry;
};
export declare const TextNode: {
    encode(message: TextNode, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): TextNode;
    fromJSON(object: any): TextNode;
    toJSON(message: TextNode): unknown;
    create<I extends Exact<DeepPartial<TextNode>, I>>(base?: I): TextNode;
    fromPartial<I extends Exact<DeepPartial<TextNode>, I>>(object: I): TextNode;
};
export declare const CommentNode: {
    encode(message: CommentNode, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CommentNode;
    fromJSON(object: any): CommentNode;
    toJSON(message: CommentNode): unknown;
    create<I extends Exact<DeepPartial<CommentNode>, I>>(base?: I): CommentNode;
    fromPartial<I extends Exact<DeepPartial<CommentNode>, I>>(object: I): CommentNode;
};
export declare const ComponentNode: {
    encode(message: ComponentNode, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ComponentNode;
    fromJSON(object: any): ComponentNode;
    toJSON(message: ComponentNode): unknown;
    create<I extends Exact<DeepPartial<ComponentNode>, I>>(base?: I): ComponentNode;
    fromPartial<I extends Exact<DeepPartial<ComponentNode>, I>>(object: I): ComponentNode;
};
export declare const ComponentNode_PropsEntry: {
    encode(message: ComponentNode_PropsEntry, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ComponentNode_PropsEntry;
    fromJSON(object: any): ComponentNode_PropsEntry;
    toJSON(message: ComponentNode_PropsEntry): unknown;
    create<I extends Exact<DeepPartial<ComponentNode_PropsEntry>, I>>(base?: I): ComponentNode_PropsEntry;
    fromPartial<I extends Exact<DeepPartial<ComponentNode_PropsEntry>, I>>(object: I): ComponentNode_PropsEntry;
};
export declare const ErrorNode: {
    encode(message: ErrorNode, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ErrorNode;
    fromJSON(object: any): ErrorNode;
    toJSON(message: ErrorNode): unknown;
    create<I extends Exact<DeepPartial<ErrorNode>, I>>(base?: I): ErrorNode;
    fromPartial<I extends Exact<DeepPartial<ErrorNode>, I>>(object: I): ErrorNode;
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
export declare const CssDocument: {
    encode(message: CssDocument, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CssDocument;
    fromJSON(object: any): CssDocument;
    toJSON(message: CssDocument): unknown;
    create<I extends Exact<DeepPartial<CssDocument>, I>>(base?: I): CssDocument;
    fromPartial<I extends Exact<DeepPartial<CssDocument>, I>>(object: I): CssDocument;
};
export declare const ComponentMetadata: {
    encode(message: ComponentMetadata, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ComponentMetadata;
    fromJSON(object: any): ComponentMetadata;
    toJSON(message: ComponentMetadata): unknown;
    create<I extends Exact<DeepPartial<ComponentMetadata>, I>>(base?: I): ComponentMetadata;
    fromPartial<I extends Exact<DeepPartial<ComponentMetadata>, I>>(object: I): ComponentMetadata;
};
export declare const FrameMetadata: {
    encode(message: FrameMetadata, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): FrameMetadata;
    fromJSON(object: any): FrameMetadata;
    toJSON(message: FrameMetadata): unknown;
    create<I extends Exact<DeepPartial<FrameMetadata>, I>>(base?: I): FrameMetadata;
    fromPartial<I extends Exact<DeepPartial<FrameMetadata>, I>>(object: I): FrameMetadata;
};
export declare const AnnotationMetadata: {
    encode(message: AnnotationMetadata, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): AnnotationMetadata;
    fromJSON(object: any): AnnotationMetadata;
    toJSON(message: AnnotationMetadata): unknown;
    create<I extends Exact<DeepPartial<AnnotationMetadata>, I>>(base?: I): AnnotationMetadata;
    fromPartial<I extends Exact<DeepPartial<AnnotationMetadata>, I>>(object: I): AnnotationMetadata;
};
export declare const AnnotationMetadata_ParamsEntry: {
    encode(message: AnnotationMetadata_ParamsEntry, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): AnnotationMetadata_ParamsEntry;
    fromJSON(object: any): AnnotationMetadata_ParamsEntry;
    toJSON(message: AnnotationMetadata_ParamsEntry): unknown;
    create<I extends Exact<DeepPartial<AnnotationMetadata_ParamsEntry>, I>>(base?: I): AnnotationMetadata_ParamsEntry;
    fromPartial<I extends Exact<DeepPartial<AnnotationMetadata_ParamsEntry>, I>>(object: I): AnnotationMetadata_ParamsEntry;
};
export declare const VDocument: {
    encode(message: VDocument, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): VDocument;
    fromJSON(object: any): VDocument;
    toJSON(message: VDocument): unknown;
    create<I extends Exact<DeepPartial<VDocument>, I>>(base?: I): VDocument;
    fromPartial<I extends Exact<DeepPartial<VDocument>, I>>(object: I): VDocument;
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
//# sourceMappingURL=vdom.d.ts.map