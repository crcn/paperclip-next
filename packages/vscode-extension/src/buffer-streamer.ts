/**
 * Per-document buffer streaming manager
 * Handles race conditions from concurrent text changes
 */

import * as grpc from '@grpc/grpc-js';
import { WorkspaceClient, PreviewUpdate, PreviewUpdateCallback } from './workspace-client';

// Production constants
const DEBOUNCE_MS = 100;

interface PendingStream {
  content: string;
  timer: NodeJS.Timeout;
  generation: number;
}

export class BufferStreamer {
  private activeStream: grpc.ClientReadableStream<any> | null = null;
  private pendingStream: PendingStream | null = null;
  private currentGeneration = 0;
  private disposed = false;

  constructor(
    private client: WorkspaceClient,
    private filePath: string,
    private onUpdate: PreviewUpdateCallback,
    private debounceMs: number = DEBOUNCE_MS
  ) {}

  /**
   * Update buffer content with debouncing and cancellation
   * Handles rapid typing by debouncing and canceling obsolete streams
   */
  updateContent(content: string): void {
    if (this.disposed) {
      return;
    }

    // Cancel pending debounced stream
    if (this.pendingStream) {
      clearTimeout(this.pendingStream.timer);
      this.pendingStream = null;
    }

    // Increment generation to invalidate previous stream
    this.currentGeneration++;
    const generation = this.currentGeneration;

    // Debounce: wait before starting new stream
    const timer = setTimeout(() => {
      this.pendingStream = null;

      // Cancel active stream if exists
      if (this.activeStream) {
        this.activeStream.cancel();
        this.activeStream = null;
      }

      // Start new stream
      this.startStream(content, generation);
    }, this.debounceMs);

    this.pendingStream = {
      content,
      timer,
      generation
    };
  }

  private startStream(content: string, generation: number): void {
    if (this.disposed) {
      console.log('[BufferStreamer] Skipping stream - disposed');
      return;
    }

    console.log(`[BufferStreamer] Starting stream for ${this.filePath}, content length: ${content.length}, generation: ${generation}`);

    try {
      const stream = this.client.streamBuffer(
        {
          clientId: this.client.getClientId(),
          filePath: this.filePath,
          content
        },
        (update: PreviewUpdate) => {
          console.log(`[BufferStreamer] Received update: patches=${update.patches?.length}, error=${update.error}`);
          // Ignore updates from stale generations
          if (generation === this.currentGeneration) {
            this.onUpdate(update);
          } else {
            console.log(`[BufferStreamer] Ignoring stale update (gen ${generation} vs current ${this.currentGeneration})`);
          }
        }
      );

      stream.on('end', () => {
        if (this.activeStream === stream) {
          this.activeStream = null;
        }
      });

      stream.on('error', (error: Error) => {
        console.error('[BufferStreamer] Stream error:', error);
        if (this.activeStream === stream) {
          this.activeStream = null;
        }
      });

      this.activeStream = stream;

    } catch (error) {
      console.error('[BufferStreamer] Failed to start stream:', error);
    }
  }

  /**
   * Force immediate update, bypassing debounce
   */
  flush(): void {
    if (this.pendingStream) {
      clearTimeout(this.pendingStream.timer);
      const { content, generation } = this.pendingStream;
      this.pendingStream = null;
      this.startStream(content, generation);
    }
  }

  dispose(): void {
    this.disposed = true;

    // Cancel pending timer
    if (this.pendingStream) {
      clearTimeout(this.pendingStream.timer);
      this.pendingStream = null;
    }

    // Cancel active stream
    if (this.activeStream) {
      this.activeStream.cancel();
      this.activeStream = null;
    }
  }

  isActive(): boolean {
    return !!this.activeStream || !!this.pendingStream;
  }
}
