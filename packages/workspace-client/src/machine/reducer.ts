/**
 * Workspace reducer - pure state transitions
 */

import type { Reducer } from '@paperclip/common/machine';
import type { WorkspaceEvent } from './events.js';
import type { WorkspaceState, DocumentState } from './state.js';

/**
 * Workspace reducer - handles all state transitions
 */
export const workspaceReducer: Reducer<WorkspaceEvent, WorkspaceState> = (
  event,
  state
) => {
  switch (event.type) {
    case 'connection-requested':
      return {
        ...state,
        connectionStatus: 'connecting' as const,
        address: event.payload.address,
        connectionError: null,
      };

    case 'connected':
      return {
        ...state,
        connectionStatus: 'connected' as const,
        address: event.payload.address,
        connectionError: null,
      };

    case 'connection-failed':
      return {
        ...state,
        connectionStatus: 'error' as const,
        connectionError: event.payload.error,
      };

    case 'disconnected':
      return {
        ...state,
        connectionStatus: 'disconnected' as const,
        address: null,
        connectionError: null,
      };

    case 'preview-requested':
      return {
        ...state,
        documents: {
          ...state.documents,
          [event.payload.filePath]: {
            ...(state.documents[event.payload.filePath] || {}),
            filePath: event.payload.filePath,
            vdom: state.documents[event.payload.filePath]?.vdom || null,
            version: state.documents[event.payload.filePath]?.version || 0,
            outline: state.documents[event.payload.filePath]?.outline || null,
            error: null,
            loading: true,
          } as DocumentState,
        },
        activeFilePath: event.payload.filePath,
      };

    case 'preview-updated': {
      const { update } = event.payload;
      const doc = state.documents[update.file_path];

      // Apply patches to VDOM
      // If there's an initialize patch, use that VDOM
      // Otherwise keep existing VDOM (patch application done by rendering client)
      let vdom = doc?.vdom || null;
      if (update.patches.length > 0 && update.patches[0].initialize) {
        vdom = update.patches[0].initialize.vdom;
      }

      return {
        ...state,
        documents: {
          ...state.documents,
          [update.file_path]: {
            filePath: update.file_path,
            vdom,
            version: update.version,
            outline: doc?.outline || null,
            error: update.error || null,
            loading: false,
          },
        },
      };
    }

    case 'mutation-requested':
      return {
        ...state,
        pendingMutations: [
          ...state.pendingMutations,
          {
            mutationId: '', // Will be set by engine
            filePath: event.payload.filePath,
            timestamp: Date.now(),
          },
        ],
      };

    case 'mutation-acknowledged': {
      const { mutationId, newVersion } = event.payload;
      return {
        ...state,
        pendingMutations: state.pendingMutations.filter(
          (m) => m.mutationId !== mutationId
        ),
        documents: Object.fromEntries(
          Object.entries(state.documents).map(([path, doc]) => [
            path,
            doc.version < newVersion ? { ...doc, version: newVersion } : doc,
          ])
        ),
      };
    }

    case 'mutation-rebased':
    case 'mutation-noop': {
      const mutationId =
        event.type === 'mutation-rebased'
          ? event.payload.originalMutationId
          : event.payload.mutationId;
      return {
        ...state,
        pendingMutations: state.pendingMutations.filter(
          (m) => m.mutationId !== mutationId
        ),
      };
    }

    case 'outline-requested':
      return {
        ...state,
        documents: {
          ...state.documents,
          [event.payload.filePath]: {
            ...(state.documents[event.payload.filePath] || {}),
            filePath: event.payload.filePath,
            vdom: state.documents[event.payload.filePath]?.vdom || null,
            version: state.documents[event.payload.filePath]?.version || 0,
            outline: state.documents[event.payload.filePath]?.outline || null,
            error: null,
            loading: true,
          } as DocumentState,
        },
      };

    case 'outline-received': {
      const { outline } = event.payload;
      const filePath = state.activeFilePath;
      if (!filePath) return state;

      return {
        ...state,
        documents: {
          ...state.documents,
          [filePath]: {
            ...state.documents[filePath],
            outline: outline.nodes,
            loading: false,
          },
        },
      };
    }

    case 'error-occurred':
      return state;

    default:
      return state;
  }
};
