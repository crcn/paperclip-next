/**
 * Integration tests for WorkspaceClient
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { WorkspaceClient, createWorkspaceClient } from './client.js';
import type { Transport } from './transport/interface.js';
import type {
  PreviewRequest,
  PreviewUpdate,
  MutationRequest,
  MutationResponse,
  OutlineRequest,
  OutlineResponse,
} from './types.js';

// Mock transport for testing
class MockTransport implements Transport {
  private connected = false;
  public connectCalls: string[] = [];
  public disconnectCalls: number = 0;

  async connect(address: string): Promise<void> {
    this.connected = true;
    this.connectCalls.push(address);
  }

  async disconnect(): Promise<void> {
    this.connected = false;
    this.disconnectCalls++;
  }

  isConnected(): boolean {
    return this.connected;
  }

  async *streamPreview(request: PreviewRequest): AsyncIterableIterator<PreviewUpdate> {
    yield {
      file_path: request.root_path,
      patches: [],
      timestamp: Date.now(),
      version: 1,
      acknowledged_mutation_ids: [],
    };
  }

  async *watchFiles(): AsyncIterableIterator<any> {
    // Empty for now
  }

  async applyMutation(request: MutationRequest): Promise<MutationResponse> {
    return {
      ack: {
        mutation_id: request.mutation.mutation_id,
        new_version: request.expected_version + 1,
        timestamp: Date.now(),
      },
    };
  }

  async getDocumentOutline(request: OutlineRequest): Promise<OutlineResponse> {
    return {
      nodes: [],
      version: 1,
    };
  }
}

describe('WorkspaceClient', () => {
  let transport: MockTransport;
  let client: WorkspaceClient;

  beforeEach(() => {
    transport = new MockTransport();
    client = createWorkspaceClient(transport);
  });

  afterEach(async () => {
    await client.disconnect();
  });

  it('should connect to server', async () => {
    await client.connect('localhost:50051');
    expect(transport.connectCalls).toContain('localhost:50051');
    expect(client.isConnected()).toBe(true);
  });

  it('should disconnect from server', async () => {
    await client.connect('localhost:50051');
    await client.disconnect();
    expect(transport.disconnectCalls).toBe(1);
    expect(client.isConnected()).toBe(false);
  });

  it('should emit connected event', async () => {
    const listener = vi.fn();
    client.on('connected', listener);

    await client.connect('localhost:50051');

    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'connected',
        address: 'localhost:50051',
      })
    );
  });

  it('should stream preview updates', async () => {
    await client.connect('localhost:50051');

    const updates: PreviewUpdate[] = [];
    for await (const update of client.streamPreview('test.pc')) {
      updates.push(update);
      break; // Just get first update
    }

    expect(updates).toHaveLength(1);
    expect(updates[0].file_path).toBe('test.pc');
  });

  it('should emit preview-updated events', async () => {
    await client.connect('localhost:50051');

    const listener = vi.fn();
    client.on('preview-updated', listener);

    for await (const _ of client.streamPreview('test.pc')) {
      break;
    }

    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'preview-updated',
        update: expect.objectContaining({
          file_path: 'test.pc',
        }),
      })
    );
  });

  it('should apply mutations with generated IDs', async () => {
    await client.connect('localhost:50051');

    const response = await client.applyMutation(
      'test.pc',
      {
        update_text: {
          node_id: 'node-1',
          content: 'Hello World',
        },
      },
      1
    );

    expect(response.ack).toBeDefined();
    expect(response.ack?.mutation_id).toBeTruthy();
    expect(response.ack?.new_version).toBe(2);
  });

  it('should emit mutation-acknowledged event', async () => {
    await client.connect('localhost:50051');

    const listener = vi.fn();
    client.on('mutation-acknowledged', listener);

    await client.applyMutation(
      'test.pc',
      {
        update_text: {
          node_id: 'node-1',
          content: 'Hello World',
        },
      },
      1
    );

    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'mutation-acknowledged',
        new_version: 2,
      })
    );
  });

  it('should get document outline', async () => {
    await client.connect('localhost:50051');

    const outline = await client.getOutline('test.pc');

    expect(outline).toEqual({
      nodes: [],
      version: 1,
    });
  });

  it('should emit outline-received event', async () => {
    await client.connect('localhost:50051');

    const listener = vi.fn();
    client.on('outline-received', listener);

    await client.getOutline('test.pc');

    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'outline-received',
        outline: expect.any(Object),
      })
    );
  });

  it('should allow unsubscribing from events', async () => {
    const listener = vi.fn();
    const unsubscribe = client.on('connected', listener);

    unsubscribe();

    await client.connect('localhost:50051');

    expect(listener).not.toHaveBeenCalled();
  });

  it('should support once listeners', async () => {
    const listener = vi.fn();
    client.once('connected', listener);

    await client.connect('localhost:50051');
    await client.disconnect();
    await client.connect('localhost:50051');

    expect(listener).toHaveBeenCalledTimes(1);
  });
});
