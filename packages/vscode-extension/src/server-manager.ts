/**
 * Server Manager - Spawns and manages the paperclip-server process
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { spawn, ChildProcess } from 'child_process';
import * as net from 'net';

export class ServerManager {
  private serverProcess: ChildProcess | null = null;
  private outputChannel: vscode.OutputChannel;
  private port: number;
  private httpPort: number;
  private isShuttingDown = false;

  constructor(port: number, httpPort: number = 3030) {
    this.port = port;
    this.httpPort = httpPort;
    this.outputChannel = vscode.window.createOutputChannel('Paperclip Server');
  }

  /**
   * Start the server, returns when server is ready
   */
  async start(): Promise<void> {
    // Check if already running on this port
    if (await this.isPortInUse(this.port)) {
      this.outputChannel.appendLine(`Server already running on port ${this.port}`);
      return;
    }

    const serverPath = await this.findServerBinary();
    if (!serverPath) {
      throw new Error(
        'Could not find paperclip-server binary. Run "cargo build --release" first.'
      );
    }

    // Find designer directory (built web app)
    const designerDir = await this.findDesignerDir();

    this.outputChannel.appendLine(`Starting server: ${serverPath}`);
    this.outputChannel.appendLine(`gRPC Port: ${this.port}`);
    this.outputChannel.appendLine(`HTTP Port: ${this.httpPort}`);
    if (designerDir) {
      this.outputChannel.appendLine(`Designer Dir: ${designerDir}`);
    }
    this.outputChannel.show(true);

    // Get workspace root
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();

    // Build command args
    const args = [
      '--port', String(this.port),
      '--http-port', String(this.httpPort),
    ];
    if (designerDir) {
      args.push('--designer-dir', designerDir);
    }
    args.push(workspaceRoot);

    // Spawn server process
    this.serverProcess = spawn(serverPath, args, {
      stdio: ['ignore', 'pipe', 'pipe'],
      detached: false,
    });

    // Pipe stdout/stderr to output channel
    this.serverProcess.stdout?.on('data', (data) => {
      this.outputChannel.append(data.toString());
    });

    this.serverProcess.stderr?.on('data', (data) => {
      this.outputChannel.append(data.toString());
    });

    this.serverProcess.on('error', (err) => {
      this.outputChannel.appendLine(`Server error: ${err.message}`);
      vscode.window.showErrorMessage(`Paperclip server error: ${err.message}`);
    });

    this.serverProcess.on('exit', (code, signal) => {
      if (!this.isShuttingDown) {
        this.outputChannel.appendLine(`Server exited with code ${code}, signal ${signal}`);
        if (code !== 0) {
          vscode.window.showWarningMessage(
            `Paperclip server exited unexpectedly (code ${code})`
          );
        }
      }
      this.serverProcess = null;
    });

    // Wait for server to be ready
    await this.waitForPort(this.port, 10000);
    this.outputChannel.appendLine('Server ready!');
  }

  /**
   * Stop the server
   */
  async stop(): Promise<void> {
    this.isShuttingDown = true;

    if (this.serverProcess) {
      this.outputChannel.appendLine('Stopping server...');

      // Try graceful shutdown first
      this.serverProcess.kill('SIGTERM');

      // Wait a bit, then force kill if needed
      await new Promise<void>((resolve) => {
        const timeout = setTimeout(() => {
          if (this.serverProcess) {
            this.serverProcess.kill('SIGKILL');
          }
          resolve();
        }, 3000);

        this.serverProcess?.on('exit', () => {
          clearTimeout(timeout);
          resolve();
        });
      });

      this.serverProcess = null;
      this.outputChannel.appendLine('Server stopped');
    }

    this.isShuttingDown = false;
  }

  /**
   * Find the paperclip-server binary
   */
  private async findServerBinary(): Promise<string | null> {
    // Resolve symlinks to get the real path (important when extension is symlinked for dev)
    const realDirname = fs.realpathSync(__dirname);
    this.outputChannel.appendLine(`__dirname: ${__dirname}`);
    this.outputChannel.appendLine(`realDirname: ${realDirname}`);

    // Strategy 1: Check relative to extension (monorepo development)
    // realDirname is packages/vscode-extension/out, so we need ../../.. to get to repo root
    const monorepoRelease = path.join(realDirname, '..', '..', '..', 'target', 'release', 'paperclip-server');
    const monorepoDebug = path.join(realDirname, '..', '..', '..', 'target', 'debug', 'paperclip-server');

    this.outputChannel.appendLine(`Looking for server at: ${monorepoRelease}`);
    this.outputChannel.appendLine(`Exists: ${fs.existsSync(monorepoRelease)}`);

    if (fs.existsSync(monorepoRelease)) {
      return monorepoRelease;
    }
    if (fs.existsSync(monorepoDebug)) {
      return monorepoDebug;
    }

    // Strategy 2: Check configured path
    const config = vscode.workspace.getConfiguration('paperclip');
    const configuredPath = config.get<string>('serverPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
      return configuredPath;
    }

    // Strategy 3: Check PATH
    const pathDirs = (process.env.PATH || '').split(path.delimiter);
    for (const dir of pathDirs) {
      const binPath = path.join(dir, 'paperclip-server');
      if (fs.existsSync(binPath)) {
        return binPath;
      }
    }

    return null;
  }

  /**
   * Find the designer app directory (built web app)
   */
  private async findDesignerDir(): Promise<string | null> {
    const realDirname = fs.realpathSync(__dirname);

    // Strategy 1: Check relative to extension (monorepo development)
    // realDirname is packages/vscode-extension/out, so we need ../../designer/dist-web
    const monorepoDesigner = path.join(realDirname, '..', '..', 'designer', 'dist-web');

    this.outputChannel.appendLine(`Looking for designer at: ${monorepoDesigner}`);
    if (fs.existsSync(monorepoDesigner)) {
      return monorepoDesigner;
    }

    // Strategy 2: Check configured path
    const config = vscode.workspace.getConfiguration('paperclip');
    const configuredPath = config.get<string>('designerPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
      return configuredPath;
    }

    this.outputChannel.appendLine('Designer directory not found');
    return null;
  }

  /**
   * Check if a port is in use
   */
  private isPortInUse(port: number): Promise<boolean> {
    return new Promise((resolve) => {
      const server = net.createServer();
      server.once('error', () => resolve(true));
      server.once('listening', () => {
        server.close();
        resolve(false);
      });
      server.listen(port, '127.0.0.1');
    });
  }

  /**
   * Wait for a port to become available
   */
  private waitForPort(port: number, timeoutMs: number): Promise<void> {
    return new Promise((resolve, reject) => {
      const startTime = Date.now();

      const tryConnect = () => {
        const socket = new net.Socket();

        socket.once('connect', () => {
          socket.destroy();
          resolve();
        });

        socket.once('error', () => {
          socket.destroy();

          if (Date.now() - startTime > timeoutMs) {
            reject(new Error(`Timeout waiting for server on port ${port}`));
          } else {
            setTimeout(tryConnect, 100);
          }
        });

        socket.connect(port, '127.0.0.1');
      };

      tryConnect();
    });
  }

  /**
   * Get the gRPC port
   */
  getPort(): number {
    return this.port;
  }

  /**
   * Get the HTTP port (for designer app)
   */
  getHttpPort(): number {
    return this.httpPort;
  }

  /**
   * Check if server is running
   */
  isRunning(): boolean {
    return this.serverProcess !== null;
  }

  /**
   * Dispose resources
   */
  dispose(): void {
    this.stop();
    this.outputChannel.dispose();
  }
}
