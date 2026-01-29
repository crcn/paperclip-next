/**
 * WebView panel for Paperclip preview
 * Production-hardened with strict CSP and visibility replay
 */

import * as vscode from 'vscode';
import { WorkspaceClient } from './workspace-client';
import { BufferStreamer } from './buffer-streamer';

export class PreviewPanel {
  private panel: vscode.WebviewPanel;
  private streamer: BufferStreamer;
  private lastViewTime: number = Date.now();
  private disposables: vscode.Disposable[] = [];
  private disposeCallbacks: Set<() => void> = new Set();
  private isVisible: boolean = true;
  private queuedUpdates: any[] = [];

  constructor(
    private context: vscode.ExtensionContext,
    private client: WorkspaceClient,
    private document: vscode.TextDocument
  ) {
    // Create WebView panel
    this.panel = vscode.window.createWebviewPanel(
      'paperclipPreview',
      `Preview: ${this.getFileName()}`,
      vscode.ViewColumn.Beside,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: []
      }
    );

    // Set initial HTML
    this.panel.webview.html = this.getWebviewContent();

    // Create buffer streamer
    const debounceMs = vscode.workspace
      .getConfiguration('paperclip')
      .get<number>('previewDebounceMs', 100);

    this.streamer = new BufferStreamer(
      this.client,
      this.document.uri.fsPath,
      (update) => this.handlePreviewUpdate(update),
      debounceMs
    );

    // Send initial content
    this.streamer.updateContent(this.document.getText());

    // Listen to document changes
    this.disposables.push(
      vscode.workspace.onDidChangeTextDocument(e => {
        if (e.document === this.document) {
          this.streamer.updateContent(e.document.getText());
        }
      })
    );

    // Track visibility changes
    this.disposables.push(
      this.panel.onDidChangeViewState(e => {
        const wasVisible = this.isVisible;
        this.isVisible = e.webviewPanel.visible;

        if (this.isVisible && !wasVisible) {
          this.lastViewTime = Date.now();
          this.replayQueuedUpdates();
        }
      })
    );

    // Handle disposal
    this.disposables.push(
      this.panel.onDidDispose(() => {
        this.dispose();
      })
    );
  }

  private getFileName(): string {
    const parts = this.document.uri.fsPath.split('/');
    return parts[parts.length - 1];
  }

  private getWebviewContent(): string {
    const nonce = this.getNonce();

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="
    default-src 'none';
    style-src 'unsafe-inline';
    script-src 'nonce-${nonce}';
  ">
  <title>Paperclip Preview</title>
  <style>
    body {
      margin: 0;
      padding: 0;
      overflow: auto;
      background: white;
    }
    #root {
      min-height: 100vh;
    }
    .error {
      color: red;
      padding: 20px;
      font-family: monospace;
      white-space: pre-wrap;
    }
    .loading {
      padding: 20px;
      color: #666;
    }
  </style>
</head>
<body>
  <div id="root" class="loading">Loading preview...</div>
  <script nonce="${nonce}">
    (function() {
      const vscode = acquireVsCodeApi();
      const root = document.getElementById('root');
      let vdom = null;

      // Listen for preview updates
      window.addEventListener('message', event => {
        const message = event.data;

        if (message.type === 'preview-update') {
          handlePreviewUpdate(message.update);
        }
      });

      function handlePreviewUpdate(update) {
        if (update.error) {
          root.className = 'error';
          root.textContent = update.error;
          return;
        }

        // Apply patches transactionally
        try {
          const newVdom = applyPatches(vdom, update.patches);
          vdom = newVdom;
          render(vdom);
        } catch (error) {
          root.className = 'error';
          root.textContent = 'Patch application failed: ' + error.message;
          console.error('Patch error:', error);
        }
      }

      function applyPatches(currentVdom, patches) {
        let result = currentVdom;

        for (const patch of patches) {
          if (patch.initialize) {
            // Full initialization
            result = patch.initialize.vdom;
          } else if (patch.create_node) {
            result = applyCreateNode(result, patch.create_node);
          } else if (patch.remove_node) {
            result = applyRemoveNode(result, patch.remove_node);
          } else if (patch.replace_node) {
            result = applyReplaceNode(result, patch.replace_node);
          } else if (patch.update_attributes) {
            result = applyUpdateAttributes(result, patch.update_attributes);
          } else if (patch.update_styles) {
            result = applyUpdateStyles(result, patch.update_styles);
          } else if (patch.update_text) {
            result = applyUpdateText(result, patch.update_text);
          }
          // Add other patch types as needed
        }

        return result;
      }

      function applyCreateNode(vdom, patch) {
        // Deep clone to avoid mutation
        const newVdom = JSON.parse(JSON.stringify(vdom));
        const parent = getNodeAtPath(newVdom, patch.path);
        if (!parent || !parent.children) {
          throw new Error('Invalid path for create_node');
        }
        parent.children.splice(patch.index, 0, patch.node);
        return newVdom;
      }

      function applyRemoveNode(vdom, patch) {
        const newVdom = JSON.parse(JSON.stringify(vdom));
        const path = patch.path;
        if (path.length === 0) {
          throw new Error('Cannot remove root node');
        }
        const parentPath = path.slice(0, -1);
        const index = path[path.length - 1];
        const parent = getNodeAtPath(newVdom, parentPath);
        if (!parent || !parent.children) {
          throw new Error('Invalid path for remove_node');
        }
        parent.children.splice(index, 1);
        return newVdom;
      }

      function applyReplaceNode(vdom, patch) {
        const newVdom = JSON.parse(JSON.stringify(vdom));
        const path = patch.path;
        if (path.length === 0) {
          return patch.new_node;
        }
        const parentPath = path.slice(0, -1);
        const index = path[path.length - 1];
        const parent = getNodeAtPath(newVdom, parentPath);
        if (!parent || !parent.children) {
          throw new Error('Invalid path for replace_node');
        }
        parent.children[index] = patch.new_node;
        return newVdom;
      }

      function applyUpdateAttributes(vdom, patch) {
        const newVdom = JSON.parse(JSON.stringify(vdom));
        const node = getNodeAtPath(newVdom, patch.path);
        if (!node) {
          throw new Error('Invalid path for update_attributes');
        }
        node.attributes = { ...node.attributes, ...patch.attributes };
        return newVdom;
      }

      function applyUpdateStyles(vdom, patch) {
        const newVdom = JSON.parse(JSON.stringify(vdom));
        const node = getNodeAtPath(newVdom, patch.path);
        if (!node) {
          throw new Error('Invalid path for update_styles');
        }
        node.styles = { ...node.styles, ...patch.styles };
        return newVdom;
      }

      function applyUpdateText(vdom, patch) {
        const newVdom = JSON.parse(JSON.stringify(vdom));
        const node = getNodeAtPath(newVdom, patch.path);
        if (!node) {
          throw new Error('Invalid path for update_text');
        }
        node.content = patch.content;
        return newVdom;
      }

      function getNodeAtPath(vdom, path) {
        let node = vdom;
        for (const index of path) {
          if (!node || !node.children || !node.children[index]) {
            return null;
          }
          node = node.children[index];
        }
        return node;
      }

      function render(vdom) {
        if (!vdom) {
          root.innerHTML = '<div class="loading">No content</div>';
          return;
        }

        root.className = '';
        root.innerHTML = '';

        if (vdom.body) {
          const bodyEl = renderNode(vdom.body);
          root.appendChild(bodyEl);
        }

        // Apply styles
        if (vdom.styles && vdom.styles.length > 0) {
          const styleEl = document.createElement('style');
          styleEl.textContent = vdom.styles.map(rule => {
            return rule.selector + ' { ' +
              Object.entries(rule.declarations || {})
                .map(([k, v]) => k + ': ' + v)
                .join('; ') +
              ' }';
          }).join('\\n');
          document.head.appendChild(styleEl);
        }
      }

      function renderNode(node) {
        if (node.text) {
          return document.createTextNode(node.text.content || '');
        }

        if (node.element) {
          const el = document.createElement(node.element.tag_name || 'div');

          // Apply attributes
          if (node.element.attributes) {
            for (const [key, value] of Object.entries(node.element.attributes)) {
              el.setAttribute(key, value);
            }
          }

          // Apply inline styles
          if (node.element.styles) {
            for (const [key, value] of Object.entries(node.element.styles)) {
              el.style[key] = value;
            }
          }

          // Render children
          if (node.element.children) {
            for (const child of node.element.children) {
              el.appendChild(renderNode(child));
            }
          }

          return el;
        }

        return document.createTextNode('');
      }

      // Notify extension we're ready
      vscode.postMessage({ type: 'ready' });
    })();
  </script>
</body>
</html>`;
  }

  private getNonce(): string {
    let text = '';
    const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    for (let i = 0; i < 32; i++) {
      text += possible.charAt(Math.floor(Math.random() * possible.length));
    }
    return text;
  }

  private handlePreviewUpdate(update: any): void {
    if (!this.isVisible) {
      // Queue updates when not visible
      this.queuedUpdates.push(update);
      return;
    }

    // Send to WebView
    this.panel.webview.postMessage({
      type: 'preview-update',
      update
    });
  }

  private replayQueuedUpdates(): void {
    if (this.queuedUpdates.length === 0) {
      return;
    }

    console.log(`[PreviewPanel] Replaying ${this.queuedUpdates.length} queued updates`);

    for (const update of this.queuedUpdates) {
      this.panel.webview.postMessage({
        type: 'preview-update',
        update
      });
    }

    this.queuedUpdates = [];
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
    // Dispose streamer
    this.streamer.dispose();

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
