"use client";
import { jsx as _jsx } from "react/jsx-runtime";
import { useRef, useMemo } from "react";
import { DispatchContext, } from "./context";
/**
 * Create dispatcher at module level - survives React remounts
 */
export function createDispatcher() {
    const registrations = [];
    let orderCounter = 0;
    const register = (registration) => {
        registration.order = orderCounter++;
        // Insert in sorted position (descending by order)
        let insertIndex = registrations.length;
        for (let i = 0; i < registrations.length; i++) {
            if (registration.order > registrations[i].order) {
                insertIndex = i;
                break;
            }
        }
        registrations.splice(insertIndex, 0, registration);
        // Start the engine after registration
        registration.instance?.start?.();
        return () => {
            const index = registrations.indexOf(registration);
            if (index !== -1) {
                registrations.splice(index, 1);
            }
        };
    };
    const dispatch = (event) => {
        for (let i = registrations.length; i--;) {
            registrations[i].instance?.handleEvent(event);
        }
    };
    return { dispatch, register };
}
/**
 * Hook to create dispatcher
 */
export function useDispatcher() {
    const registrationsRef = useRef([]);
    const orderCounterRef = useRef(0);
    return useMemo(() => {
        const register = (registration) => {
            const registrations = registrationsRef.current;
            registration.order = orderCounterRef.current++;
            let insertIndex = registrations.length;
            for (let i = 0; i < registrations.length; i++) {
                if (registration.order > registrations[i].order) {
                    insertIndex = i;
                    break;
                }
            }
            registrations.splice(insertIndex, 0, registration);
            registration.instance?.start?.();
            return () => {
                const index = registrations.indexOf(registration);
                if (index !== -1) {
                    registrations.splice(index, 1);
                }
            };
        };
        const dispatch = (event) => {
            const registrations = registrationsRef.current;
            for (let i = registrations.length; i--;) {
                registrations[i].instance?.handleEvent(event);
            }
        };
        return { dispatch, register };
    }, []);
}
/**
 * Root provider that sets up global dispatch with event bubbling
 */
export function DispatchProvider({ children, value }) {
    const internalValue = useDispatcher();
    const contextValue = value ?? internalValue;
    return (_jsx(DispatchContext.Provider, { value: contextValue, children: children }));
}
//# sourceMappingURL=dispatchProvider.js.map