/**
 * Machine instance - creates and manages machine state
 */

import {
  AnyEvent,
  Disposable,
  EngineFactory,
  MachineHandle,
  MachineInstance,
  PropsRef,
  Reducer,
} from "./types";

type Listener<State> = (state: State, prevState: State) => void;

/**
 * Creates a machine instance with state management and optional engine
 */
export function createMachineInstance<Event extends AnyEvent, State, Props>(options: {
  reducer: Reducer<Event, State>;
  engine?: EngineFactory<Event, State, Props>;
  initialState: State;
  propsRef: PropsRef<Props>;
  dispatch: (event: Event) => void;
}): MachineInstance<Event, State, Props> {
  const { reducer, engine: engineFactory, initialState, dispatch } = options;

  let state = Object.freeze(initialState);
  let currentProps = options.propsRef.current;
  let disposed = false;
  const listeners: Listener<State>[] = [];

  // Props ref that always returns current props
  const propsRef: PropsRef<Props> = {
    get current() {
      return currentProps;
    },
  };

  // Machine handle for engines
  const machine: MachineHandle<Event, State> = {
    dispatch(event: Event) {
      if (disposed) return;
      dispatch(event);
    },
    getState() {
      return state;
    },
  };

  let _started = false;

  const instance: MachineInstance<Event, State, Props> = {
    getState() {
      return state;
    },

    subscribe(listener: Listener<State>): Disposable {
      listeners.push(listener);
      return {
        dispose() {
          const index = listeners.indexOf(listener);
          if (index !== -1) {
            listeners.splice(index, 1);
          }
        },
      };
    },

    handleEvent(event: Event) {
      if (disposed) return;

      console.log("[Instance] handleEvent:", event.type, "listeners:", listeners.length);

      const prevState = state;
      const newState = Object.freeze(reducer(event, prevState));
      state = newState;

      // Notify listeners (LIFO order)
      console.log("[Instance] Notifying", listeners.length, "listeners");
      for (let i = listeners.length - 1; i >= 0; i--) {
        listeners[i](newState, prevState);
      }

      // Run engine side effects
      engine?.handleEvent(event, prevState);
    },

    handlePropsChange(prevProps: Props, nextProps: Props) {
      if (disposed) return;
      engine?.handlePropsChange?.(prevProps, nextProps);
    },

    updateProps(props: Props) {
      currentProps = props;
    },

    start() {
      if (_started) return;
      _started = true;
      engine?.start?.();
    },

    dispose() {
      disposed = true;
      engine?.dispose?.();
      listeners.length = 0;
    },
  };

  // Create engine
  const engine = engineFactory?.(propsRef, machine);

  return instance;
}
