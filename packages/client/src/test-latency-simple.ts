/**
 * End-to-end latency measurement test
 * Measures time from file write → preview update
 * Target: <40ms
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { writeFileSync } from 'fs';
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

interface LatencyResult {
  latencyMs: number;
  version: number;
  patchCount: number;
  testName: string;
}

async function measureLatency(
  testName: string,
  filePath: string,
  content: string
): Promise<LatencyResult> {
  return new Promise((resolve, reject) => {
    const client = new WorkspaceService(
      '127.0.0.1:50051',
      grpc.credentials.createInsecure()
    );

    let updateCount = 0;
    let writeTime = 0;

    const call = client.StreamPreview({ root_path: filePath });

    call.on('data', (update: any) => {
      updateCount++;

      // First update is initial state
      if (updateCount === 1) {
        // Write file after receiving initial state
        setTimeout(() => {
          const fullPath = join(__dirname, '../../../examples', filePath);
          writeTime = Date.now();
          writeFileSync(fullPath, content);
        }, 50);
        return;
      }

      // Second update is from our file change
      const receiveTime = Date.now();
      const latencyMs = receiveTime - writeTime;

      call.cancel();
      client.close();

      resolve({
        latencyMs,
        version: update.version,
        patchCount: update.patches?.length || 0,
        testName
      });
    });

    call.on('error', (error: Error) => {
      reject(error);
    });

    // Timeout after 10 seconds
    setTimeout(() => {
      call.cancel();
      client.close();
      reject(new Error('Timeout waiting for update'));
    }, 10000);
  });
}

async function runTests() {
  console.log('═'.repeat(70));
  console.log('Paperclip End-to-End Latency Test');
  console.log('═'.repeat(70));
  console.log();

  const testCases = [
    {
      name: 'Simple button',
      file: 'test-latency.pc',
      content: `public component Button {
  render button {
    style {
      padding: 8px 16px
      background: #3366FF
    }
    text "Click me"
  }
}`
    },
    {
      name: 'Card with multiple elements',
      file: 'test-latency.pc',
      content: `public component Card {
  render div {
    style {
      padding: 16px
    }
    div { text "Title" }
    div { text "Content" }
    button { text "Action" }
  }
}`
    },
    {
      name: 'Nested components',
      file: 'test-latency.pc',
      content: `public component Button {
  render button {
    text "Click"
  }
}

public component Card {
  render div {
    Button {}
    Button {}
  }
}`
    }
  ];

  const results: LatencyResult[] = [];

  for (const testCase of testCases) {
    console.log(`Test: ${testCase.name}`);

    try {
      // Wait between tests
      await new Promise(resolve => setTimeout(resolve, 300));

      const result = await measureLatency(
        testCase.name,
        testCase.file,
        testCase.content
      );

      results.push(result);

      const status = result.latencyMs < 40 ? '✓ PASS' : '⚠ SLOW';
      console.log(`  Latency: ${result.latencyMs}ms - ${status}`);
      console.log(`  Version: ${result.version}`);
      console.log(`  Patches: ${result.patchCount}`);
    } catch (error) {
      console.log(`  ✗ Error: ${(error as Error).message}`);
    }

    console.log();
  }

  // Summary
  console.log('═'.repeat(70));
  console.log('Summary');
  console.log('═'.repeat(70));
  console.log();

  if (results.length > 0) {
    const latencies = results.map(r => r.latencyMs);
    const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
    const minLatency = Math.min(...latencies);
    const maxLatency = Math.max(...latencies);
    const passedTests = results.filter(r => r.latencyMs < 40).length;
    const passRate = (passedTests / results.length) * 100;

    console.log(`Tests run:      ${results.length}`);
    console.log(`Average latency: ${avgLatency.toFixed(2)}ms`);
    console.log(`Min latency:     ${minLatency}ms`);
    console.log(`Max latency:     ${maxLatency}ms`);
    console.log();
    console.log(`Target:          <40ms`);
    console.log(`Pass rate:       ${passRate.toFixed(0)}% (${passedTests}/${results.length})`);
    console.log();

    if (passRate === 100) {
      console.log('✓ All tests passed! Pipeline meets <40ms target.');
    } else if (passRate > 0) {
      console.log('⚠ Some tests exceeded target latency');
    } else {
      console.log('✗ No tests met target latency');
    }

    console.log();
    console.log('Individual Results:');
    results.forEach(r => {
      const mark = r.latencyMs < 40 ? '✓' : '✗';
      console.log(`  ${mark} ${r.testName}: ${r.latencyMs}ms`);
    });
  } else {
    console.log('No tests completed successfully');
  }

  console.log();
  process.exit(0);
}

runTests().catch((error) => {
  console.error('Test suite failed:', error);
  process.exit(1);
});
