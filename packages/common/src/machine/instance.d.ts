/**
 * Machine instance - creates and manages machine state
 */
import { AnyEvent, EngineFactory, MachineInstance, PropsRef, Reducer } from "./types";
/**
 * Creates a machine instance with state management and optional engine
 */
export declare function createMachineInstance<Event extends AnyEvent, State, Props>(options: {
    reducer: Reducer<Event, State>;
    engine?: EngineFactory<Event, State, Props>;
    initialState: State;
    propsRef: PropsRef<Props>;
    dispatch: (event: Event) => void;
}): MachineInstance<Event, State, Props>;
//# sourceMappingURL=instance.d.ts.map