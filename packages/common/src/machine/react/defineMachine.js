"use client";
import { jsx as _jsx } from "react/jsx-runtime";
import React, { useContext, useEffect, useLayoutEffect, useMemo, useRef, useState, } from "react";
import { createMachineInstance } from "../instance";
import { DispatchContext } from "./context";
const useIsomorphicLayoutEffect = typeof window !== "undefined" ? useLayoutEffect : useEffect;
/**
 * Defines a machine with its reducer, engine, and initial state
 */
export function defineMachine(options) {
    const { reducer, engine, initialState } = options;
    const MachineContext = React.createContext(null);
    const createInstance = (props, dispatch) => {
        const propsRef = { current: props };
        return createMachineInstance({
            reducer,
            engine,
            initialState,
            propsRef: propsRef,
            dispatch,
        });
    };
    const Provider = ({ children, ...rest }) => {
        const props = rest.props ?? {};
        const externalInstance = rest.instance;
        const dispatchCtx = useContext(DispatchContext);
        const dispatchCtxRef = useRef();
        dispatchCtxRef.current = dispatchCtx;
        if (!dispatchCtx) {
            throw new Error("Machine Provider must be used within a DispatchProvider");
        }
        const propsRef = useRef(props);
        const prevPropsRef = useRef(null);
        const isFirstMount = useRef(true);
        const instance = externalInstance ??
            useMemo(() => {
                return createMachineInstance({
                    reducer,
                    engine,
                    initialState,
                    propsRef: propsRef,
                    dispatch: (v) => dispatchCtxRef.current.dispatch(v),
                });
            }, []);
        useEffect(() => {
            if (externalInstance)
                return;
            const unregister = dispatchCtxRef.current.register({
                instance: instance,
                order: 0,
            });
            // Start engine after registration
            instance.start?.();
            return () => {
                unregister?.();
                instance.dispose();
            };
        }, [instance, dispatchCtx, externalInstance]);
        useIsomorphicLayoutEffect(() => {
            const prevProps = prevPropsRef.current;
            const nextProps = props;
            prevPropsRef.current = nextProps;
            if (isFirstMount.current) {
                isFirstMount.current = false;
                instance.handlePropsChange(nextProps, nextProps);
            }
            else if (prevProps !== null && prevProps !== nextProps) {
                instance.handlePropsChange(prevProps, nextProps);
            }
        });
        return (_jsx(MachineContext.Provider, { value: instance, children: children }));
    };
    const useSelector = (selector) => {
        const instance = useContext(MachineContext);
        if (!instance) {
            throw new Error("useSelector must be used within the corresponding Machine Provider");
        }
        // Use ref to avoid resubscribing when selector changes
        const selectorRef = useRef(selector);
        selectorRef.current = selector;
        const [value, setValue] = useState(() => {
            const initialValue = selector(instance.getState());
            console.log("[useSelector] Initial value:", initialValue);
            return initialValue;
        });
        const valueRef = useRef(value);
        useEffect(() => {
            console.log("[useSelector] Subscribing to instance");
            // Sync with current state in case we missed updates
            const currentValue = selectorRef.current(instance.getState());
            if (!Object.is(valueRef.current, currentValue)) {
                console.log("[useSelector] Syncing missed update");
                valueRef.current = currentValue;
                setValue(currentValue);
            }
            const subscription = instance.subscribe((state) => {
                const newValue = selectorRef.current(state);
                const changed = !Object.is(valueRef.current, newValue);
                console.log("[useSelector] State update - changed:", changed, "newValue:", newValue);
                if (changed) {
                    valueRef.current = newValue;
                    setValue(newValue);
                }
            });
            return () => {
                console.log("[useSelector] Unsubscribing");
                subscription.dispose();
            };
        }, [instance]); // Only depend on instance, not selector
        return value;
    };
    return {
        Provider,
        useSelector,
        createInstance,
    };
}
//# sourceMappingURL=defineMachine.js.map