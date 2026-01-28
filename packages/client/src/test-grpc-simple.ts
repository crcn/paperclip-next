/**
 * Simple gRPC connection test
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('Loading proto files...');

const PROTO_PATH = join(__dirname, '../../../proto/workspace.proto');
console.log('Proto path:', PROTO_PATH);

const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
  includeDirs: [join(__dirname, '../../../proto')]
});

console.log('Proto loaded successfully');

const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;
console.log('Package definition keys:', Object.keys(protoDescriptor));

const WorkspaceService = protoDescriptor.paperclip?.workspace?.WorkspaceService;

if (!WorkspaceService) {
  console.error('WorkspaceService not found in proto');
  console.error('Available:', JSON.stringify(protoDescriptor, null, 2));
  process.exit(1);
}

console.log('Creating client...');
const client = new WorkspaceService(
  '127.0.0.1:50051',
  grpc.credentials.createInsecure()
);

console.log('Calling StreamPreview...');
const call = client.StreamPreview({ root_path: 'button.pc' });

let updateCount = 0;

call.on('data', (update: any) => {
  updateCount++;
  console.log(`\nUpdate #${updateCount}:`);
  console.log('  File:', update.file_path);
  console.log('  Version:', update.version);
  console.log('  Patches:', update.patches?.length || 0);
  console.log('  Error:', update.error || 'none');
  console.log('  Timestamp:', new Date(Number(update.timestamp)).toISOString());

  if (updateCount >= 1) {
    console.log('\nReceived initial update, closing...');
    call.cancel();
    client.close();
    process.exit(0);
  }
});

call.on('error', (error: Error) => {
  console.error('\nStream error:', error.message);
  console.error('Error details:', error);
  process.exit(1);
});

call.on('end', () => {
  console.log('\nStream ended');
  process.exit(0);
});

setTimeout(() => {
  console.error('\nTimeout - no response received after 5 seconds');
  call.cancel();
  client.close();
  process.exit(1);
}, 5000);

console.log('Waiting for updates...');
