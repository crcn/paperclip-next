"use client";

import React from "react";
import { AnyEvent, MachineInstance } from "../types";

/**
 * Registration for a mounted machine instance
 */
export type MachineRegistration<Event extends AnyEvent = AnyEvent> = {
  instance: MachineInstance<Event, unknown>;
  order: number;
};

/**
 * Global dispatch context value
 */
export type MachineRegistry<Event extends AnyEvent = AnyEvent> = {
  dispatch: (event: Event) => void;
  register: (registration: MachineRegistration<Event>) => () => void;
};

/**
 * Global dispatch context
 */
export const DispatchContext = React.createContext<MachineRegistry | null>(
  null,
);
