/**
 * End-to-end latency measurement - Simple and reliable
 * Measures: file write → gRPC update received
 * Target: <40ms
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { writeFileSync, unlinkSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

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

interface LatencyMeasurement {
  testName: string;
  writeTimestamp: number;
  receiveTimestamp: number;
  latencyMs: number;
  version: number;
}

async function measureSingleLatency(
  testName: string,
  fileName: string,
  content: string
): Promise<LatencyMeasurement> {
  return new Promise((resolve, reject) => {
    const client = new WorkspaceService(
      '127.0.0.1:50051',
      grpc.credentials.createInsecure()
    );

    const filePath = join(__dirname, '../../../examples', fileName);
    let writeTimestamp = 0;

    // Write the file immediately
    writeTimestamp = Date.now();
    writeFileSync(filePath, content);

    // Start streaming - should get update for the file we just wrote
    const call = client.StreamPreview({ root_path: fileName });

    call.on('data', (update: any) => {
      const receiveTimestamp = Date.now();
      const latencyMs = receiveTimestamp - writeTimestamp;

      call.cancel();
      client.close();

      resolve({
        testName,
        writeTimestamp,
        receiveTimestamp,
        latencyMs,
        version: update.version
      });
    });

    call.on('error', (error: Error) => {
      client.close();
      reject(error);
    });

    setTimeout(() => {
      call.cancel();
      client.close();
      reject(new Error('Timeout'));
    }, 5000);
  });
}

async function runTests() {
  console.log('═'.repeat(70));
  console.log('Paperclip End-to-End Latency Test');
  console.log('Target: <40ms (file write → gRPC update received)');
  console.log('═'.repeat(70));
  console.log();

  const testCases = [
    {
      name: 'Simple button',
      file: 'latency-test-1.pc',
      content: `public component Button {
  render button {
    style {
      padding: 8px 16px
      background: #3366FF
      color: white
    }
    text "Click me"
  }
}`
    },
    {
      name: 'Card with nested elements',
      file: 'latency-test-2.pc',
      content: `public component Card {
  render div {
    style {
      padding: 20px
      background: white
      border-radius: 8px
    }
    div {
      style { margin-bottom: 12px }
      text "Title"
    }
    div { text "Content here" }
    button {
      style { padding: 8px 12px }
      text "Action"
    }
  }
}`
    },
    {
      name: 'Multiple components',
      file: 'latency-test-3.pc',
      content: `public component Button {
  render button {
    style { padding: 6px 12px }
    text "Click"
  }
}

public component Card {
  render div {
    style { padding: 16px }
    Button {}
    Button {}
  }
}`
    }
  ];

  const results: LatencyMeasurement[] = [];

  for (let i = 0; i < testCases.length; i++) {
    const testCase = testCases[i];
    console.log(`Test ${i + 1}/${testCases.length}: ${testCase.name}`);

    try {
      // Small delay between tests
      await new Promise(resolve => setTimeout(resolve, 200));

      const result = await measureSingleLatency(
        testCase.name,
        testCase.file,
        testCase.content
      );

      results.push(result);

      const status = result.latencyMs < 40 ? '✓ PASS' : '⚠ SLOW';
      console.log(`  Latency: ${result.latencyMs}ms ${status}`);
      console.log(`  Version: ${result.version}`);
    } catch (error) {
      console.log(`  ✗ Error: ${(error as Error).message}`);
    }

    console.log();
  }

  // Cleanup test files
  console.log('Cleaning up test files...');
  for (const testCase of testCases) {
    try {
      const filePath = join(__dirname, '../../../examples', testCase.file);
      unlinkSync(filePath);
    } catch (e) {
      // Ignore cleanup errors
    }
  }
  console.log();

  // Summary
  console.log('═'.repeat(70));
  console.log('Summary');
  console.log('═'.repeat(70));
  console.log();

  if (results.length === 0) {
    console.log('❌ No tests completed successfully');
    process.exit(1);
  }

  const latencies = results.map(r => r.latencyMs);
  const avg = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const min = Math.min(...latencies);
  const max = Math.max(...latencies);
  const passed = results.filter(r => r.latencyMs < 40).length;
  const passRate = (passed / results.length) * 100;

  console.log(`Tests completed:  ${results.length}/${testCases.length}`);
  console.log(`Average latency:  ${avg.toFixed(2)}ms`);
  console.log(`Min latency:      ${min}ms`);
  console.log(`Max latency:      ${max}ms`);
  console.log();
  console.log(`Target:           <40ms`);
  console.log(`Pass rate:        ${passRate.toFixed(0)}% (${passed}/${results.length})`);
  console.log();

  // Individual results
  console.log('Individual Results:');
  results.forEach(r => {
    const mark = r.latencyMs < 40 ? '✓' : '✗';
    console.log(`  ${mark} ${r.testName}: ${r.latencyMs}ms`);
  });
  console.log();

  // Final verdict
  if (passRate === 100) {
    console.log('✅ SUCCESS: All tests passed! Pipeline meets <40ms target.');
    console.log();
    process.exit(0);
  } else if (passRate >= 50) {
    console.log('⚠️  PARTIAL: Most tests passed, but some exceeded target.');
    console.log();
    process.exit(0);
  } else {
    console.log('❌ FAILED: Most tests exceeded target latency.');
    console.log();
    process.exit(1);
  }
}

console.log('Starting latency measurement...\n');
runTests().catch((error) => {
  console.error('Test suite failed:', error);
  process.exit(1);
});
