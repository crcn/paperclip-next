import { CRDTDocument, TextDelta, ChangeOptions, ChangeHandler } from './crdt';

/**
 * Transport interface for server communication.
 * Implementations can use gRPC, WebSocket, etc.
 */
export interface SyncTransport {
  /**
   * Send a CRDT update to the server.
   */
  sendUpdate(filePath: string, update: Uint8Array, stateVector: Uint8Array): Promise<void>;

  /**
   * Subscribe to CRDT updates from server.
   */
  onUpdate(callback: (update: Uint8Array) => void): () => void;

  /**
   * Subscribe to VDOM updates from server.
   */
  onVDOM(callback: (vdom: any) => void): () => void;

  /**
   * Subscribe to CSSOM updates from server.
   */
  onCSSOM(callback: (cssom: any) => void): () => void;
}

export type VDOMHandler = (vdom: any) => void;
export type CSSOMHandler = (cssom: any) => void;

/**
 * A session for editing a single file with CRDT synchronization.
 *
 * This class combines:
 * - CRDT document (for collaborative editing)
 * - Server sync (for persistence and broadcasting)
 * - VDOM/CSSOM rendering (from server-side evaluation)
 *
 * Usage:
 *   const session = await client.openFile('/path/to/file.pc');
 *   session.insert(0, '<div>');
 *   session.onVDOMChange((vdom) => render(vdom));
 */
export class DocumentSession {
  private crdt: CRDTDocument;
  private vdom: any = null;
  private cssom: any = null;
  private vdomHandlers = new Set<VDOMHandler>();
  private cssomHandlers = new Set<CSSOMHandler>();
  private textHandlers = new Set<ChangeHandler>();
  private disposed = false;
  private sendPending = false;
  private sendTimer: ReturnType<typeof setTimeout> | null = null;
  private lastSentStateVector: Uint8Array | null = null;
  private unsubscribers: Array<() => void> = [];

  // Debounce interval for batching rapid edits
  private static readonly SEND_DEBOUNCE_MS = 5;

  constructor(
    private filePath: string,
    private transport: SyncTransport
  ) {
    this.crdt = new CRDTDocument();

    // Subscribe to local CRDT changes
    const unsubCrdt = this.crdt.onChange((delta, origin) => {
      if (this.disposed) return;

      // Notify text handlers
      for (const handler of this.textHandlers) {
        handler(delta, origin);
      }

      // Only send to server for local changes
      if (origin !== 'remote') {
        this.scheduleSend();
      }
    });
    this.unsubscribers.push(unsubCrdt);

    // Subscribe to server updates
    const unsubUpdate = transport.onUpdate((update) => {
      if (this.disposed) return;
      this.crdt.applyUpdate(update, { origin: 'remote' });
    });
    this.unsubscribers.push(unsubUpdate);

    // Subscribe to VDOM updates
    const unsubVdom = transport.onVDOM((vdom) => {
      if (this.disposed) return;
      this.vdom = vdom;
      for (const handler of this.vdomHandlers) {
        handler(vdom);
      }
    });
    this.unsubscribers.push(unsubVdom);

    // Subscribe to CSSOM updates
    const unsubCssom = transport.onCSSOM((cssom) => {
      if (this.disposed) return;
      this.cssom = cssom;
      for (const handler of this.cssomHandlers) {
        handler(cssom);
      }
    });
    this.unsubscribers.push(unsubCssom);
  }

  // ============ TEXT EDITING ============

  /**
   * Get current text content.
   */
  getText(): string {
    return this.crdt.getText();
  }

  /**
   * Insert text at position.
   */
  insert(index: number, text: string, options?: ChangeOptions): void {
    this.crdt.insert(index, text, options);
  }

  /**
   * Delete text at position.
   */
  delete(index: number, length: number, options?: ChangeOptions): void {
    this.crdt.delete(index, length, options);
  }

  /**
   * Replace entire document content.
   */
  setText(content: string, options?: ChangeOptions): void {
    this.crdt.setText(content, options);
  }

  /**
   * Execute multiple operations as a single transaction.
   */
  transaction(fn: () => void, options?: ChangeOptions): void {
    this.crdt.transaction(fn, options);
  }

  /**
   * Subscribe to text changes.
   */
  onTextChange(handler: ChangeHandler): () => void {
    this.textHandlers.add(handler);
    return () => {
      this.textHandlers.delete(handler);
    };
  }

  // ============ RENDERING ============

  /**
   * Get current VDOM (may be null if not yet received).
   */
  getVDOM(): any {
    return this.vdom;
  }

  /**
   * Get current CSSOM (may be null if not yet received).
   */
  getCSSOM(): any {
    return this.cssom;
  }

  /**
   * Subscribe to VDOM changes from server.
   */
  onVDOMChange(handler: VDOMHandler): () => void {
    this.vdomHandlers.add(handler);
    return () => {
      this.vdomHandlers.delete(handler);
    };
  }

  /**
   * Subscribe to CSSOM changes from server.
   */
  onCSSOMChange(handler: CSSOMHandler): () => void {
    this.cssomHandlers.add(handler);
    return () => {
      this.cssomHandlers.delete(handler);
    };
  }

  // ============ SYNC ============

  /**
   * Get underlying CRDT document for advanced operations.
   */
  getDocument(): CRDTDocument {
    return this.crdt;
  }

  /**
   * Apply an update directly (for multi-client sync testing).
   */
  applyUpdate(update: Uint8Array): void {
    this.crdt.applyUpdate(update, { origin: 'remote' });
  }

  /**
   * Check if document has unsynced changes.
   */
  isDirty(): boolean {
    return this.crdt.isDirty();
  }

  /**
   * Mark document as clean (after server confirms AST is valid).
   */
  markClean(): void {
    this.crdt.markClean();
  }

  // ============ INTERNAL ============

  private scheduleSend(): void {
    if (this.sendPending) return;
    this.sendPending = true;

    if (this.sendTimer) {
      clearTimeout(this.sendTimer);
    }

    this.sendTimer = setTimeout(() => {
      this.sendTimer = null;
      this.sendPending = false;
      this.doSend();
    }, DocumentSession.SEND_DEBOUNCE_MS);
  }

  private doSend(): void {
    if (this.disposed) return;

    const stateVector = this.crdt.getStateVector();
    let update: Uint8Array;

    if (this.lastSentStateVector) {
      // Send delta since last sync
      update = this.crdt.encodeDelta(this.lastSentStateVector);
    } else {
      // First sync - send full state
      update = this.crdt.encodeState();
    }

    this.lastSentStateVector = stateVector;

    // Fire and forget - transport handles errors
    this.transport.sendUpdate(this.filePath, update, stateVector).catch(err => {
      console.error('[DocumentSession] Failed to send update:', err);
    });
  }

  /**
   * Clean up resources.
   */
  dispose(): void {
    this.disposed = true;

    if (this.sendTimer) {
      clearTimeout(this.sendTimer);
      this.sendTimer = null;
    }

    for (const unsub of this.unsubscribers) {
      unsub();
    }
    this.unsubscribers = [];

    this.vdomHandlers.clear();
    this.cssomHandlers.clear();
    this.textHandlers.clear();

    this.crdt.dispose();
  }
}
