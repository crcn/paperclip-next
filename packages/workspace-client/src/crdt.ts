import * as Y from 'yjs';

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
 * CRDT-backed text document using Yjs.
 *
 * This class wraps a Y.Text document and provides a clean API for:
 * - Text editing (insert, delete, setText)
 * - Origin tracking (to prevent feedback loops in VS Code)
 * - State vector sync (for efficient delta synchronization)
 * - Transaction batching (for grouping rapid edits)
 * - Dirty flag (for AST validity tracking)
 */
export class CRDTDocument {
  private doc: Y.Doc;
  private text: Y.Text;
  private dirty = false;
  private handlers: Set<ChangeHandler> = new Set();
  private currentOrigin: string | null = null;
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

  /**
   * Get the current text content.
   */
  getText(): string {
    return this.text.toString();
  }

  /**
   * Insert text at a position.
   */
  insert(index: number, text: string, options?: ChangeOptions): void {
    const origin = options?.origin ?? 'local';
    this.doc.transact(() => {
      this.text.insert(index, text);
    }, origin);
  }

  /**
   * Delete text starting at a position.
   */
  delete(index: number, length: number, options?: ChangeOptions): void {
    if (length <= 0) return;

    const currentLength = this.text.length;
    if (currentLength === 0 || index >= currentLength) return;

    // Clamp to valid range
    const actualLength = Math.min(length, currentLength - index);
    if (actualLength <= 0) return;

    const origin = options?.origin ?? 'local';
    this.doc.transact(() => {
      this.text.delete(index, actualLength);
    }, origin);
  }

  /**
   * Replace entire document content.
   */
  setText(content: string, options?: ChangeOptions): void {
    const origin = options?.origin ?? 'local';
    this.doc.transact(() => {
      this.text.delete(0, this.text.length);
      if (content.length > 0) {
        this.text.insert(0, content);
      }
    }, origin);
  }

  /**
   * Execute multiple operations as a single transaction.
   * Only emits one change event for the entire batch.
   */
  transaction(fn: () => void, options?: ChangeOptions): void {
    const origin = options?.origin ?? 'local';
    this.doc.transact(fn, origin);
  }

  /**
   * Subscribe to text changes.
   * Returns unsubscribe function.
   */
  onChange(handler: ChangeHandler): () => void {
    this.handlers.add(handler);
    return () => {
      this.handlers.delete(handler);
    };
  }

  /**
   * Get the current state vector for delta sync.
   */
  getStateVector(): Uint8Array {
    return Y.encodeStateVector(this.doc);
  }

  /**
   * Encode the full document state.
   */
  encodeState(): Uint8Array {
    return Y.encodeStateAsUpdate(this.doc);
  }

  /**
   * Encode only changes since a given state vector.
   */
  encodeDelta(stateVector: Uint8Array): Uint8Array {
    return Y.encodeStateAsUpdate(this.doc, stateVector);
  }

  /**
   * Apply an update from another document or server.
   */
  applyUpdate(update: Uint8Array, options?: ChangeOptions): void {
    const origin = options?.origin ?? 'remote';
    Y.applyUpdate(this.doc, update, origin);
  }

  /**
   * Check if document has uncommitted changes (AST may be stale).
   */
  isDirty(): boolean {
    return this.dirty;
  }

  /**
   * Mark document as clean (AST is up to date).
   */
  markClean(): void {
    this.dirty = false;
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
