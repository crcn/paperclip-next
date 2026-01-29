/**
 * Preview pool manager
 * Enforces max concurrent previews and manages lifecycle
 */

import * as vscode from 'vscode';
import { WorkspaceClient } from './workspace-client';
import { PreviewPanel } from './preview-panel';

export class PreviewManager {
  private previews: Map<string, PreviewPanel> = new Map();
  private maxPreviews: number;

  constructor(
    private context: vscode.ExtensionContext,
    private client: WorkspaceClient
  ) {
    this.maxPreviews = vscode.workspace
      .getConfiguration('paperclip')
      .get<number>('maxPreviewPanels', 10);

    // Watch for config changes
    context.subscriptions.push(
      vscode.workspace.onDidChangeConfiguration(e => {
        if (e.affectsConfiguration('paperclip.maxPreviewPanels')) {
          this.maxPreviews = vscode.workspace
            .getConfiguration('paperclip')
            .get<number>('maxPreviewPanels', 10);
          this.enforceMaxPreviews();
        }
      })
    );
  }

  /**
   * Open or focus preview for document
   */
  async openPreview(document: vscode.TextDocument): Promise<void> {
    const uri = document.uri.toString();

    // If already exists, focus it
    const existing = this.previews.get(uri);
    if (existing) {
      existing.reveal();
      return;
    }

    // Enforce max previews before creating new one
    this.enforceMaxPreviews();

    // Create new preview
    const panel = new PreviewPanel(
      this.context,
      this.client,
      document
    );

    // Track preview
    this.previews.set(uri, panel);

    // Remove from map when disposed
    panel.onDispose(() => {
      this.previews.delete(uri);
    });
  }

  /**
   * Enforce max preview limit by closing least recently used
   */
  private enforceMaxPreviews(): void {
    if (this.previews.size < this.maxPreviews) {
      return;
    }

    // Find least recently viewed panel
    let lruPanel: PreviewPanel | null = null;
    let lruTime = Infinity;

    for (const panel of this.previews.values()) {
      const viewTime = panel.getLastViewTime();
      if (viewTime < lruTime) {
        lruTime = viewTime;
        lruPanel = panel;
      }
    }

    // Close LRU panel
    if (lruPanel) {
      console.log('[PreviewManager] Closing LRU panel to enforce max previews');
      lruPanel.dispose();
    }
  }

  /**
   * Close all previews
   */
  disposeAll(): void {
    for (const panel of this.previews.values()) {
      panel.dispose();
    }
    this.previews.clear();
  }

  /**
   * Get active preview count
   */
  getActiveCount(): number {
    return this.previews.size;
  }
}
