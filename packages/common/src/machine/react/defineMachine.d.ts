import React from "react";
import { AnyEvent, EngineFactory, MachineInstance, Reducer } from "../types";
export type DefineMachineOptions<Event extends AnyEvent, State, Props> = {
    reducer: Reducer<Event, State>;
    engine?: EngineFactory<Event, State, Props>;
    initialState: State;
};
export type MachineProviderProps<Props, Event extends AnyEvent, State> = Props extends Record<string, never> ? {
    children?: React.ReactNode;
    instance?: MachineInstance<Event, State, Props>;
} : {
    props: Props;
    children?: React.ReactNode;
    instance?: MachineInstance<Event, State, Props>;
};
export type DefineMachineResult<Event extends AnyEvent, State, Props> = {
    Provider: React.FC<MachineProviderProps<Props, Event, State>>;
    useSelector: <R>(selector: (state: State) => R) => R;
    createInstance: (props: Props, dispatch: (event: Event) => void) => MachineInstance<Event, State, Props>;
};
/**
 * Defines a machine with its reducer, engine, and initial state
 */
export declare function defineMachine<Event extends AnyEvent, State, Props = Record<string, never>>(options: DefineMachineOptions<Event, State, Props>): DefineMachineResult<Event, State, Props>;
//# sourceMappingURL=defineMachine.d.ts.map