"use client";

import React, {
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { createMachineInstance } from "../instance";
import {
  AnyEvent,
  EngineFactory,
  MachineInstance,
  PropsRef,
  Reducer,
} from "../types";
import { DispatchContext, MachineRegistration } from "./context";

const useIsomorphicLayoutEffect =
  typeof window !== "undefined" ? useLayoutEffect : useEffect;

export type DefineMachineOptions<Event extends AnyEvent, State, Props> = {
  reducer: Reducer<Event, State>;
  engine?: EngineFactory<Event, State, Props>;
  initialState: State;
};

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

export type DefineMachineResult<Event extends AnyEvent, State, Props> = {
  Provider: React.FC<MachineProviderProps<Props, Event, State>>;
  useSelector: <R>(selector: (state: State) => R) => R;
  createInstance: (
    props: Props,
    dispatch: (event: Event) => void,
  ) => MachineInstance<Event, State, Props>;
};

/**
 * Defines a machine with its reducer, engine, and initial state
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

    const propsRef = useRef<Props>(props);
    const prevPropsRef = useRef<Props | null>(null);
    const isFirstMount = useRef(true);

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

    useEffect(() => {
      if (externalInstance) return;

      const unregister = dispatchCtxRef.current.register({
        instance: instance as MachineInstance<AnyEvent, unknown>,
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
        instance!.handlePropsChange(nextProps, nextProps);
      } else if (prevProps !== null && prevProps !== nextProps) {
        instance!.handlePropsChange(prevProps, nextProps);
      }
    });

    return (
      <MachineContext.Provider value={instance}>
        {children}
      </MachineContext.Provider>
    );
  };

  const useSelector = <R,>(selector: (state: State) => R): R => {
    const instance = useContext(MachineContext);

    if (!instance) {
      throw new Error(
        "useSelector must be used within the corresponding Machine Provider",
      );
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
