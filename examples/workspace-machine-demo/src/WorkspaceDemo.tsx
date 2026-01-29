import React, { useState, useContext } from 'react';
import { DispatchContext } from '@paperclip/common/machine/react';
import { WorkspaceMachine } from '@paperclip/workspace-client/machine';
import type { WorkspaceEvent } from '@paperclip/workspace-client/machine';
import './WorkspaceDemo.css';

export function WorkspaceDemo() {
  const dispatch = useContext(DispatchContext);
  const [address, setAddress] = useState('localhost:50051');
  const [filePath, setFilePath] = useState('button.pc');

  // Select state
  const connectionStatus = WorkspaceMachine.useSelector(
    (state) => state.connectionStatus
  );
  const connectionError = WorkspaceMachine.useSelector(
    (state) => state.connectionError
  );
  const activeDoc = WorkspaceMachine.useSelector((state) => {
    const path = state.activeFilePath;
    return path ? state.documents[path] : null;
  });
  const documents = WorkspaceMachine.useSelector((state) => state.documents);

  // Dispatch events
  const connect = () => {
    dispatch?.dispatch({
      type: 'connection-requested',
      payload: { address },
    } as WorkspaceEvent);
  };

  const disconnect = () => {
    dispatch?.dispatch({
      type: 'disconnected',
    } as WorkspaceEvent);
  };

  const loadPreview = () => {
    dispatch?.dispatch({
      type: 'preview-requested',
      payload: { filePath },
    } as WorkspaceEvent);
  };

  const loadOutline = () => {
    dispatch?.dispatch({
      type: 'outline-requested',
      payload: { filePath },
    } as WorkspaceEvent);
  };

  return (
    <div className="workspace-demo">
      <header className="header">
        <h1>Workspace Machine Demo</h1>
        <p>Real-time preview using machine/engine pattern</p>
      </header>

      <div className="container">
        {/* Connection Panel */}
        <div className="panel">
          <h2>Connection</h2>
          <div className="status-badge" data-status={connectionStatus}>
            {connectionStatus}
          </div>

          {connectionError && (
            <div className="error">
              <strong>Error:</strong> {connectionError}
            </div>
          )}

          <div className="form-group">
            <label>Server Address:</label>
            <input
              type="text"
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              placeholder="localhost:50051"
              disabled={connectionStatus === 'connected'}
            />
          </div>

          <div className="button-group">
            <button
              onClick={connect}
              disabled={connectionStatus === 'connected' || connectionStatus === 'connecting'}
            >
              Connect
            </button>
            <button
              onClick={disconnect}
              disabled={connectionStatus === 'disconnected'}
            >
              Disconnect
            </button>
          </div>
        </div>

        {/* File Preview Panel */}
        <div className="panel">
          <h2>File Preview</h2>

          <div className="form-group">
            <label>File Path:</label>
            <input
              type="text"
              value={filePath}
              onChange={(e) => setFilePath(e.target.value)}
              placeholder="button.pc"
            />
          </div>

          <div className="button-group">
            <button
              onClick={loadPreview}
              disabled={connectionStatus !== 'connected'}
            >
              Load Preview
            </button>
            <button
              onClick={loadOutline}
              disabled={connectionStatus !== 'connected'}
            >
              Load Outline
            </button>
          </div>

          {activeDoc && (
            <div className="doc-info">
              <div className="info-row">
                <span className="label">Version:</span>
                <span className="value">{activeDoc.version}</span>
              </div>
              <div className="info-row">
                <span className="label">Loading:</span>
                <span className="value">{activeDoc.loading ? 'Yes' : 'No'}</span>
              </div>
              {activeDoc.error && (
                <div className="error">
                  <strong>Error:</strong> {activeDoc.error}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Documents List */}
        <div className="panel">
          <h2>Documents ({Object.keys(documents).length})</h2>
          <div className="documents-list">
            {Object.entries(documents).map(([path, doc]) => (
              <div key={path} className="document-item">
                <div className="doc-path">{path}</div>
                <div className="doc-meta">
                  Version: {doc.version} |
                  {doc.loading && ' Loading...'}
                  {doc.vdom && ' ✓ VDOM'}
                  {doc.outline && ` ✓ Outline (${doc.outline.length})`}
                </div>
              </div>
            ))}
            {Object.keys(documents).length === 0 && (
              <div className="empty-state">
                No documents loaded yet
              </div>
            )}
          </div>
        </div>

        {/* VDOM Preview */}
        {activeDoc?.vdom && (
          <div className="panel vdom-panel">
            <h2>VDOM Preview</h2>
            <pre className="vdom-preview">
              {JSON.stringify(activeDoc.vdom, null, 2)}
            </pre>
          </div>
        )}

        {/* Outline Preview */}
        {activeDoc?.outline && (
          <div className="panel outline-panel">
            <h2>Document Outline</h2>
            <div className="outline-tree">
              {activeDoc.outline.map((node) => (
                <div key={node.node_id} className="outline-node">
                  <span className="node-label">{node.label || node.node_id}</span>
                  <span className="node-type">{node.type}</span>
                  {node.child_ids.length > 0 && (
                    <span className="node-children">
                      {node.child_ids.length} children
                    </span>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
