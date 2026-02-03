import React from "react";
import { createRoot } from "react-dom/client";
import { Designer } from "./components/Designer";

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

  return <Designer filePath={filePath} style={{ width: "100%", height: "100%" }} />;
}

// Mount the app
const root = createRoot(document.getElementById("root")!);
root.render(<App />);
