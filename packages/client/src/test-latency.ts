/**
 * End-to-end latency measurement test
 * Measures time from file write → preview update
 * Target: <40ms
 */

import { WorkspaceClient, PreviewUpdate } from './grpc-client.js';
import { writeFileSync } from 'fs';
import { join } from 'path';

interface LatencyMeasurement {
  writeTime: number;
  receiveTime: number;
  latencyMs: number;
  version: number;
  patchCount: number;
}

async function measureLatency(
  client: WorkspaceClient,
  testFilePath: string,
  content: string
): Promise<LatencyMeasurement> {
  return new Promise((resolve, reject) => {
    let writeTime = 0;
    let firstUpdate = true;

    const stream = client.streamPreview(
      testFilePath,
      (update: PreviewUpdate) => {
        const receiveTime = Date.now();

        // Skip initial update, wait for change
        if (firstUpdate) {
          firstUpdate = false;

          // Write the file to trigger update
          setTimeout(() => {
            writeTime = Date.now();
            const fullPath = join(process.cwd(), '../../examples', testFilePath);
            writeFileSync(fullPath, content);
          }, 100);

          return;
        }

        // Calculate latency
        const latencyMs = receiveTime - writeTime;

        stream.cancel();
        resolve({
          writeTime,
          receiveTime,
          latencyMs,
          version: update.version,
          patchCount: update.patches.length
        });
      },
      (error) => {
        reject(error);
      }
    );
  });
}

async function runLatencyTests() {
  console.log('═'.repeat(60));
  console.log('Paperclip End-to-End Latency Test');
  console.log('═'.repeat(60));
  console.log();

  const client = new WorkspaceClient();

  const testCases = [
    {
      name: 'Simple button component',
      file: 'test-latency.pc',
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
      name: 'Component with multiple elements',
      file: 'test-latency.pc',
      content: `public component Card {
  render div {
    style {
      padding: 16px
      background: white
      border-radius: 8px
    }
    div {
      text "Title"
    }
    div {
      text "Content"
    }
    button {
      text "Action"
    }
  }
}`
    },
    {
      name: 'Nested components',
      file: 'test-latency.pc',
      content: `public component Button {
  render button {
    style {
      padding: 8px 16px
    }
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

  const results: LatencyMeasurement[] = [];

  for (const testCase of testCases) {
    console.log(`Test: ${testCase.name}`);
    console.log(`File: ${testCase.file}`);

    try {
      // Wait a bit between tests
      await new Promise(resolve => setTimeout(resolve, 200));

      const measurement = await measureLatency(
        client,
        testCase.file,
        testCase.content
      );

      results.push(measurement);

      console.log(`✓ Latency: ${measurement.latencyMs}ms`);
      console.log(`  Version: ${measurement.version}`);
      console.log(`  Patches: ${measurement.patchCount}`);

      if (measurement.latencyMs < 40) {
        console.log(`  Status: ✓ PASS (target: <40ms)`);
      } else {
        console.log(`  Status: ⚠ SLOW (target: <40ms)`);
      }
    } catch (error) {
      console.log(`✗ Error: ${(error as Error).message}`);
    }

    console.log();
  }

  client.close();

  // Summary
  console.log('═'.repeat(60));
  console.log('Summary');
  console.log('═'.repeat(60));
  console.log();

  if (results.length > 0) {
    const latencies = results.map(r => r.latencyMs);
    const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
    const minLatency = Math.min(...latencies);
    const maxLatency = Math.max(...latencies);

    console.log(`Tests run: ${results.length}`);
    console.log(`Average latency: ${avgLatency.toFixed(2)}ms`);
    console.log(`Min latency: ${minLatency}ms`);
    console.log(`Max latency: ${maxLatency}ms`);
    console.log();

    const passedTests = results.filter(r => r.latencyMs < 40).length;
    const passRate = (passedTests / results.length) * 100;

    console.log(`Target: <40ms`);
    console.log(`Pass rate: ${passRate.toFixed(0)}% (${passedTests}/${results.length})`);
    console.log();

    if (passRate === 100) {
      console.log('✓ All tests passed!');
    } else {
      console.log('⚠ Some tests exceeded target latency');
    }
  } else {
    console.log('No tests completed successfully');
  }

  console.log();
}

// Run tests
console.log('Starting latency tests...');
console.log('Make sure the gRPC server is running:');
console.log('  cargo run --bin paperclip-server examples');
console.log();

setTimeout(() => {
  runLatencyTests().catch((error) => {
    console.error('Test failed:', error);
    process.exit(1);
  });
}, 1000);
