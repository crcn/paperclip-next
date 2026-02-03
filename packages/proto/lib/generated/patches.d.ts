import * as _m0 from "protobufjs/minimal";
import { CssRule, VDocument, VNode } from "./vdom";
export declare const protobufPackage = "paperclip.patches";
/** VDocument patch operations */
export interface VDocPatch {
    initialize?: InitializePatch | undefined;
    createNode?: CreateNodePatch | undefined;
    removeNode?: RemoveNodePatch | undefined;
    replaceNode?: ReplaceNodePatch | undefined;
    updateAttributes?: UpdateAttributesPatch | undefined;
    updateStyles?: UpdateStylesPatch | undefined;
    updateText?: UpdateTextPatch | undefined;
    addStyleRule?: AddStyleRulePatch | undefined;
    removeStyleRule?: RemoveStyleRulePatch | undefined;
    /** NEW: For semantic reordering */
    moveChild?: MoveChildPatch | undefined;
}
/** Patch path supporting both positional and semantic IDs */
export interface PatchPath {
    positional?: PositionalPath | undefined;
    /** e.g., "Card{Card-0}::div[d4f5]" */
    semantic?: string | undefined;
}
/** Positional path (array of indices) */
export interface PositionalPath {
    indices: number[];
}
export interface InitializePatch {
    vdom?: VDocument | undefined;
}
export interface CreateNodePatch {
    path: number[];
    node?: VNode | undefined;
    index: number;
}
export interface RemoveNodePatch {
    path: number[];
}
export interface ReplaceNodePatch {
    path: number[];
    newNode?: VNode | undefined;
}
export interface UpdateAttributesPatch {
    path: number[];
    attributes: {
        [key: string]: string;
    };
}
export interface UpdateAttributesPatch_AttributesEntry {
    key: string;
    value: string;
}
export interface UpdateStylesPatch {
    path: number[];
    styles: {
        [key: string]: string;
    };
}
export interface UpdateStylesPatch_StylesEntry {
    key: string;
    value: string;
}
export interface UpdateTextPatch {
    path: number[];
    content: string;
}
export interface AddStyleRulePatch {
    rule?: CssRule | undefined;
}
export interface RemoveStyleRulePatch {
    index: number;
}
/** NEW: Move child patch for efficient reordering (semantic ID support) */
export interface MoveChildPatch {
    parent?: PatchPath | undefined;
    childSemanticId: string;
    fromIndex: number;
    toIndex: number;
}
export declare const VDocPatch: {
    encode(message: VDocPatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): VDocPatch;
    fromJSON(object: any): VDocPatch;
    toJSON(message: VDocPatch): unknown;
    create<I extends Exact<DeepPartial<VDocPatch>, I>>(base?: I): VDocPatch;
    fromPartial<I extends Exact<DeepPartial<VDocPatch>, I>>(object: I): VDocPatch;
};
export declare const PatchPath: {
    encode(message: PatchPath, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): PatchPath;
    fromJSON(object: any): PatchPath;
    toJSON(message: PatchPath): unknown;
    create<I extends Exact<DeepPartial<PatchPath>, I>>(base?: I): PatchPath;
    fromPartial<I extends Exact<DeepPartial<PatchPath>, I>>(object: I): PatchPath;
};
export declare const PositionalPath: {
    encode(message: PositionalPath, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): PositionalPath;
    fromJSON(object: any): PositionalPath;
    toJSON(message: PositionalPath): unknown;
    create<I extends Exact<DeepPartial<PositionalPath>, I>>(base?: I): PositionalPath;
    fromPartial<I extends Exact<DeepPartial<PositionalPath>, I>>(object: I): PositionalPath;
};
export declare const InitializePatch: {
    encode(message: InitializePatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): InitializePatch;
    fromJSON(object: any): InitializePatch;
    toJSON(message: InitializePatch): unknown;
    create<I extends Exact<DeepPartial<InitializePatch>, I>>(base?: I): InitializePatch;
    fromPartial<I extends Exact<DeepPartial<InitializePatch>, I>>(object: I): InitializePatch;
};
export declare const CreateNodePatch: {
    encode(message: CreateNodePatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): CreateNodePatch;
    fromJSON(object: any): CreateNodePatch;
    toJSON(message: CreateNodePatch): unknown;
    create<I extends Exact<DeepPartial<CreateNodePatch>, I>>(base?: I): CreateNodePatch;
    fromPartial<I extends Exact<DeepPartial<CreateNodePatch>, I>>(object: I): CreateNodePatch;
};
export declare const RemoveNodePatch: {
    encode(message: RemoveNodePatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): RemoveNodePatch;
    fromJSON(object: any): RemoveNodePatch;
    toJSON(message: RemoveNodePatch): unknown;
    create<I extends Exact<DeepPartial<RemoveNodePatch>, I>>(base?: I): RemoveNodePatch;
    fromPartial<I extends Exact<DeepPartial<RemoveNodePatch>, I>>(object: I): RemoveNodePatch;
};
export declare const ReplaceNodePatch: {
    encode(message: ReplaceNodePatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): ReplaceNodePatch;
    fromJSON(object: any): ReplaceNodePatch;
    toJSON(message: ReplaceNodePatch): unknown;
    create<I extends Exact<DeepPartial<ReplaceNodePatch>, I>>(base?: I): ReplaceNodePatch;
    fromPartial<I extends Exact<DeepPartial<ReplaceNodePatch>, I>>(object: I): ReplaceNodePatch;
};
export declare const UpdateAttributesPatch: {
    encode(message: UpdateAttributesPatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): UpdateAttributesPatch;
    fromJSON(object: any): UpdateAttributesPatch;
    toJSON(message: UpdateAttributesPatch): unknown;
    create<I extends Exact<DeepPartial<UpdateAttributesPatch>, I>>(base?: I): UpdateAttributesPatch;
    fromPartial<I extends Exact<DeepPartial<UpdateAttributesPatch>, I>>(object: I): UpdateAttributesPatch;
};
export declare const UpdateAttributesPatch_AttributesEntry: {
    encode(message: UpdateAttributesPatch_AttributesEntry, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): UpdateAttributesPatch_AttributesEntry;
    fromJSON(object: any): UpdateAttributesPatch_AttributesEntry;
    toJSON(message: UpdateAttributesPatch_AttributesEntry): unknown;
    create<I extends Exact<DeepPartial<UpdateAttributesPatch_AttributesEntry>, I>>(base?: I): UpdateAttributesPatch_AttributesEntry;
    fromPartial<I extends Exact<DeepPartial<UpdateAttributesPatch_AttributesEntry>, I>>(object: I): UpdateAttributesPatch_AttributesEntry;
};
export declare const UpdateStylesPatch: {
    encode(message: UpdateStylesPatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): UpdateStylesPatch;
    fromJSON(object: any): UpdateStylesPatch;
    toJSON(message: UpdateStylesPatch): unknown;
    create<I extends Exact<DeepPartial<UpdateStylesPatch>, I>>(base?: I): UpdateStylesPatch;
    fromPartial<I extends Exact<DeepPartial<UpdateStylesPatch>, I>>(object: I): UpdateStylesPatch;
};
export declare const UpdateStylesPatch_StylesEntry: {
    encode(message: UpdateStylesPatch_StylesEntry, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): UpdateStylesPatch_StylesEntry;
    fromJSON(object: any): UpdateStylesPatch_StylesEntry;
    toJSON(message: UpdateStylesPatch_StylesEntry): unknown;
    create<I extends Exact<DeepPartial<UpdateStylesPatch_StylesEntry>, I>>(base?: I): UpdateStylesPatch_StylesEntry;
    fromPartial<I extends Exact<DeepPartial<UpdateStylesPatch_StylesEntry>, I>>(object: I): UpdateStylesPatch_StylesEntry;
};
export declare const UpdateTextPatch: {
    encode(message: UpdateTextPatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): UpdateTextPatch;
    fromJSON(object: any): UpdateTextPatch;
    toJSON(message: UpdateTextPatch): unknown;
    create<I extends Exact<DeepPartial<UpdateTextPatch>, I>>(base?: I): UpdateTextPatch;
    fromPartial<I extends Exact<DeepPartial<UpdateTextPatch>, I>>(object: I): UpdateTextPatch;
};
export declare const AddStyleRulePatch: {
    encode(message: AddStyleRulePatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): AddStyleRulePatch;
    fromJSON(object: any): AddStyleRulePatch;
    toJSON(message: AddStyleRulePatch): unknown;
    create<I extends Exact<DeepPartial<AddStyleRulePatch>, I>>(base?: I): AddStyleRulePatch;
    fromPartial<I extends Exact<DeepPartial<AddStyleRulePatch>, I>>(object: I): AddStyleRulePatch;
};
export declare const RemoveStyleRulePatch: {
    encode(message: RemoveStyleRulePatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): RemoveStyleRulePatch;
    fromJSON(object: any): RemoveStyleRulePatch;
    toJSON(message: RemoveStyleRulePatch): unknown;
    create<I extends Exact<DeepPartial<RemoveStyleRulePatch>, I>>(base?: I): RemoveStyleRulePatch;
    fromPartial<I extends Exact<DeepPartial<RemoveStyleRulePatch>, I>>(object: I): RemoveStyleRulePatch;
};
export declare const MoveChildPatch: {
    encode(message: MoveChildPatch, writer?: _m0.Writer): _m0.Writer;
    decode(input: _m0.Reader | Uint8Array, length?: number): MoveChildPatch;
    fromJSON(object: any): MoveChildPatch;
    toJSON(message: MoveChildPatch): unknown;
    create<I extends Exact<DeepPartial<MoveChildPatch>, I>>(base?: I): MoveChildPatch;
    fromPartial<I extends Exact<DeepPartial<MoveChildPatch>, I>>(object: I): MoveChildPatch;
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
//# sourceMappingURL=patches.d.ts.map