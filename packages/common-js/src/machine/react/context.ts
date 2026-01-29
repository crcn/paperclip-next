"use client";

// @ts-ignore - Using Yarn PnP
import React from "react";
import { AnyEvent, MachineInstance } from "../types.js";

/**
 * Registration for a mounted machine instance
 */
export type MachineRegistration<Event extends AnyEvent = AnyEvent> = {
  instance: MachineInstance<Event, unknown>;
  order: number; // For bottom-up ordering
};

/**
 * Global dispatch context value
 */
export type MachineRegistry<Event extends AnyEvent = AnyEvent> = {
  dispatch: (event: Event) => void;
  register: (registration: MachineRegistration<Event>) => () => void;
};

/**
 * Global dispatch context - single dispatch that bubbles through all machines
 */
export const DispatchContext = React.createContext<MachineRegistry | null>(
  null,
);
