/**
 * Workspace state
 */

import type { VDocument, OutlineNode } from '@paperclip/workspace-client';

/**
 * Connection status
 */
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

/**
 * Document state for a single file
 */
export interface DocumentState {
  filePath: string;
  vdom: VDocument | null;
  version: number;
  outline: OutlineNode[] | null;
  error: string | null;
  loading: boolean;
}

/**
 * Workspace state
 */
export interface WorkspaceState {
  /**
   * Connection status
   */
  connectionStatus: ConnectionStatus;

  /**
   * Server address
   */
  address: string | null;

  /**
   * Last connection error
   */
  connectionError: string | null;

  /**
   * Documents by file path
   */
  documents: Record<string, DocumentState>;

  /**
   * Currently active file path
   */
  activeFilePath: string | null;

  /**
   * Pending mutations (optimistic updates)
   */
  pendingMutations: Array<{
    mutationId: string;
    filePath: string;
    timestamp: number;
  }>;
}

/**
 * Initial workspace state
 */
export const initialState: WorkspaceState = {
  connectionStatus: 'disconnected',
  address: null,
  connectionError: null,
  documents: {},
  activeFilePath: null,
  pendingMutations: [],
};
