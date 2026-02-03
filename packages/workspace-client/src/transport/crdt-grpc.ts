/**
 * gRPC transport for CRDT synchronization.
 * Implements SyncTransport interface using bidirectional gRPC streaming.
 */

import type { SyncTransport } from '../sync.js';

/**
 * CRDT sync messages (matching proto definitions)
 */
export interface CrdtJoin {
  stateVector: Uint8Array;
}

export interface CrdtUpdate {
  update: Uint8Array;
  stateVector: Uint8Array;
  origin: string;
}

export interface CrdtWelcome {
  documentState: Uint8Array;
  stateVector: Uint8Array;
  version: number;
  clientCount: number;
}

export interface CrdtVdomPatch {
  patches: any[];
  version: number;
  originClientId: string;
}

export interface CrdtParseError {
  error: string;
  line: number;
  column: number;
}

export type CrdtSyncResponse =
  | { type: 'welcome'; welcome: CrdtWelcome }
  | { type: 'remoteUpdate'; update: CrdtUpdate }
  | { type: 'vdomPatch'; patch: CrdtVdomPatch }
  | { type: 'parseError'; error: CrdtParseError };

/**
 * gRPC CRDT stream interface.
 * This is the expected interface from the gRPC client.
 */
export interface CrdtGrpcStream {
  write(request: {
    clientId: string;
    filePath: string;
    messageType:
      | { join: CrdtJoin }
      | { update: CrdtUpdate }
      | { ack: { sequence: number } };
  }): void;

  on(event: 'data', handler: (response: any) => void): void;
  on(event: 'error', handler: (error: Error) => void): void;
  on(event: 'end', handler: () => void): void;

  end(): void;
  cancel(): void;
}

/**
 * gRPC client interface for CRDT sync.
 */
export interface CrdtGrpcClient {
  crdtSync(): CrdtGrpcStream;
}

/**
 * Configuration for CrdtGrpcTransport.
 */
export interface CrdtGrpcTransportConfig {
  client: CrdtGrpcClient;
  clientId: string;
  filePath: string;
}

/**
 * gRPC-based transport for CRDT synchronization.
 */
export class CrdtGrpcTransport implements SyncTransport {
  private stream: CrdtGrpcStream | null = null;
  private updateHandlers = new Set<(update: Uint8Array) => void>();
  private vdomHandlers = new Set<(vdom: any) => void>();
  private cssomHandlers = new Set<(cssom: any) => void>();
  private errorHandlers = new Set<(error: Error) => void>();
  private connected = false;
  private clientId: string;
  private filePath: string;
  private grpcClient: CrdtGrpcClient;

  constructor(config: CrdtGrpcTransportConfig) {
    this.grpcClient = config.client;
    this.clientId = config.clientId;
    this.filePath = config.filePath;
  }

  /**
   * Connect to the server and join the CRDT session.
   */
  async connect(initialStateVector?: Uint8Array): Promise<CrdtWelcome> {
    return new Promise((resolve, reject) => {
      this.stream = this.grpcClient.crdtSync();

      // Handle incoming messages
      this.stream.on('data', (response: any) => {
        this.handleResponse(response, resolve);
      });

      this.stream.on('error', (error: Error) => {
        this.connected = false;
        for (const handler of this.errorHandlers) {
          handler(error);
        }
        reject(error);
      });

      this.stream.on('end', () => {
        this.connected = false;
      });

      // Send join request
      this.stream.write({
        clientId: this.clientId,
        filePath: this.filePath,
        messageType: {
          join: {
            stateVector: initialStateVector || new Uint8Array(),
          },
        },
      });
    });
  }

  /**
   * Handle incoming response from server.
   */
  private handleResponse(response: any, welcomeResolver?: (welcome: CrdtWelcome) => void): void {
    const msgType = response.messageType;

    if (msgType?.welcome) {
      this.connected = true;
      const welcome: CrdtWelcome = {
        documentState: new Uint8Array(msgType.welcome.documentState || []),
        stateVector: new Uint8Array(msgType.welcome.stateVector || []),
        version: msgType.welcome.version || 0,
        clientCount: msgType.welcome.clientCount || 1,
      };
      welcomeResolver?.(welcome);
    } else if (msgType?.remoteUpdate) {
      const update = new Uint8Array(msgType.remoteUpdate.update || []);
      for (const handler of this.updateHandlers) {
        handler(update);
      }
    } else if (msgType?.vdomPatch) {
      const vdom = {
        patches: msgType.vdomPatch.patches || [],
        version: msgType.vdomPatch.version,
        originClientId: msgType.vdomPatch.originClientId,
      };
      for (const handler of this.vdomHandlers) {
        handler(vdom);
      }
    } else if (msgType?.cssomPatch) {
      const cssom = {
        rules: msgType.cssomPatch.rules || [],
        version: msgType.cssomPatch.version,
      };
      for (const handler of this.cssomHandlers) {
        handler(cssom);
      }
    } else if (msgType?.parseError) {
      // Parse errors are also delivered via VDOM handler with error info
      const errorInfo = {
        error: msgType.parseError.error,
        line: msgType.parseError.line,
        column: msgType.parseError.column,
      };
      for (const handler of this.vdomHandlers) {
        handler({ error: errorInfo, patches: [] });
      }
    }
  }

  /**
   * Send a CRDT update to the server.
   */
  async sendUpdate(filePath: string, update: Uint8Array, stateVector: Uint8Array): Promise<void> {
    if (!this.stream || !this.connected) {
      throw new Error('Not connected to CRDT server');
    }

    this.stream.write({
      clientId: this.clientId,
      filePath,
      messageType: {
        update: {
          update,
          stateVector,
          origin: 'local',
        },
      },
    });
  }

  /**
   * Subscribe to CRDT updates from server.
   */
  onUpdate(callback: (update: Uint8Array) => void): () => void {
    this.updateHandlers.add(callback);
    return () => {
      this.updateHandlers.delete(callback);
    };
  }

  /**
   * Subscribe to VDOM updates from server.
   */
  onVDOM(callback: (vdom: any) => void): () => void {
    this.vdomHandlers.add(callback);
    return () => {
      this.vdomHandlers.delete(callback);
    };
  }

  /**
   * Subscribe to CSSOM updates from server.
   */
  onCSSOM(callback: (cssom: any) => void): () => void {
    this.cssomHandlers.add(callback);
    return () => {
      this.cssomHandlers.delete(callback);
    };
  }

  /**
   * Subscribe to transport errors.
   */
  onError(callback: (error: Error) => void): () => void {
    this.errorHandlers.add(callback);
    return () => {
      this.errorHandlers.delete(callback);
    };
  }

  /**
   * Check if connected.
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Disconnect from server.
   */
  disconnect(): void {
    if (this.stream) {
      this.stream.end();
      this.stream = null;
    }
    this.connected = false;
    this.updateHandlers.clear();
    this.vdomHandlers.clear();
    this.cssomHandlers.clear();
    this.errorHandlers.clear();
  }
}

/**
 * Create a mock transport for testing without a real server.
 */
export function createMockCrdtTransport(): SyncTransport & {
  simulateRemoteUpdate(update: Uint8Array): void;
  simulateVDOM(vdom: any): void;
  simulateCSSOM(cssom: any): void;
  getSentUpdates(): Array<{ filePath: string; update: Uint8Array; stateVector: Uint8Array }>;
} {
  const updateHandlers = new Set<(update: Uint8Array) => void>();
  const vdomHandlers = new Set<(vdom: any) => void>();
  const cssomHandlers = new Set<(cssom: any) => void>();
  const sentUpdates: Array<{ filePath: string; update: Uint8Array; stateVector: Uint8Array }> = [];

  return {
    async sendUpdate(filePath: string, update: Uint8Array, stateVector: Uint8Array): Promise<void> {
      sentUpdates.push({ filePath, update, stateVector });
    },

    onUpdate(callback: (update: Uint8Array) => void): () => void {
      updateHandlers.add(callback);
      return () => updateHandlers.delete(callback);
    },

    onVDOM(callback: (vdom: any) => void): () => void {
      vdomHandlers.add(callback);
      return () => vdomHandlers.delete(callback);
    },

    onCSSOM(callback: (cssom: any) => void): () => void {
      cssomHandlers.add(callback);
      return () => cssomHandlers.delete(callback);
    },

    // Test helpers
    simulateRemoteUpdate(update: Uint8Array): void {
      for (const handler of updateHandlers) {
        handler(update);
      }
    },

    simulateVDOM(vdom: any): void {
      for (const handler of vdomHandlers) {
        handler(vdom);
      }
    },

    simulateCSSOM(cssom: any): void {
      for (const handler of cssomHandlers) {
        handler(cssom);
      }
    },

    getSentUpdates() {
      return sentUpdates;
    },
  };
}
