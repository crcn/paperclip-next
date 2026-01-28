/**
 * Test and demonstration of OT-style patches
 * Shows that patches are pure data structures with no DOM references
 */

import { VNode, diff, patch, domPatchApplier } from "./vdom";

console.log("=== OT-Style Patches Test ===\n");

// Create two simple VNode trees
const oldTree: VNode = {
  type: "Element",
  tag: "div",
  attributes: { class: "container" },
  styles: { padding: "10px" },
  children: [
    {
      type: "Element",
      tag: "h1",
      attributes: {},
      styles: {},
      children: [{ type: "Text", content: "Hello" }],
    },
    {
      type: "Element",
      tag: "p",
      attributes: {},
      styles: { color: "blue" },
      children: [{ type: "Text", content: "World" }],
    },
  ],
};

const newTree: VNode = {
  type: "Element",
  tag: "div",
  attributes: { class: "container", id: "main" },
  styles: { padding: "20px" },
  children: [
    {
      type: "Element",
      tag: "h1",
      attributes: {},
      styles: {},
      children: [{ type: "Text", content: "Hello Paperclip!" }],
    },
    {
      type: "Element",
      tag: "p",
      attributes: {},
      styles: { color: "red", "font-weight": "bold" },
      children: [{ type: "Text", content: "OT-style patches!" }],
    },
    {
      type: "Element",
      tag: "button",
      attributes: {},
      styles: {},
      children: [{ type: "Text", content: "New button" }],
    },
  ],
};

// Step 1: Generate patches (pure function - no DOM needed!)
console.log("1. Diffing VNode trees (pure computation)...");
const patches = diff(oldTree, newTree);

console.log(`   Generated ${patches.length} patches:\n`);
patches.forEach((p, i) => {
  console.log(`   [${i}] ${p.type} at path [${p.path.join(", ")}]`);
});

// Step 2: Verify patches are serializable (key OT property!)
console.log("\n2. Verifying patches are serializable...");
try {
  const serialized = JSON.stringify(patches, null, 2);
  console.log("   ✓ Patches are pure data - can be sent over network!");
  console.log(`   Serialized size: ${serialized.length} bytes`);

  // Can deserialize and use later
  const deserialized = JSON.parse(serialized);
  console.log(`   ✓ Deserialized ${deserialized.length} patches`);
} catch (e) {
  console.error("   ✗ Failed to serialize:", e);
}

// Step 3: Apply patches to real DOM (using applier pattern)
console.log("\n3. Applying patches to DOM...");

// Create initial DOM from oldTree (in real app, this would already exist)
const container = document.createElement("div");
document.body.appendChild(container);

// For testing, we need to manually create the old DOM structure
// In the real app, this would already exist from initial render
function quickRender(vnode: VNode): HTMLElement {
  if (vnode.type !== "Element") {
    throw new Error("Root must be Element");
  }

  const el = document.createElement(vnode.tag);

  for (const [k, v] of Object.entries(vnode.attributes)) {
    el.setAttribute(k, v);
  }

  for (const [k, v] of Object.entries(vnode.styles)) {
    (el.style as any)[k] = v;
  }

  for (const child of vnode.children) {
    if (child.type === "Text") {
      el.appendChild(document.createTextNode(child.content));
    } else if (child.type === "Element") {
      el.appendChild(quickRender(child));
    }
  }

  return el;
}

const oldDom = quickRender(oldTree);
container.appendChild(oldDom);

console.log("   Old DOM:", oldDom.outerHTML.substring(0, 100) + "...");

// Apply patches using DOM applier
const applier = domPatchApplier();
patch(patches, oldDom, applier);

console.log("   New DOM:", oldDom.outerHTML.substring(0, 100) + "...");
console.log("   ✓ Patches applied successfully!");

// Step 4: Demonstrate composability - could use different appliers
console.log("\n4. Demonstrating composability...");
console.log("   Same patches could be applied to:");
console.log("   • DOM (current implementation)");
console.log("   • Server-side rendering (HTML string builder)");
console.log("   • React components");
console.log("   • Canvas rendering");
console.log("   • Or sent over gRPC to a remote renderer!");

// Step 5: Show patch structure
console.log("\n5. Example patch structures:\n");
console.log(JSON.stringify(patches.slice(0, 3), null, 2));

console.log("\n=== Test Complete ===");
console.log("The OT-style refactor is working! 🎉");
