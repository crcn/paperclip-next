import * as Y from 'yjs';

/**
 * A span in the source text, with both absolute positions and relative positions.
 * RelativePositions survive concurrent edits to other parts of the document.
 */
export interface SourceSpan {
  /** Absolute start position (may be stale after concurrent edits) */
  start: number;
  /** Absolute end position (may be stale after concurrent edits) */
  end: number;
  /** Relative start position (survives concurrent edits) */
  relStart: Y.RelativePosition;
  /** Relative end position (survives concurrent edits) */
  relEnd: Y.RelativePosition;
}

/**
 * A node in the AST with its source span.
 */
export interface ASTNode {
  id: string;
  type: string;
  span: SourceSpan;
  children?: ASTNode[];
  /** For style nodes: property declarations */
  properties?: Map<string, { value: string; span: SourceSpan }>;
  /** For frame annotations */
  frame?: { x: number; y: number; width: number; height: number; span: SourceSpan };
}

/**
 * Result of resolving a RelativePosition.
 */
export interface ResolvedSpan {
  start: number;
  end: number;
  /** Whether the resolution might be inaccurate (target was modified) */
  maybeStale: boolean;
}

export interface TextDelta {
  insert?: string;
  delete?: number;
  retain?: number;
}

export interface ChangeOptions {
  origin?: string;
}

export type ChangeHandler = (delta: TextDelta[], origin: string | null) => void;

/**
 * CRDT document with RelativePosition support for safe mutation translation.
 *
 * This is a standalone implementation that provides all the features needed
 * for the Designer mutation → Y.Text flow.
 */
export class CRDTDocumentWithRelPos {
  private doc: Y.Doc;
  private text: Y.Text;
  private dirty = false;
  private handlers: Set<ChangeHandler> = new Set();
  private disposed = false;

  constructor() {
    this.doc = new Y.Doc();
    this.text = this.doc.getText('content');

    // Listen to Y.Text changes
    this.text.observe((event, transaction) => {
      if (this.disposed) return;

      this.dirty = true;

      // Convert Y.js delta to our format
      const delta = event.delta as TextDelta[];
      const origin = transaction.origin as string | null ?? 'local';

      // Notify all handlers
      for (const handler of this.handlers) {
        handler(delta, origin);
      }
    });
  }

  // ==========================================================================
  // Basic text operations
  // ==========================================================================

  getText(): string {
    return this.text.toString();
  }

  insert(index: number, content: string, options?: ChangeOptions): void {
    const origin = options?.origin ?? 'local';
    this.doc.transact(() => {
      this.text.insert(index, content);
    }, origin);
  }

  delete(index: number, length: number, options?: ChangeOptions): void {
    if (length <= 0) return;
    const currentLength = this.text.length;
    if (currentLength === 0 || index >= currentLength) return;

    const actualLength = Math.min(length, currentLength - index);
    if (actualLength <= 0) return;

    const origin = options?.origin ?? 'local';
    this.doc.transact(() => {
      this.text.delete(index, actualLength);
    }, origin);
  }

  setText(content: string, options?: ChangeOptions): void {
    const origin = options?.origin ?? 'local';
    this.doc.transact(() => {
      this.text.delete(0, this.text.length);
      if (content.length > 0) {
        this.text.insert(0, content);
      }
    }, origin);
  }

  transaction(fn: () => void, options?: ChangeOptions): void {
    const origin = options?.origin ?? 'local';
    this.doc.transact(fn, origin);
  }

  // ==========================================================================
  // Change tracking
  // ==========================================================================

  onChange(handler: ChangeHandler): () => void {
    this.handlers.add(handler);
    return () => {
      this.handlers.delete(handler);
    };
  }

  isDirty(): boolean {
    return this.dirty;
  }

  markClean(): void {
    this.dirty = false;
  }

  // ==========================================================================
  // State sync
  // ==========================================================================

  getStateVector(): Uint8Array {
    return Y.encodeStateVector(this.doc);
  }

  encodeState(): Uint8Array {
    return Y.encodeStateAsUpdate(this.doc);
  }

  encodeDelta(stateVector: Uint8Array): Uint8Array {
    return Y.encodeStateAsUpdate(this.doc, stateVector);
  }

  applyUpdate(update: Uint8Array, options?: ChangeOptions): void {
    const origin = options?.origin ?? 'remote';
    Y.applyUpdate(this.doc, update, origin);
  }

  // ==========================================================================
  // RelativePosition support - THE CRITICAL PART
  // ==========================================================================

  /**
   * Create a RelativePosition for an absolute index.
   * RelativePositions track a logical position in the document history,
   * not a byte offset - they survive concurrent edits.
   */
  createRelativePosition(index: number): Y.RelativePosition {
    return Y.createRelativePositionFromTypeIndex(this.text, index);
  }

  /**
   * Resolve a RelativePosition to an absolute index.
   * Returns null if the position no longer exists (rare edge case).
   */
  resolveRelativePosition(relPos: Y.RelativePosition): number | null {
    const absPos = Y.createAbsolutePositionFromRelativePosition(relPos, this.doc);
    return absPos?.index ?? null;
  }

  /**
   * Create a SourceSpan for a range in the current document state.
   */
  createSpan(start: number, end: number): SourceSpan {
    return {
      start,
      end,
      relStart: this.createRelativePosition(start),
      relEnd: this.createRelativePosition(end),
    };
  }

  /**
   * Resolve a SourceSpan to current absolute positions.
   * Also checks if the content at that span matches expected content.
   */
  resolveSpan(span: SourceSpan, expectedContent?: string): ResolvedSpan | null {
    const start = this.resolveRelativePosition(span.relStart);
    const end = this.resolveRelativePosition(span.relEnd);

    if (start === null || end === null) {
      return null;
    }

    // Check if content matches expected (to detect conflicts)
    let maybeStale = false;
    if (expectedContent !== undefined) {
      const actualContent = this.getText().slice(start, end);
      maybeStale = actualContent !== expectedContent;
    }

    return { start, end, maybeStale };
  }

  /**
   * Apply a text edit at a span, using RelativePositions for safety.
   *
   * @param span - The span to replace
   * @param newText - The new text to insert
   * @param expectedContent - If provided, verifies the span content before editing
   * @returns true if edit succeeded, false if conflict detected
   */
  editAtSpan(
    span: SourceSpan,
    newText: string,
    expectedContent?: string,
    options?: ChangeOptions
  ): boolean {
    const resolved = this.resolveSpan(span, expectedContent);

    if (!resolved) {
      // Position no longer exists
      return false;
    }

    if (resolved.maybeStale) {
      // Content changed - conflict
      return false;
    }

    // Apply the edit
    this.transaction(() => {
      const length = resolved.end - resolved.start;
      if (length > 0) {
        this.delete(resolved.start, length);
      }
      if (newText.length > 0) {
        this.insert(resolved.start, newText);
      }
    }, options);

    return true;
  }

  /**
   * Get the underlying Y.Doc for advanced operations.
   */
  getYDoc(): Y.Doc {
    return this.doc;
  }

  /**
   * Clean up resources.
   */
  dispose(): void {
    this.disposed = true;
    this.handlers.clear();
    this.doc.destroy();
  }
}

/**
 * Mutation types that can be applied to the document.
 */
export type Mutation =
  | { type: 'SetFrameBounds'; nodeId: string; bounds: { x: number; y: number; width: number; height: number } }
  | { type: 'SetStyleProperty'; nodeId: string; property: string; value: string }
  | { type: 'DeleteStyleProperty'; nodeId: string; property: string }
  | { type: 'SetTextContent'; nodeId: string; content: string }
  | { type: 'DeleteNode'; nodeId: string }
  | { type: 'MoveNode'; nodeId: string; newParentId: string; index: number }
  | { type: 'InsertNode'; parentId: string; index: number; source: string };

/**
 * Result of applying a mutation.
 */
export type MutationResult =
  | { success: true }
  | { success: false; reason: 'node_not_found' | 'conflict' | 'invalid_mutation' | 'parse_error' };

/**
 * AST index for looking up nodes by ID.
 */
export class ASTIndex {
  private nodes: Map<string, ASTNode> = new Map();
  private root: ASTNode | null = null;

  constructor(ast?: ASTNode) {
    if (ast) {
      this.rebuild(ast);
    }
  }

  rebuild(ast: ASTNode): void {
    this.nodes.clear();
    this.root = ast;
    this.indexNode(ast);
  }

  private indexNode(node: ASTNode): void {
    this.nodes.set(node.id, node);
    if (node.children) {
      for (const child of node.children) {
        this.indexNode(child);
      }
    }
  }

  getNode(id: string): ASTNode | undefined {
    return this.nodes.get(id);
  }

  getRoot(): ASTNode | null {
    return this.root;
  }

  getAllNodes(): ASTNode[] {
    return Array.from(this.nodes.values());
  }
}

/**
 * Handles applying mutations to Y.Text via AST-aware translation.
 */
export class MutationHandler {
  constructor(
    private doc: CRDTDocumentWithRelPos,
    private astIndex: ASTIndex
  ) {}

  /**
   * Apply a mutation to the document.
   */
  apply(mutation: Mutation, options?: ChangeOptions): MutationResult {
    switch (mutation.type) {
      case 'SetFrameBounds':
        return this.applySetFrameBounds(mutation, options);
      case 'SetStyleProperty':
        return this.applySetStyleProperty(mutation, options);
      case 'DeleteStyleProperty':
        return this.applyDeleteStyleProperty(mutation, options);
      case 'SetTextContent':
        return this.applySetTextContent(mutation, options);
      case 'DeleteNode':
        return this.applyDeleteNode(mutation, options);
      case 'MoveNode':
        return this.applyMoveNode(mutation, options);
      case 'InsertNode':
        return this.applyInsertNode(mutation, options);
      default:
        return { success: false, reason: 'invalid_mutation' };
    }
  }

  private applySetFrameBounds(
    mutation: Extract<Mutation, { type: 'SetFrameBounds' }>,
    options?: ChangeOptions
  ): MutationResult {
    const node = this.astIndex.getNode(mutation.nodeId);
    if (!node || !node.frame) {
      return { success: false, reason: 'node_not_found' };
    }

    const { x, y, width, height } = mutation.bounds;
    const newFrameText = `@frame(x: ${x}, y: ${y}, width: ${width}, height: ${height})`;

    // Get expected content by resolving RelativePositions first
    // This is critical - we must use resolved positions, not stale absolute positions
    const resolved = this.doc.resolveSpan(node.frame.span);
    if (!resolved) {
      return { success: false, reason: 'conflict' };
    }

    const text = this.doc.getText();
    const expectedContent = text.slice(resolved.start, resolved.end);

    const success = this.doc.editAtSpan(node.frame.span, newFrameText, expectedContent, options);
    return success ? { success: true } : { success: false, reason: 'conflict' };
  }

  private applySetStyleProperty(
    mutation: Extract<Mutation, { type: 'SetStyleProperty' }>,
    options?: ChangeOptions
  ): MutationResult {
    const node = this.astIndex.getNode(mutation.nodeId);
    if (!node || !node.properties) {
      return { success: false, reason: 'node_not_found' };
    }

    const prop = node.properties.get(mutation.property);
    if (!prop) {
      // Property doesn't exist - would need to insert
      // For now, return not found
      return { success: false, reason: 'node_not_found' };
    }

    const text = this.doc.getText();
    const expectedContent = text.slice(prop.span.start, prop.span.end);
    const newText = `${mutation.property}: ${mutation.value}`;

    const success = this.doc.editAtSpan(prop.span, newText, expectedContent, options);
    return success ? { success: true } : { success: false, reason: 'conflict' };
  }

  private applyDeleteStyleProperty(
    mutation: Extract<Mutation, { type: 'DeleteStyleProperty' }>,
    options?: ChangeOptions
  ): MutationResult {
    const node = this.astIndex.getNode(mutation.nodeId);
    if (!node || !node.properties) {
      return { success: false, reason: 'node_not_found' };
    }

    const prop = node.properties.get(mutation.property);
    if (!prop) {
      return { success: false, reason: 'node_not_found' };
    }

    const text = this.doc.getText();
    const expectedContent = text.slice(prop.span.start, prop.span.end);

    // Delete the property line (including newline)
    const success = this.doc.editAtSpan(prop.span, '', expectedContent, options);
    return success ? { success: true } : { success: false, reason: 'conflict' };
  }

  private applySetTextContent(
    mutation: Extract<Mutation, { type: 'SetTextContent' }>,
    options?: ChangeOptions
  ): MutationResult {
    const node = this.astIndex.getNode(mutation.nodeId);
    if (!node || node.type !== 'text') {
      return { success: false, reason: 'node_not_found' };
    }

    const text = this.doc.getText();
    const expectedContent = text.slice(node.span.start, node.span.end);

    // Replace the text content (assuming format: text "content")
    const newText = `text "${mutation.content}"`;
    const success = this.doc.editAtSpan(node.span, newText, expectedContent, options);
    return success ? { success: true } : { success: false, reason: 'conflict' };
  }

  private applyDeleteNode(
    mutation: Extract<Mutation, { type: 'DeleteNode' }>,
    options?: ChangeOptions
  ): MutationResult {
    const node = this.astIndex.getNode(mutation.nodeId);
    if (!node) {
      return { success: false, reason: 'node_not_found' };
    }

    const text = this.doc.getText();
    const expectedContent = text.slice(node.span.start, node.span.end);

    const success = this.doc.editAtSpan(node.span, '', expectedContent, options);
    return success ? { success: true } : { success: false, reason: 'conflict' };
  }

  private applyMoveNode(
    mutation: Extract<Mutation, { type: 'MoveNode' }>,
    options?: ChangeOptions
  ): MutationResult {
    const node = this.astIndex.getNode(mutation.nodeId);
    const newParent = this.astIndex.getNode(mutation.newParentId);

    if (!node || !newParent) {
      return { success: false, reason: 'node_not_found' };
    }

    const text = this.doc.getText();
    const nodeContent = text.slice(node.span.start, node.span.end);

    // Find insertion point in new parent
    // This is simplified - real implementation would need proper child index handling
    if (!newParent.children || newParent.children.length === 0) {
      return { success: false, reason: 'invalid_mutation' };
    }

    // Get the target position
    let insertPos: number;
    if (mutation.index >= newParent.children.length) {
      // Insert at end
      const lastChild = newParent.children[newParent.children.length - 1];
      insertPos = lastChild.span.end;
    } else {
      // Insert before the child at index
      insertPos = newParent.children[mutation.index].span.start;
    }

    // Determine order: if source is before target, delete first then insert
    // If source is after target, insert first then delete
    const sourceStart = node.span.start;
    const sourceEnd = node.span.end;

    this.doc.transaction(() => {
      if (sourceStart < insertPos) {
        // Source before target: insert first (at adjusted position), then delete
        // After delete, positions shift, so insert at insertPos - deletedLength
        const adjustedInsertPos = insertPos - (sourceEnd - sourceStart);
        this.doc.delete(sourceStart, sourceEnd - sourceStart);
        this.doc.insert(adjustedInsertPos, nodeContent);
      } else {
        // Source after target: insert first, then delete (at adjusted position)
        this.doc.insert(insertPos, nodeContent);
        // After insert, source positions shift by inserted length
        const adjustedSourceStart = sourceStart + nodeContent.length;
        this.doc.delete(adjustedSourceStart, sourceEnd - sourceStart);
      }
    }, options);

    return { success: true };
  }

  private applyInsertNode(
    mutation: Extract<Mutation, { type: 'InsertNode' }>,
    options?: ChangeOptions
  ): MutationResult {
    const parent = this.astIndex.getNode(mutation.parentId);
    if (!parent) {
      return { success: false, reason: 'node_not_found' };
    }

    // Find insertion point
    let insertPos: number;
    if (!parent.children || parent.children.length === 0) {
      // Insert at parent's content start (after opening brace)
      // This is simplified
      insertPos = parent.span.start + 1;
    } else if (mutation.index >= parent.children.length) {
      const lastChild = parent.children[parent.children.length - 1];
      insertPos = lastChild.span.end;
    } else {
      insertPos = parent.children[mutation.index].span.start;
    }

    this.doc.insert(insertPos, mutation.source, options);
    return { success: true };
  }
}
