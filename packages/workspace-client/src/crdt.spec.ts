import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CRDTDocument } from './crdt';

describe('CRDTDocument', () => {
  let doc: CRDTDocument;

  beforeEach(() => {
    doc = new CRDTDocument();
  });

  describe('basic text operations', () => {
    it('starts with empty text', () => {
      expect(doc.getText()).toBe('');
    });

    it('inserts text at position', () => {
      doc.insert(0, 'hello');
      expect(doc.getText()).toBe('hello');
    });

    it('inserts text at middle position', () => {
      doc.insert(0, 'hllo');
      doc.insert(1, 'e');
      expect(doc.getText()).toBe('hello');
    });

    it('appends text at end', () => {
      doc.insert(0, 'hello');
      doc.insert(5, ' world');
      expect(doc.getText()).toBe('hello world');
    });

    it('deletes text', () => {
      doc.insert(0, 'hello world');
      doc.delete(5, 6); // delete " world"
      expect(doc.getText()).toBe('hello');
    });

    it('deletes text from middle', () => {
      doc.insert(0, 'hello world');
      doc.delete(2, 3); // delete "llo"
      expect(doc.getText()).toBe('he world');
    });

    it('replaces text via delete then insert', () => {
      doc.insert(0, 'hello world');
      doc.delete(6, 5); // delete "world"
      doc.insert(6, 'there');
      expect(doc.getText()).toBe('hello there');
    });
  });

  describe('setText convenience method', () => {
    it('sets entire document content', () => {
      doc.setText('hello world');
      expect(doc.getText()).toBe('hello world');
    });

    it('replaces existing content', () => {
      doc.insert(0, 'old content');
      doc.setText('new content');
      expect(doc.getText()).toBe('new content');
    });

    it('handles empty string', () => {
      doc.insert(0, 'content');
      doc.setText('');
      expect(doc.getText()).toBe('');
    });
  });

  describe('origin tracking', () => {
    it('emits changes with local origin by default', () => {
      const changes: Array<{ origin: string | null }> = [];
      doc.onChange((delta, origin) => {
        changes.push({ origin });
      });

      doc.insert(0, 'hello');

      expect(changes.length).toBe(1);
      expect(changes[0].origin).toBe('local');
    });

    it('emits changes with custom origin', () => {
      const changes: Array<{ origin: string | null }> = [];
      doc.onChange((delta, origin) => {
        changes.push({ origin });
      });

      doc.insert(0, 'hello', { origin: 'vscode' });

      expect(changes.length).toBe(1);
      expect(changes[0].origin).toBe('vscode');
    });

    it('allows filtering changes by origin', () => {
      const localChanges: string[] = [];
      const remoteChanges: string[] = [];

      doc.onChange((delta, origin) => {
        if (origin === 'local') {
          localChanges.push(doc.getText());
        } else if (origin === 'remote') {
          remoteChanges.push(doc.getText());
        }
      });

      doc.insert(0, 'local', { origin: 'local' });
      doc.insert(5, '-remote', { origin: 'remote' });

      expect(localChanges).toEqual(['local']);
      expect(remoteChanges).toEqual(['local-remote']);
    });
  });

  describe('state vector sync', () => {
    it('returns state vector', () => {
      doc.insert(0, 'hello');
      const stateVector = doc.getStateVector();
      expect(stateVector).toBeInstanceOf(Uint8Array);
      expect(stateVector.length).toBeGreaterThan(0);
    });

    it('encodes state as update', () => {
      doc.insert(0, 'hello');
      const update = doc.encodeState();
      expect(update).toBeInstanceOf(Uint8Array);
      expect(update.length).toBeGreaterThan(0);
    });

    it('encodes delta from state vector', () => {
      doc.insert(0, 'hello');
      const sv1 = doc.getStateVector();

      doc.insert(5, ' world');
      const delta = doc.encodeDelta(sv1);

      expect(delta).toBeInstanceOf(Uint8Array);
      expect(delta.length).toBeGreaterThan(0);
    });

    it('applies update from another document', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      doc1.insert(0, 'hello');
      const update = doc1.encodeState();

      doc2.applyUpdate(update);
      expect(doc2.getText()).toBe('hello');
    });

    it('merges concurrent edits', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      // Initial sync
      doc1.insert(0, 'hello');
      doc2.applyUpdate(doc1.encodeState());

      // Concurrent edits
      doc1.insert(5, '!');
      doc2.insert(0, '> ');

      // Sync both ways
      doc1.applyUpdate(doc2.encodeState());
      doc2.applyUpdate(doc1.encodeState());

      // Both should converge to same state
      expect(doc1.getText()).toBe(doc2.getText());
      // The exact order depends on client IDs, but both chars should be present
      expect(doc1.getText()).toContain('>');
      expect(doc1.getText()).toContain('!');
      expect(doc1.getText()).toContain('hello');
    });
  });

  describe('delta encoding for efficient sync', () => {
    it('produces smaller delta than full state for incremental changes', () => {
      doc.insert(0, 'a'.repeat(1000)); // Large initial content
      const sv = doc.getStateVector();

      doc.insert(1000, 'b'); // Small change

      const fullState = doc.encodeState();
      const delta = doc.encodeDelta(sv);

      // Delta should be much smaller than full state
      expect(delta.length).toBeLessThan(fullState.length / 2);
    });
  });

  describe('change events', () => {
    it('emits onChange for inserts', () => {
      const handler = vi.fn();
      doc.onChange(handler);

      doc.insert(0, 'hello');

      expect(handler).toHaveBeenCalledTimes(1);
    });

    it('emits onChange for deletes', () => {
      doc.insert(0, 'hello');

      const handler = vi.fn();
      doc.onChange(handler);

      doc.delete(0, 2);

      expect(handler).toHaveBeenCalledTimes(1);
    });

    it('emits onChange when applying updates', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      const handler = vi.fn();
      doc2.onChange(handler);

      doc1.insert(0, 'hello');
      doc2.applyUpdate(doc1.encodeState());

      expect(handler).toHaveBeenCalled();
    });

    it('provides delta information in change event', () => {
      let capturedDelta: any;
      doc.onChange((delta) => {
        capturedDelta = delta;
      });

      doc.insert(0, 'hello');

      expect(capturedDelta).toBeDefined();
      expect(Array.isArray(capturedDelta)).toBe(true);
      expect(capturedDelta[0].insert).toBe('hello');
    });

    it('can unsubscribe from changes', () => {
      const handler = vi.fn();
      const unsubscribe = doc.onChange(handler);

      doc.insert(0, 'hello');
      expect(handler).toHaveBeenCalledTimes(1);

      unsubscribe();
      doc.insert(5, ' world');
      expect(handler).toHaveBeenCalledTimes(1); // Still 1, not called again
    });
  });

  describe('transaction batching', () => {
    it('batches multiple operations into single change event', () => {
      const handler = vi.fn();
      doc.onChange(handler);

      doc.transaction(() => {
        doc.insert(0, 'hello');
        doc.insert(5, ' ');
        doc.insert(6, 'world');
      });

      // Should emit only once for the entire transaction
      expect(handler).toHaveBeenCalledTimes(1);
      expect(doc.getText()).toBe('hello world');
    });

    it('transaction origin applies to all operations', () => {
      const origins: Array<string | null> = [];
      doc.onChange((_, origin) => {
        origins.push(origin);
      });

      doc.transaction(() => {
        doc.insert(0, 'a');
        doc.insert(1, 'b');
      }, { origin: 'batch' });

      expect(origins).toEqual(['batch']);
    });
  });

  describe('dirty flag for AST validity', () => {
    it('starts clean', () => {
      expect(doc.isDirty()).toBe(false);
    });

    it('marks dirty on local edit', () => {
      doc.insert(0, 'hello');
      expect(doc.isDirty()).toBe(true);
    });

    it('marks dirty on remote update', () => {
      const doc1 = new CRDTDocument();
      doc1.insert(0, 'hello');

      doc.applyUpdate(doc1.encodeState());
      expect(doc.isDirty()).toBe(true);
    });

    it('clears dirty flag with markClean', () => {
      doc.insert(0, 'hello');
      expect(doc.isDirty()).toBe(true);

      doc.markClean();
      expect(doc.isDirty()).toBe(false);
    });

    it('becomes dirty again after clean', () => {
      doc.insert(0, 'hello');
      doc.markClean();
      doc.insert(5, '!');
      expect(doc.isDirty()).toBe(true);
    });
  });

  describe('dispose', () => {
    it('removes all listeners on dispose', () => {
      const handler = vi.fn();
      doc.onChange(handler);

      doc.dispose();
      doc.insert(0, 'hello');

      expect(handler).not.toHaveBeenCalled();
    });
  });
});
