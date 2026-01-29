/**
 * Machine/Engine pattern for state management with side effects
 *
 * A clean alternative to Redux + middleware:
 * - Reducer: Pure state transitions
 * - Engine: Side effects handler (API calls, subscriptions, etc.)
 * - No middleware, no thunks - just functions
 */

// Core types
export type {
  AnyEvent,
  BaseEvent,
  Dispatcher,
  Disposable,
  Engine,
  EngineFactory,
  MachineDefinition,
  MachineHandle,
  MachineInstance,
  PickEvent,
  PropsRef,
  Reducer,
} from './types.js';

// Utilities
export { changed } from './types.js';

// Machine instance creation (for advanced use cases)
export { createMachineInstance } from './instance.js';
