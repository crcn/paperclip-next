/**
 * gRPC transport for Node.js
 * Uses @grpc/grpc-js for native gRPC support
 */

// @ts-ignore - Using Yarn PnP
import * as grpc from '@grpc/grpc-js';
// @ts-ignore - Using Yarn PnP
import * as protoLoader from '@grpc/proto-loader';
// @ts-ignore - Using Yarn PnP
import { dirname } from 'path';
// @ts-ignore - Using Yarn PnP
import { fileURLToPath } from 'url';
// @ts-ignore - Using Yarn PnP
import { createRequire } from 'module';
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

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const require = createRequire(import.meta.url);

/**
 * Resolve proto files from @paperclip/proto package
 */
function resolveProtoPath(): { protoPath: string; includePath: string } {
  try {
    // Resolve the @paperclip/proto package location
    const protoPackagePath = dirname(require.resolve('@paperclip/proto/package.json'));
    return {
      protoPath: `${protoPackagePath}/src/workspace.proto`,
      includePath: `${protoPackagePath}/src`,
    };
  } catch {
    // Fallback to relative path for development
    return {
      protoPath: `${__dirname}/../../../../proto/src/workspace.proto`,
      includePath: `${__dirname}/../../../../proto/src`,
    };
  }
}

/**
 * Configuration for GrpcTransport
 */
export interface GrpcTransportConfig {
  /**
   * Path to workspace.proto file
   * Auto-resolved from @paperclip/proto package if not specified
   */
  protoPath?: string;

  /**
   * Directory containing proto files
   * Auto-resolved from @paperclip/proto package if not specified
   */
  protoIncludePath?: string;

  /**
   * gRPC credentials (default: insecure)
   */
  credentials?: grpc.ChannelCredentials;

  /**
   * gRPC channel options
   */
  channelOptions?: grpc.ChannelOptions;
}

/**
 * gRPC transport implementation for Node.js
 */
export class GrpcTransport implements Transport {
  private client: any | null = null;
  private address: string | null = null;
  private config: Required<GrpcTransportConfig>;

  constructor(config: GrpcTransportConfig = {}) {
    const defaultPaths = resolveProtoPath();
    this.config = {
      protoPath: config.protoPath || defaultPaths.protoPath,
      protoIncludePath: config.protoIncludePath || defaultPaths.includePath,
      credentials: config.credentials || grpc.credentials.createInsecure(),
      channelOptions: config.channelOptions || {},
    };
  }

  async connect(address: string): Promise<void> {
    if (this.client) {
      throw new ConnectionError('Already connected');
    }

    try {
      // Load proto files
      const packageDefinition = protoLoader.loadSync(this.config.protoPath, {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true,
        includeDirs: [this.config.protoIncludePath],
      });

      const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;
      const WorkspaceService =
        protoDescriptor.paperclip.workspace.WorkspaceService;

      this.client = new WorkspaceService(
        address,
        this.config.credentials,
        this.config.channelOptions
      );

      this.address = address;

      // Wait for the channel to be ready
      await this.waitForReady();
    } catch (error) {
      throw new ConnectionError(
        `Failed to connect to ${address}`,
        error
      );
    }
  }

  async disconnect(): Promise<void> {
    if (this.client) {
      this.client.close();
      this.client = null;
      this.address = null;
    }
  }

  isConnected(): boolean {
    return this.client !== null;
  }

  async *streamPreview(
    request: PreviewRequest
  ): AsyncIterableIterator<PreviewUpdate> {
    if (!this.client) {
      throw new ConnectionError('Not connected');
    }

    const call = this.client.StreamPreview(request);

    try {
      for await (const update of this.streamToAsyncIterator<PreviewUpdate>(call)) {
        yield update;
      }
    } catch (error) {
      throw new RpcError('StreamPreview failed', undefined, error);
    }
  }

  async *watchFiles(request: WatchRequest): AsyncIterableIterator<FileEvent> {
    if (!this.client) {
      throw new ConnectionError('Not connected');
    }

    const call = this.client.WatchFiles(request);

    try {
      for await (const event of this.streamToAsyncIterator<FileEvent>(call)) {
        yield event;
      }
    } catch (error) {
      throw new RpcError('WatchFiles failed', undefined, error);
    }
  }

  async applyMutation(request: MutationRequest): Promise<MutationResponse> {
    if (!this.client) {
      throw new ConnectionError('Not connected');
    }

    return new Promise((resolve, reject) => {
      this.client.ApplyMutation(request, (error: any, response: MutationResponse) => {
        if (error) {
          reject(new RpcError('ApplyMutation failed', error.code, error));
        } else {
          resolve(response);
        }
      });
    });
  }

  async getDocumentOutline(
    request: OutlineRequest
  ): Promise<OutlineResponse> {
    if (!this.client) {
      throw new ConnectionError('Not connected');
    }

    return new Promise((resolve, reject) => {
      this.client.GetDocumentOutline(
        request,
        (error: any, response: OutlineResponse) => {
          if (error) {
            reject(new RpcError('GetDocumentOutline failed', error.code, error));
          } else {
            resolve(response);
          }
        }
      );
    });
  }

  /**
   * Wait for the gRPC channel to be ready
   */
  private async waitForReady(timeoutMs: number = 5000): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.client) {
        reject(new ConnectionError('Client not initialized'));
        return;
      }

      const deadline = new Date(Date.now() + timeoutMs);
      this.client.waitForReady(deadline, (error: Error | undefined) => {
        if (error) {
          reject(new ConnectionError('Failed to connect', error));
        } else {
          resolve();
        }
      });
    });
  }

  /**
   * Convert a gRPC stream to an async iterator
   */
  private async *streamToAsyncIterator<T>(
    stream: grpc.ClientReadableStream<T>
  ): AsyncIterableIterator<T> {
    const queue: T[] = [];
    let done = false;
    let error: Error | null = null;
    let resolve: ((value: T | PromiseLike<T>) => void) | null = null;
    let reject: ((reason?: any) => void) | null = null;

    stream.on('data', (data: T) => {
      if (resolve) {
        resolve(data);
        resolve = null;
        reject = null;
      } else {
        queue.push(data);
      }
    });

    stream.on('error', (err: Error) => {
      error = err;
      if (reject) {
        reject(err);
        resolve = null;
        reject = null;
      }
    });

    stream.on('end', () => {
      done = true;
      if (resolve) {
        resolve = null;
        reject = null;
      }
    });

    while (true) {
      if (queue.length > 0) {
        yield queue.shift()!;
      } else if (done) {
        break;
      } else if (error) {
        throw error;
      } else {
        await new Promise<T>((res, rej) => {
          resolve = res;
          reject = rej;
        }).then((value) => {
          return value;
        });

        if (queue.length > 0) {
          yield queue.shift()!;
        }
      }
    }
  }
}
