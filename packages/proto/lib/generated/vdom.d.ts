import _m0 from "protobufjs/minimal";
export declare const protobufPackage = "paperclip.vdom";
/** Virtual DOM node (discriminated union) */
export interface VNode {
    element?: ElementNode | undefined;
    text?: TextNode | undefined;
    comment?: CommentNode | undefined;
    component?: ComponentNode | undefined;
}
export interface ElementNode {
    tag: string;
    attributes: {
        [key: string]: string;
    };
    styles: {
        [key: string]: string;
    };
    /** Field 5 (id) removed - use semantic_id instead */
    children: VNode[];
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
    /** Field 4 (id) removed - use semantic_id instead */
    children: VNode[];
}
export interface ComponentNode_PropsEntry {
    key: string;
    value: string;
}
export interface VDocument {
    nodes: VNode[];
    styles: CssRule[];
}
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
export declare const VDocument: {
    encode(message: VDocument, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): VDocument;
    fromJSON(object: any): VDocument;
    toJSON(message: VDocument): unknown;
    create<I extends Exact<DeepPartial<VDocument>, I>>(base?: I): VDocument;
    fromPartial<I extends Exact<DeepPartial<VDocument>, I>>(object: I): VDocument;
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