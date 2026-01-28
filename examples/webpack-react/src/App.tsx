import React from 'react';
import './App.css';

// Copy the components from ../vite-react/src/components/ to use them here
// Example:
// import { Button } from './components/Button.pc';
// import { Card } from './components/Card.pc';

function App() {
  return (
    <div className="app">
      <div className="hero">
        <h1>Paperclip + Webpack + React</h1>
        <p>Copy Paperclip components from the Vite example to get started!</p>
        <div className="instructions">
          <h2>Quick Setup</h2>
          <ol>
            <li>Copy components from <code>../vite-react/src/components/</code></li>
            <li>Import them in this file</li>
            <li>Start building!</li>
          </ol>
        </div>
      </div>
    </div>
  );
}

export default App;
