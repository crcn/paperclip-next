/**
 * Demonstrates that OT-style patches are pure, serializable data structures
 * Perfect for sending over gRPC or storing for replay
 */

import { VNode, diff } from "./vdom";

const oldTree: VNode = {
  type: "Element",
  tag: "button",
  attributes: { class: "btn" },
  styles: { padding: "8px", background: "blue" },
  children: [{ type: "Text", content: "Click me" }],
};

const newTree: VNode = {
  type: "Element",
  tag: "button",
  attributes: { class: "btn-primary", disabled: "true" },
  styles: { padding: "12px", background: "red" },
  children: [{ type: "Text", content: "Click me now!" }],
};

console.log("🔬 OT-Style Patches Demonstration\n");

// 1. Pure diffing
const patches = diff(oldTree, newTree);
console.log(`Generated ${patches.length} patches:`);
patches.forEach((p) => {
  console.log(`  • ${p.type} at [${p.path.join(",")}]`);
});

// 2. Serialize to JSON (for network transmission)
const json = JSON.stringify(patches, null, 2);
console.log(`\n📦 Serialized to ${json.length} bytes`);

// 3. Show serialized structure
console.log("\n📄 Patch structure:\n");
console.log(json);

// 4. Deserialize (simulate receiving from server)
const received = JSON.parse(json);
console.log(`\n✅ Deserialized ${received.length} patches successfully`);

// 5. Show benefits
console.log("\n🎯 Benefits of OT-style patches:");
console.log("  ✓ Pure functions - no side effects");
console.log("  ✓ Serializable - can send over gRPC/WebSocket");
console.log("  ✓ Testable - no DOM needed for diffing");
console.log("  ✓ Composable - different appliers for DOM/SSR/React");
console.log("  ✓ Platform-agnostic - works anywhere");
console.log("  ✓ OT-compatible - ready for collaborative editing");
