/**
 * Proof of concept: RelativePositions survive concurrent edits
 *
 * This validates the core assumption of our architecture:
 * If we store RelativePositions when parsing, they will correctly
 * resolve to the new positions after concurrent edits.
 */

import * as Y from 'yjs';

function proof() {
  const doc = new Y.Doc();
  const text = doc.getText('content');

  // Initial content
  const source = `/**
 * @frame(x: 0, y: 0, width: 100, height: 100)
 */
component Button {
    render div {
        text "Click me"
    }
}`;

  text.insert(0, source);

  // Find @frame and store RelativePositions
  const frameStart = source.indexOf('@frame');
  const frameEnd = source.indexOf(')', frameStart) + 1;

  console.log('=== Initial State ===');
  console.log(`Frame at absolute positions: ${frameStart}-${frameEnd}`);
  console.log(`Frame content: "${source.slice(frameStart, frameEnd)}"`);

  // Create RelativePositions
  const relStart = Y.createRelativePositionFromTypeIndex(text, frameStart);
  const relEnd = Y.createRelativePositionFromTypeIndex(text, frameEnd);

  console.log('\n=== Simulating VS Code edit: insert comment at beginning ===');

  // VS Code inserts a comment at the beginning
  const comment = '// Added by VS Code\n';
  text.insert(0, comment);

  console.log(`Inserted "${comment.trim()}" at position 0`);

  // Resolve RelativePositions to new absolute positions
  const absStart = Y.createAbsolutePositionFromRelativePosition(relStart, doc);
  const absEnd = Y.createAbsolutePositionFromRelativePosition(relEnd, doc);

  const newStart = absStart?.index ?? -1;
  const newEnd = absEnd?.index ?? -1;

  console.log(`\n=== After VS Code Edit ===`);
  console.log(`Frame now at absolute positions: ${newStart}-${newEnd}`);
  console.log(`Frame content: "${text.toString().slice(newStart, newEnd)}"`);

  // Verify the content is still correct
  const expectedContent = '@frame(x: 0, y: 0, width: 100, height: 100)';
  const actualContent = text.toString().slice(newStart, newEnd);

  console.log(`\n=== Verification ===`);
  console.log(`Expected: "${expectedContent}"`);
  console.log(`Actual:   "${actualContent}"`);
  console.log(`Match: ${expectedContent === actualContent ? '✅ YES' : '❌ NO'}`);

  // Now apply a mutation (change frame bounds)
  console.log('\n=== Applying Designer Mutation: SetFrameBounds ===');

  const newFrameText = '@frame(x: 100, y: 200, width: 300, height: 400)';

  // Delete old frame, insert new
  doc.transact(() => {
    text.delete(newStart, newEnd - newStart);
    text.insert(newStart, newFrameText);
  });

  console.log(`New document:\n${text.toString()}`);
  console.log(`\nFrame updated: ${text.toString().includes(newFrameText) ? '✅ YES' : '❌ NO'}`);
  console.log(`Comment preserved: ${text.toString().includes('// Added by VS Code') ? '✅ YES' : '❌ NO'}`);
}

proof();
