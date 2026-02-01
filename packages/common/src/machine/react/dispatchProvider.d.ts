import React from "react";
import { MachineRegistry } from "./context";
type DispatchProviderProps = {
    children: React.ReactNode;
    value?: MachineRegistry;
};
/**
 * Create dispatcher at module level - survives React remounts
 */
export declare function createDispatcher(): MachineRegistry;
/**
 * Hook to create dispatcher
 */
export declare function useDispatcher(): MachineRegistry;
/**
 * Root provider that sets up global dispatch with event bubbling
 */
export declare function DispatchProvider({ children, value }: DispatchProviderProps): import("react/jsx-runtime").JSX.Element;
export {};
//# sourceMappingURL=dispatchProvider.d.ts.map