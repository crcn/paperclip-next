"use client";

import React from "react";
import { DispatchProvider } from "@paperclip/common";
import { DesignerMachine } from "../machine";
import { Canvas } from "./Canvas";

export interface DesignerProps {
  filePath: string;
  serverUrl?: string;
  className?: string;
  style?: React.CSSProperties;
}

export function Designer({ filePath, serverUrl, className, style }: DesignerProps) {
  return (
    <DispatchProvider>
      <DesignerMachine.Provider props={{ filePath, serverUrl }}>
        <Canvas className={className} style={style} />
      </DesignerMachine.Provider>
    </DispatchProvider>
  );
}
