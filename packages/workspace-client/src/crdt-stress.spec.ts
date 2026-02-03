/**
 * CRDT Stress Tests
 *
 * These tests are designed to break the CRDT implementation by:
 * - Simulating rapid concurrent edits
 * - Testing edge cases in conflict resolution
 * - Verifying convergence under adversarial conditions
 * - Testing memory and performance characteristics
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CRDTDocument } from './crdt';
import { DocumentSession } from './sync';
import { createMockCrdtTransport } from './transport/crdt-grpc';

describe('CRDT Stress Tests', () => {
  describe('rapid concurrent edits', () => {
    it('handles 100 rapid sequential inserts', () => {
      const doc = new CRDTDocument();

      for (let i = 0; i < 100; i++) {
        doc.insert(i, String(i % 10));
      }

      expect(doc.getText().length).toBe(100);
    });

    it('handles alternating insert/delete cycles', () => {
      const doc = new CRDTDocument();

      for (let i = 0; i < 50; i++) {
        doc.insert(0, 'abc');
        doc.delete(0, 2);
      }

      // Should have 50 'c' characters
      expect(doc.getText()).toBe('c'.repeat(50));
    });

    it('handles interleaved operations from multiple logical clients', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();
      const doc3 = new CRDTDocument();

      // Simulate concurrent typing
      for (let i = 0; i < 20; i++) {
        doc1.insert(doc1.getText().length, 'a');
        doc2.insert(doc2.getText().length, 'b');
        doc3.insert(doc3.getText().length, 'c');

        // Sync all pairs
        doc2.applyUpdate(doc1.encodeState());
        doc3.applyUpdate(doc1.encodeState());
        doc1.applyUpdate(doc2.encodeState());
        doc3.applyUpdate(doc2.encodeState());
        doc1.applyUpdate(doc3.encodeState());
        doc2.applyUpdate(doc3.encodeState());
      }

      // All should converge
      expect(doc1.getText()).toBe(doc2.getText());
      expect(doc2.getText()).toBe(doc3.getText());

      // Should contain all characters
      const text = doc1.getText();
      expect(text.split('a').length - 1).toBe(20);
      expect(text.split('b').length - 1).toBe(20);
      expect(text.split('c').length - 1).toBe(20);
    });

    it('handles burst of 1000 character insertions', () => {
      const doc = new CRDTDocument();

      // Burst insert
      const content = 'x'.repeat(1000);
      doc.setText(content);

      expect(doc.getText().length).toBe(1000);

      // Now do incremental edits
      doc.delete(500, 100);
      expect(doc.getText().length).toBe(900);

      doc.insert(500, 'y'.repeat(100));
      expect(doc.getText().length).toBe(1000);
    });
  });

  describe('convergence under adversarial conditions', () => {
    it('converges with delayed sync (simulating network lag)', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      // Initial content
      doc1.setText('hello world');
      doc2.applyUpdate(doc1.encodeState());

      // Make many edits before syncing
      const pendingUpdates1: Uint8Array[] = [];
      const pendingUpdates2: Uint8Array[] = [];

      for (let i = 0; i < 10; i++) {
        const sv1 = doc1.getStateVector();
        doc1.insert(0, `a${i}`);
        pendingUpdates1.push(doc1.encodeDelta(sv1));

        const sv2 = doc2.getStateVector();
        doc2.insert(doc2.getText().length, `b${i}`);
        pendingUpdates2.push(doc2.encodeDelta(sv2));
      }

      // Apply all pending updates
      for (const update of pendingUpdates1) {
        doc2.applyUpdate(update);
      }
      for (const update of pendingUpdates2) {
        doc1.applyUpdate(update);
      }

      // Should converge
      expect(doc1.getText()).toBe(doc2.getText());
    });

    it('converges with out-of-order updates', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      doc1.setText('base');
      doc2.applyUpdate(doc1.encodeState());

      // Generate updates
      const sv1 = doc1.getStateVector();
      doc1.insert(4, '1');
      const update1 = doc1.encodeDelta(sv1);

      const sv2 = doc1.getStateVector();
      doc1.insert(5, '2');
      const update2 = doc1.encodeDelta(sv2);

      const sv3 = doc1.getStateVector();
      doc1.insert(6, '3');
      const update3 = doc1.encodeDelta(sv3);

      // Apply in reverse order (should still work)
      doc2.applyUpdate(update3);
      doc2.applyUpdate(update2);
      doc2.applyUpdate(update1);

      // May or may not converge to same text (Yjs handles causality)
      // but applying full state should definitely converge
      doc2.applyUpdate(doc1.encodeState());
      expect(doc2.getText()).toBe(doc1.getText());
    });

    it('converges with duplicate updates', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      doc1.setText('hello');
      const update = doc1.encodeState();

      // Apply same update multiple times
      doc2.applyUpdate(update);
      doc2.applyUpdate(update);
      doc2.applyUpdate(update);

      expect(doc2.getText()).toBe('hello');
    });

    it('handles same-position concurrent inserts with convergence', () => {
      // This is the classic CRDT challenge - two users insert at same position
      // The exact ordering depends on client IDs (which are random), but
      // the key property is that both documents CONVERGE to the same state.

      for (let trial = 0; trial < 10; trial++) {
        const doc1 = new CRDTDocument();
        const doc2 = new CRDTDocument();

        doc1.setText('x');
        doc2.applyUpdate(doc1.encodeState());

        // Both insert at position 1 (after 'x')
        doc1.insert(1, 'A');
        doc2.insert(1, 'B');

        // Sync
        doc1.applyUpdate(doc2.encodeState());
        doc2.applyUpdate(doc1.encodeState());

        // Key CRDT property: both converge to same state
        expect(doc1.getText()).toBe(doc2.getText());

        // Both characters should be present
        const text = doc1.getText();
        expect(text).toContain('x');
        expect(text).toContain('A');
        expect(text).toContain('B');
        expect(text.length).toBe(3);
      }
    });

    it('handles character-by-character typing simulation', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      const text1 = 'Hello from user 1';
      const text2 = 'Hi from user 2';

      // Simulate typing character by character
      for (let i = 0; i < Math.max(text1.length, text2.length); i++) {
        if (i < text1.length) {
          doc1.insert(i, text1[i]);
        }
        if (i < text2.length) {
          doc2.insert(i, text2[i]);
        }

        // Sync after each character (like real-time collaboration)
        doc1.applyUpdate(doc2.encodeState());
        doc2.applyUpdate(doc1.encodeState());
      }

      // Both should have same content
      expect(doc1.getText()).toBe(doc2.getText());
      // Both strings should be present (interleaved)
      const finalText = doc1.getText();
      // Can't predict exact interleaving, but all chars should be there
      expect(finalText.length).toBe(text1.length + text2.length);
    });
  });

  describe('edge cases', () => {
    it('handles empty document operations', () => {
      const doc = new CRDTDocument();

      // Delete from empty doc (should be no-op, not crash)
      doc.delete(0, 10);
      expect(doc.getText()).toBe('');

      // Insert still works
      doc.insert(0, 'test');
      expect(doc.getText()).toBe('test');

      // Delete all and try again
      doc.delete(0, 4);
      expect(doc.getText()).toBe('');

      // Insert at position 0 after emptying
      doc.insert(0, 'hello');
      expect(doc.getText()).toBe('hello');
    });

    it('handles very long content', () => {
      const doc = new CRDTDocument();

      // 100KB of content
      const largeContent = 'x'.repeat(100_000);
      doc.setText(largeContent);

      expect(doc.getText().length).toBe(100_000);

      // Can still do operations
      doc.insert(50_000, 'MIDDLE');
      expect(doc.getText().length).toBe(100_006);
    });

    it('handles unicode content', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      // Emoji and multibyte characters
      doc1.setText('Hello 👋 World 🌍!');
      doc2.applyUpdate(doc1.encodeState());

      expect(doc2.getText()).toBe('Hello 👋 World 🌍!');

      // Insert more unicode
      doc1.insert(6, '你好');
      doc2.applyUpdate(doc1.encodeState());

      expect(doc2.getText()).toContain('你好');
    });

    it('handles newlines and special characters', () => {
      const doc = new CRDTDocument();

      const content = 'line1\nline2\r\nline3\ttab';
      doc.setText(content);

      expect(doc.getText()).toBe(content);
    });

    it('handles zero-length operations', () => {
      const doc = new CRDTDocument();
      doc.setText('hello');

      // Insert empty string
      doc.insert(2, '');
      expect(doc.getText()).toBe('hello');

      // Delete zero characters
      doc.delete(2, 0);
      expect(doc.getText()).toBe('hello');
    });
  });

  describe('memory and performance', () => {
    it('state vector remains reasonably sized after many operations', () => {
      const doc = new CRDTDocument();

      // Many small operations
      for (let i = 0; i < 1000; i++) {
        doc.insert(0, 'x');
      }

      const stateVector = doc.getStateVector();
      // State vector should be small (just client IDs and counters)
      expect(stateVector.length).toBeLessThan(1000);
    });

    it('delta updates are efficient', () => {
      const doc = new CRDTDocument();

      // Initial large content
      doc.setText('x'.repeat(10_000));
      const sv = doc.getStateVector();

      // Small change
      doc.insert(5000, 'Y');

      const delta = doc.encodeDelta(sv);
      const fullState = doc.encodeState();

      // Delta should be much smaller than full state
      expect(delta.length).toBeLessThan(fullState.length / 10);
    });

    it('handles transaction batching correctly', () => {
      const doc = new CRDTDocument();
      const changeCount = { value: 0 };

      doc.onChange(() => {
        changeCount.value++;
      });

      // Without transaction - multiple events
      doc.insert(0, 'a');
      doc.insert(1, 'b');
      doc.insert(2, 'c');
      expect(changeCount.value).toBe(3);

      // With transaction - single event
      changeCount.value = 0;
      doc.transaction(() => {
        doc.insert(3, 'd');
        doc.insert(4, 'e');
        doc.insert(5, 'f');
      });
      expect(changeCount.value).toBe(1);
    });
  });

  describe('DocumentSession integration', () => {
    it('session handles rapid edits with debouncing', async () => {
      const transport = createMockCrdtTransport();
      const session = new DocumentSession('/test.pc', transport);

      // Rapid edits
      for (let i = 0; i < 100; i++) {
        session.insert(i, 'x');
      }

      // Wait for debounce
      await new Promise(resolve => setTimeout(resolve, 50));

      // Should batch into fewer sends
      const sentUpdates = transport.getSentUpdates();
      expect(sentUpdates.length).toBeLessThan(100);
      expect(session.getText().length).toBe(100);

      session.dispose();
    });

    it('session filters updates by origin correctly', async () => {
      const transport = createMockCrdtTransport();
      const session = new DocumentSession('/test.pc', transport);

      const localChanges: number[] = [];
      const remoteChanges: number[] = [];

      session.onTextChange((delta, origin) => {
        if (origin === 'local') {
          localChanges.push(1);
        } else if (origin === 'remote') {
          remoteChanges.push(1);
        }
      });

      // Local edit
      session.insert(0, 'local');

      // Simulate remote update
      const remoteDoc = new CRDTDocument();
      remoteDoc.setText('remote');
      transport.simulateRemoteUpdate(remoteDoc.encodeState());

      expect(localChanges.length).toBe(1);
      expect(remoteChanges.length).toBe(1);

      session.dispose();
    });

    it('multiple sessions converge', async () => {
      const transport1 = createMockCrdtTransport();
      const transport2 = createMockCrdtTransport();

      const session1 = new DocumentSession('/test.pc', transport1);
      const session2 = new DocumentSession('/test.pc', transport2);

      // Both start with same content
      session1.setText('base');
      session2.applyUpdate(session1.getDocument().encodeState());

      // Concurrent edits
      session1.insert(4, '1');
      session2.insert(4, '2');

      // Sync
      session1.applyUpdate(session2.getDocument().encodeState());
      session2.applyUpdate(session1.getDocument().encodeState());

      expect(session1.getText()).toBe(session2.getText());

      session1.dispose();
      session2.dispose();
    });
  });

  describe('adversarial scenarios', () => {
    it('survives "edit war" - alternating overwrites', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      for (let i = 0; i < 20; i++) {
        // User 1 replaces all content
        doc1.setText(`User1 version ${i}`);
        doc2.applyUpdate(doc1.encodeState());

        // User 2 immediately replaces all content
        doc2.setText(`User2 version ${i}`);
        doc1.applyUpdate(doc2.encodeState());
      }

      // Should converge (last write wins for full replacement)
      expect(doc1.getText()).toBe(doc2.getText());
    });

    it('handles pathological interleaving pattern', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      doc1.setText('0123456789');
      doc2.applyUpdate(doc1.encodeState());

      // Alternating deletions from both sides
      for (let i = 0; i < 5; i++) {
        if (doc1.getText().length > 0) {
          doc1.delete(0, 1); // Delete from start
        }
        if (doc2.getText().length > 0) {
          doc2.delete(doc2.getText().length - 1, 1); // Delete from end
        }

        doc1.applyUpdate(doc2.encodeState());
        doc2.applyUpdate(doc1.encodeState());
      }

      // Should converge to empty or same remaining content
      expect(doc1.getText()).toBe(doc2.getText());
    });

    it('handles insert-at-deleted-position gracefully', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      doc1.setText('hello');
      doc2.applyUpdate(doc1.encodeState());

      // User 1 deletes "ell"
      doc1.delete(1, 3);

      // User 2 (before seeing delete) inserts at position 2
      doc2.insert(2, 'X');

      // Sync
      doc1.applyUpdate(doc2.encodeState());
      doc2.applyUpdate(doc1.encodeState());

      // Should converge - X might be preserved or not depending on Yjs semantics
      expect(doc1.getText()).toBe(doc2.getText());
    });

    it('handles concurrent replace-all operations', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();
      const doc3 = new CRDTDocument();

      doc1.setText('original');
      doc2.applyUpdate(doc1.encodeState());
      doc3.applyUpdate(doc1.encodeState());

      // All three replace content simultaneously
      doc1.setText('from user 1');
      doc2.setText('from user 2');
      doc3.setText('from user 3');

      // Merge all
      const state1 = doc1.encodeState();
      const state2 = doc2.encodeState();
      const state3 = doc3.encodeState();

      doc1.applyUpdate(state2);
      doc1.applyUpdate(state3);
      doc2.applyUpdate(state1);
      doc2.applyUpdate(state3);
      doc3.applyUpdate(state1);
      doc3.applyUpdate(state2);

      // All should converge
      expect(doc1.getText()).toBe(doc2.getText());
      expect(doc2.getText()).toBe(doc3.getText());
    });
  });

  describe('dirty flag behavior', () => {
    it('tracks dirty state through multiple operations', () => {
      const doc = new CRDTDocument();

      expect(doc.isDirty()).toBe(false);

      doc.insert(0, 'hello');
      expect(doc.isDirty()).toBe(true);

      doc.markClean();
      expect(doc.isDirty()).toBe(false);

      doc.delete(0, 2);
      expect(doc.isDirty()).toBe(true);

      doc.markClean();
      doc.setText('new content');
      expect(doc.isDirty()).toBe(true);
    });

    it('becomes dirty on remote updates', () => {
      const doc1 = new CRDTDocument();
      const doc2 = new CRDTDocument();

      doc1.setText('hello');
      doc2.markClean();
      expect(doc2.isDirty()).toBe(false);

      doc2.applyUpdate(doc1.encodeState());
      expect(doc2.isDirty()).toBe(true);
    });
  });
});
