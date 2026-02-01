"use client";

import { useContext } from "react";
import { AnyEvent } from "../types";
import { DispatchContext } from "./context";

/**
 * Get the global dispatch function
 */
export function useDispatch<Event extends AnyEvent>(): (event: Event) => void {
  const ctx = useContext(DispatchContext);

  if (!ctx) {
    throw new Error("useDispatch must be used within a DispatchProvider");
  }

  return ctx.dispatch as (event: Event) => void;
}
