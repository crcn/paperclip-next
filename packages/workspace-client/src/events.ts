/**
 * Workspace event types (past tense - describing what happened)
 */

import type {
  PreviewUpdate,
  FileEvent,
  MutationResponse,
  OutlineResponse,
} from './types.js';

/**
 * Base workspace event
 */
export interface WorkspaceEvent {
  type: string;
  timestamp: number;
}

/**
 * Connection was established
 */
export interface Connected extends WorkspaceEvent {
  type: 'connected';
  address: string;
}

/**
 * Connection was lost
 */
export interface Disconnected extends WorkspaceEvent {
  type: 'disconnected';
  reason?: string;
}

/**
 * Preview update was received
 */
export interface PreviewUpdated extends WorkspaceEvent {
  type: 'preview-updated';
  update: PreviewUpdate;
}

/**
 * File event was received
 */
export interface FileChanged extends WorkspaceEvent {
  type: 'file-changed';
  event: FileEvent;
}

/**
 * Mutation was acknowledged by server
 */
export interface MutationAcknowledged extends WorkspaceEvent {
  type: 'mutation-acknowledged';
  mutation_id: string;
  new_version: number;
}

/**
 * Mutation was rebased (transformed due to concurrent changes)
 */
export interface MutationRebased extends WorkspaceEvent {
  type: 'mutation-rebased';
  original_mutation_id: string;
  new_version: number;
  reason: string;
}

/**
 * Mutation had no effect
 */
export interface MutationNoop extends WorkspaceEvent {
  type: 'mutation-noop';
  mutation_id: string;
  reason: string;
}

/**
 * Document outline was received
 */
export interface OutlineReceived extends WorkspaceEvent {
  type: 'outline-received';
  outline: OutlineResponse;
}

/**
 * Connection error occurred
 */
export interface ConnectionFailed extends WorkspaceEvent {
  type: 'connection-failed';
  error: Error;
}

/**
 * RPC error occurred
 */
export interface RpcFailed extends WorkspaceEvent {
  type: 'rpc-failed';
  method: string;
  error: Error;
}

/**
 * Union of all workspace events
 */
export type WorkspaceEventUnion =
  | Connected
  | Disconnected
  | PreviewUpdated
  | FileChanged
  | MutationAcknowledged
  | MutationRebased
  | MutationNoop
  | OutlineReceived
  | ConnectionFailed
  | RpcFailed;

/**
 * Event listener callback
 */
export type EventListener<T extends WorkspaceEvent = WorkspaceEvent> = (
  event: T
) => void;

/**
 * Simple event emitter for workspace events
 */
export class EventEmitter {
  private listeners = new Map<string, Set<EventListener>>();

  /**
   * Register an event listener
   */
  on<T extends WorkspaceEvent>(
    eventType: T['type'],
    listener: EventListener<T>
  ): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }

    this.listeners.get(eventType)!.add(listener as EventListener);

    // Return unsubscribe function
    return () => {
      this.off(eventType, listener);
    };
  }

  /**
   * Register a one-time event listener
   */
  once<T extends WorkspaceEvent>(
    eventType: T['type'],
    listener: EventListener<T>
  ): () => void {
    const wrappedListener = (event: WorkspaceEvent) => {
      this.off(eventType, wrappedListener as EventListener<T>);
      listener(event as T);
    };

    return this.on(eventType, wrappedListener as EventListener<T>);
  }

  /**
   * Unregister an event listener
   */
  off<T extends WorkspaceEvent>(
    eventType: T['type'],
    listener: EventListener<T>
  ): void {
    const listeners = this.listeners.get(eventType);
    if (listeners) {
      listeners.delete(listener as EventListener);
    }
  }

  /**
   * Emit an event to all registered listeners
   */
  emit<T extends WorkspaceEvent>(event: T): void {
    const listeners = this.listeners.get(event.type);
    if (listeners) {
      for (const listener of listeners) {
        try {
          listener(event);
        } catch (error) {
          console.error(`Error in ${event.type} listener:`, error);
        }
      }
    }
  }

  /**
   * Remove all listeners for a specific event type
   * If no event type is provided, removes all listeners
   */
  removeAllListeners(eventType?: string): void {
    if (eventType) {
      this.listeners.delete(eventType);
    } else {
      this.listeners.clear();
    }
  }
}
