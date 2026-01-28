/**
 * Node.js gRPC client for Paperclip workspace service
 * Connects to the Rust server and streams preview updates
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load proto files
const PROTO_PATH = join(__dirname, '../../../proto/workspace.proto');

const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
  includeDirs: [join(__dirname, '../../../proto')]
});

const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;
const WorkspaceService = protoDescriptor.paperclip.workspace.WorkspaceService;

export interface PreviewUpdate {
  file_path: string;
  patches: any[];
  error?: string;
  timestamp: number;
  version: number;
}

export class WorkspaceClient {
  private client: any;
  private serverAddress: string;

  constructor(serverAddress: string = '127.0.0.1:50051') {
    this.serverAddress = serverAddress;
    this.client = new WorkspaceService(
      serverAddress,
      grpc.credentials.createInsecure()
    );
  }

  /**
   * Stream preview updates for a file
   */
  streamPreview(
    filePath: string,
    onUpdate: (update: PreviewUpdate) => void,
    onError?: (error: Error) => void,
    onEnd?: () => void
  ): grpc.ClientReadableStream<PreviewUpdate> {
    const call = this.client.StreamPreview({ root_path: filePath });

    call.on('data', (update: PreviewUpdate) => {
      onUpdate(update);
    });

    call.on('error', (error: Error) => {
      if (onError) {
        onError(error);
      } else {
        console.error('Stream error:', error);
      }
    });

    call.on('end', () => {
      if (onEnd) {
        onEnd();
      }
    });

    return call;
  }

  /**
   * Close the client connection
   */
  close() {
    this.client.close();
  }
}

// CLI usage
if (import.meta.url === `file://${process.argv[1]}`) {
  const filePath = process.argv[2] || 'button.pc';

  console.log(`Connecting to workspace server at 127.0.0.1:50051...`);
  console.log(`Streaming preview updates for: ${filePath}\n`);

  const client = new WorkspaceClient();

  const stream = client.streamPreview(
    filePath,
    (update) => {
      console.log('─'.repeat(60));
      console.log(`Update received at ${new Date(Number(update.timestamp)).toISOString()}`);
      console.log(`File: ${update.file_path}`);
      console.log(`Version: ${update.version}`);

      if (update.error) {
        console.log(`Error: ${update.error}`);
      } else {
        console.log(`Patches: ${update.patches.length}`);

        // Show patch types
        const patchTypes = update.patches.map((p: any) => {
          const type = Object.keys(p.patch_type || {})[0] || 'unknown';
          return type;
        });
        console.log(`Patch types: ${patchTypes.join(', ')}`);
      }
      console.log('─'.repeat(60));
      console.log();
    },
    (error) => {
      console.error('Stream error:', error.message);
      process.exit(1);
    },
    () => {
      console.log('Stream ended');
      process.exit(0);
    }
  );

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    console.log('\nClosing connection...');
    stream.cancel();
    client.close();
    process.exit(0);
  });
}
