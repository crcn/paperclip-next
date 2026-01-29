/**
 * Workspace client for interacting with Paperclip workspace service
 * Event-driven architecture with Redux integration
 */

import type { Transport } from './transport/interface.js';
import { ConnectionError, RpcError } from './transport/interface.js';
import { EventEmitter, type WorkspaceEventUnion } from './events.js';
import type {
  PreviewRequest,
  PreviewUpdate,
  WatchRequest,
  FileEvent,
  Mutation,
  MutationRequest,
  MutationResponse,
  OutlineRequest,
  OutlineResponse,
} from './types.js';

/**
 * Configuration for WorkspaceClient
 */
export interface WorkspaceClientConfig {
  /**
   * Client ID for tracking mutations
   * Defaults to a random UUID
   */
  clientId?: string;

  /**
   * Enable automatic reconnection on disconnect
   * Default: true
   */
  autoReconnect?: boolean;

  /**
   * Maximum reconnection attempts
   * Default: 10
   */
  maxReconnectAttempts?: number;

  /**
   * Initial reconnection delay in milliseconds
   * Default: 1000
   */
  reconnectDelayMs?: number;

  /**
   * Maximum reconnection delay in milliseconds
   * Default: 30000 (30 seconds)
   */
  maxReconnectDelayMs?: number;
}

/**
 * Workspace client for real-time preview and editing
 */
export class WorkspaceClient {
  private transport: Transport;
  private events = new EventEmitter();
  private config: Required<WorkspaceClientConfig>;
  private address: string | null = null;
  private reconnectAttempts = 0;
  private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

  constructor(transport: Transport, config: WorkspaceClientConfig = {}) {
    this.transport = transport;
    this.config = {
      clientId: config.clientId || this.generateClientId(),
      autoReconnect: config.autoReconnect ?? true,
      maxReconnectAttempts: config.maxReconnectAttempts ?? 10,
      reconnectDelayMs: config.reconnectDelayMs ?? 1000,
      maxReconnectDelayMs: config.maxReconnectDelayMs ?? 30000,
    };
  }

  /**
   * Connect to workspace server
   */
  async connect(address: string): Promise<void> {
    try {
      await this.transport.connect(address);
      this.address = address;
      this.reconnectAttempts = 0;

      this.events.emit({
        type: 'connected',
        address,
        timestamp: Date.now(),
      });
    } catch (error) {
      this.events.emit({
        type: 'connection-failed',
        error: error as Error,
        timestamp: Date.now(),
      });

      if (this.config.autoReconnect) {
        this.scheduleReconnect();
      }

      throw error;
    }
  }

  /**
   * Disconnect from workspace server
   */
  async disconnect(): Promise<void> {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    await this.transport.disconnect();

    this.events.emit({
      type: 'disconnected',
      timestamp: Date.now(),
    });
  }

  /**
   * Check if client is connected
   */
  isConnected(): boolean {
    return this.transport.isConnected();
  }

  /**
   * Stream preview updates for a file
   * Returns an async iterator that yields preview updates
   */
  async *streamPreview(
    filePath: string
  ): AsyncIterableIterator<PreviewUpdate> {
    const request: PreviewRequest = { root_path: filePath };

    try {
      for await (const update of this.transport.streamPreview(request)) {
        // Emit event
        this.events.emit({
          type: 'preview-updated',
          update,
          timestamp: Date.now(),
        });

        yield update;
      }
    } catch (error) {
      this.handleRpcError('streamPreview', error as Error);
      throw error;
    }
  }

  /**
   * Watch files in a directory
   * Returns an async iterator that yields file events
   */
  async *watchFiles(
    directory: string,
    patterns: string[] = ['*.pc']
  ): AsyncIterableIterator<FileEvent> {
    const request: WatchRequest = { directory, patterns };

    try {
      for await (const event of this.transport.watchFiles(request)) {
        // Emit event
        this.events.emit({
          type: 'file-changed',
          event,
          timestamp: Date.now(),
        });

        yield event;
      }
    } catch (error) {
      this.handleRpcError('watchFiles', error as Error);
      throw error;
    }
  }

  /**
   * Apply a mutation to a document
   */
  async applyMutation(
    filePath: string,
    mutation: Omit<Mutation, 'mutation_id' | 'timestamp'>,
    expectedVersion: number
  ): Promise<MutationResponse> {
    // Generate mutation ID
    const mutationWithId: Mutation = {
      ...mutation,
      mutation_id: this.generateMutationId(),
      timestamp: Date.now(),
    };

    const request: MutationRequest = {
      client_id: this.config.clientId,
      file_path: filePath,
      mutation: mutationWithId,
      expected_version: expectedVersion,
    };

    try {
      const response = await this.transport.applyMutation(request);

      // Emit appropriate event based on response type
      if (response.ack) {
        this.events.emit({
          type: 'mutation-acknowledged',
          mutation_id: response.ack.mutation_id,
          new_version: response.ack.new_version,
          timestamp: Date.now(),
        });
      } else if (response.rebased) {
        this.events.emit({
          type: 'mutation-rebased',
          original_mutation_id: response.rebased.original_mutation_id,
          new_version: response.rebased.new_version,
          reason: response.rebased.reason,
          timestamp: Date.now(),
        });
      } else if (response.noop) {
        this.events.emit({
          type: 'mutation-noop',
          mutation_id: response.noop.mutation_id,
          reason: response.noop.reason,
          timestamp: Date.now(),
        });
      }

      return response;
    } catch (error) {
      this.handleRpcError('applyMutation', error as Error);
      throw error;
    }
  }

  /**
   * Get document outline (AST structure)
   */
  async getOutline(filePath: string): Promise<OutlineResponse> {
    const request: OutlineRequest = { file_path: filePath };

    try {
      const response = await this.transport.getDocumentOutline(request);

      this.events.emit({
        type: 'outline-received',
        outline: response,
        timestamp: Date.now(),
      });

      return response;
    } catch (error) {
      this.handleRpcError('getOutline', error as Error);
      throw error;
    }
  }

  /**
   * Register an event listener
   * Returns an unsubscribe function
   */
  on<T extends WorkspaceEventUnion>(
    eventType: T['type'],
    listener: (event: T) => void
  ): () => void {
    return this.events.on(eventType, listener);
  }

  /**
   * Register a one-time event listener
   * Returns an unsubscribe function
   */
  once<T extends WorkspaceEventUnion>(
    eventType: T['type'],
    listener: (event: T) => void
  ): () => void {
    return this.events.once(eventType, listener);
  }

  /**
   * Unregister an event listener
   */
  off<T extends WorkspaceEventUnion>(
    eventType: T['type'],
    listener: (event: T) => void
  ): void {
    this.events.off(eventType, listener);
  }

  /**
   * Generate a unique client ID
   */
  private generateClientId(): string {
    return `client-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
  }

  /**
   * Generate a unique mutation ID
   */
  private generateMutationId(): string {
    return `mutation-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
  }

  /**
   * Handle RPC errors and emit events
   */
  private handleRpcError(method: string, error: Error): void {
    this.events.emit({
      type: 'rpc-failed',
      method,
      error,
      timestamp: Date.now(),
    });

    // If connection-related error and auto-reconnect enabled
    if (
      (error instanceof ConnectionError || error instanceof RpcError) &&
      this.config.autoReconnect
    ) {
      this.scheduleReconnect();
    }
  }

  /**
   * Schedule a reconnection attempt with exponential backoff
   */
  private scheduleReconnect(): void {
    if (this.reconnectTimeout) {
      return; // Already scheduled
    }

    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      this.events.emit({
        type: 'disconnected',
        reason: 'Max reconnection attempts reached',
        timestamp: Date.now(),
      });
      return;
    }

    const delay = Math.min(
      this.config.reconnectDelayMs * Math.pow(2, this.reconnectAttempts),
      this.config.maxReconnectDelayMs
    );

    this.reconnectTimeout = setTimeout(async () => {
      this.reconnectTimeout = null;
      this.reconnectAttempts++;

      if (this.address) {
        try {
          await this.connect(this.address);
        } catch (error) {
          // connect() will schedule another reconnect if needed
        }
      }
    }, delay);
  }
}

/**
 * Factory function to create a WorkspaceClient
 */
export function createWorkspaceClient(
  transport: Transport,
  config?: WorkspaceClientConfig
): WorkspaceClient {
  return new WorkspaceClient(transport, config);
}
