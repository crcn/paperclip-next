"use client";

// @ts-ignore - Using Yarn PnP
import React, { useRef, useMemo } from "react";
import { AnyEvent } from "../types.js";
import {
  DispatchContext,
  MachineRegistry,
  MachineRegistration,
} from "./context.js";

type DispatchProviderProps = {
  children: React.ReactNode;
  value?: MachineRegistry;
};

/**
 * Create dispatcher at module level - survives React remounts
 */
export function createDispatcher(): MachineRegistry {
  const registrations: MachineRegistration[] = [];
  let orderCounter = 0;

  const register = (registration: MachineRegistration) => {
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

    // Start the engine after registration so dispatched events work
    registration.instance?.start?.();

    return () => {
      const index = registrations.indexOf(registration);
      if (index !== -1) {
        registrations.splice(index, 1);
      }
    };
  };

  const dispatch = (event: AnyEvent) => {
    for (let i = registrations.length; i--; ) {
      registrations[i].instance?.handleEvent(event);
    }
  };

  return { dispatch, register };
}

/**
 * Hook to create dispatcher - use at stable component level (e.g. root layout)
 */
export function useDispatcher(): MachineRegistry {
  const registrationsRef = useRef<MachineRegistration[]>([]);
  const orderCounterRef = useRef(0);

  return useMemo<MachineRegistry>(() => {
    const register = (registration: MachineRegistration) => {
      const registrations = registrationsRef.current;
      registration.order = orderCounterRef.current++;

      // Insert in sorted position (descending by order)
      let insertIndex = registrations.length;
      for (let i = 0; i < registrations.length; i++) {
        if (registration.order > registrations[i].order) {
          insertIndex = i;
          break;
        }
      }
      registrations.splice(insertIndex, 0, registration);

      // Start the engine after registration so dispatched events work
      registration.instance?.start?.();

      return () => {
        const index = registrations.indexOf(registration);
        if (index !== -1) {
          registrations.splice(index, 1);
        }
      };
    };

    const dispatch = (event: AnyEvent) => {
      const registrations = registrationsRef.current;
      for (let i = registrations.length; i--; ) {
        registrations[i].instance?.handleEvent(event);
      }
    };

    return { dispatch, register };
  }, []);
}

/**
 * Root provider that sets up global dispatch with event bubbling.
 * All machine providers must be descendants of this provider.
 *
 * Events bubble bottom-up through all registered machines.
 */
export function DispatchProvider({ children, value }: DispatchProviderProps) {
  const internalValue = useDispatcher();
  const contextValue = value ?? internalValue;

  return React.createElement(
    DispatchContext.Provider,
    { value: contextValue },
    children,
  );
}
