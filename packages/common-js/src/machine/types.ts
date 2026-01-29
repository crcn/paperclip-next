/**
 * Core machine/engine types
 * Ported from @shay/common machine pattern
 */

export type Disposable = {
  dispose: () => void;
};

/**
 * Base event type - all events follow this pattern
 */
export type BaseEvent<
  Type extends string,
  Payload = undefined,
> = Payload extends undefined
  ? { type: Type }
  : { type: Type; payload: Payload };

export type AnyEvent = { type: string; payload?: unknown };

/**
 * Extract a specific event type from an event union
 */
export type PickEvent<Event extends AnyEvent, Type extends string> = Extract<
  Event,
  { type: Type }
>;

/**
 * Reducer - pure state transition function
 * (event, state) => newState
 */
export type Reducer<Event extends AnyEvent, State> = (
  event: Event,
  state: State,
) => State;

/**
 * Props ref - stable reference with current props
 */
export type PropsRef<Props> = { readonly current: Props };

/**
 * Engine - handles side effects
 */
export type Engine<Event extends AnyEvent, State = unknown, Props = unknown> = {
  /** Called after the machine is registered and ready to dispatch events */
  start?(): void;
  handleEvent(event: Event, prevState: State): void;
  handlePropsChange?(prevProps: Props, nextProps: Props): void;
  dispose?: () => void;
};

/**
 * Machine handle - minimal interface for engines to interact with machine
 */
export type MachineHandle<Event extends AnyEvent, State> = {
  dispatch(event: Event): void;
  getState(): State;
};

/**
 * Engine factory - creates an engine with props ref and machine handle
 */
export type EngineFactory<Event extends AnyEvent, State, Props> = (
  props: PropsRef<Props>,
  machine: MachineHandle<Event, State>,
) => Engine<Event, State>;

/**
 * Machine definition - the configuration for a machine
 */
export type MachineDefinition<Event extends AnyEvent, State, Props> = {
  reducer: Reducer<Event, State>;
  engine?: EngineFactory<Event, State, Props>;
  initialState: State;
};

/**
 * Machine instance - a running machine with state
 */
export type MachineInstance<Event extends AnyEvent, State, Props = unknown> = {
  getState(): State;
  subscribe(listener: (state: State, prevState: State) => void): Disposable;
  handleEvent(event: Event): void;
  handlePropsChange(prevProps: Props, nextProps: Props): void;
  updateProps?(props: Props): void;
  /** Called by registry after registration to start the engine */
  start?(): void;
  dispose(): void;
};

/**
 * Dispatcher function type
 */
export type Dispatcher<Event extends AnyEvent> = (event: Event) => void;

/**
 * Deep equality comparison
 */
function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (a == null || b == null) return a === b;
  if (typeof a !== typeof b) return false;

  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
      if (!deepEqual(a[i], b[i])) return false;
    }
    return true;
  }

  if (typeof a === "object" && typeof b === "object") {
    const aKeys = Object.keys(a as object);
    const bKeys = Object.keys(b as object);
    if (aKeys.length !== bKeys.length) return false;
    for (const key of aKeys) {
      if (
        !bKeys.includes(key) ||
        !deepEqual(
          (a as Record<string, unknown>)[key],
          (b as Record<string, unknown>)[key],
        )
      ) {
        return false;
      }
    }
    return true;
  }

  return false;
}

/**
 * Check if a selector's result has changed between two states
 * Uses deep equality for comparison
 */
export function changed<State, T>(
  prevState: State,
  newState: State,
  selector: (state: State) => T,
): boolean {
  const prevValue = selector(prevState);
  const newValue = selector(newState);
  return !deepEqual(prevValue, newValue);
}
