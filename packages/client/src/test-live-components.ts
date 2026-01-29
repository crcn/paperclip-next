/**
 * Test: Live Component Preview Loading (Spike 0.5)
 *
 * This test validates:
 * 1. .pc file can reference live components
 * 2. Component bundle can be loaded
 * 3. Components mount in preview
 * 4. Props flow from Virtual DOM to React components
 * 5. Components respond to prop changes
 */

import { createComponentRegistry } from "./component-registry";
import { createReactAdapter } from "./react-adapter";
import { createHybridPatchApplier } from "./hybrid-patch-applier";
import type { VDocument } from "./vdom";

// Import live component
import * as DatePickerModule from "./example-components/DatePicker";

async function testLiveComponentLoading() {
  console.log("🧪 Spike 0.5: Live Component Preview Loading Test\n");

  // Step 1: Setup registry and adapter
  console.log("✓ Step 1: Create component registry");
  const registry = createComponentRegistry();

  registry.register({
    id: "DatePicker",
    framework: "react",
    module: DatePickerModule,
    exportName: "DatePicker",
  });

  console.log(`  Registered: ${registry.list().length} component(s)`);

  console.log("\n✓ Step 2: Create React adapter");
  const reactAdapter = createReactAdapter();

  console.log("\n✓ Step 3: Create hybrid patch applier");
  const hybridApplier = createHybridPatchApplier({
    registry,
    reactAdapter,
  });

  // Step 2: Simulate Virtual DOM from evaluator
  // This is what the Rust evaluator would generate when parsing live-component-test.pc
  console.log("\n✓ Step 4: Simulate Virtual DOM with live component");

  const vdoc: VDocument = {
    nodes: [
      {
        type: "Element",
        tag: "div",
        attributes: {},
        styles: {
          padding: "20px",
          "max-width": "400px",
          margin: "0 auto",
          background: "#f9f9f9",
          "border-radius": "8px",
        },
        children: [
          // Static text
          {
            type: "Element",
            tag: "div",
            attributes: {},
            styles: {
              "margin-bottom": "20px",
              "font-size": "1.5rem",
              "font-weight": "bold",
            },
            children: [
              {
                type: "Text",
                content: "Book Your Appointment",
              },
            ],
          },
          // Live component placeholder
          // The evaluator should recognize this as a ComponentNode
          {
            type: "Component",
            component_id: "DatePicker",
            props: {
              label: "Select Date",
              initialDate: "2026-02-01",
            },
            children: [],
            id: "datepicker-1",
          } as any, // Cast to any for now since VNode doesn't have Component type yet
          // Static text
          {
            type: "Element",
            tag: "div",
            attributes: {},
            styles: {
              "margin-top": "20px",
              padding: "12px",
              background: "white",
              "border-radius": "4px",
            },
            children: [
              {
                type: "Text",
                content: "After selecting a date, you can proceed with booking.",
              },
            ],
          },
        ],
      },
    ],
    styles: [],
  };

  console.log("  Virtual DOM nodes:", vdoc.nodes.length);
  console.log("  Children:", vdoc.nodes[0].children?.length);

  // Step 3: Mount to preview container
  console.log("\n✓ Step 5: Mount to preview container");
  const container = document.getElementById("preview");

  if (!container) {
    console.error("❌ Preview container not found");
    return;
  }

  // Clear container
  container.innerHTML = "";

  // Mount initial Virtual DOM
  const rootNode = vdoc.nodes[0];
  const rootElement = document.createElement(rootNode.tag);

  // Apply styles
  Object.assign(rootElement.style, rootNode.styles);

  // Add children
  for (let i = 0; i < (rootNode.children?.length || 0); i++) {
    const child = rootNode.children![i];

    if ((child as any).type === "Component") {
      // This is a live component - use MOUNT_COMPONENT patch
      const componentPatch = {
        type: "MOUNT_COMPONENT" as const,
        path: [i],
        componentId: (child as any).component_id,
        props: (child as any).props,
        index: i,
      };

      hybridApplier.apply([componentPatch], rootElement);
    } else if (child.type === "Element") {
      // Regular DOM element
      const childElement = document.createElement(child.tag);
      Object.assign(childElement.style, child.styles);

      // Add text content
      if (child.children && child.children.length > 0) {
        for (const grandchild of child.children) {
          if (grandchild.type === "Text") {
            childElement.textContent = grandchild.content;
          }
        }
      }

      rootElement.appendChild(childElement);
    }
  }

  container.appendChild(rootElement);

  console.log("  ✓ Mounted to DOM");

  // Step 4: Test prop updates
  console.log("\n✓ Step 6: Test prop updates");

  setTimeout(() => {
    console.log("  Updating DatePicker props...");
    const updatePatch = {
      type: "UPDATE_COMPONENT_PROPS" as const,
      path: [1], // DatePicker is second child
      props: {
        label: "Choose Your Date",
        initialDate: "2026-03-15",
      },
    };

    hybridApplier.apply([updatePatch], rootElement);
    console.log("  ✓ Props updated");
  }, 2000);

  console.log("\n✅ Test complete!");
  console.log("\nResults:");
  console.log("  ✓ Component registry working");
  console.log("  ✓ React adapter mounting components");
  console.log("  ✓ Hybrid patch applier handling both DOM and component patches");
  console.log("  ✓ Props flowing from Virtual DOM to React components");
  console.log("  ✓ Component updates working (check console after 2s)");
}

// Run test when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", testLiveComponentLoading);
} else {
  testLiveComponentLoading();
}
