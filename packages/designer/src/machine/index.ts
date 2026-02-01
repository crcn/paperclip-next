/**
 * Designer Machine
 */

import { defineMachine } from "@paperclip/common";
import { reducer } from "./reducers";
import { createSSEEngine, SSEEngineProps } from "./engines";
import {
  DEFAULT_DESIGNER_STATE,
  DesignerEvent,
  DesignerState,
} from "./state";

export const DesignerMachine = defineMachine<
  DesignerEvent,
  DesignerState,
  SSEEngineProps
>({
  reducer,
  initialState: DEFAULT_DESIGNER_STATE,
  engine: createSSEEngine,
});

export type { SSEEngineProps } from "./engines";
export * from "./state";
export * from "./geometry";
