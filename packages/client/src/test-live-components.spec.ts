/**
 * Test: Live Component Preview Loading (Spike 0.5) - Automated Tests
 *
 * This validates the hybrid rendering architecture programmatically:
 * 1. Component registry can register and retrieve components
 * 2. React adapter can mount/update/unmount components
 * 3. Hybrid patch applier processes component lifecycle patches
 * 4. Props flow correctly from Virtual DOM to React components
 */

import { createComponentRegistry } from "./component-registry";
import { createReactAdapter } from "./react-adapter";
import { createHybridPatchApplier } from "./hybrid-patch-applier";
import * as DatePickerModule from "./example-components/DatePicker";

describe("Spike 0.5: Live Component Preview Loading", () => {
  test("Component registry can register and retrieve components", () => {
    const registry = createComponentRegistry();

    registry.register({
      id: "DatePicker",
      framework: "react",
      module: DatePickerModule,
      exportName: "DatePicker",
    });

    expect(registry.has("DatePicker")).toBe(true);
    expect(registry.list()).toHaveLength(1);

    const component = registry.get("DatePicker");
    expect(component).toBeDefined();
    expect(component?.id).toBe("DatePicker");
    expect(component?.framework).toBe("react");
  });

  test("React adapter can mount components", () => {
    const adapter = createReactAdapter();
    const container = document.createElement("div");
    document.body.appendChild(container); // Add to document

    const metadata = {
      id: "DatePicker",
      framework: "react" as const,
      module: DatePickerModule,
      exportName: "DatePicker",
    };

    const props = {
      label: "Select Date",
      initialDate: "2026-02-01",
    };

    // Mount component
    const mount = adapter.mount(container, metadata, props);

    // Verify mount was created
    expect(mount.container).toBe(container);
    expect(mount.props).toEqual(props);

    // Cleanup
    adapter.unmount(mount);
    document.body.removeChild(container);
  });

  test("React adapter can update component props", () => {
    const adapter = createReactAdapter();
    const container = document.createElement("div");
    document.body.appendChild(container);

    const metadata = {
      id: "DatePicker",
      framework: "react" as const,
      module: DatePickerModule,
      exportName: "DatePicker",
    };

    // Mount with initial props
    const mount = adapter.mount(container, metadata, {
      label: "Select Date",
      initialDate: "2026-02-01",
    });

    // Update props
    const newProps = {
      label: "Choose Your Date",
      initialDate: "2026-03-15",
    };
    adapter.update(mount, newProps);

    // Verify props were updated
    expect(mount.props).toEqual(newProps);

    // Cleanup
    adapter.unmount(mount);
    document.body.removeChild(container);
  });

  test("React adapter can unmount components", () => {
    const adapter = createReactAdapter();
    const container = document.createElement("div");
    document.body.appendChild(container);

    const metadata = {
      id: "DatePicker",
      framework: "react" as const,
      module: DatePickerModule,
      exportName: "DatePicker",
    };

    // Mount component
    const mount = adapter.mount(container, metadata, {
      label: "Select Date",
      initialDate: "2026-02-01",
    });

    // Unmount
    adapter.unmount(mount);

    // Verify unmount succeeded (no error thrown)
    expect(true).toBe(true);

    // Cleanup
    document.body.removeChild(container);
  });

  test("Hybrid patch applier can mount components", () => {
    const registry = createComponentRegistry();
    registry.register({
      id: "DatePicker",
      framework: "react",
      module: DatePickerModule,
      exportName: "DatePicker",
    });

    const reactAdapter = createReactAdapter();
    const hybridApplier = createHybridPatchApplier({
      registry,
      reactAdapter,
    });

    const rootElement = document.createElement("div");
    document.body.appendChild(rootElement);

    const mountPatch = {
      type: "MOUNT_COMPONENT" as const,
      path: [0],
      componentId: "DatePicker",
      props: {
        label: "Select Date",
        initialDate: "2026-02-01",
      },
      index: 0,
    };

    hybridApplier.apply([mountPatch], rootElement);

    // Verify component was mounted
    expect(rootElement.children.length).toBeGreaterThan(0);

    // Cleanup
    const unmountPatch = {
      type: "UNMOUNT_COMPONENT" as const,
      path: [0],
    };
    hybridApplier.apply([unmountPatch], rootElement);
    document.body.removeChild(rootElement);
  });

  test("Hybrid patch applier can update component props", () => {
    const registry = createComponentRegistry();
    registry.register({
      id: "DatePicker",
      framework: "react",
      module: DatePickerModule,
      exportName: "DatePicker",
    });

    const reactAdapter = createReactAdapter();
    const hybridApplier = createHybridPatchApplier({
      registry,
      reactAdapter,
    });

    const rootElement = document.createElement("div");
    document.body.appendChild(rootElement);

    // Mount first
    const mountPatch = {
      type: "MOUNT_COMPONENT" as const,
      path: [0],
      componentId: "DatePicker",
      props: {
        label: "Select Date",
        initialDate: "2026-02-01",
      },
      index: 0,
    };

    hybridApplier.apply([mountPatch], rootElement);
    expect(rootElement.children.length).toBeGreaterThan(0);

    // Update props
    const updatePatch = {
      type: "UPDATE_COMPONENT_PROPS" as const,
      path: [0],
      props: {
        label: "Choose Your Date",
        initialDate: "2026-03-15",
      },
    };

    // Should not throw
    expect(() => {
      hybridApplier.apply([updatePatch], rootElement);
    }).not.toThrow();

    // Cleanup
    const unmountPatch = {
      type: "UNMOUNT_COMPONENT" as const,
      path: [0],
    };
    hybridApplier.apply([unmountPatch], rootElement);
    document.body.removeChild(rootElement);
  });

  test("Hybrid patch applier can unmount components", () => {
    const registry = createComponentRegistry();
    registry.register({
      id: "DatePicker",
      framework: "react",
      module: DatePickerModule,
      exportName: "DatePicker",
    });

    const reactAdapter = createReactAdapter();
    const hybridApplier = createHybridPatchApplier({
      registry,
      reactAdapter,
    });

    const rootElement = document.createElement("div");
    document.body.appendChild(rootElement);

    // Mount first
    const mountPatch = {
      type: "MOUNT_COMPONENT" as const,
      path: [0],
      componentId: "DatePicker",
      props: {
        label: "Select Date",
        initialDate: "2026-02-01",
      },
      index: 0,
    };

    hybridApplier.apply([mountPatch], rootElement);
    expect(rootElement.children.length).toBeGreaterThan(0);

    // Unmount
    const unmountPatch = {
      type: "UNMOUNT_COMPONENT" as const,
      path: [0],
    };

    hybridApplier.apply([unmountPatch], rootElement);

    // Component should be removed
    expect(rootElement.children.length).toBe(0);

    // Cleanup
    document.body.removeChild(rootElement);
  });

  test("Full hybrid rendering flow", () => {
    const registry = createComponentRegistry();
    registry.register({
      id: "DatePicker",
      framework: "react",
      module: DatePickerModule,
      exportName: "DatePicker",
    });

    const reactAdapter = createReactAdapter();
    const hybridApplier = createHybridPatchApplier({
      registry,
      reactAdapter,
    });

    // Create a container with static content + live component
    const rootElement = document.createElement("div");
    document.body.appendChild(rootElement);

    // Add static div
    const staticDiv = document.createElement("div");
    staticDiv.textContent = "Book Your Appointment";
    rootElement.appendChild(staticDiv);

    // Mount live component at index 1
    const mountPatch = {
      type: "MOUNT_COMPONENT" as const,
      path: [1],
      componentId: "DatePicker",
      props: {
        label: "Select Date",
        initialDate: "2026-02-01",
      },
      index: 1,
    };

    hybridApplier.apply([mountPatch], rootElement);

    // Verify structure: static div + component
    expect(rootElement.children.length).toBe(2);
    expect(rootElement.children[0].textContent).toBe("Book Your Appointment");

    // Update component props
    const updatePatch = {
      type: "UPDATE_COMPONENT_PROPS" as const,
      path: [1],
      props: {
        label: "Choose Your Date",
        initialDate: "2026-03-15",
      },
    };

    hybridApplier.apply([updatePatch], rootElement);

    // Structure should remain the same
    expect(rootElement.children.length).toBe(2);

    // Unmount component
    const unmountPatch = {
      type: "UNMOUNT_COMPONENT" as const,
      path: [1],
    };

    hybridApplier.apply([unmountPatch], rootElement);

    // Only static div should remain
    expect(rootElement.children.length).toBe(1);
    expect(rootElement.children[0].textContent).toBe("Book Your Appointment");

    // Cleanup
    document.body.removeChild(rootElement);
  });
});
