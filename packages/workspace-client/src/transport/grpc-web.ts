/**
 * gRPC-web transport for browsers
 * Uses @grpc/grpc-web for browser compatibility
 *
 * NOTE: This is a stub implementation. The full implementation requires:
 * 1. @grpc/grpc-web dependency
 * 2. Browser-compatible protobuf generation
 * 3. gRPC-web proxy (like Envoy) in front of the server
 *
 * For now, this serves as the interface definition for browser transport.
 */

import type { Transport } from './interface.js';
import { ConnectionError, RpcError } from './interface.js';
import type {
  PreviewRequest,
  PreviewUpdate,
  WatchRequest,
  FileEvent,
  MutationRequest,
  MutationResponse,
  OutlineRequest,
  OutlineResponse,
} from '../types.js';

/**
 * Configuration for GrpcWebTransport
 */
export interface GrpcWebTransportConfig {
  /**
   * Enable text mode (base64 encoding) instead of binary
   * Required for some proxy configurations
   */
  textMode?: boolean;

  /**
   * Custom headers to include with requests
   */
  headers?: Record<string, string>;

  /**
   * Request timeout in milliseconds
   */
  timeoutMs?: number;
}

/**
 * gRPC-web transport implementation for browsers
 *
 * @example
 * ```typescript
 * import { GrpcWebTransport } from '@paperclip/workspace-client/grpc-web';
 *
 * const transport = new GrpcWebTransport({
 *   textMode: true,
 *   headers: { 'x-api-key': 'your-key' }
 * });
 *
 * await transport.connect('https://localhost:8080');
 * ```
 */
export class GrpcWebTransport implements Transport {
  private config: Required<GrpcWebTransportConfig>;
  private connected = false;
  private address: string | null = null;

  constructor(config: GrpcWebTransportConfig = {}) {
    this.config = {
      textMode: config.textMode ?? false,
      headers: config.headers ?? {},
      timeoutMs: config.timeoutMs ?? 30000,
    };
  }

  async connect(address: string): Promise<void> {
    if (this.connected) {
      throw new ConnectionError('Already connected');
    }

    // TODO: Initialize grpc-web client
    // For now, just mark as connected for interface compatibility
    this.address = address;
    this.connected = true;
  }

  async disconnect(): Promise<void> {
    this.connected = false;
    this.address = null;
  }

  isConnected(): boolean {
    return this.connected;
  }

  async *streamPreview(
    request: PreviewRequest
  ): AsyncIterableIterator<PreviewUpdate> {
    if (!this.connected) {
      throw new ConnectionError('Not connected');
    }

    // TODO: Implement grpc-web streaming
    // This requires:
    // 1. @grpc/grpc-web client setup
    // 2. Generated client stubs from .proto files
    // 3. Server-sent events or WebSocket fallback

    throw new Error('GrpcWebTransport.streamPreview not yet implemented');
  }

  async *watchFiles(request: WatchRequest): AsyncIterableIterator<FileEvent> {
    if (!this.connected) {
      throw new ConnectionError('Not connected');
    }

    // TODO: Implement grpc-web streaming
    throw new Error('GrpcWebTransport.watchFiles not yet implemented');
  }

  async applyMutation(request: MutationRequest): Promise<MutationResponse> {
    if (!this.connected) {
      throw new ConnectionError('Not connected');
    }

    // TODO: Implement grpc-web unary call
    throw new Error('GrpcWebTransport.applyMutation not yet implemented');
  }

  async getDocumentOutline(
    request: OutlineRequest
  ): Promise<OutlineResponse> {
    if (!this.connected) {
      throw new ConnectionError('Not connected');
    }

    // TODO: Implement grpc-web unary call
    throw new Error('GrpcWebTransport.getDocumentOutline not yet implemented');
  }
}
