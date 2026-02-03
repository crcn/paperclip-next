"use client";

import React from "react";
import { DispatchProvider } from "@paperclip/common";
import { DesignerMachine } from "../machine";
import { Canvas } from "./Canvas";
import { LayerPanel } from "./LayerPanel";
import { StylePanel } from "./StylePanel";

const styles = {
  container: {
    display: "flex",
    height: "100%",
    width: "100%",
  },
  leftPanel: {
    width: "240px",
    flexShrink: 0,
    borderRight: "1px solid #333",
    overflow: "hidden",
  },
  canvasArea: {
    flex: 1,
    minWidth: 0,
    position: "relative" as const,
  },
  rightPanel: {
    width: "280px",
    flexShrink: 0,
    borderLeft: "1px solid #333",
    overflow: "hidden",
  },
};

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
        <div style={{ ...styles.container, ...style }} className={className}>
          <div style={styles.leftPanel}>
            <LayerPanel />
          </div>
          <div style={styles.canvasArea}>
            <Canvas style={{ width: "100%", height: "100%" }} />
          </div>
          <div style={styles.rightPanel}>
            <StylePanel />
          </div>
        </div>
      </DesignerMachine.Provider>
    </DispatchProvider>
  );
}
