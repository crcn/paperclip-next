/**
 * Shared gRPC client for Paperclip workspace service
 * Production-hardened with automatic reconnection and backoff
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import * as path from 'path';
import { createRequire } from 'module';

/**
 * Resolve proto files from @paperclip/proto package
 */
function resolveProtoPath(): { protoPath: string; includePath: string } {
  try {
    // For CommonJS (VS Code extensions)
    const protoPackagePath = path.dirname(require.resolve('@paperclip/proto/package.json'));
    return {
      protoPath: path.join(protoPackagePath, 'src', 'workspace.proto'),
      includePath: path.join(protoPackagePath, 'src'),
    };
  } catch {
    // Fallback - shouldn't happen if @paperclip/proto is installed
    throw new Error('@paperclip/proto package not found. Make sure it is installed.');
  }
}

// Production constants
const INITIAL_BACKOFF_MS = 1000;
const MAX_BACKOFF_MS = 30000;
const JITTER_FACTOR = 0.3;
const HEARTBEAT_INTERVAL_MS = 60000; // 1 minute

export interface PreviewUpdate {
  filePath: string;
  patches: any[];
  error?: string;
  timestamp: number;
  version: number;
  acknowledgedMutationIds: string[];
  changedByClientId?: string;
}

export interface StreamBufferRequest {
  clientId: string;
  filePath: string;
  content: string;
  expectedStateVersion?: number;
}

export type PreviewUpdateCallback = (update: PreviewUpdate) => void;
export type ConnectionStateCallback = (connected: boolean) => void;

export class WorkspaceClient {
  private client: any;
  private clientId: string;
  private currentBackoff: number = INITIAL_BACKOFF_MS;
  private reconnectTimer?: NodeJS.Timeout;
  private heartbeatTimer?: NodeJS.Timeout;
  private connectionStateCallbacks: Set<ConnectionStateCallback> = new Set();
  private isShuttingDown = false;
  private protoPath: string;
  private protoIncludePath: string;

  constructor(private serverAddress: string) {
    this.clientId = `vscode-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const paths = resolveProtoPath();
    this.protoPath = paths.protoPath;
    this.protoIncludePath = paths.includePath;
  }

  async connect(): Promise<void> {
    if (this.isShuttingDown) {
      throw new Error('Client is shutting down');
    }

    try {
      const packageDefinition = await protoLoader.load(this.protoPath, {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true,
        includeDirs: [this.protoIncludePath]
      });

      const proto: any = grpc.loadPackageDefinition(packageDefinition);

      this.client = new proto.paperclip.workspace.WorkspaceService(
        this.serverAddress,
        grpc.credentials.createInsecure()
      );

      // Start heartbeat
      this.startHeartbeat();

      // Reset backoff on successful connection
      this.currentBackoff = INITIAL_BACKOFF_MS;

      // Notify connection state
      this.notifyConnectionState(true);

    } catch (error) {
      console.error('[WorkspaceClient] Connection failed:', error);
      this.scheduleReconnect();
      throw error;
    }
  }

  private startHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
    }

    this.heartbeatTimer = setInterval(async () => {
      if (this.isShuttingDown) return;

      try {
        await this.sendHeartbeat();
      } catch (error) {
        console.error('[WorkspaceClient] Heartbeat failed:', error);
        this.scheduleReconnect();
      }
    }, HEARTBEAT_INTERVAL_MS);
  }

  private async sendHeartbeat(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.client) {
        reject(new Error('Client not connected'));
        return;
      }

      this.client.Heartbeat(
        { clientId: this.clientId },
        (error: Error | null, response: any) => {
          if (error) {
            reject(error);
          } else {
            resolve();
          }
        }
      );
    });
  }

  private scheduleReconnect(): void {
    if (this.isShuttingDown || this.reconnectTimer) {
      return;
    }

    // Notify disconnection
    this.notifyConnectionState(false);

    // Exponential backoff with jitter
    const jitter = this.currentBackoff * JITTER_FACTOR * (Math.random() - 0.5);
    const delay = Math.min(this.currentBackoff + jitter, MAX_BACKOFF_MS);

    console.log(`[WorkspaceClient] Reconnecting in ${delay}ms`);

    this.reconnectTimer = setTimeout(async () => {
      this.reconnectTimer = undefined;
      this.currentBackoff = Math.min(this.currentBackoff * 2, MAX_BACKOFF_MS);

      try {
        await this.connect();
      } catch {
        // connect() will schedule another reconnect
      }
    }, delay);
  }

  streamBuffer(
    request: StreamBufferRequest,
    onUpdate: PreviewUpdateCallback
  ): grpc.ClientReadableStream<any> {
    if (!this.client) {
      throw new Error('Client not connected');
    }

    const stream = this.client.StreamBuffer(request);

    stream.on('data', (update: any) => {
      onUpdate({
        filePath: update.file_path,
        patches: update.patches || [],
        error: update.error,
        timestamp: Number(update.timestamp),
        version: Number(update.version),
        acknowledgedMutationIds: update.acknowledged_mutation_ids || [],
        changedByClientId: update.changed_by_client_id
      });
    });

    stream.on('error', (error: Error) => {
      console.error('[WorkspaceClient] Stream error:', error);
      this.scheduleReconnect();
    });

    stream.on('end', () => {
      console.log('[WorkspaceClient] Stream ended');
    });

    return stream;
  }

  async closePreview(): Promise<void> {
    if (!this.client) {
      return;
    }

    return new Promise((resolve, reject) => {
      this.client.ClosePreview(
        { clientId: this.clientId },
        (error: Error | null, response: any) => {
          if (error) {
            console.error('[WorkspaceClient] ClosePreview failed:', error);
            reject(error);
          } else {
            console.log('[WorkspaceClient] Preview closed:', response);
            resolve();
          }
        }
      );
    });
  }

  onConnectionStateChange(callback: ConnectionStateCallback): void {
    this.connectionStateCallbacks.add(callback);
  }

  offConnectionStateChange(callback: ConnectionStateCallback): void {
    this.connectionStateCallbacks.delete(callback);
  }

  private notifyConnectionState(connected: boolean): void {
    this.connectionStateCallbacks.forEach(cb => {
      try {
        cb(connected);
      } catch (error) {
        console.error('[WorkspaceClient] Connection state callback error:', error);
      }
    });
  }

  async dispose(): Promise<void> {
    this.isShuttingDown = true;

    // Stop timers
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = undefined;
    }
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = undefined;
    }

    // Close preview state
    try {
      await this.closePreview();
    } catch (error) {
      console.error('[WorkspaceClient] Failed to close preview on dispose:', error);
    }

    // Close client
    if (this.client) {
      this.client.close();
      this.client = null;
    }

    this.connectionStateCallbacks.clear();
  }

  getClientId(): string {
    return this.clientId;
  }

  isConnected(): boolean {
    return !!this.client && !this.isShuttingDown;
  }
}
