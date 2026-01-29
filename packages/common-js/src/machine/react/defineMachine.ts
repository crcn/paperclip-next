"use client";

// @ts-ignore - Using Yarn PnP
import React, {
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { createMachineInstance } from "../instance.js";
import {
  AnyEvent,
  EngineFactory,
  MachineInstance,
  PropsRef,
  Reducer,
} from "../types.js";
import { DispatchContext } from "./context.js";

// Use useLayoutEffect on client, useEffect on server
const useIsomorphicLayoutEffect =
  typeof window !== "undefined" ? useLayoutEffect : useEffect;

/**
 * Options for defineMachine
 */
export type DefineMachineOptions<Event extends AnyEvent, State, Props> = {
  reducer: Reducer<Event, State>;
  engine?: EngineFactory<Event, State, Props>;
  initialState: State;
};

/**
 * Props for the generated Provider component
 */
export type MachineProviderProps<Props, Event extends AnyEvent, State> =
  Props extends Record<string, never>
    ? {
        children?: React.ReactNode;
        instance?: MachineInstance<Event, State, Props>;
      }
    : {
        props: Props;
        children?: React.ReactNode;
        instance?: MachineInstance<Event, State, Props>;
      };

/**
 * Return type of defineMachine
 */
export type DefineMachineResult<Event extends AnyEvent, State, Props> = {
  /**
   * Provider component - shares machine state with descendants
   */
  Provider: React.FC<MachineProviderProps<Props, Event, State>>;

  /**
   * Typed, memoized selector hook for this machine
   */
  useSelector: <R>(selector: (state: State) => R) => R;

  /**
   * Create a machine instance outside of React (for stable references)
   */
  createInstance: (
    props: Props,
    dispatch: (event: Event) => void,
  ) => MachineInstance<Event, State, Props>;
};

/**
 * Defines a machine with its reducer, engine, and initial state.
 * Returns a Provider component and a typed useSelector hook.
 */
export function defineMachine<
  Event extends AnyEvent,
  State,
  Props = Record<string, never>,
>(
  options: DefineMachineOptions<Event, State, Props>,
): DefineMachineResult<Event, State, Props> {
  const { reducer, engine, initialState } = options;
  const MachineContext = React.createContext<MachineInstance<
    Event,
    State
  > | null>(null);

  // Factory to create instance outside React
  const createInstance = (
    props: Props,
    dispatch: (event: Event) => void,
  ): MachineInstance<Event, State, Props> => {
    const propsRef = { current: props };
    return createMachineInstance<Event, State, Props>({
      reducer,
      engine,
      initialState,
      propsRef: propsRef as PropsRef<Props>,
      dispatch,
    });
  };

  // Provider component
  const Provider: React.FC<MachineProviderProps<Props, Event, State>> = ({
    children,
    ...rest
  }) => {
    const props = (rest as any).props ?? ({} as Props);
    const externalInstance = (rest as any).instance as
      | MachineInstance<Event, State, Props>
      | undefined;
    const dispatchCtx = useContext(DispatchContext);
    const dispatchCtxRef = useRef<any>();
    dispatchCtxRef.current = dispatchCtx;

    if (!dispatchCtx) {
      throw new Error(
        "Machine Provider must be used within a DispatchProvider",
      );
    }

    // Props refs for tracking changes
    const propsRef = useRef<Props>(props);
    const prevPropsRef = useRef<Props | null>(null);
    const isFirstMount = useRef(true);

    // Use external instance if provided, otherwise create one
    const instance =
      externalInstance ??
      useMemo(() => {
        return createMachineInstance<Event, State, Props>({
          reducer,
          engine,
          initialState,
          propsRef: propsRef as PropsRef<Props>,
          dispatch: (v) => dispatchCtxRef.current.dispatch(v),
        });
      }, []);

    // Register with dispatcher (calls instance.start() after registration)
    // Skip if external instance is provided (caller handles registration)
    useEffect(() => {
      if (externalInstance) return;

      const unregister = dispatchCtxRef.current.register({
        instance: instance as MachineInstance<AnyEvent, unknown>,
        order: 0,
      });

      return () => {
        unregister?.();
        instance.dispose();
      };
    }, [instance, dispatchCtx, externalInstance]);

    // Detect prop changes and notify engine
    useIsomorphicLayoutEffect(() => {
      const prevProps = prevPropsRef.current;
      const nextProps = props;

      // Update prev props ref for next comparison
      prevPropsRef.current = nextProps;

      // On first mount or when props change, notify engine
      if (isFirstMount.current) {
        isFirstMount.current = false;
        // Always sync on mount in case the cached instance has stale subscription
        instance!.handlePropsChange(nextProps, nextProps);
      } else if (prevProps !== null && prevProps !== nextProps) {
        instance!.handlePropsChange(prevProps, nextProps);
      }
    });

    return React.createElement(
      MachineContext.Provider,
      { value: instance },
      children,
    );
  };

  // Typed, memoized selector hook
  const useSelector = <R>(selector: (state: State) => R): R => {
    const instance = useContext(MachineContext);

    if (!instance) {
      throw new Error(
        "useSelector must be used within the corresponding Machine Provider",
      );
    }

    const [value, setValue] = useState(() => selector(instance.getState()));
    const valueRef = useRef(value);

    useEffect(() => {
      const subscription = instance.subscribe((state) => {
        const newValue = selector(state);
        // Only update if value actually changed
        if (!Object.is(valueRef.current, newValue)) {
          valueRef.current = newValue;
          setValue(newValue);
        }
      });

      return () => subscription.dispose();
    }, [instance, selector]);

    return value;
  };

  return {
    Provider,
    useSelector,
    createInstance,
  };
}
