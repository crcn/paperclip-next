/**
 * Paperclip VSCode Extension
 * Entry point for extension activation
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { WorkspaceClient } from './workspace-client';
import { PreviewManager } from './preview-manager';

let client: WorkspaceClient | null = null;
let previewManager: PreviewManager | null = null;

export async function activate(context: vscode.ExtensionContext) {
  console.log('Paperclip extension is now active');

  try {
    // Get configuration
    const config = vscode.workspace.getConfiguration('paperclip');
    const serverPort = config.get<number>('serverPort', 50051);
    const serverAddress = `localhost:${serverPort}`;

    // Locate proto file
    const protoPath = path.join(
      context.extensionPath,
      '..',
      '..',
      'proto',
      'workspace.proto'
    );

    // Initialize workspace client
    client = new WorkspaceClient(serverAddress, protoPath);
    await client.connect();

    // Initialize preview manager
    previewManager = new PreviewManager(context, client);

    // Register preview command
    const previewCommand = vscode.commands.registerCommand(
      'paperclip.preview',
      async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showWarningMessage('No active editor');
          return;
        }

        const document = editor.document;
        if (!document.fileName.endsWith('.pc')) {
          vscode.window.showWarningMessage('Not a Paperclip (.pc) file');
          return;
        }

        if (!previewManager) {
          vscode.window.showErrorMessage('Preview manager not initialized');
          return;
        }

        await previewManager.openPreview(document);
      }
    );

    context.subscriptions.push(previewCommand);

    // Show connection status
    vscode.window.showInformationMessage('Paperclip: Connected to server');

    // Monitor connection state
    client.onConnectionStateChange(connected => {
      if (connected) {
        vscode.window.showInformationMessage('Paperclip: Reconnected to server');
      } else {
        vscode.window.showWarningMessage('Paperclip: Disconnected from server');
      }
    });

  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    vscode.window.showErrorMessage(`Paperclip: Failed to start - ${message}`);
    console.error('Paperclip activation error:', error);
  }
}

export async function deactivate() {
  console.log('Paperclip extension is now deactivated');

  // Dispose preview manager
  if (previewManager) {
    previewManager.disposeAll();
    previewManager = null;
  }

  // Dispose client
  if (client) {
    await client.dispose();
    client = null;
  }
}
