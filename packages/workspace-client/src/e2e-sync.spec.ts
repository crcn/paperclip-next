/**
 * End-to-End Sync Integration Tests
 *
 * These tests verify the complete cycle:
 * 1. CRDT document editing
 * 2. Sync between multiple editors
 * 3. Server parsing and VDOM generation
 * 4. VDOM patch delivery to all clients
 * 5. Rendering updates
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CRDTDocument } from './crdt';
import { DocumentSession, SyncTransport } from './sync';

/**
 * Simulated server that parses documents and generates VDOM patches.
 * This mimics what the Rust server does.
 */
class MockServer {
  private sessions = new Map<string, {
    document: CRDTDocument;
    clients: Map<string, MockServerClient>;
    version: number;
  }>();

  /**
   * Simple parser that extracts component structure.
   * Real server uses paperclip-parser.
   */
  private parse(source: string): { vdom: any; error?: string } {
    // Very simple parsing - just check for basic structure
    const componentMatch = source.match(/component\s+(\w+)\s*\{/);
    if (!componentMatch) {
      return {
        vdom: null,
        error: `Parse error: No component found`,
      };
    }

    const componentName = componentMatch[1];

    // Check for render block
    const renderMatch = source.match(/render\s+(\w+)\s*\{/);
    if (!renderMatch) {
      return {
        vdom: null,
        error: `Parse error: No render block in component ${componentName}`,
      };
    }

    const tagName = renderMatch[1];

    // Extract text content
    const textMatches = [...source.matchAll(/text\s+"([^"]+)"/g)];
    const children = textMatches.map((m, i) => ({
      type: 'text',
      id: `text-${i}`,
      content: m[1],
    }));

    // Extract nested elements
    const divMatches = [...source.matchAll(/(\w+)\s*\{[^}]*\}/g)];

    return {
      vdom: {
        type: 'element',
        tag: tagName,
        id: `${componentName}-root`,
        attributes: {},
        children,
        componentName,
      },
    };
  }

  /**
   * Join or create a session.
   */
  join(filePath: string, clientId: string, client: MockServerClient, initialStateVector?: Uint8Array): {
    documentState: Uint8Array;
    stateVector: Uint8Array;
    vdom: any;
    version: number;
  } {
    let session = this.sessions.get(filePath);

    if (!session) {
      session = {
        document: new CRDTDocument(),
        clients: new Map(),
        version: 0,
      };
      this.sessions.set(filePath, session);
    }

    session.clients.set(clientId, client);

    const docState = session.document.encodeState();
    const stateVector = session.document.getStateVector();
    const parseResult = this.parse(session.document.getText());

    return {
      documentState: docState,
      stateVector,
      vdom: parseResult.vdom,
      version: session.version,
    };
  }

  /**
   * Apply CRDT update from a client.
   */
  applyUpdate(filePath: string, clientId: string, update: Uint8Array): void {
    const session = this.sessions.get(filePath);
    if (!session) return;

    // Apply to server's document
    session.document.applyUpdate(update);
    session.version++;

    // Parse and generate VDOM
    const source = session.document.getText();
    const parseResult = this.parse(source);

    // Broadcast CRDT update to other clients
    for (const [id, client] of session.clients) {
      if (id !== clientId) {
        client.receiveUpdate(update);
      }
    }

    // Broadcast VDOM to all clients
    for (const [id, client] of session.clients) {
      if (parseResult.vdom) {
        client.receiveVDOM({
          patches: [{ type: 'replace', vdom: parseResult.vdom }],
          version: session.version,
          originClientId: clientId,
        });
      } else if (parseResult.error) {
        client.receiveVDOM({
          error: parseResult.error,
          patches: [],
          version: session.version,
        });
      }
    }
  }

  /**
   * Get current document text (for assertions).
   */
  getDocumentText(filePath: string): string {
    return this.sessions.get(filePath)?.document.getText() ?? '';
  }

  /**
   * Disconnect a client.
   */
  disconnect(filePath: string, clientId: string): void {
    const session = this.sessions.get(filePath);
    if (session) {
      session.clients.delete(clientId);
      if (session.clients.size === 0) {
        this.sessions.delete(filePath);
      }
    }
  }
}

/**
 * Mock client connection to the server.
 */
class MockServerClient {
  public receivedUpdates: Uint8Array[] = [];
  public receivedVDOM: any[] = [];
  public receivedCSSOM: any[] = [];

  private updateHandler: ((update: Uint8Array) => void) | null = null;
  private vdomHandler: ((vdom: any) => void) | null = null;
  private cssomHandler: ((cssom: any) => void) | null = null;

  receiveUpdate(update: Uint8Array): void {
    this.receivedUpdates.push(update);
    this.updateHandler?.(update);
  }

  receiveVDOM(vdom: any): void {
    this.receivedVDOM.push(vdom);
    this.vdomHandler?.(vdom);
  }

  receiveCSSOM(cssom: any): void {
    this.receivedCSSOM.push(cssom);
    this.cssomHandler?.(cssom);
  }

  onUpdate(handler: (update: Uint8Array) => void): void {
    this.updateHandler = handler;
  }

  onVDOM(handler: (vdom: any) => void): void {
    this.vdomHandler = handler;
  }

  onCSSOM(handler: (cssom: any) => void): void {
    this.cssomHandler = handler;
  }
}

/**
 * Transport that connects to the mock server.
 */
function createMockServerTransport(
  server: MockServer,
  filePath: string,
  clientId: string
): SyncTransport & { client: MockServerClient; connect: () => void } {
  const client = new MockServerClient();
  let connected = false;

  const updateHandlers = new Set<(update: Uint8Array) => void>();
  const vdomHandlers = new Set<(vdom: any) => void>();
  const cssomHandlers = new Set<(cssom: any) => void>();

  // Wire up client to handlers
  client.onUpdate((update) => {
    for (const handler of updateHandlers) {
      handler(update);
    }
  });

  client.onVDOM((vdom) => {
    for (const handler of vdomHandlers) {
      handler(vdom);
    }
  });

  client.onCSSOM((cssom) => {
    for (const handler of cssomHandlers) {
      handler(cssom);
    }
  });

  return {
    client,

    connect(): void {
      const result = server.join(filePath, clientId, client);
      connected = true;
    },

    async sendUpdate(filePath: string, update: Uint8Array, stateVector: Uint8Array): Promise<void> {
      if (!connected) throw new Error('Not connected');
      server.applyUpdate(filePath, clientId, update);
    },

    onUpdate(callback: (update: Uint8Array) => void): () => void {
      updateHandlers.add(callback);
      return () => updateHandlers.delete(callback);
    },

    onVDOM(callback: (vdom: any) => void): () => void {
      vdomHandlers.add(callback);
      return () => vdomHandlers.delete(callback);
    },

    onCSSOM(callback: (cssom: any) => void): () => void {
      cssomHandlers.add(callback);
      return () => cssomHandlers.delete(callback);
    },
  };
}

describe('End-to-End Sync Integration', () => {
  let server: MockServer;

  beforeEach(() => {
    server = new MockServer();
  });

  describe('single editor workflow', () => {
    it('edits document and receives VDOM updates', async () => {
      const transport = createMockServerTransport(server, '/test.pc', 'editor1');
      transport.connect();

      const session = new DocumentSession('/test.pc', transport);
      const vdomUpdates: any[] = [];

      session.onVDOMChange((vdom) => {
        vdomUpdates.push(vdom);
      });

      // Write a valid component
      session.setText('component Button { render div { text "Click me" } }');

      // Wait for sync
      await new Promise(resolve => setTimeout(resolve, 20));

      // Should receive VDOM update
      expect(vdomUpdates.length).toBeGreaterThan(0);
      expect(vdomUpdates[0].patches).toBeDefined();

      session.dispose();
    });

    it('receives parse errors for invalid content', async () => {
      const transport = createMockServerTransport(server, '/test.pc', 'editor1');
      transport.connect();

      const session = new DocumentSession('/test.pc', transport);
      const vdomUpdates: any[] = [];

      session.onVDOMChange((vdom) => {
        vdomUpdates.push(vdom);
      });

      // Write invalid content (no component)
      session.setText('not a valid component');

      await new Promise(resolve => setTimeout(resolve, 20));

      // Should receive error
      expect(vdomUpdates.length).toBeGreaterThan(0);
      expect(vdomUpdates[0].error).toBeDefined();

      session.dispose();
    });

    it('tracks dirty state through edit cycle', async () => {
      const transport = createMockServerTransport(server, '/test.pc', 'editor1');
      transport.connect();

      const session = new DocumentSession('/test.pc', transport);

      expect(session.isDirty()).toBe(false);

      session.setText('component A { render div {} }');
      expect(session.isDirty()).toBe(true);

      // Simulate server acknowledging (mark clean)
      session.markClean();
      expect(session.isDirty()).toBe(false);

      // Another edit makes it dirty again
      session.insert(0, '// comment\n');
      expect(session.isDirty()).toBe(true);

      session.dispose();
    });
  });

  describe('two editor synchronization', () => {
    it('syncs edits from editor A to editor B', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');

      transportA.connect();
      transportB.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);

      // Editor A makes a change
      sessionA.setText('component Button { render div { text "Hello" } }');

      await new Promise(resolve => setTimeout(resolve, 20));

      // Editor B should receive the update via CRDT
      expect(sessionB.getText()).toBe(sessionA.getText());

      sessionA.dispose();
      sessionB.dispose();
    });

    it('syncs edits from editor B to editor A', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');

      transportA.connect();
      transportB.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);

      // Editor B makes a change
      sessionB.setText('component Card { render section { text "Content" } }');

      await new Promise(resolve => setTimeout(resolve, 20));

      // Editor A should receive the update
      expect(sessionA.getText()).toBe(sessionB.getText());

      sessionA.dispose();
      sessionB.dispose();
    });

    it('handles concurrent edits from both editors', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');

      transportA.connect();
      transportB.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);

      // Both start with same content
      sessionA.setText('component Test { render div {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      // Concurrent edits
      sessionA.insert(sessionA.getText().indexOf('div'), 'main-');
      sessionB.insert(sessionB.getText().indexOf('{}'), ' text "Hi" ');

      await new Promise(resolve => setTimeout(resolve, 50));

      // Both should converge
      expect(sessionA.getText()).toBe(sessionB.getText());

      // Both edits should be present
      const text = sessionA.getText();
      expect(text).toContain('main-div');
      expect(text).toContain('text "Hi"');

      sessionA.dispose();
      sessionB.dispose();
    });

    it('both editors receive VDOM updates', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');

      transportA.connect();
      transportB.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);

      const vdomA: any[] = [];
      const vdomB: any[] = [];

      sessionA.onVDOMChange((vdom) => vdomA.push(vdom));
      sessionB.onVDOMChange((vdom) => vdomB.push(vdom));

      // Editor A makes a change
      sessionA.setText('component Button { render button { text "Click" } }');

      await new Promise(resolve => setTimeout(resolve, 20));

      // Both should receive VDOM
      expect(vdomA.length).toBeGreaterThan(0);
      expect(vdomB.length).toBeGreaterThan(0);

      sessionA.dispose();
      sessionB.dispose();
    });
  });

  describe('three editor synchronization', () => {
    it('syncs across three editors', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');
      const transportC = createMockServerTransport(server, '/test.pc', 'editorC');

      transportA.connect();
      transportB.connect();
      transportC.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);
      const sessionC = new DocumentSession('/test.pc', transportC);

      // Each editor adds content
      sessionA.setText('component A { render div {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      sessionB.insert(sessionB.getText().length, '\ncomponent B { render span {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      sessionC.insert(sessionC.getText().length, '\ncomponent C { render p {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      // All should converge
      expect(sessionA.getText()).toBe(sessionB.getText());
      expect(sessionB.getText()).toBe(sessionC.getText());

      // All components should be present
      const text = sessionA.getText();
      expect(text).toContain('component A');
      expect(text).toContain('component B');
      expect(text).toContain('component C');

      sessionA.dispose();
      sessionB.dispose();
      sessionC.dispose();
    });
  });

  describe('editor join/leave scenarios', () => {
    it('new editor joins and gets current state', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      transportA.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      sessionA.setText('component Existing { render div { text "Content" } }');

      await new Promise(resolve => setTimeout(resolve, 20));

      // New editor joins
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');
      transportB.connect();

      const sessionB = new DocumentSession('/test.pc', transportB);

      // Sync the initial state
      sessionB.applyUpdate(sessionA.getDocument().encodeState());

      // New editor should have the existing content
      expect(sessionB.getText()).toBe(sessionA.getText());

      sessionA.dispose();
      sessionB.dispose();
    });

    it('continues working when an editor disconnects', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');

      transportA.connect();
      transportB.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);

      sessionA.setText('component Test { render div {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      // Editor A disconnects
      sessionA.dispose();
      server.disconnect('/test.pc', 'editorA');

      // Editor B can still work
      sessionB.insert(sessionB.getText().indexOf('{}'), ' text "Still working" ');
      await new Promise(resolve => setTimeout(resolve, 20));

      expect(sessionB.getText()).toContain('Still working');

      sessionB.dispose();
    });
  });

  describe('realistic editing patterns', () => {
    it('handles character-by-character typing from two editors', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');

      transportA.connect();
      transportB.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);

      // Base content
      sessionA.setText('component Test { render div { text "" } }');
      await new Promise(resolve => setTimeout(resolve, 20));

      // Find the position between quotes
      const quotePos = sessionA.getText().indexOf('""') + 1;

      // Both editors type into the text content
      const textA = 'Hello';
      const textB = 'World';

      for (let i = 0; i < Math.max(textA.length, textB.length); i++) {
        if (i < textA.length) {
          const pos = sessionA.getText().indexOf('""') + 1 + i;
          sessionA.insert(pos, textA[i]);
        }
        if (i < textB.length) {
          const pos = sessionB.getText().lastIndexOf('"');
          sessionB.insert(pos, textB[i]);
        }
        await new Promise(resolve => setTimeout(resolve, 5));
      }

      await new Promise(resolve => setTimeout(resolve, 50));

      // Both should converge
      expect(sessionA.getText()).toBe(sessionB.getText());

      // Both strings should be present (interleaved)
      const finalText = sessionA.getText();
      for (const char of textA) {
        expect(finalText).toContain(char);
      }
      for (const char of textB) {
        expect(finalText).toContain(char);
      }

      sessionA.dispose();
      sessionB.dispose();
    });

    it('handles rapid undo/redo simulation (delete + reinsert)', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      transportA.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const vdomUpdates: any[] = [];

      sessionA.onVDOMChange((vdom) => vdomUpdates.push(vdom));

      // Initial content
      sessionA.setText('component Test { render div { text "Hello" } }');
      await new Promise(resolve => setTimeout(resolve, 20));

      const initialVdomCount = vdomUpdates.length;

      // Simulate rapid undo/redo (delete and reinsert)
      for (let i = 0; i < 10; i++) {
        const text = sessionA.getText();
        const helloPos = text.indexOf('Hello');
        if (helloPos >= 0) {
          sessionA.delete(helloPos, 5);
          sessionA.insert(helloPos, 'Hello');
        }
      }

      await new Promise(resolve => setTimeout(resolve, 50));

      // Content should be stable
      expect(sessionA.getText()).toContain('Hello');

      // Should still receive VDOM updates (content is valid)
      expect(vdomUpdates.length).toBeGreaterThan(initialVdomCount);

      sessionA.dispose();
    });

    it('handles copy-paste of large blocks', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');

      transportA.connect();
      transportB.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);

      // Large component
      const largeComponent = `
component LargeForm {
  render form {
    div {
      text "Field 1"
    }
    div {
      text "Field 2"
    }
    div {
      text "Field 3"
    }
    div {
      text "Field 4"
    }
    div {
      text "Field 5"
    }
  }
}`;

      // Editor A pastes large content
      sessionA.setText(largeComponent);
      await new Promise(resolve => setTimeout(resolve, 30));

      // Editor B should have it
      expect(sessionB.getText()).toBe(sessionA.getText());

      // Editor B adds another component
      sessionB.insert(sessionB.getText().length, '\ncomponent Small { render span {} }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Both should have both components
      expect(sessionA.getText()).toBe(sessionB.getText());
      expect(sessionA.getText()).toContain('LargeForm');
      expect(sessionA.getText()).toContain('Small');

      sessionA.dispose();
      sessionB.dispose();
    });
  });

  describe('error recovery', () => {
    it('recovers from temporary invalid state', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      transportA.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const vdomUpdates: any[] = [];

      sessionA.onVDOMChange((vdom) => vdomUpdates.push(vdom));

      // Valid content
      sessionA.setText('component Test { render div {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      const validCount = vdomUpdates.filter(v => !v.error).length;

      // Break it temporarily (simulate mid-edit)
      sessionA.setText('component Test { render ');
      await new Promise(resolve => setTimeout(resolve, 20));

      // Should have error
      const errorCount = vdomUpdates.filter(v => v.error).length;
      expect(errorCount).toBeGreaterThan(0);

      // Fix it
      sessionA.setText('component Test { render span {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      // Should have new valid VDOM
      const finalValidCount = vdomUpdates.filter(v => !v.error).length;
      expect(finalValidCount).toBeGreaterThan(validCount);

      sessionA.dispose();
    });

    it('other editors still work during error state', async () => {
      const transportA = createMockServerTransport(server, '/test.pc', 'editorA');
      const transportB = createMockServerTransport(server, '/test.pc', 'editorB');

      transportA.connect();
      transportB.connect();

      const sessionA = new DocumentSession('/test.pc', transportA);
      const sessionB = new DocumentSession('/test.pc', transportB);

      // Valid content
      sessionA.setText('component Test { render div {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      // Editor A breaks it
      sessionA.setText('broken content');
      await new Promise(resolve => setTimeout(resolve, 20));

      // Editor B can still see the content (even if invalid)
      expect(sessionB.getText()).toBe('broken content');

      // Editor B fixes it
      sessionB.setText('component Fixed { render span {} }');
      await new Promise(resolve => setTimeout(resolve, 20));

      // Both should have the fix
      expect(sessionA.getText()).toBe(sessionB.getText());
      expect(sessionA.getText()).toContain('Fixed');

      sessionA.dispose();
      sessionB.dispose();
    });
  });
});
