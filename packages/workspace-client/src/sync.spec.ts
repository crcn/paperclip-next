import { describe, it, expect, vi, beforeEach } from 'vitest';
import { DocumentSession, SyncTransport } from './sync';
import { CRDTDocument } from './crdt';

// Mock transport for testing
class MockTransport implements SyncTransport {
  public sentUpdates: Array<{ filePath: string; update: Uint8Array; stateVector: Uint8Array }> = [];
  public onUpdateCallback: ((update: Uint8Array) => void) | null = null;
  public onVDOMCallback: ((vdom: any) => void) | null = null;
  public onCSSOMCallback: ((cssom: any) => void) | null = null;
  public connected = false;

  async sendUpdate(filePath: string, update: Uint8Array, stateVector: Uint8Array): Promise<void> {
    this.sentUpdates.push({ filePath, update, stateVector });
  }

  onUpdate(callback: (update: Uint8Array) => void): () => void {
    this.onUpdateCallback = callback;
    return () => { this.onUpdateCallback = null; };
  }

  onVDOM(callback: (vdom: any) => void): () => void {
    this.onVDOMCallback = callback;
    return () => { this.onVDOMCallback = null; };
  }

  onCSSOM(callback: (cssom: any) => void): () => void {
    this.onCSSOMCallback = callback;
    return () => { this.onCSSOMCallback = null; };
  }

  // Test helper: simulate receiving an update from server
  simulateRemoteUpdate(update: Uint8Array): void {
    this.onUpdateCallback?.(update);
  }

  // Test helper: simulate receiving VDOM from server
  simulateVDOM(vdom: any): void {
    this.onVDOMCallback?.(vdom);
  }

  // Test helper: simulate receiving CSSOM from server
  simulateCSSOM(cssom: any): void {
    this.onCSSOMCallback?.(cssom);
  }
}

describe('DocumentSession', () => {
  let transport: MockTransport;
  let session: DocumentSession;

  beforeEach(() => {
    transport = new MockTransport();
    session = new DocumentSession('/path/to/file.pc', transport);
  });

  describe('text editing', () => {
    it('exposes getText() for current content', () => {
      expect(session.getText()).toBe('');
    });

    it('allows insert operations', () => {
      session.insert(0, 'hello');
      expect(session.getText()).toBe('hello');
    });

    it('allows delete operations', () => {
      session.insert(0, 'hello world');
      session.delete(5, 6);
      expect(session.getText()).toBe('hello');
    });

    it('allows setText for full replacement', () => {
      session.setText('new content');
      expect(session.getText()).toBe('new content');
    });
  });

  describe('sending updates to server', () => {
    it('sends update after local edit', async () => {
      session.insert(0, 'hello');

      // Allow microtask queue to flush (debounced send)
      await new Promise(resolve => setTimeout(resolve, 10));

      expect(transport.sentUpdates.length).toBe(1);
      expect(transport.sentUpdates[0].filePath).toBe('/path/to/file.pc');
      expect(transport.sentUpdates[0].update).toBeInstanceOf(Uint8Array);
      expect(transport.sentUpdates[0].stateVector).toBeInstanceOf(Uint8Array);
    });

    it('debounces rapid edits', async () => {
      session.insert(0, 'h');
      session.insert(1, 'e');
      session.insert(2, 'l');
      session.insert(3, 'l');
      session.insert(4, 'o');

      await new Promise(resolve => setTimeout(resolve, 50));

      // Should batch into fewer sends
      expect(transport.sentUpdates.length).toBeLessThanOrEqual(2);
    });

    it('includes state vector for delta sync', async () => {
      session.insert(0, 'hello');
      await new Promise(resolve => setTimeout(resolve, 10));

      const sent = transport.sentUpdates[0];
      expect(sent.stateVector.length).toBeGreaterThan(0);
    });
  });

  describe('receiving updates from server', () => {
    it('applies remote updates to document', () => {
      // Create remote document with changes
      const remoteDoc = new CRDTDocument();
      remoteDoc.insert(0, 'from server');
      const update = remoteDoc.encodeState();

      // Simulate server sending update
      transport.simulateRemoteUpdate(update);

      expect(session.getText()).toBe('from server');
    });

    it('merges concurrent edits', async () => {
      // Local edit
      session.insert(0, 'local');

      // Remote edit (on same initial state)
      const remoteDoc = new CRDTDocument();
      remoteDoc.insert(0, 'remote');
      transport.simulateRemoteUpdate(remoteDoc.encodeState());

      // Both should be present
      const text = session.getText();
      expect(text).toContain('local');
      expect(text).toContain('remote');
    });

    it('does not trigger send for remote updates', async () => {
      const remoteDoc = new CRDTDocument();
      remoteDoc.insert(0, 'from server');
      transport.simulateRemoteUpdate(remoteDoc.encodeState());

      await new Promise(resolve => setTimeout(resolve, 50));

      // Should not send anything back for remote-originated updates
      expect(transport.sentUpdates.length).toBe(0);
    });
  });

  describe('VDOM rendering', () => {
    it('provides getVDOM() for current state', () => {
      expect(session.getVDOM()).toBeNull(); // Initially null
    });

    it('updates VDOM when server sends it', () => {
      const vdom = { type: 'div', children: [] };
      transport.simulateVDOM(vdom);

      expect(session.getVDOM()).toEqual(vdom);
    });

    it('emits onVDOMChange when VDOM updates', () => {
      const handler = vi.fn();
      session.onVDOMChange(handler);

      const vdom = { type: 'div', children: [] };
      transport.simulateVDOM(vdom);

      expect(handler).toHaveBeenCalledWith(vdom);
    });

    it('can unsubscribe from VDOM changes', () => {
      const handler = vi.fn();
      const unsub = session.onVDOMChange(handler);

      transport.simulateVDOM({ type: 'div' });
      expect(handler).toHaveBeenCalledTimes(1);

      unsub();
      transport.simulateVDOM({ type: 'span' });
      expect(handler).toHaveBeenCalledTimes(1);
    });
  });

  describe('CSSOM rendering', () => {
    it('provides getCSSOM() for current state', () => {
      expect(session.getCSSOM()).toBeNull(); // Initially null
    });

    it('updates CSSOM when server sends it', () => {
      const cssom = { rules: ['.foo { color: red }'] };
      transport.simulateCSSOM(cssom);

      expect(session.getCSSOM()).toEqual(cssom);
    });

    it('emits onCSSOMChange when CSSOM updates', () => {
      const handler = vi.fn();
      session.onCSSOMChange(handler);

      const cssom = { rules: [] };
      transport.simulateCSSOM(cssom);

      expect(handler).toHaveBeenCalledWith(cssom);
    });
  });

  describe('origin tracking for VS Code integration', () => {
    it('marks local edits with vscode origin', async () => {
      session.insert(0, 'hello', { origin: 'vscode' });
      await new Promise(resolve => setTimeout(resolve, 10));

      // The update should still be sent (origin is for local filtering)
      expect(transport.sentUpdates.length).toBe(1);
    });

    it('allows filtering onChange by origin', () => {
      const localHandler = vi.fn();
      const remoteHandler = vi.fn();

      session.onTextChange((delta, origin) => {
        if (origin === 'local' || origin === 'vscode') {
          localHandler(delta);
        } else {
          remoteHandler(delta);
        }
      });

      // Local edit
      session.insert(0, 'local');

      // Remote edit
      const remoteDoc = new CRDTDocument();
      remoteDoc.insert(0, 'remote');
      transport.simulateRemoteUpdate(remoteDoc.encodeState());

      expect(localHandler).toHaveBeenCalled();
      expect(remoteHandler).toHaveBeenCalled();
    });
  });

  describe('dirty flag', () => {
    it('tracks when AST needs reparsing', () => {
      expect(session.isDirty()).toBe(false);

      session.insert(0, 'hello');
      expect(session.isDirty()).toBe(true);

      session.markClean();
      expect(session.isDirty()).toBe(false);
    });
  });

  describe('dispose', () => {
    it('cleans up resources', () => {
      const handler = vi.fn();
      session.onVDOMChange(handler);
      session.onTextChange(handler);

      session.dispose();

      // Should not call handlers after dispose
      transport.simulateVDOM({ type: 'div' });
      session.insert(0, 'test');

      expect(handler).not.toHaveBeenCalled();
    });
  });
});

describe('multi-client synchronization', () => {
  it('converges to same state with concurrent edits', async () => {
    const transport1 = new MockTransport();
    const transport2 = new MockTransport();

    const session1 = new DocumentSession('/file.pc', transport1);
    const session2 = new DocumentSession('/file.pc', transport2);

    // Both start with same content
    session1.setText('hello');
    session2.applyUpdate(session1.getDocument().encodeState());

    // Concurrent edits
    session1.insert(5, '!'); // "hello!"
    session2.insert(0, '> '); // "> hello"

    // Sync updates between sessions
    session2.applyUpdate(session1.getDocument().encodeState());
    session1.applyUpdate(session2.getDocument().encodeState());

    // Both should have same content
    expect(session1.getText()).toBe(session2.getText());
    expect(session1.getText()).toContain('hello');
    expect(session1.getText()).toContain('!');
    expect(session1.getText()).toContain('>');
  });

  it('handles three-way merge correctly', async () => {
    const t1 = new MockTransport();
    const t2 = new MockTransport();
    const t3 = new MockTransport();

    const s1 = new DocumentSession('/file.pc', t1);
    const s2 = new DocumentSession('/file.pc', t2);
    const s3 = new DocumentSession('/file.pc', t3);

    // All start with same base
    s1.setText('base');
    const baseState = s1.getDocument().encodeState();
    s2.applyUpdate(baseState);
    s3.applyUpdate(baseState);

    // Three concurrent edits at different positions
    s1.insert(0, 'A'); // "Abase"
    s2.insert(4, 'B'); // "baseB"
    s3.insert(2, 'C'); // "baCse"

    // Merge all states
    const state1 = s1.getDocument().encodeState();
    const state2 = s2.getDocument().encodeState();
    const state3 = s3.getDocument().encodeState();

    s1.applyUpdate(state2);
    s1.applyUpdate(state3);
    s2.applyUpdate(state1);
    s2.applyUpdate(state3);
    s3.applyUpdate(state1);
    s3.applyUpdate(state2);

    // All three should converge
    expect(s1.getText()).toBe(s2.getText());
    expect(s2.getText()).toBe(s3.getText());

    // All edits should be present
    const final = s1.getText();
    expect(final).toContain('A');
    expect(final).toContain('B');
    expect(final).toContain('C');
    // Original 'base' is split by C insert at position 2: 'ba' + 'C' + 'se'
    expect(final).toContain('ba');
    expect(final).toContain('se');
  });
});
