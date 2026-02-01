import React from "react";
import { createRoot } from "react-dom/client";
import { DispatchProvider } from "@paperclip/common";
import { DesignerMachine } from "./machine";
import { Canvas } from "./components/Canvas";

/**
 * Root App component - wires up providers
 */
function App() {
  const params = new URLSearchParams(window.location.search);
  const filePath = params.get("file");

  if (!filePath) {
    return (
      <div style={{ padding: 20, fontFamily: "system-ui" }}>
        <h1>Paperclip Designer</h1>
        <p>No file specified. Add ?file=/path/to/file.pc to the URL.</p>
      </div>
    );
  }

  return (
    <DispatchProvider>
      <DesignerMachine.Provider props={{ filePath }}>
        <Canvas style={{ width: "100%", height: "100%" }} />
      </DesignerMachine.Provider>
    </DispatchProvider>
  );
}

// Mount the app
const root = createRoot(document.getElementById("root")!);
root.render(<App />);
