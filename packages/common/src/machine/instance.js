/**
 * Machine instance - creates and manages machine state
 */
/**
 * Creates a machine instance with state management and optional engine
 */
export function createMachineInstance(options) {
    const { reducer, engine: engineFactory, initialState, dispatch } = options;
    let state = Object.freeze(initialState);
    let currentProps = options.propsRef.current;
    let disposed = false;
    const listeners = [];
    // Props ref that always returns current props
    const propsRef = {
        get current() {
            return currentProps;
        },
    };
    // Machine handle for engines
    const machine = {
        dispatch(event) {
            if (disposed)
                return;
            dispatch(event);
        },
        getState() {
            return state;
        },
    };
    let _started = false;
    const instance = {
        getState() {
            return state;
        },
        subscribe(listener) {
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
        handleEvent(event) {
            if (disposed)
                return;
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
        handlePropsChange(prevProps, nextProps) {
            if (disposed)
                return;
            engine?.handlePropsChange?.(prevProps, nextProps);
        },
        updateProps(props) {
            currentProps = props;
        },
        start() {
            if (_started)
                return;
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
//# sourceMappingURL=instance.js.map