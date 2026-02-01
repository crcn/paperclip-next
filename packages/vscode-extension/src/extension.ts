/**
 * Paperclip VSCode Extension
 * Entry point for extension activation
 */

import * as vscode from 'vscode';
import { WorkspaceClient } from './workspace-client';
import { PreviewManager } from './preview-manager';
import { ServerManager } from './server-manager';

let serverManager: ServerManager | null = null;
let client: WorkspaceClient | null = null;
let previewManager: PreviewManager | null = null;

export async function activate(context: vscode.ExtensionContext) {
  console.log('Paperclip extension is now active');

  // Register preview command immediately (before server connection)
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
        vscode.window.showErrorMessage(
          'Paperclip server not connected. Make sure the server is running.'
        );
        return;
      }

      await previewManager.openPreview(document);
    }
  );

  context.subscriptions.push(previewCommand);

  try {
    // Get configuration
    const config = vscode.workspace.getConfiguration('paperclip');
    const serverPort = config.get<number>('serverPort', 50051);
    const httpPort = config.get<number>('httpPort', 3030);

    // Start the server (or connect if already running)
    serverManager = new ServerManager(serverPort, httpPort);
    await serverManager.start();

    // Initialize workspace client
    const serverAddress = `localhost:${serverPort}`;
    client = new WorkspaceClient(serverAddress);
    await client.connect();

    // Initialize preview manager with HTTP port for designer iframe
    previewManager = new PreviewManager(context, client, serverManager.getHttpPort());

    // Show connection status
    vscode.window.showInformationMessage('Paperclip: Ready');

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

  // Stop server
  if (serverManager) {
    await serverManager.stop();
    serverManager = null;
  }
}
