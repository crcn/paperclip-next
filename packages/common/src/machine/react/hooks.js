"use client";
import { useContext } from "react";
import { DispatchContext } from "./context";
/**
 * Get the global dispatch function
 */
export function useDispatch() {
    const ctx = useContext(DispatchContext);
    if (!ctx) {
        throw new Error("useDispatch must be used within a DispatchProvider");
    }
    return ctx.dispatch;
}
//# sourceMappingURL=hooks.js.map