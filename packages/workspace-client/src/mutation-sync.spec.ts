/**
 * COMPREHENSIVE TESTS FOR DESIGNER MUTATION → Y.TEXT SYNC
 *
 * This is the most critical flow in the system:
 * 1. Designer mutation → Apply to AST → Convert to Y.Text edit (via RelPos) → Y.Text
 * 2. VS Code edit → Y.Text directly (Yjs sync)
 *
 * These tests try to BREAK the system by:
 * - Concurrent edits from multiple sources
 * - Race conditions between mutation translation and remote edits
 * - Edge cases in RelativePosition resolution
 * - Malformed inputs
 * - Rapid sequential mutations
 * - Large documents
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import * as Y from 'yjs';
import {
  CRDTDocumentWithRelPos,
  ASTIndex,
  ASTNode,
  MutationHandler,
  SourceSpan,
  Mutation,
} from './crdt-relpos';

// ============================================================================
// TEST HELPERS
// ============================================================================

/**
 * Create a simple Paperclip document for testing.
 */
function createTestDocument(): string {
  return `/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Button {
    render div {
        style {
            padding: 16px
            color: blue
        }
        text "Click me"
    }
}`;
}

/**
 * Create a mock AST with source spans.
 * In production, this comes from the parser.
 */
function createMockAST(doc: CRDTDocumentWithRelPos, source: string): ASTNode {
  // Find @frame annotation
  const frameMatch = source.match(/@frame\(x: (\d+), y: (\d+), width: (\d+), height: (\d+)\)/);
  const frameStart = source.indexOf('@frame');
  const frameEnd = frameStart + (frameMatch?.[0]?.length ?? 0);

  // Find style block
  const styleStart = source.indexOf('style {');
  const styleEnd = source.indexOf('}', styleStart) + 1;

  // Find padding property
  const paddingMatch = source.match(/padding: (\d+px)/);
  const paddingStart = source.indexOf('padding:');
  const paddingEnd = paddingStart + (paddingMatch?.[0]?.length ?? 'padding: 16px'.length);

  // Find color property
  const colorMatch = source.match(/color: (\w+)/);
  const colorStart = source.indexOf('color:', paddingEnd);
  const colorEnd = colorStart + (colorMatch?.[0]?.length ?? 'color: blue'.length);

  // Find text node
  const textStart = source.indexOf('text "');
  const textEnd = source.indexOf('"', textStart + 6) + 1;

  // Find div
  const divStart = source.indexOf('render div');
  const divEnd = source.lastIndexOf('}', source.lastIndexOf('}') - 1) + 1;

  // Create spans with RelativePositions
  const frameSpan = doc.createSpan(frameStart, frameEnd);
  const styleSpan = doc.createSpan(styleStart, styleEnd);
  const paddingSpan = doc.createSpan(paddingStart, paddingEnd);
  const colorSpan = doc.createSpan(colorStart, colorEnd);
  const textSpan = doc.createSpan(textStart, textEnd);
  const divSpan = doc.createSpan(divStart, divEnd);

  const styleNode: ASTNode = {
    id: 'style-1',
    type: 'style',
    span: styleSpan,
    properties: new Map([
      ['padding', { value: '16px', span: paddingSpan }],
      ['color', { value: 'blue', span: colorSpan }],
    ]),
  };

  const textNode: ASTNode = {
    id: 'text-1',
    type: 'text',
    span: textSpan,
  };

  const divNode: ASTNode = {
    id: 'div-1',
    type: 'element',
    span: divSpan,
    children: [styleNode, textNode],
  };

  const componentNode: ASTNode = {
    id: 'component-1',
    type: 'component',
    span: doc.createSpan(0, source.length),
    frame: {
      x: 0,
      y: 0,
      width: 100,
      height: 100,
      span: frameSpan,
    },
    children: [divNode],
  };

  return componentNode;
}

/**
 * Simulate a VS Code edit (direct Y.Text modification).
 */
function simulateVSCodeEdit(doc: CRDTDocumentWithRelPos, index: number, deleteCount: number, insertText: string): void {
  doc.transaction(() => {
    if (deleteCount > 0) {
      doc.delete(index, deleteCount);
    }
    if (insertText.length > 0) {
      doc.insert(index, insertText);
    }
  }, { origin: 'vscode' });
}

/**
 * Create two synced documents (simulating server + client or two clients).
 */
function createSyncedDocs(): [CRDTDocumentWithRelPos, CRDTDocumentWithRelPos] {
  const doc1 = new CRDTDocumentWithRelPos();
  const doc2 = new CRDTDocumentWithRelPos();

  // Set up bidirectional sync
  doc1.onChange(() => {
    const update = doc1.encodeState();
    doc2.applyUpdate(update, { origin: 'sync' });
  });

  doc2.onChange(() => {
    const update = doc2.encodeState();
    doc1.applyUpdate(update, { origin: 'sync' });
  });

  return [doc1, doc2];
}

// ============================================================================
// BASIC MUTATION TESTS
// ============================================================================

describe('Basic Mutation Application', () => {
  let doc: CRDTDocumentWithRelPos;
  let astIndex: ASTIndex;
  let handler: MutationHandler;

  beforeEach(() => {
    doc = new CRDTDocumentWithRelPos();
    const source = createTestDocument();
    doc.setText(source);
    doc.markClean();

    const ast = createMockAST(doc, source);
    astIndex = new ASTIndex(ast);
    handler = new MutationHandler(doc, astIndex);
  });

  it('applies SetFrameBounds mutation', () => {
    const result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 100, y: 200, width: 300, height: 400 },
    });

    expect(result.success).toBe(true);
    expect(doc.getText()).toContain('@frame(x: 100, y: 200, width: 300, height: 400)');
  });

  it('applies SetStyleProperty mutation', () => {
    const result = handler.apply({
      type: 'SetStyleProperty',
      nodeId: 'style-1',
      property: 'padding',
      value: '32px',
    });

    expect(result.success).toBe(true);
    expect(doc.getText()).toContain('padding: 32px');
  });

  it('applies SetTextContent mutation', () => {
    const result = handler.apply({
      type: 'SetTextContent',
      nodeId: 'text-1',
      content: 'Hello World',
    });

    expect(result.success).toBe(true);
    expect(doc.getText()).toContain('text "Hello World"');
  });

  it('fails gracefully for non-existent node', () => {
    const result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'non-existent',
      bounds: { x: 0, y: 0, width: 100, height: 100 },
    });

    expect(result.success).toBe(false);
    expect(result.reason).toBe('node_not_found');
  });
});

// ============================================================================
// RELATIVE POSITION TESTS
// ============================================================================

describe('RelativePosition Survival', () => {
  let doc: CRDTDocumentWithRelPos;

  beforeEach(() => {
    doc = new CRDTDocumentWithRelPos();
    doc.setText('AAABBBCCC');
  });

  it('RelativePosition survives insertion before', () => {
    // Create RelPos pointing to 'BBB' (index 3)
    const relPos = doc.createRelativePosition(3);

    // Insert text at beginning
    doc.insert(0, 'XXX');

    // Resolve - should now be at index 6
    const resolved = doc.resolveRelativePosition(relPos);
    expect(resolved).toBe(6);
    expect(doc.getText().charAt(resolved!)).toBe('B');
  });

  it('RelativePosition survives insertion after', () => {
    const relPos = doc.createRelativePosition(3);

    // Insert text at end
    doc.insert(9, 'XXX');

    // Should still be at index 3
    const resolved = doc.resolveRelativePosition(relPos);
    expect(resolved).toBe(3);
  });

  it('RelativePosition survives deletion before', () => {
    const relPos = doc.createRelativePosition(6); // 'C'

    // Delete 'AAA'
    doc.delete(0, 3);

    // Should now be at index 3
    const resolved = doc.resolveRelativePosition(relPos);
    expect(resolved).toBe(3);
    expect(doc.getText().charAt(resolved!)).toBe('C');
  });

  it('RelativePosition survives deletion after', () => {
    const relPos = doc.createRelativePosition(3);

    // Delete 'CCC'
    doc.delete(6, 3);

    // Should still be at index 3
    const resolved = doc.resolveRelativePosition(relPos);
    expect(resolved).toBe(3);
  });

  it('RelativePosition handles deletion AT the position', () => {
    const relPos = doc.createRelativePosition(3); // Start of 'BBB'

    // Delete 'BBB'
    doc.delete(3, 3);

    // Position should still resolve (to where BBB was)
    const resolved = doc.resolveRelativePosition(relPos);
    expect(resolved).not.toBeNull();
    // After deletion, text is 'AAACCC', position 3 is 'C'
  });

  it('Span survives multiple edits', () => {
    const span = doc.createSpan(3, 6); // 'BBB'

    // Multiple edits
    doc.insert(0, 'XXX'); // 'XXXAAABBBCCC'
    doc.delete(9, 3); // 'XXXAAABBB'
    doc.insert(9, 'YYY'); // 'XXXAAABBBYYYY'

    const resolved = doc.resolveSpan(span);
    expect(resolved).not.toBeNull();
    expect(resolved!.start).toBe(6); // Shifted by 'XXX' insertion
    expect(resolved!.end).toBe(9);
    expect(doc.getText().slice(resolved!.start, resolved!.end)).toBe('BBB');
  });
});

// ============================================================================
// CONCURRENT EDIT TESTS - THE CRITICAL ONES
// ============================================================================

describe('Concurrent Designer + VS Code Edits', () => {
  let doc: CRDTDocumentWithRelPos;
  let astIndex: ASTIndex;
  let handler: MutationHandler;

  beforeEach(() => {
    doc = new CRDTDocumentWithRelPos();
    const source = createTestDocument();
    doc.setText(source);
    doc.markClean();

    const ast = createMockAST(doc, source);
    astIndex = new ASTIndex(ast);
    handler = new MutationHandler(doc, astIndex);
  });

  it('Designer mutation survives VS Code edit BEFORE target', () => {
    // VS Code adds comment at the very beginning
    simulateVSCodeEdit(doc, 0, 0, '// Added by VS Code\n');

    // Designer applies mutation to frame (which has shifted)
    const result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 999, y: 999, width: 999, height: 999 },
    });

    expect(result.success).toBe(true);
    expect(doc.getText()).toContain('@frame(x: 999, y: 999, width: 999, height: 999)');
    expect(doc.getText()).toContain('// Added by VS Code');
  });

  it('Designer mutation survives VS Code edit AFTER target', () => {
    // VS Code adds comment at the end
    const endPos = doc.getText().length;
    simulateVSCodeEdit(doc, endPos, 0, '\n// Added by VS Code');

    // Designer applies mutation to frame
    const result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 111, y: 222, width: 333, height: 444 },
    });

    expect(result.success).toBe(true);
    expect(doc.getText()).toContain('@frame(x: 111, y: 222, width: 333, height: 444)');
    expect(doc.getText()).toContain('// Added by VS Code');
  });

  it('Designer mutation detects conflict when target was modified by VS Code', () => {
    // VS Code modifies the frame annotation directly
    const frameStart = doc.getText().indexOf('@frame');
    simulateVSCodeEdit(doc, frameStart, 5, 'XXXXX'); // Corrupt @frame

    // Designer tries to apply mutation - should detect conflict
    const result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 0, y: 0, width: 0, height: 0 },
    });

    expect(result.success).toBe(false);
    expect(result.reason).toBe('conflict');
  });

  it('Multiple Designer mutations in sequence', () => {
    // First mutation
    let result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 10, y: 20, width: 30, height: 40 },
    });
    expect(result.success).toBe(true);

    // Need to rebuild AST after mutation
    const newAst = createMockAST(doc, doc.getText());
    astIndex.rebuild(newAst);

    // Second mutation
    result = handler.apply({
      type: 'SetStyleProperty',
      nodeId: 'style-1',
      property: 'padding',
      value: '24px',
    });
    expect(result.success).toBe(true);

    expect(doc.getText()).toContain('@frame(x: 10, y: 20, width: 30, height: 40)');
    expect(doc.getText()).toContain('padding: 24px');
  });

  it('Interleaved Designer and VS Code edits', () => {
    // Designer mutation 1
    handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 1, y: 1, width: 1, height: 1 },
    });

    // VS Code edit (add comment)
    simulateVSCodeEdit(doc, 0, 0, '// Comment 1\n');

    // Rebuild AST
    let ast = createMockAST(doc, doc.getText());
    astIndex.rebuild(ast);
    handler = new MutationHandler(doc, astIndex);

    // Designer mutation 2
    handler.apply({
      type: 'SetStyleProperty',
      nodeId: 'style-1',
      property: 'padding',
      value: '99px',
    });

    // VS Code edit (add another comment)
    simulateVSCodeEdit(doc, 0, 0, '// Comment 2\n');

    // All edits should be present
    const text = doc.getText();
    expect(text).toContain('// Comment 1');
    expect(text).toContain('// Comment 2');
    expect(text).toContain('@frame(x: 1, y: 1, width: 1, height: 1)');
    expect(text).toContain('padding: 99px');
  });
});

// ============================================================================
// MULTI-CLIENT SYNC TESTS
// ============================================================================

describe('Multi-Client Synchronization', () => {
  it('Two VS Code clients typing concurrently converge', () => {
    const doc1 = new CRDTDocumentWithRelPos();
    const doc2 = new CRDTDocumentWithRelPos();

    // Initial sync
    doc1.setText('Hello');
    doc2.applyUpdate(doc1.encodeState());

    // Concurrent edits
    doc1.insert(5, ' World'); // 'Hello World'
    doc2.insert(0, 'Say '); // 'Say Hello'

    // Cross-sync
    doc1.applyUpdate(doc2.encodeState());
    doc2.applyUpdate(doc1.encodeState());

    // Should converge
    expect(doc1.getText()).toBe(doc2.getText());
    expect(doc1.getText()).toContain('Say');
    expect(doc1.getText()).toContain('Hello');
    expect(doc1.getText()).toContain('World');
  });

  it('Designer on client A, VS Code on client B, both sync correctly', () => {
    const serverDoc = new CRDTDocumentWithRelPos();
    const designerDoc = new CRDTDocumentWithRelPos();
    const vscodeDoc = new CRDTDocumentWithRelPos();

    const source = createTestDocument();
    serverDoc.setText(source);

    // Initial sync
    designerDoc.applyUpdate(serverDoc.encodeState());
    vscodeDoc.applyUpdate(serverDoc.encodeState());

    // Designer applies mutation (on server)
    const ast = createMockAST(serverDoc, serverDoc.getText());
    const astIndex = new ASTIndex(ast);
    const handler = new MutationHandler(serverDoc, astIndex);

    handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 500, y: 500, width: 500, height: 500 },
    });

    // VS Code types (on vscode client)
    vscodeDoc.insert(0, '// VS Code was here\n');

    // Sync all
    const serverUpdate = serverDoc.encodeState();
    const vscodeUpdate = vscodeDoc.encodeState();

    designerDoc.applyUpdate(serverUpdate);
    designerDoc.applyUpdate(vscodeUpdate);

    vscodeDoc.applyUpdate(serverUpdate);

    serverDoc.applyUpdate(vscodeUpdate);

    // All should converge
    expect(serverDoc.getText()).toBe(vscodeDoc.getText());
    expect(serverDoc.getText()).toBe(designerDoc.getText());
    expect(serverDoc.getText()).toContain('@frame(x: 500, y: 500, width: 500, height: 500)');
    expect(serverDoc.getText()).toContain('// VS Code was here');
  });
});

// ============================================================================
// STRESS TESTS - TRYING TO BREAK IT
// ============================================================================

describe('Stress Tests', () => {
  it('Rapid sequential mutations (simulating drag)', () => {
    const doc = new CRDTDocumentWithRelPos();
    const source = createTestDocument();
    doc.setText(source);

    // Simulate 60fps drag with frame updates
    for (let i = 0; i < 60; i++) {
      const ast = createMockAST(doc, doc.getText());
      const astIndex = new ASTIndex(ast);
      const handler = new MutationHandler(doc, astIndex);

      const result = handler.apply({
        type: 'SetFrameBounds',
        nodeId: 'component-1',
        bounds: { x: i, y: i, width: 100 + i, height: 100 + i },
      });

      expect(result.success).toBe(true);
    }

    // Final state should have last values
    expect(doc.getText()).toContain('@frame(x: 59, y: 59, width: 159, height: 159)');
  });

  it('Large document with many edits', () => {
    const doc = new CRDTDocumentWithRelPos();

    // Create a large document
    let source = '';
    for (let i = 0; i < 100; i++) {
      source += `/**
 * @frame(x: ${i}, y: ${i}, width: 100, height: 100)
 */
component Component${i} {
    render div {
        style {
            padding: 16px
        }
        text "Component ${i}"
    }
}

`;
    }
    doc.setText(source);

    // Create span at beginning
    const span = doc.createSpan(0, 10);

    // Make many edits throughout the document
    for (let i = 0; i < 50; i++) {
      const pos = Math.floor(Math.random() * doc.getText().length);
      doc.insert(pos, 'X');
    }

    // Span should still resolve
    const resolved = doc.resolveSpan(span);
    expect(resolved).not.toBeNull();
  });

  it('Concurrent edits from many clients', () => {
    const docs: CRDTDocumentWithRelPos[] = [];
    const numClients = 10;

    // Create clients
    for (let i = 0; i < numClients; i++) {
      docs.push(new CRDTDocumentWithRelPos());
    }

    // Initial content
    docs[0].setText('START');
    for (let i = 1; i < numClients; i++) {
      docs[i].applyUpdate(docs[0].encodeState());
    }

    // Each client makes an edit
    for (let i = 0; i < numClients; i++) {
      docs[i].insert(docs[i].getText().length, `_CLIENT${i}`);
    }

    // Sync all to all
    for (let i = 0; i < numClients; i++) {
      const update = docs[i].encodeState();
      for (let j = 0; j < numClients; j++) {
        if (i !== j) {
          docs[j].applyUpdate(update);
        }
      }
    }

    // All should converge to same content
    const finalText = docs[0].getText();
    for (let i = 1; i < numClients; i++) {
      expect(docs[i].getText()).toBe(finalText);
    }

    // All client markers should be present
    for (let i = 0; i < numClients; i++) {
      expect(finalText).toContain(`_CLIENT${i}`);
    }
  });
});

// ============================================================================
// EDGE CASES - MORE WAYS TO BREAK IT
// ============================================================================

describe('Edge Cases', () => {
  it('Empty document', () => {
    const doc = new CRDTDocumentWithRelPos();
    expect(doc.getText()).toBe('');

    const span = doc.createSpan(0, 0);
    const resolved = doc.resolveSpan(span);
    expect(resolved).not.toBeNull();
    expect(resolved!.start).toBe(0);
    expect(resolved!.end).toBe(0);
  });

  it('Delete entire document content', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('Hello World');

    const span = doc.createSpan(0, 5); // 'Hello'

    // Delete everything
    doc.delete(0, doc.getText().length);

    // Span resolution after total deletion
    const resolved = doc.resolveSpan(span);
    // Should still resolve (to position 0)
    expect(resolved).not.toBeNull();
  });

  it('RelativePosition at document end', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('Hello');

    const relPos = doc.createRelativePosition(5); // End

    doc.insert(5, ' World');

    // Position should track to middle now (after Hello, before World)
    const resolved = doc.resolveRelativePosition(relPos);
    expect(resolved).not.toBeNull();
  });

  it('Very long single line', () => {
    const doc = new CRDTDocumentWithRelPos();
    const longLine = 'A'.repeat(100000);
    doc.setText(longLine);

    const span = doc.createSpan(50000, 50010);

    // Edit at beginning
    doc.insert(0, 'XXX');

    const resolved = doc.resolveSpan(span);
    expect(resolved!.start).toBe(50003);
    expect(resolved!.end).toBe(50013);
  });

  it('Unicode content', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('Hello 🎉 World 🌍');

    // Create span around emoji
    const emojiIndex = doc.getText().indexOf('🎉');
    const span = doc.createSpan(emojiIndex, emojiIndex + 2); // Emoji is 2 chars

    // Edit before
    doc.insert(0, 'XXX');

    const resolved = doc.resolveSpan(span);
    expect(resolved).not.toBeNull();
    expect(doc.getText().slice(resolved!.start, resolved!.end)).toBe('🎉');
  });

  it('Mutation to deleted node should fail gracefully', () => {
    const doc = new CRDTDocumentWithRelPos();
    const source = createTestDocument();
    doc.setText(source);

    const ast = createMockAST(doc, source);
    const astIndex = new ASTIndex(ast);
    const handler = new MutationHandler(doc, astIndex);

    // Simulate VS Code deleting the frame annotation
    const frameStart = doc.getText().indexOf('/**');
    const frameEnd = doc.getText().indexOf('*/') + 2;
    simulateVSCodeEdit(doc, frameStart, frameEnd - frameStart, '');

    // Designer tries to modify the now-deleted frame
    const result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 999, y: 999, width: 999, height: 999 },
    });

    // Should fail because the content at the span doesn't match
    expect(result.success).toBe(false);
  });

  it('Mutation with negative coordinates', () => {
    const doc = new CRDTDocumentWithRelPos();
    const source = createTestDocument();
    doc.setText(source);

    const ast = createMockAST(doc, source);
    const astIndex = new ASTIndex(ast);
    const handler = new MutationHandler(doc, astIndex);

    const result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: -100, y: -200, width: 300, height: 400 },
    });

    expect(result.success).toBe(true);
    expect(doc.getText()).toContain('@frame(x: -100, y: -200, width: 300, height: 400)');
  });

  it('Mutation with very large numbers', () => {
    const doc = new CRDTDocumentWithRelPos();
    const source = createTestDocument();
    doc.setText(source);

    const ast = createMockAST(doc, source);
    const astIndex = new ASTIndex(ast);
    const handler = new MutationHandler(doc, astIndex);

    const result = handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 999999999, y: 999999999, width: 999999999, height: 999999999 },
    });

    expect(result.success).toBe(true);
  });
});

// ============================================================================
// CONFLICT DETECTION TESTS
// ============================================================================

describe('Conflict Detection', () => {
  it('Detects when span content changed', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('AAABBBCCC');

    const span = doc.createSpan(3, 6); // 'BBB'

    // Modify content within span
    doc.delete(4, 1); // 'AAABBCCC'
    doc.insert(4, 'X'); // 'AAABXBCCC'

    // Check with expected content
    const resolved = doc.resolveSpan(span, 'BBB');
    expect(resolved!.maybeStale).toBe(true);
  });

  it('No false positive when content unchanged', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('AAABBBCCC');

    const span = doc.createSpan(3, 6); // 'BBB'

    // Edit elsewhere
    doc.insert(0, 'XXX');
    doc.insert(doc.getText().length, 'YYY');

    const resolved = doc.resolveSpan(span, 'BBB');
    expect(resolved!.maybeStale).toBe(false);
    expect(doc.getText().slice(resolved!.start, resolved!.end)).toBe('BBB');
  });

  it('editAtSpan refuses to edit when content changed', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('AAABBBCCC');

    const span = doc.createSpan(3, 6); // 'BBB'

    // Modify content within span
    doc.delete(3, 3);
    doc.insert(3, 'XXX');

    // Try to edit with expected content
    const success = doc.editAtSpan(span, 'NEW', 'BBB');
    expect(success).toBe(false);
  });

  it('editAtSpan succeeds when content matches', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('AAABBBCCC');

    const span = doc.createSpan(3, 6); // 'BBB'

    // Edit elsewhere
    doc.insert(0, 'XXX');

    // Edit should succeed
    const success = doc.editAtSpan(span, 'NEW', 'BBB');
    expect(success).toBe(true);
    expect(doc.getText()).toBe('XXXAAANEWCCC');
  });
});

// ============================================================================
// ORIGIN TRACKING TESTS
// ============================================================================

describe('Origin Tracking', () => {
  it('Designer mutations have designer origin', () => {
    const doc = new CRDTDocumentWithRelPos();
    const source = createTestDocument();
    doc.setText(source);

    const ast = createMockAST(doc, source);
    const astIndex = new ASTIndex(ast);
    const handler = new MutationHandler(doc, astIndex);

    const origins: string[] = [];
    doc.onChange((_, origin) => {
      if (origin) origins.push(origin);
    });

    handler.apply({
      type: 'SetFrameBounds',
      nodeId: 'component-1',
      bounds: { x: 1, y: 2, width: 3, height: 4 },
    }, { origin: 'designer' });

    expect(origins).toContain('designer');
  });

  it('VS Code edits have vscode origin', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('Hello');

    const origins: string[] = [];
    doc.onChange((_, origin) => {
      if (origin) origins.push(origin);
    });

    simulateVSCodeEdit(doc, 5, 0, ' World');

    expect(origins).toContain('vscode');
  });

  it('Can filter by origin to prevent feedback loops', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('Hello');

    const nonLocalChanges: string[] = [];
    doc.onChange((_, origin) => {
      if (origin !== 'local') {
        nonLocalChanges.push(origin ?? 'null');
      }
    });

    // Local edit
    doc.insert(5, '!', { origin: 'local' });

    // Remote edit
    doc.insert(0, '> ', { origin: 'remote' });

    expect(nonLocalChanges).toEqual(['remote']);
  });
});

// ============================================================================
// TRANSACTION BATCHING TESTS
// ============================================================================

describe('Transaction Batching', () => {
  it('Multiple edits in transaction emit single change', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('AAABBBCCC');

    let changeCount = 0;
    doc.onChange(() => changeCount++);

    doc.transaction(() => {
      doc.delete(3, 3);
      doc.insert(3, 'XXX');
      doc.insert(6, 'YYY');
    });

    expect(changeCount).toBe(1);
    expect(doc.getText()).toBe('AAAXXXYYYCC');
  });

  it('MoveNode mutation uses transaction for atomicity', () => {
    const doc = new CRDTDocumentWithRelPos();
    doc.setText('[A][B][C]');

    let changeCount = 0;
    doc.onChange(() => changeCount++);

    // Simulate move: cut [B] and paste before [A]
    doc.transaction(() => {
      const bContent = '[B]';
      doc.delete(3, 3); // Remove [B]
      doc.insert(0, bContent); // Insert before [A]
    });

    expect(changeCount).toBe(1);
    expect(doc.getText()).toBe('[B][A][C]');
  });
});
