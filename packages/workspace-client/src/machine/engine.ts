/**
 * Workspace engine - handles side effects
 */

import type {
  Engine,
  EngineFactory,
  MachineHandle,
  PropsRef,
} from '@paperclip/common/machine';
import type { WorkspaceClient } from '../client.js';
import type { WorkspaceEvent } from './events.js';
import type { WorkspaceState } from './state.js';

/**
 * Props for workspace engine
 */
export interface WorkspaceEngineProps {
  client: WorkspaceClient;
}

/**
 * Workspace engine - wraps WorkspaceClient and handles all side effects
 */
export const workspaceEngine: EngineFactory<
  WorkspaceEvent,
  WorkspaceState,
  WorkspaceEngineProps
> = (
  props: PropsRef<WorkspaceEngineProps>,
  machine: MachineHandle<WorkspaceEvent, WorkspaceState>
) => {
  const { client } = props.current;
  let previewStreamAbort: AbortController | null = null;

  // Forward workspace client events to machine
  const unsubscribeConnected = client.on('connected', (event) => {
    machine.dispatch({
      type: 'connected',
      payload: { address: event.address },
    });
  });

  const unsubscribePreviewUpdated = client.on('preview-updated', (event) => {
    machine.dispatch({
      type: 'preview-updated',
      payload: { update: event.update },
    });
  });

  const unsubscribeMutationAcknowledged = client.on(
    'mutation-acknowledged',
    (event) => {
      machine.dispatch({
        type: 'mutation-acknowledged',
        payload: {
          mutationId: event.mutation_id,
          newVersion: event.new_version,
        },
      });
    }
  );

  const unsubscribeMutationRebased = client.on('mutation-rebased', (event) => {
    machine.dispatch({
      type: 'mutation-rebased',
      payload: {
        originalMutationId: event.original_mutation_id,
        newVersion: event.new_version,
        reason: event.reason,
      },
    });
  });

  const unsubscribeMutationNoop = client.on('mutation-noop', (event) => {
    machine.dispatch({
      type: 'mutation-noop',
      payload: {
        mutationId: event.mutation_id,
        reason: event.reason,
      },
    });
  });

  const unsubscribeOutlineReceived = client.on('outline-received', (event) => {
    machine.dispatch({
      type: 'outline-received',
      payload: { outline: event.outline },
    });
  });

  const unsubscribeConnectionFailed = client.on('connection-failed', (event) => {
    machine.dispatch({
      type: 'connection-failed',
      payload: { error: event.error.message },
    });
  });

  const unsubscribeRpcFailed = client.on('rpc-failed', (event) => {
    machine.dispatch({
      type: 'error-occurred',
      payload: {
        operation: event.method,
        error: event.error.message,
      },
    });
  });

  const engine: Engine<WorkspaceEvent, WorkspaceState> = {
    start() {
      // Engine started, client is ready
    },

    async handleEvent(event: WorkspaceEvent, prevState: WorkspaceState) {
      switch (event.type) {
        case 'connection-requested':
          try {
            await client.connect(event.payload.address);
          } catch (error) {
            machine.dispatch({
              type: 'connection-failed',
              payload: { error: (error as Error).message },
            });
          }
          break;

        case 'preview-requested': {
          const { filePath } = event.payload;

          // Cancel previous stream if any
          if (previewStreamAbort) {
            previewStreamAbort.abort();
          }

          previewStreamAbort = new AbortController();

          // Start new stream
          (async () => {
            try {
              for await (const update of client.streamPreview(filePath)) {
                // Events are already dispatched by client event listeners
              }
            } catch (error) {
              if ((error as Error).name !== 'AbortError') {
                machine.dispatch({
                  type: 'error-occurred',
                  payload: {
                    operation: 'streamPreview',
                    error: (error as Error).message,
                  },
                });
              }
            }
          })();
          break;
        }

        case 'mutation-requested': {
          const { filePath, mutation, expectedVersion } = event.payload;
          try {
            await client.applyMutation(filePath, mutation, expectedVersion);
          } catch (error) {
            machine.dispatch({
              type: 'error-occurred',
              payload: {
                operation: 'applyMutation',
                error: (error as Error).message,
              },
            });
          }
          break;
        }

        case 'outline-requested': {
          const { filePath } = event.payload;
          try {
            await client.getOutline(filePath);
          } catch (error) {
            machine.dispatch({
              type: 'error-occurred',
              payload: {
                operation: 'getOutline',
                error: (error as Error).message,
              },
            });
          }
          break;
        }
      }
    },

    dispose() {
      // Clean up subscriptions
      unsubscribeConnected();
      unsubscribePreviewUpdated();
      unsubscribeMutationAcknowledged();
      unsubscribeMutationRebased();
      unsubscribeMutationNoop();
      unsubscribeOutlineReceived();
      unsubscribeConnectionFailed();
      unsubscribeRpcFailed();

      // Abort any running streams
      if (previewStreamAbort) {
        previewStreamAbort.abort();
      }

      // Disconnect client
      client.disconnect();
    },
  };

  return engine;
};
