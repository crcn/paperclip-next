import React, { useMemo } from 'react';
import { DispatchProvider } from '@paperclip/common/machine/react';
import { GrpcTransport, createWorkspaceClient } from '@paperclip/workspace-client';
import { WorkspaceMachine } from '@paperclip/workspace-client/machine';
import { WorkspaceDemo } from './WorkspaceDemo';

export function App() {
  // Create client once
  const client = useMemo(() => {
    const transport = new GrpcTransport();
    return createWorkspaceClient(transport);
  }, []);

  return (
    <DispatchProvider>
      <WorkspaceMachine.Provider props={{ client }}>
        <WorkspaceDemo />
      </WorkspaceMachine.Provider>
    </DispatchProvider>
  );
}
