/**
 * Hybrid Patch Applier - handles both DOM and React component patches
 */

import type { Patch, VNode, PatchApplier } from "./vdom";
import { domPatchApplier } from "./vdom";
import type { ComponentRegistry } from "./component-registry";
import type { ReactAdapter, ReactComponentMount } from "./react-adapter";

export interface HybridPatchApplierOptions {
  registry: ComponentRegistry;
  reactAdapter: ReactAdapter;
}

export function createHybridPatchApplier(
  options: HybridPatchApplierOptions
): PatchApplier<Element> {
  const { registry, reactAdapter } = options;

  // Track component mounts by path (as string key)
  const componentMounts = new Map<string, ReactComponentMount>();

  // Get standard DOM patch applier
  const domApplier = domPatchApplier();

  // Helper to serialize path to string key
  const pathKey = (path: number[]): string => path.join(",");

  // Helper to get node at path
  const getNodeAtPath = (root: Element, path: number[]): Node | null => {
    let current: Node | null = root;

    for (const index of path) {
      if (!current || !current.childNodes) return null;
      current = current.childNodes[index] || null;
    }

    return current;
  };

  return {
    apply(patches, element): Element {
      for (const patch of patches) {
        switch (patch.type) {
          case "MOUNT_COMPONENT": {
            const metadata = registry.get(patch.componentId);
            if (!metadata) {
              console.error(
                `Component ${patch.componentId} not found in registry`
              );
              continue;
            }

            // Create wrapper container for React component
            const container = document.createElement("div");
            container.dataset.componentId = patch.componentId;
            container.dataset.componentPath = pathKey(patch.path);

            // Mount React component
            const mount = reactAdapter.mount(container, metadata, patch.props);
            componentMounts.set(pathKey(patch.path), mount);

            // Insert container into DOM
            const parent = getNodeAtPath(
              element,
              patch.path.slice(0, -1)
            ) as Element;
            if (parent) {
              if (patch.index < parent.childNodes.length) {
                parent.insertBefore(container, parent.childNodes[patch.index]);
              } else {
                parent.appendChild(container);
              }
            }
            break;
          }

          case "UPDATE_COMPONENT_PROPS": {
            const mount = componentMounts.get(pathKey(patch.path));
            if (mount) {
              reactAdapter.update(mount, patch.props);
            }
            break;
          }

          case "UNMOUNT_COMPONENT": {
            const mount = componentMounts.get(pathKey(patch.path));
            if (mount) {
              reactAdapter.unmount(mount);
              mount.container.remove();
              componentMounts.delete(pathKey(patch.path));
            }
            break;
          }

          default:
            // Delegate to standard DOM applier
            element = domApplier.apply([patch], element);
            break;
        }
      }
      return element;
    },
  };
}
