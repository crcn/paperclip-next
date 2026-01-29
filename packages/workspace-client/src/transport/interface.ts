/**
 * Transport abstraction for gRPC (Node.js) and gRPC-web (browser)
 */

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

export interface Transport {
  /**
   * Connect to the workspace server
   */
  connect(address: string): Promise<void>;

  /**
   * Disconnect from the workspace server
   */
  disconnect(): Promise<void>;

  /**
   * Check if transport is currently connected
   */
  isConnected(): boolean;

  /**
   * Stream preview updates for a file
   * Returns an async iterator that yields PreviewUpdate events
   */
  streamPreview(request: PreviewRequest): AsyncIterableIterator<PreviewUpdate>;

  /**
   * Watch files in a directory
   * Returns an async iterator that yields FileEvent events
   */
  watchFiles(request: WatchRequest): AsyncIterableIterator<FileEvent>;

  /**
   * Apply a mutation to a document
   * Returns a Promise with the mutation response
   */
  applyMutation(request: MutationRequest): Promise<MutationResponse>;

  /**
   * Get document outline (AST structure)
   * Returns a Promise with the outline response
   */
  getDocumentOutline(request: OutlineRequest): Promise<OutlineResponse>;
}

/**
 * Base error class for transport errors
 */
export class TransportError extends Error {
  constructor(
    message: string,
    public readonly code?: string,
    public readonly details?: unknown
  ) {
    super(message);
    this.name = 'TransportError';
  }
}

/**
 * Error thrown when connection fails
 */
export class ConnectionError extends TransportError {
  constructor(message: string, details?: unknown) {
    super(message, 'CONNECTION_ERROR', details);
    this.name = 'ConnectionError';
  }
}

/**
 * Error thrown when an RPC call fails
 */
export class RpcError extends TransportError {
  constructor(message: string, code?: string, details?: unknown) {
    super(message, code, details);
    this.name = 'RpcError';
  }
}
