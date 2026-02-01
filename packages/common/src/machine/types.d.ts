/**
 * Machine types - core state management infrastructure
 */
export type Disposable = {
    dispose: () => void;
};
/**
 * Base event type - all events follow this pattern
 */
export type BaseEvent<Type extends string, Payload = undefined> = Payload extends undefined ? {
    type: Type;
} : {
    type: Type;
    payload: Payload;
};
export type AnyEvent = {
    type: string;
    payload?: unknown;
};
/**
 * Extract a specific event type from an event union
 */
export type PickEvent<Event extends AnyEvent, Type extends string> = Extract<Event, {
    type: Type;
}>;
/**
 * Reducer - pure state transition function
 */
export type Reducer<Event extends AnyEvent, State> = (event: Event, state: State) => State;
/**
 * Props ref - stable reference with current props
 */
export type PropsRef<Props> = {
    readonly current: Props;
};
/**
 * Engine - handles side effects
 */
export type Engine<Event extends AnyEvent, State = unknown, Props = unknown> = {
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
export type EngineFactory<Event extends AnyEvent, State, Props> = (props: PropsRef<Props>, machine: MachineHandle<Event, State>) => Engine<Event, State>;
/**
 * Machine definition
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
    start?(): void;
    dispose(): void;
};
/**
 * Check if a selector's result has changed
 */
export declare function changed<State, T>(prevState: State, newState: State, selector: (state: State) => T): boolean;
//# sourceMappingURL=types.d.ts.map