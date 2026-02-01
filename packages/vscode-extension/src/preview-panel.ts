/**
 * WebView panel for Paperclip preview
 *
 * Architecture:
 *   VS Code Extension <-> Workspace Server <-> Designer (in iframe)
 *
 * - VS Code sends buffer changes to workspace server via gRPC
 * - Designer in iframe receives updates from server via SSE
 */

import * as vscode from 'vscode';
import { WorkspaceClient } from './workspace-client';
import * as grpc from '@grpc/grpc-js';

export class PreviewPanel {
  private panel: vscode.WebviewPanel;
  private lastViewTime: number = Date.now();
  private disposables: vscode.Disposable[] = [];
  private disposeCallbacks: Set<() => void> = new Set();
  private isVisible: boolean = true;
  private httpPort: number;
  private currentStream: grpc.ClientReadableStream<any> | null = null;
  private debounceTimer: NodeJS.Timeout | null = null;

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

    // Set HTML with iframe pointing to designer
    this.panel.webview.html = this.getWebviewContent();

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

    // Listen for document changes and send to server
    this.disposables.push(
      vscode.workspace.onDidChangeTextDocument(e => {
        if (e.document.uri.toString() === this.document.uri.toString()) {
          this.onDocumentChange();
        }
      })
    );

    // Send initial buffer content
    this.sendBufferUpdate();
  }

  private getFileName(): string {
    const parts = this.document.uri.fsPath.split('/');
    return parts[parts.length - 1];
  }

  private onDocumentChange(): void {
    // Debounce rapid changes (100ms)
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    this.debounceTimer = setTimeout(() => {
      this.sendBufferUpdate();
    }, 100);
  }

  private sendBufferUpdate(): void {
    // Cancel previous stream
    if (this.currentStream) {
      this.currentStream.cancel();
      this.currentStream = null;
    }

    const content = this.document.getText();
    const filePath = this.document.uri.fsPath;

    console.log(`[PreviewPanel] Sending buffer update for ${filePath}, ${content.length} chars`);

    try {
      this.currentStream = this.client.streamBuffer(
        {
          clientId: this.client.getClientId(),
          filePath,
          content,
        },
        (update) => {
          // Server processes the buffer - designer will get update via SSE
          console.log(`[PreviewPanel] Buffer processed v${update.version}`);
        }
      );
    } catch (error) {
      console.error('[PreviewPanel] Failed to send buffer update:', error);
    }
  }

  private getWebviewContent(): string {
    const filePath = this.document.uri.fsPath;
    const designerUrl = `http://localhost:${this.httpPort}?file=${encodeURIComponent(filePath)}`;

    // Create iframe via JavaScript to avoid CSP issues
    return `<!DOCTYPE html>
<html lang="en">
<head>
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
</script>
</html>`;
  }

  updateFilePath(document: vscode.TextDocument): void {
    this.document = document;
    this.panel.webview.html = this.getWebviewContent();
    this.panel.title = `Preview: ${this.getFileName()}`;
    this.sendBufferUpdate();
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
