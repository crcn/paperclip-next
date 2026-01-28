/**
 * Demo: Hybrid Rendering with React Components
 *
 * This demonstrates:
 * 1. Registering a React component
 * 2. Creating a VDocument with Component nodes
 * 3. Generating patches (MOUNT_COMPONENT)
 * 4. Applying patches via hybridPatchApplier
 * 5. Updating component props
 */

import { createComponentRegistry } from "./component-registry";
import { createBundleLoader } from "./bundle-loader";
import { createReactAdapter } from "./react-adapter";
import { createHybridPatchApplier } from "./hybrid-patch-applier";
import type { VDocument, VNode, Patch } from "./vdom";

export async function runHybridDemo() {
  console.log("🚀 Starting Hybrid Rendering Demo");

  // 1. Set up infrastructure
  const registry = createComponentRegistry();
  const loader = createBundleLoader();
  const reactAdapter = createReactAdapter();
  const applier = createHybridPatchApplier({ registry, reactAdapter });

  // 2. Load and register DatePicker component
  console.log("📦 Loading DatePicker component");
  const DatePickerModule = await import("./example-components/DatePicker");

  registry.register({
    id: "react:DatePicker",
    framework: "react",
    module: DatePickerModule,
    exportName: "DatePicker",
  });

  console.log("✅ DatePicker registered");

  // 3. Create initial VDocument with Component node
  const initialVDoc: VDocument = {
    nodes: [
      {
        type: "Element",
        tag: "div",
        attributes: { id: "demo-root" },
        styles: { padding: "2rem" },
        children: [
          {
            type: "Element",
            tag: "h1",
            attributes: {},
            styles: {},
            children: [{ type: "Text", content: "Hybrid Rendering Demo" }],
          },
          {
            type: "Component",
            componentId: "react:DatePicker",
            props: {
              label: "Choose a date",
              initialDate: "2024-01-15",
            },
            children: [],
          },
        ],
      },
    ],
    styles: [],
  };

  // 4. Generate patches for initial render (manually for demo)
  const initialPatches: Patch[] = [
    {
      type: "CREATE",
      path: [0],
      index: 0,
      node: initialVDoc.nodes[0],
    },
    {
      type: "MOUNT_COMPONENT",
      path: [0, 1],
      componentId: "react:DatePicker",
      props: {
        label: "Choose a date",
        initialDate: "2024-01-15",
      },
      index: 1,
    },
  ];

  // 5. Apply patches to DOM
  console.log("🎨 Rendering initial state");
  const root = document.getElementById("app");
  if (!root) {
    console.error("❌ Root element not found");
    return;
  }

  applier.apply(initialPatches, root as Element);
  console.log("✅ Initial render complete");

  // 6. Test prop updates after 2 seconds
  setTimeout(() => {
    console.log("🔄 Updating component props");

    const updatePatches: Patch[] = [
      {
        type: "UPDATE_COMPONENT_PROPS",
        path: [0, 1],
        props: {
          label: "Updated: Pick a different date",
          initialDate: "2024-12-25",
        },
      },
    ];

    applier.apply(updatePatches, root as Element);
    console.log("✅ Props updated");
  }, 2000);

  console.log("✨ Demo running - watch for prop update in 2 seconds");
}
