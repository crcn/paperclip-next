/**
 * React integration for machine/engine pattern
 */

export {
  createDispatcher,
  useDispatcher,
  DispatchProvider,
} from './dispatchProvider.js';

export { defineMachine } from './defineMachine.js';
export type {
  DefineMachineOptions,
  DefineMachineResult,
  MachineProviderProps,
} from './defineMachine.js';

export { DispatchContext } from './context.js';
export type { MachineRegistry, MachineRegistration } from './context.js';
