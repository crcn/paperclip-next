/**
 * Proof of concept: Conflict detection when target is modified
 *
 * This validates that we can detect when VS Code modifies the
 * exact content that a designer mutation wants to change.
 */

import * as Y from 'yjs';

function proof() {
  const doc = new Y.Doc();
  const text = doc.getText('content');

  // Initial content
  const source = `@frame(x: 0, y: 0, width: 100, height: 100)`;
  text.insert(0, source);

  // Store RelativePositions for @frame
  const frameStart = 0;
  const frameEnd = source.length;
  const relStart = Y.createRelativePositionFromTypeIndex(text, frameStart);
  const relEnd = Y.createRelativePositionFromTypeIndex(text, frameEnd);

  // Store expected content at parse time
  const expectedContent = source;

  console.log('=== Initial State ===');
  console.log(`Content: "${text.toString()}"`);
  console.log(`Expected: "${expectedContent}"`);

  // VS Code modifies the frame directly (conflict!)
  console.log('\n=== VS Code modifies the frame ===');
  text.delete(7, 1); // Delete 'x'
  text.insert(7, 'X'); // Insert 'X' (capital)
  console.log(`Content after VS Code edit: "${text.toString()}"`);

  // Resolve positions
  const absStart = Y.createAbsolutePositionFromRelativePosition(relStart, doc);
  const absEnd = Y.createAbsolutePositionFromRelativePosition(relEnd, doc);
  const newStart = absStart?.index ?? -1;
  const newEnd = absEnd?.index ?? -1;

  // Check if content matches expected
  const actualContent = text.toString().slice(newStart, newEnd);

  console.log('\n=== Conflict Detection ===');
  console.log(`Expected: "${expectedContent}"`);
  console.log(`Actual:   "${actualContent}"`);
  console.log(`Conflict detected: ${expectedContent !== actualContent ? '✅ YES (correctly detected)' : '❌ NO (missed conflict!)'}`);

  // Designer should NOT apply mutation when conflict detected
  if (expectedContent !== actualContent) {
    console.log('\n=== Designer mutation rejected due to conflict ===');
  } else {
    console.log('\n=== BUG: Designer would apply mutation over VS Code changes ===');
  }
}

function proofNoConflict() {
  console.log('\n\n========================================');
  console.log('=== CASE 2: No conflict (edit elsewhere) ===');
  console.log('========================================\n');

  const doc = new Y.Doc();
  const text = doc.getText('content');

  const source = `// comment\n@frame(x: 0, y: 0, width: 100, height: 100)`;
  text.insert(0, source);

  // Store RelativePositions for @frame (starts after comment)
  const frameStart = source.indexOf('@frame');
  const frameEnd = source.length;
  const relStart = Y.createRelativePositionFromTypeIndex(text, frameStart);
  const relEnd = Y.createRelativePositionFromTypeIndex(text, frameEnd);

  const expectedContent = '@frame(x: 0, y: 0, width: 100, height: 100)';

  console.log('=== Initial State ===');
  console.log(`Frame content: "${expectedContent}"`);

  // VS Code modifies the COMMENT, not the frame
  console.log('\n=== VS Code modifies the comment (not frame) ===');
  text.delete(0, 10); // Delete '// comment'
  text.insert(0, '// MODIFIED'); // Replace with different comment
  console.log(`Content after VS Code edit: "${text.toString()}"`);

  // Resolve positions
  const absStart = Y.createAbsolutePositionFromRelativePosition(relStart, doc);
  const absEnd = Y.createAbsolutePositionFromRelativePosition(relEnd, doc);
  const newStart = absStart?.index ?? -1;
  const newEnd = absEnd?.index ?? -1;

  const actualContent = text.toString().slice(newStart, newEnd);

  console.log('\n=== Conflict Detection ===');
  console.log(`Expected: "${expectedContent}"`);
  console.log(`Actual:   "${actualContent}"`);
  console.log(`No conflict: ${expectedContent === actualContent ? '✅ YES (correctly allowed)' : '❌ NO (false positive!)'}`);
}

proof();
proofNoConflict();
