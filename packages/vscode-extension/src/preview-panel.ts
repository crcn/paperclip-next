/**
 * WebView panel for Paperclip preview
 *
 * Architecture:
 *   VS Code Extension <-> Workspace Server <-> Designer (in iframe)
 *
 * - VS Code sends buffer content to server via gRPC StreamBuffer
 * - Server evaluates and sends VDOM patches back
 * - Designer in iframe receives VDOM updates from server via SSE
 */

import * as vscode from 'vscode';
import * as grpc from '@grpc/grpc-js';
import { WorkspaceClient, PreviewUpdate, MutationRequest } from './workspace-client';

// Debounce delay for sending updates to server
const DEBOUNCE_MS = 100;

export class PreviewPanel {
  private panel: vscode.WebviewPanel;
  private lastViewTime: number = Date.now();
  private disposables: vscode.Disposable[] = [];
  private disposeCallbacks: Set<() => void> = new Set();
  private isVisible: boolean = true;
  private httpPort: number;
  private currentStream: grpc.ClientReadableStream<any> | null = null;
  private debounceTimer: NodeJS.Timeout | null = null;
  private stateVersion: number = 0;

  constructor(
    private context: vscode.ExtensionContext,
    private client: WorkspaceClient,
    private document: vscode.TextDocument,
    httpPort: number = 3030
  ) {
    this.httpPort = httpPort;

    // Create WebView panel
    this.panel = vscode.window.createWebviewPanel(
      'paperclipPreview',
      `Preview: ${this.getFileName()}`,
      vscode.ViewColumn.Beside,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
      }
    );

    // Set HTML with iframe pointing to designer (async)
    this.setWebviewContent();

    // Track visibility changes
    this.disposables.push(
      this.panel.onDidChangeViewState(e => {
        this.isVisible = e.webviewPanel.visible;
        if (this.isVisible) {
          this.lastViewTime = Date.now();
        }
      })
    );

    // Handle disposal
    this.disposables.push(
      this.panel.onDidDispose(() => {
        this.dispose();
      })
    );

    // Handle messages from webview (including mutations from designer iframe)
    this.disposables.push(
      this.panel.webview.onDidReceiveMessage(async (message) => {
        await this.handleWebviewMessage(message);
      })
    );

    // Start streaming buffer
    this.startStreamBuffer();
  }

  /**
   * Handle messages from the webview (forwarded from designer iframe)
   */
  private async handleWebviewMessage(message: any): Promise<void> {
    console.log('[PreviewPanel] Received webview message:', message);

    if (message.type === 'mutation') {
      const { mutationId, mutationType, payload } = message;
      const filePath = this.document.uri.fsPath;

      const mutationRequest: MutationRequest = {
        clientId: this.client.getClientId(),
        filePath,
        mutation: {
          mutationId,
          timestamp: Date.now(),
        },
      };

      // Build mutation based on type
      if (mutationType === 'SetInlineStyle') {
        mutationRequest.mutation.setInlineStyle = {
          nodeId: payload.node_id,
          property: payload.property,
          value: payload.value,
        };
      } else if (mutationType === 'DeleteInlineStyle') {
        // Delete is a SetInlineStyle with empty value (or we could add a delete mutation type)
        mutationRequest.mutation.setInlineStyle = {
          nodeId: payload.node_id,
          property: payload.property,
          value: '', // Empty value to delete
        };
      } else if (mutationType === 'SetFrameBounds') {
        mutationRequest.mutation.setFrameBounds = {
          frameId: payload.frame_id,
          bounds: payload.bounds,
        };
      } else {
        console.warn('[PreviewPanel] Unknown mutation type:', mutationType);
        return;
      }

      try {
        const result = await this.client.applyMutation(mutationRequest);
        console.log('[PreviewPanel] Mutation result:', result);

        // Send result back to designer iframe
        this.panel.webview.postMessage({
          type: 'mutationResult',
          mutationId,
          success: result.success,
          newVersion: result.newVersion,
          error: result.error,
        });
      } catch (error) {
        console.error('[PreviewPanel] Mutation error:', error);
        this.panel.webview.postMessage({
          type: 'mutationResult',
          mutationId,
          success: false,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }
  }

  private getFileName(): string {
    const parts = this.document.uri.fsPath.split('/');
    return parts[parts.length - 1];
  }

  /**
   * Start streaming buffer to server for live preview.
   */
  private startStreamBuffer(): void {
    const filePath = this.document.uri.fsPath;
    const content = this.document.getText();
    console.log(`[PreviewPanel] Starting stream for ${filePath}`);

    // Cancel any existing stream
    if (this.currentStream) {
      this.currentStream.cancel();
      this.currentStream = null;
    }

    try {
      this.currentStream = this.client.streamBuffer(
        {
          clientId: this.client.getClientId(),
          filePath,
          content,
          expectedStateVersion: this.stateVersion,
        },
        (update: PreviewUpdate) => {
          this.stateVersion = update.version;
          // Designer in iframe receives updates via SSE, nothing to do here
        }
      );
    } catch (error) {
      console.error('[PreviewPanel] Failed to start stream:', error);
    }

    // Listen for document changes
    this.disposables.push(
      vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document !== this.document) return;
        this.scheduleUpdate();
      })
    );
  }

  private scheduleUpdate(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    this.debounceTimer = setTimeout(() => {
      this.debounceTimer = null;
      this.startStreamBuffer();
    }, DEBOUNCE_MS);
  }

  private async setWebviewContent(): Promise<void> {
    this.panel.webview.html = await this.getWebviewContent();
  }

  private async getWebviewContent(): Promise<string> {
    const filePath = this.document.uri.fsPath;

    // Use asExternalUri for proper VSCode/Codespaces authorization
    const designerHost = await vscode.env.asExternalUri(
      vscode.Uri.parse(`http://localhost:${this.httpPort}`)
    );
    // Add timestamp to bust VSCode webview cache
    const cacheBust = Date.now();
    const designerUrl = `${designerHost}?file=${encodeURIComponent(filePath)}&_t=${cacheBust}`;

    console.log(`[PreviewPanel] Opening preview: ${designerUrl}`);

    // Create iframe with CSP that allows localhost
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; frame-src ${designerHost} http://localhost:* https://localhost:*; style-src 'unsafe-inline'; script-src 'unsafe-inline';">
  <style>
    html, body {
      margin: 0;
      padding: 0;
      width: 100vw;
      height: 100vh;
      overflow: hidden;
      background: white;
    }
    .loader {
      width: 12px;
      height: 12px;
      background: #000000;
      opacity: 0.3;
      border-radius: 50%;
      box-shadow: 20px 0 #00000022, -20px 0 #000000;
      animation: loader 1s infinite linear alternate;
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      z-index: -1;
    }
    @keyframes loader {
      0% { box-shadow: 20px 0 #000000, -20px 0 #00000022; background: #000000 }
      33% { box-shadow: 20px 0 #000000, -20px 0 #00000022; background: #00000022 }
      66% { box-shadow: 20px 0 #00000022, -20px 0 #000000; background: #00000022 }
    }
  </style>
</head>
<body>
  <div class="loader"></div>
</body>
<script>
  // Get VSCode webview API for messaging
  const vscode = acquireVsCodeApi();

  const iframe = document.createElement("iframe");
  iframe.src = "${designerUrl}";
  Object.assign(iframe.style, {
    width: "100vw",
    height: "100vh",
    border: "none",
    background: "transparent",
    position: "absolute",
    top: 0,
    left: 0
  });
  document.body.appendChild(iframe);

  // Handle all messages
  window.addEventListener("message", (event) => {
    console.log("[Webview] Received message:", event.data, "source:", event.source === iframe.contentWindow ? "iframe" : event.source === window ? "window" : "unknown");

    // Message from iframe (designer) - forward to VSCode extension
    if (event.source === iframe.contentWindow) {
      if (event.data && event.data.type === "mutation") {
        console.log("[Webview] Forwarding mutation to extension:", event.data);
        vscode.postMessage(event.data);
      }
      return;
    }

    // Message from VSCode extension - forward to iframe
    if (event.data && event.data.type === "mutationResult") {
      console.log("[Webview] Forwarding mutation result to iframe:", event.data);
      iframe.contentWindow?.postMessage(event.data, "*");
    }
  });
</script>
</html>`;
  }

  async updateFilePath(document: vscode.TextDocument): Promise<void> {
    // Cancel existing stream
    if (this.currentStream) {
      this.currentStream.cancel();
      this.currentStream = null;
    }

    this.document = document;
    this.stateVersion = 0;
    this.panel.webview.html = await this.getWebviewContent();
    this.panel.title = `Preview: ${this.getFileName()}`;

    // Start new stream for new document
    this.startStreamBuffer();
  }

  reveal(): void {
    this.panel.reveal();
    this.lastViewTime = Date.now();
  }

  getLastViewTime(): number {
    return this.lastViewTime;
  }

  onDispose(callback: () => void): void {
    this.disposeCallbacks.add(callback);
  }

  dispose(): void {
    // Cancel stream
    if (this.currentStream) {
      this.currentStream.cancel();
      this.currentStream = null;
    }

    // Clear debounce timer
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }

    // Dispose panel
    this.panel.dispose();

    // Dispose listeners
    for (const disposable of this.disposables) {
      disposable.dispose();
    }
    this.disposables = [];

    // Notify callbacks
    this.disposeCallbacks.forEach(cb => {
      try {
        cb();
      } catch (error) {
        console.error('[PreviewPanel] Dispose callback error:', error);
      }
    });
    this.disposeCallbacks.clear();
  }
}
