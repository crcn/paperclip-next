/**
 * Workspace machine - state management for Paperclip workspace
 */

import { defineMachine } from '@paperclip/common/machine/react';
import { workspaceReducer } from './reducer.js';
import { workspaceEngine } from './engine.js';
import { initialState } from './state.js';
import type { WorkspaceEngineProps } from './engine.js';
import type { WorkspaceEvent } from './events.js';
import type { WorkspaceState } from './state.js';

// Export types
export type { WorkspaceEvent } from './events.js';
export type { WorkspaceState, DocumentState } from './state.js';
export type { WorkspaceEngineProps } from './engine.js';

/**
 * Workspace machine definition
 *
 * Usage:
 * ```tsx
 * import { DispatchProvider } from '@paperclip/common/machine/react';
 * import { WorkspaceMachine } from '@paperclip/workspace-machine';
 * import { GrpcTransport, createWorkspaceClient } from '@paperclip/workspace-client';
 *
 * const transport = new GrpcTransport();
 * const client = createWorkspaceClient(transport);
 *
 * function App() {
 *   return (
 *     <DispatchProvider>
 *       <WorkspaceMachine.Provider props={{ client }}>
 *         <Editor />
 *       </WorkspaceMachine.Provider>
 *     </DispatchProvider>
 *   );
 * }
 *
 * function Editor() {
 *   const status = WorkspaceMachine.useSelector(state => state.connectionStatus);
 *   const dispatch = useDispatch<WorkspaceEvent>();
 *
 *   return (
 *     <button onClick={() => dispatch({
 *       type: 'connection-requested',
 *       payload: { address: 'localhost:50051' }
 *     })}>
 *       Connect ({status})
 *     </button>
 *   );
 * }
 * ```
 */
export const WorkspaceMachine = defineMachine<
  WorkspaceEvent,
  WorkspaceState,
  WorkspaceEngineProps
>({
  reducer: workspaceReducer,
  engine: workspaceEngine,
  initialState,
});
