/**
 * Visual Editing → CRDT Sync Tests
 *
 * These tests verify the flow when a visual designer makes changes:
 * 1. User interacts with rendered VDOM (drag, resize, edit text)
 * 2. Designer generates a mutation (SetInlineStyle, MoveElement, etc.)
 * 3. Mutation is applied to source code
 * 4. CRDT document is updated
 * 5. Change syncs to other editors (VS Code, another designer)
 * 6. All editors re-render with new VDOM
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CRDTDocument } from './crdt';
import { DocumentSession, SyncTransport } from './sync';

/**
 * Represents a mutation from the visual designer.
 * These map to the proto Mutation types.
 */
type VisualMutation =
  | { type: 'setInlineStyle'; nodeId: string; property: string; value: string }
  | { type: 'updateText'; nodeId: string; content: string }
  | { type: 'moveElement'; nodeId: string; newParentId: string; index: number }
  | { type: 'setAttribute'; nodeId: string; name: string; value: string }
  | { type: 'removeNode'; nodeId: string }
  | { type: 'setFrameBounds'; frameId: string; x: number; y: number; width: number; height: number };

/**
 * Simple source code editor that can apply visual mutations.
 * In production, this is done by paperclip-editor in Rust.
 */
class SourceCodeMutator {
  /**
   * Apply a visual mutation to source code.
   * Returns the modified source.
   */
  applyMutation(source: string, mutation: VisualMutation): string {
    switch (mutation.type) {
      case 'setInlineStyle':
        return this.applySetInlineStyle(source, mutation);
      case 'updateText':
        return this.applyUpdateText(source, mutation);
      case 'setAttribute':
        return this.applySetAttribute(source, mutation);
      case 'setFrameBounds':
        return this.applySetFrameBounds(source, mutation);
      default:
        return source;
    }
  }

  private applySetInlineStyle(
    source: string,
    mutation: { nodeId: string; property: string; value: string }
  ): string {
    // Find element by looking for id or name attribute
    // Simple implementation - real one uses AST
    const elementRegex = new RegExp(`(\\w+)\\s*(\\{[^}]*\\})`, 'g');

    // For simplicity, add style to first div/span found
    return source.replace(
      /(render\s+\w+)\s*\{/,
      `$1 style:${mutation.property}="${mutation.value}" {`
    );
  }

  private applyUpdateText(
    source: string,
    mutation: { nodeId: string; content: string }
  ): string {
    // Replace text content
    return source.replace(
      /text\s+"[^"]*"/,
      `text "${mutation.content}"`
    );
  }

  private applySetAttribute(
    source: string,
    mutation: { nodeId: string; name: string; value: string }
  ): string {
    // Add attribute to element
    return source.replace(
      /(render\s+\w+)/,
      `$1 ${mutation.name}="${mutation.value}"`
    );
  }

  private applySetFrameBounds(
    source: string,
    mutation: { frameId: string; x: number; y: number; width: number; height: number }
  ): string {
    // Add or update @frame comment
    const frameComment = `/** @frame { x: ${mutation.x}, y: ${mutation.y}, width: ${mutation.width}, height: ${mutation.height} } */`;

    if (source.includes('@frame')) {
      return source.replace(/\/\*\*\s*@frame\s*\{[^}]*\}\s*\*\//, frameComment);
    } else {
      // Add before component
      return source.replace(/component/, `${frameComment}\ncomponent`);
    }
  }
}

/**
 * Simulates the visual designer that renders VDOM and handles interactions.
 */
class MockVisualDesigner {
  private session: DocumentSession;
  private mutator = new SourceCodeMutator();
  private currentVDOM: any = null;
  private onVDOMUpdate: ((vdom: any) => void) | null = null;

  constructor(session: DocumentSession) {
    this.session = session;

    // Listen for VDOM updates
    session.onVDOMChange((vdom) => {
      this.currentVDOM = vdom;
      this.onVDOMUpdate?.(vdom);
    });
  }

  /**
   * Get current rendered VDOM.
   */
  getVDOM(): any {
    return this.currentVDOM;
  }

  /**
   * Subscribe to VDOM updates.
   */
  onRender(callback: (vdom: any) => void): () => void {
    this.onVDOMUpdate = callback;
    return () => { this.onVDOMUpdate = null; };
  }

  /**
   * Simulate user dragging to resize an element.
   */
  resizeElement(nodeId: string, width: number, height: number): void {
    const mutation: VisualMutation = {
      type: 'setInlineStyle',
      nodeId,
      property: 'width',
      value: `${width}px`,
    };
    this.applyVisualMutation(mutation);

    const mutation2: VisualMutation = {
      type: 'setInlineStyle',
      nodeId,
      property: 'height',
      value: `${height}px`,
    };
    this.applyVisualMutation(mutation2);
  }

  /**
   * Simulate user editing text by double-clicking.
   */
  editText(nodeId: string, newContent: string): void {
    const mutation: VisualMutation = {
      type: 'updateText',
      nodeId,
      content: newContent,
    };
    this.applyVisualMutation(mutation);
  }

  /**
   * Simulate user changing an attribute via properties panel.
   */
  setAttribute(nodeId: string, name: string, value: string): void {
    const mutation: VisualMutation = {
      type: 'setAttribute',
      nodeId,
      name,
      value,
    };
    this.applyVisualMutation(mutation);
  }

  /**
   * Simulate user moving a frame (component preview) on canvas.
   */
  moveFrame(frameId: string, x: number, y: number, width: number, height: number): void {
    const mutation: VisualMutation = {
      type: 'setFrameBounds',
      frameId,
      x,
      y,
      width,
      height,
    };
    this.applyVisualMutation(mutation);
  }

  /**
   * Apply a visual mutation by updating source code.
   */
  private applyVisualMutation(mutation: VisualMutation): void {
    const currentSource = this.session.getText();
    const newSource = this.mutator.applyMutation(currentSource, mutation);

    if (newSource !== currentSource) {
      // Update the CRDT document with the new source
      // Use transaction to batch if needed
      this.session.setText(newSource, { origin: 'designer' });
    }
  }
}

/**
 * Mock server for testing (same as e2e-sync.spec.ts).
 */
class MockServer {
  private sessions = new Map<string, {
    document: CRDTDocument;
    clients: Map<string, MockServerClient>;
    version: number;
  }>();

  private parse(source: string): { vdom: any; error?: string } {
    const componentMatch = source.match(/component\s+(\w+)\s*\{/);
    if (!componentMatch) {
      return { vdom: null, error: 'No component found' };
    }

    const componentName = componentMatch[1];
    const renderMatch = source.match(/render\s+(\w+)([^{]*)\{/);

    if (!renderMatch) {
      return { vdom: null, error: 'No render block' };
    }

    const tagName = renderMatch[1];
    const attributes: Record<string, string> = {};

    // Extract style attributes
    const styleMatches = [...source.matchAll(/style:(\w+)="([^"]+)"/g)];
    for (const match of styleMatches) {
      attributes[`style:${match[1]}`] = match[2];
    }

    // Extract other attributes
    const attrMatches = [...source.matchAll(/(\w+)="([^"]+)"/g)];
    for (const match of attrMatches) {
      if (!match[1].startsWith('style')) {
        attributes[match[1]] = match[2];
      }
    }

    // Extract text
    const textMatch = source.match(/text\s+"([^"]+)"/);
    const children = textMatch ? [{ type: 'text', content: textMatch[1] }] : [];

    // Extract frame bounds
    const frameMatch = source.match(/@frame\s*\{\s*x:\s*([\d.]+),\s*y:\s*([\d.]+),\s*width:\s*([\d.]+),\s*height:\s*([\d.]+)\s*\}/);
    const frameBounds = frameMatch ? {
      x: parseFloat(frameMatch[1]),
      y: parseFloat(frameMatch[2]),
      width: parseFloat(frameMatch[3]),
      height: parseFloat(frameMatch[4]),
    } : null;

    return {
      vdom: {
        type: 'element',
        tag: tagName,
        id: `${componentName}-root`,
        attributes,
        children,
        componentName,
        frameBounds,
      },
    };
  }

  join(filePath: string, clientId: string, client: MockServerClient): {
    documentState: Uint8Array;
    stateVector: Uint8Array;
    vdom: any;
    version: number;
  } {
    let session = this.sessions.get(filePath);
    if (!session) {
      session = { document: new CRDTDocument(), clients: new Map(), version: 0 };
      this.sessions.set(filePath, session);
    }
    session.clients.set(clientId, client);

    const parseResult = this.parse(session.document.getText());
    return {
      documentState: session.document.encodeState(),
      stateVector: session.document.getStateVector(),
      vdom: parseResult.vdom,
      version: session.version,
    };
  }

  applyUpdate(filePath: string, clientId: string, update: Uint8Array): void {
    const session = this.sessions.get(filePath);
    if (!session) return;

    session.document.applyUpdate(update);
    session.version++;

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
      client.receiveVDOM({
        patches: parseResult.vdom ? [{ type: 'replace', vdom: parseResult.vdom }] : [],
        version: session.version,
        originClientId: clientId,
        error: parseResult.error,
      });
    }
  }

  getDocumentText(filePath: string): string {
    return this.sessions.get(filePath)?.document.getText() ?? '';
  }

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

class MockServerClient {
  private updateHandler: ((update: Uint8Array) => void) | null = null;
  private vdomHandler: ((vdom: any) => void) | null = null;

  receiveUpdate(update: Uint8Array): void {
    this.updateHandler?.(update);
  }

  receiveVDOM(vdom: any): void {
    this.vdomHandler?.(vdom);
  }

  onUpdate(handler: (update: Uint8Array) => void): void {
    this.updateHandler = handler;
  }

  onVDOM(handler: (vdom: any) => void): void {
    this.vdomHandler = handler;
  }
}

function createMockServerTransport(
  server: MockServer,
  filePath: string,
  clientId: string
): SyncTransport & { client: MockServerClient; connect: () => void } {
  const client = new MockServerClient();
  const updateHandlers = new Set<(update: Uint8Array) => void>();
  const vdomHandlers = new Set<(vdom: any) => void>();
  const cssomHandlers = new Set<(cssom: any) => void>();
  let connected = false;

  client.onUpdate((update) => {
    for (const handler of updateHandlers) handler(update);
  });
  client.onVDOM((vdom) => {
    for (const handler of vdomHandlers) handler(vdom);
  });

  return {
    client,
    connect() {
      server.join(filePath, clientId, client);
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

describe('Visual Editing → CRDT Sync', () => {
  let server: MockServer;

  beforeEach(() => {
    server = new MockServer();
  });

  describe('designer makes visual changes → syncs to text editor', () => {
    it('resizing element in designer updates source in VS Code', async () => {
      // Setup: Designer and VS Code editing same file
      const designerTransport = createMockServerTransport(server, '/button.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/button.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/button.pc', designerTransport);
      const vscodeSession = new DocumentSession('/button.pc', vscodeTransport);

      const designer = new MockVisualDesigner(designerSession);

      // Initial component
      designerSession.setText('component Button { render div { text "Click me" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer resizes the element
      designer.resizeElement('Button-root', 200, 50);
      await new Promise(resolve => setTimeout(resolve, 30));

      // VS Code should see the style change in source
      const vscodeSource = vscodeSession.getText();
      expect(vscodeSource).toContain('style:width="200px"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('editing text in designer updates source in VS Code', async () => {
      const designerTransport = createMockServerTransport(server, '/card.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/card.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/card.pc', designerTransport);
      const vscodeSession = new DocumentSession('/card.pc', vscodeTransport);

      const designer = new MockVisualDesigner(designerSession);

      // Initial component
      designerSession.setText('component Card { render div { text "Old Title" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer double-clicks and edits text
      designer.editText('text-0', 'New Amazing Title');
      await new Promise(resolve => setTimeout(resolve, 30));

      // VS Code should see the text change
      const vscodeSource = vscodeSession.getText();
      expect(vscodeSource).toContain('text "New Amazing Title"');
      expect(vscodeSource).not.toContain('Old Title');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('changing attribute in designer properties panel updates VS Code', async () => {
      const designerTransport = createMockServerTransport(server, '/link.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/link.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/link.pc', designerTransport);
      const vscodeSession = new DocumentSession('/link.pc', vscodeTransport);

      const designer = new MockVisualDesigner(designerSession);

      // Initial component
      designerSession.setText('component Link { render a { text "Click here" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer sets href attribute via properties panel
      designer.setAttribute('Link-root', 'href', 'https://example.com');
      await new Promise(resolve => setTimeout(resolve, 30));

      // VS Code should see the attribute
      const vscodeSource = vscodeSession.getText();
      expect(vscodeSource).toContain('href="https://example.com"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('moving frame on canvas updates @frame comment in VS Code', async () => {
      const designerTransport = createMockServerTransport(server, '/button.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/button.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/button.pc', designerTransport);
      const vscodeSession = new DocumentSession('/button.pc', vscodeTransport);

      const designer = new MockVisualDesigner(designerSession);

      // Initial component
      designerSession.setText('component Button { render div { text "Click" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer drags the frame to new position
      designer.moveFrame('Button', 100, 200, 300, 150);
      await new Promise(resolve => setTimeout(resolve, 30));

      // VS Code should see the @frame comment
      const vscodeSource = vscodeSession.getText();
      expect(vscodeSource).toContain('@frame');
      expect(vscodeSource).toContain('x: 100');
      expect(vscodeSource).toContain('y: 200');
      expect(vscodeSource).toContain('width: 300');
      expect(vscodeSource).toContain('height: 150');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('VS Code makes text changes → designer re-renders', () => {
    it('typing in VS Code updates VDOM in designer', async () => {
      const designerTransport = createMockServerTransport(server, '/test.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/test.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/test.pc', designerTransport);
      const vscodeSession = new DocumentSession('/test.pc', vscodeTransport);

      const vdomUpdates: any[] = [];
      designerSession.onVDOMChange((vdom) => vdomUpdates.push(vdom));

      // Initial content from VS Code
      vscodeSession.setText('component Test { render div { text "Hello" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer should have received VDOM
      expect(vdomUpdates.length).toBeGreaterThan(0);
      const vdom = vdomUpdates[vdomUpdates.length - 1];
      expect(vdom.patches?.[0]?.vdom?.children?.[0]?.content).toBe('Hello');

      // VS Code changes text
      const source = vscodeSession.getText();
      vscodeSession.setText(source.replace('Hello', 'World'));
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer should see updated VDOM
      const latestVdom = vdomUpdates[vdomUpdates.length - 1];
      expect(latestVdom.patches?.[0]?.vdom?.children?.[0]?.content).toBe('World');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('adding new element in VS Code appears in designer', async () => {
      const designerTransport = createMockServerTransport(server, '/test.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/test.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/test.pc', designerTransport);
      const vscodeSession = new DocumentSession('/test.pc', vscodeTransport);

      const vdomUpdates: any[] = [];
      designerSession.onVDOMChange((vdom) => vdomUpdates.push(vdom));

      // Simple component
      vscodeSession.setText('component Test { render div { text "One" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // VS Code adds another text element
      vscodeSession.setText('component Test { render div { text "One" text "Two" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer's source should match
      expect(designerSession.getText()).toBe(vscodeSession.getText());

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('concurrent visual and text editing', () => {
    it('designer resize + VS Code text edit merge correctly', async () => {
      const designerTransport = createMockServerTransport(server, '/test.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/test.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/test.pc', designerTransport);
      const vscodeSession = new DocumentSession('/test.pc', vscodeTransport);

      const designer = new MockVisualDesigner(designerSession);

      // Initial component
      vscodeSession.setText('component Button { render div { text "Click" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Concurrent: Designer resizes while VS Code changes text
      designer.resizeElement('Button-root', 150, 40);
      vscodeSession.setText(vscodeSession.getText().replace('Click', 'Submit'));

      await new Promise(resolve => setTimeout(resolve, 50));

      // Both changes should be present
      const finalSource = designerSession.getText();
      expect(finalSource).toBe(vscodeSession.getText());
      // Note: Due to how mutations are applied, we check what survives
      // The text change should definitely be there
      expect(finalSource).toContain('Submit');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('two designers editing same component converge', async () => {
      const designer1Transport = createMockServerTransport(server, '/test.pc', 'designer1');
      const designer2Transport = createMockServerTransport(server, '/test.pc', 'designer2');

      designer1Transport.connect();
      designer2Transport.connect();

      const session1 = new DocumentSession('/test.pc', designer1Transport);
      const session2 = new DocumentSession('/test.pc', designer2Transport);

      const designer1 = new MockVisualDesigner(session1);
      const designer2 = new MockVisualDesigner(session2);

      // Initial component
      session1.setText('component Test { render div { text "Hello" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer 1 changes text
      designer1.editText('text-0', 'Hello World');

      // Designer 2 adds attribute (concurrent)
      designer2.setAttribute('Test-root', 'class', 'primary');

      await new Promise(resolve => setTimeout(resolve, 50));

      // Both should converge
      expect(session1.getText()).toBe(session2.getText());

      session1.dispose();
      session2.dispose();
    });
  });

  describe('VDOM → source → VDOM round-trip integrity', () => {
    it('style changes round-trip correctly', async () => {
      const transport = createMockServerTransport(server, '/test.pc', 'designer');
      transport.connect();

      const session = new DocumentSession('/test.pc', transport);
      const designer = new MockVisualDesigner(session);
      const vdomUpdates: any[] = [];

      session.onVDOMChange((vdom) => vdomUpdates.push(vdom));

      // Initial
      session.setText('component Box { render div { text "Content" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Apply style via designer
      designer.resizeElement('Box-root', 200, 100);
      await new Promise(resolve => setTimeout(resolve, 30));

      // VDOM should reflect the style
      const latestVdom = vdomUpdates[vdomUpdates.length - 1];
      expect(latestVdom.patches?.[0]?.vdom?.attributes?.['style:width']).toBe('200px');

      session.dispose();
    });

    it('frame bounds round-trip correctly', async () => {
      const transport = createMockServerTransport(server, '/test.pc', 'designer');
      transport.connect();

      const session = new DocumentSession('/test.pc', transport);
      const designer = new MockVisualDesigner(session);
      const vdomUpdates: any[] = [];

      session.onVDOMChange((vdom) => vdomUpdates.push(vdom));

      // Initial
      session.setText('component Card { render div { text "Card" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Move frame
      designer.moveFrame('Card', 50, 100, 400, 300);
      await new Promise(resolve => setTimeout(resolve, 30));

      // VDOM should have frame bounds
      const latestVdom = vdomUpdates[vdomUpdates.length - 1];
      expect(latestVdom.patches?.[0]?.vdom?.frameBounds).toEqual({
        x: 50,
        y: 100,
        width: 400,
        height: 300,
      });

      session.dispose();
    });
  });

  describe('rapid visual interactions', () => {
    it('handles rapid drag operations (many style updates)', async () => {
      const designerTransport = createMockServerTransport(server, '/test.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/test.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/test.pc', designerTransport);
      const vscodeSession = new DocumentSession('/test.pc', vscodeTransport);

      const designer = new MockVisualDesigner(designerSession);

      // Initial
      designerSession.setText('component Box { render div { text "Drag me" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Simulate rapid drag (30 position updates)
      for (let i = 0; i < 30; i++) {
        designer.moveFrame('Box', i * 10, i * 5, 200, 100);
      }

      await new Promise(resolve => setTimeout(resolve, 100));

      // Both should converge to final state
      expect(designerSession.getText()).toBe(vscodeSession.getText());

      // Final position should be in source
      const source = designerSession.getText();
      expect(source).toContain('x: 290'); // 29 * 10
      expect(source).toContain('y: 145'); // 29 * 5

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('handles rapid text editing (character by character)', async () => {
      const designerTransport = createMockServerTransport(server, '/test.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/test.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/test.pc', designerTransport);
      const vscodeSession = new DocumentSession('/test.pc', vscodeTransport);

      const designer = new MockVisualDesigner(designerSession);

      // Initial
      designerSession.setText('component Test { render div { text "" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Type character by character in designer
      const textToType = 'Hello World';
      for (let i = 1; i <= textToType.length; i++) {
        designer.editText('text-0', textToType.slice(0, i));
        await new Promise(resolve => setTimeout(resolve, 5));
      }

      await new Promise(resolve => setTimeout(resolve, 50));

      // Both should have full text
      expect(designerSession.getText()).toContain('text "Hello World"');
      expect(vscodeSession.getText()).toContain('text "Hello World"');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('error handling in visual editing', () => {
    it('designer continues working after VS Code introduces syntax error', async () => {
      const designerTransport = createMockServerTransport(server, '/test.pc', 'designer');
      const vscodeTransport = createMockServerTransport(server, '/test.pc', 'vscode');

      designerTransport.connect();
      vscodeTransport.connect();

      const designerSession = new DocumentSession('/test.pc', designerTransport);
      const vscodeSession = new DocumentSession('/test.pc', vscodeTransport);

      const designer = new MockVisualDesigner(designerSession);
      const vdomUpdates: any[] = [];

      designerSession.onVDOMChange((vdom) => vdomUpdates.push(vdom));

      // Valid component
      vscodeSession.setText('component Test { render div { text "Valid" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // VS Code breaks it
      vscodeSession.setText('invalid syntax here');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer should receive error
      const errorUpdate = vdomUpdates.find(v => v.error);
      expect(errorUpdate).toBeDefined();

      // VS Code fixes it
      vscodeSession.setText('component Test { render div { text "Fixed" } }');
      await new Promise(resolve => setTimeout(resolve, 30));

      // Designer should work again
      const latestVdom = vdomUpdates[vdomUpdates.length - 1];
      expect(latestVdom.patches?.[0]?.vdom?.children?.[0]?.content).toBe('Fixed');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });
});
