/**
 * Workspace events (past tense - describing what happened)
 */

import type {
  PreviewUpdate,
  FileEvent,
  MutationResponse,
  OutlineResponse,
  VDocument,
} from '../types.js';
import type { BaseEvent } from '@paperclip/common/machine';

/**
 * Connection was requested
 */
export type ConnectionRequested = BaseEvent<
  'connection-requested',
  { address: string }
>;

/**
 * Connection established
 */
export type Connected = BaseEvent<'connected', { address: string }>;

/**
 * Connection failed
 */
export type ConnectionFailed = BaseEvent<
  'connection-failed',
  { error: string }
>;

/**
 * Disconnected from server
 */
export type Disconnected = BaseEvent<'disconnected'>;

/**
 * File preview was requested
 */
export type PreviewRequested = BaseEvent<'preview-requested', { filePath: string }>;

/**
 * Preview update received
 */
export type PreviewUpdated = BaseEvent<'preview-updated', { update: PreviewUpdate }>;

/**
 * File was changed on disk
 */
export type FileChanged = BaseEvent<'file-changed', { event: FileEvent }>;

/**
 * Mutation was requested
 */
export type MutationRequested = BaseEvent<
  'mutation-requested',
  {
    filePath: string;
    mutation: any; // Will use proper mutation type
    expectedVersion: number;
  }
>;

/**
 * Mutation was acknowledged by server
 */
export type MutationAcknowledged = BaseEvent<
  'mutation-acknowledged',
  {
    mutationId: string;
    newVersion: number;
  }
>;

/**
 * Mutation was rebased due to concurrent changes
 */
export type MutationRebased = BaseEvent<
  'mutation-rebased',
  {
    originalMutationId: string;
    newVersion: number;
    reason: string;
  }
>;

/**
 * Mutation had no effect
 */
export type MutationNoop = BaseEvent<
  'mutation-noop',
  {
    mutationId: string;
    reason: string;
  }
>;

/**
 * Document outline was requested
 */
export type OutlineRequested = BaseEvent<'outline-requested', { filePath: string }>;

/**
 * Document outline received
 */
export type OutlineReceived = BaseEvent<
  'outline-received',
  { outline: OutlineResponse }
>;

/**
 * Error occurred
 */
export type ErrorOccurred = BaseEvent<
  'error-occurred',
  {
    operation: string;
    error: string;
  }
>;

/**
 * Union of all workspace events
 */
export type WorkspaceEvent =
  | ConnectionRequested
  | Connected
  | ConnectionFailed
  | Disconnected
  | PreviewRequested
  | PreviewUpdated
  | FileChanged
  | MutationRequested
  | MutationAcknowledged
  | MutationRebased
  | MutationNoop
  | OutlineRequested
  | OutlineReceived
  | ErrorOccurred;
