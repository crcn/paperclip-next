/**
 * VS Code CRDT synchronization bridge.
 *
 * Uses workspace-client's CrdtGrpcTransport and DocumentSession.
 * Only the VS Code-specific bridging logic lives here.
 */

import * as vscode from 'vscode';
import {
  DocumentSession,
  CRDTDocument,
  CrdtGrpcTransport,
  CrdtGrpcClient,
} from '@paperclip/workspace-client';
import { WorkspaceClient } from './workspace-client';

/**
 * Bridges a VS Code TextDocument with workspace-client's DocumentSession.
 * Provides instant local feedback while syncing with the server.
 */
export class VSCodeDocumentBridge {
  private session: DocumentSession;
  private transport: CrdtGrpcTransport;
  private disposables: vscode.Disposable[] = [];
  private unsubscribers: Array<() => void> = [];

  // Prevent infinite loops when applying remote changes
  private applyingRemoteCount = 0;

  constructor(
    private document: vscode.TextDocument,
    client: WorkspaceClient
  ) {
    const filePath = document.uri.fsPath;
    const clientId = client.getClientId();

    // Get the raw gRPC client and create adapter for workspace-client's interface
    const rawClient = client.getRawClient();
    const grpcClientAdapter: CrdtGrpcClient = {
      crdtSync: () => rawClient.CrdtSync(),
    };

    // Create transport using workspace-client's implementation
    this.transport = new CrdtGrpcTransport({
      client: grpcClientAdapter,
      clientId,
      filePath,
    });

    // Create session using workspace-client's DocumentSession
    this.session = new DocumentSession(filePath, this.transport);

    this.setupBridge();
  }

  /**
   * Initialize the sync session.
   */
  async initialize(): Promise<void> {
    // Connect and join the CRDT session
    await this.transport.connect();

    // Initialize CRDT with current document content
    const content = this.document.getText();
    if (content.length > 0) {
      this.session.setText(content, { origin: 'init' });
    }
  }

  private setupBridge(): void {
    // VS Code -> CRDT: Listen to document changes
    this.disposables.push(
      vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document !== this.document) return;
        if (this.applyingRemoteCount > 0) return;

        // Apply each change to the CRDT session
        this.session.transaction(() => {
          for (const change of e.contentChanges) {
            const offset = change.rangeOffset;

            if (change.rangeLength > 0) {
              this.session.delete(offset, change.rangeLength);
            }

            if (change.text) {
              this.session.insert(offset, change.text);
            }
          }
        });
      })
    );

    // CRDT -> VS Code: Listen to remote text changes
    const unsubText = this.session.onTextChange((delta, origin) => {
      if (origin !== 'remote') return;
      this.syncToVSCode();
    });
    this.unsubscribers.push(unsubText);
  }

  /**
   * Sync CRDT content to VS Code document.
   */
  private async syncToVSCode(): Promise<void> {
    const newContent = this.session.getText();
    const currentContent = this.document.getText();

    if (newContent === currentContent) return;

    this.applyingRemoteCount++;

    try {
      const edit = new vscode.WorkspaceEdit();
      const fullRange = new vscode.Range(
        this.document.positionAt(0),
        this.document.positionAt(currentContent.length)
      );
      edit.replace(this.document.uri, fullRange, newContent);

      await vscode.workspace.applyEdit(edit);
    } finally {
      this.applyingRemoteCount--;
    }
  }

  /**
   * Get the underlying DocumentSession.
   */
  getSession(): DocumentSession {
    return this.session;
  }

  /**
   * Subscribe to VDOM updates.
   */
  onVDOMChange(callback: (vdom: any) => void): () => void {
    return this.session.onVDOMChange(callback);
  }

  /**
   * Clean up resources.
   */
  dispose(): void {
    for (const disposable of this.disposables) {
      disposable.dispose();
    }
    this.disposables.length = 0;

    for (const unsub of this.unsubscribers) {
      unsub();
    }
    this.unsubscribers.length = 0;

    this.session.dispose();
    this.transport.disconnect();
  }
}
