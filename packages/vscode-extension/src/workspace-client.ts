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
  const fs = require('fs');

  // Strategy 1: Try require.resolve (works when deps are in node_modules)
  try {
    const protoPackagePath = path.dirname(require.resolve('@paperclip/proto/package.json'));
    return {
      protoPath: path.join(protoPackagePath, 'src', 'workspace.proto'),
      includePath: path.join(protoPackagePath, 'src'),
    };
  } catch {
    // Continue to fallback
  }

  // Resolve symlinks to get the real path (important when extension is symlinked for dev)
  const realDirname = fs.realpathSync(__dirname);

  // Strategy 2: Resolve relative to this file (for symlinked extension in monorepo)
  // Extension is at packages/vscode-extension, proto is at packages/proto
  const protoPath = path.join(realDirname, '..', '..', 'proto', 'src', 'workspace.proto');
  if (fs.existsSync(protoPath)) {
    return {
      protoPath,
      includePath: path.join(realDirname, '..', '..', 'proto', 'src'),
    };
  }

  // Strategy 3: Check root node_modules (yarn workspaces hoisting)
  const rootProtoPath = path.join(realDirname, '..', '..', '..', 'node_modules', '@paperclip', 'proto', 'src', 'workspace.proto');
  if (fs.existsSync(rootProtoPath)) {
    return {
      protoPath: rootProtoPath,
      includePath: path.join(realDirname, '..', '..', '..', 'node_modules', '@paperclip', 'proto', 'src'),
    };
  }

  throw new Error('@paperclip/proto package not found. Make sure it is installed.');
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

export interface MutationRequest {
  clientId: string;
  filePath: string;
  mutation: {
    mutationId: string;
    timestamp: number;
    setInlineStyle?: {
      nodeId: string;
      property: string;
      value: string;
    };
    setFrameBounds?: {
      frameId: string;
      bounds: { x: number; y: number; width: number; height: number };
    };
    updateText?: {
      nodeId: string;
      content: string;
    };
  };
  expectedVersion?: number;
}

export interface MutationResult {
  success: boolean;
  mutationId: string;
  newVersion: number;
  error?: string;
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
        keepCase: false,  // Convert snake_case to camelCase for JS
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
      console.error('[WorkspaceClient] streamBuffer called but client not connected');
      throw new Error('Client not connected');
    }

    console.log(`[WorkspaceClient] streamBuffer: clientId=${request.clientId}, filePath=${request.filePath}, contentLen=${request.content.length}`);
    const stream = this.client.StreamBuffer(request);

    stream.on('data', (update: any) => {
      console.log(`[WorkspaceClient] stream.on('data'):`, JSON.stringify(update, null, 2));
      onUpdate({
        filePath: update.filePath,
        patches: update.patches || [],
        error: update.error,
        timestamp: Number(update.timestamp),
        version: Number(update.version),
        acknowledgedMutationIds: update.acknowledgedMutationIds || [],
        changedByClientId: update.changedByClientId
      });
    });

    stream.on('error', (error: any) => {
      // Don't reconnect for cancelled streams (happens when user types more)
      // or for normal stream completion
      const code = error?.code;
      const isCancelled = code === 1; // grpc.status.CANCELLED
      const isOk = code === 0; // grpc.status.OK

      console.log(`[WorkspaceClient] stream.on('error'): code=${code}, message=${error?.message}`);

      if (!isCancelled && !isOk) {
        console.error('[WorkspaceClient] Stream error:', error);
        // Only reconnect for actual connection errors
        if (code === 14) { // grpc.status.UNAVAILABLE
          this.scheduleReconnect();
        }
      }
    });

    stream.on('end', () => {
      // Stream ended normally - this is expected for single-response streams
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

  /**
   * Apply a mutation to a document via gRPC
   */
  async applyMutation(request: MutationRequest): Promise<MutationResult> {
    if (!this.client) {
      throw new Error('Client not connected');
    }

    return new Promise((resolve, reject) => {
      // Build the proto mutation object
      const protoMutation: any = {
        mutationId: request.mutation.mutationId,
        timestamp: request.mutation.timestamp,
      };

      if (request.mutation.setInlineStyle) {
        protoMutation.setInlineStyle = {
          nodeId: request.mutation.setInlineStyle.nodeId,
          property: request.mutation.setInlineStyle.property,
          value: request.mutation.setInlineStyle.value,
        };
      } else if (request.mutation.setFrameBounds) {
        protoMutation.setFrameBounds = {
          frameId: request.mutation.setFrameBounds.frameId,
          bounds: request.mutation.setFrameBounds.bounds,
        };
      } else if (request.mutation.updateText) {
        protoMutation.updateText = {
          nodeId: request.mutation.updateText.nodeId,
          content: request.mutation.updateText.content,
        };
      }

      const protoRequest = {
        clientId: request.clientId,
        filePath: request.filePath,
        mutation: protoMutation,
        expectedVersion: request.expectedVersion || 0,
      };

      console.log('[WorkspaceClient] applyMutation:', JSON.stringify(protoRequest, null, 2));

      this.client.ApplyMutation(
        protoRequest,
        (error: Error | null, response: any) => {
          if (error) {
            console.error('[WorkspaceClient] ApplyMutation failed:', error);
            resolve({
              success: false,
              mutationId: request.mutation.mutationId,
              newVersion: 0,
              error: error.message,
            });
          } else {
            console.log('[WorkspaceClient] ApplyMutation response:', response);
            // Parse response - could be ack, rebased, or noop
            if (response.ack) {
              resolve({
                success: true,
                mutationId: response.ack.mutationId,
                newVersion: Number(response.ack.newVersion),
              });
            } else if (response.noop) {
              resolve({
                success: true,
                mutationId: response.noop.mutationId,
                newVersion: 0,
                error: response.noop.reason,
              });
            } else if (response.rebased) {
              resolve({
                success: true,
                mutationId: response.rebased.originalMutationId,
                newVersion: Number(response.rebased.newVersion),
              });
            } else {
              resolve({
                success: false,
                mutationId: request.mutation.mutationId,
                newVersion: 0,
                error: 'Unknown response format',
              });
            }
          }
        }
      );
    });
  }

  /**
   * Get the raw gRPC client for advanced usage (e.g., with CrdtGrpcTransport).
   */
  getRawClient(): any {
    return this.client;
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
