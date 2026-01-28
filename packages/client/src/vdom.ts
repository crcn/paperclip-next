/**
 * Virtual DOM types matching the Rust evaluator output
 *
 * IMPORTANT: This module uses pure functions with explicit dependencies.
 * Avoid globals in web-related code! See DEVELOPMENT.md for guidelines.
 */

export type VNode =
  | {
      type: "Element";
      tag: string;
      attributes: Record<string, string>;
      styles: Record<string, string>;
      children: VNode[];
      id?: string;
    }
  | {
      type: "Text";
      content: string;
    }
  | {
      type: "Comment";
      content: string;
    }
  | {
      type: "Component";
      componentId: string;
      props: Record<string, any>;
      children: VNode[];
      id?: string;
    };

export interface VDocument {
  nodes: VNode[];
  styles: CssRule[];
}

export interface CssRule {
  selector: string;
  properties: Record<string, string>;
}

/**
 * OT-style patches with path-based addressing
 * Paths are arrays of child indices: [0, 2, 1] = first child → third child → second child
 */
export type Patch =
  | { type: "CREATE"; path: number[]; node: VNode; index: number }
  | { type: "REMOVE"; path: number[] }
  | { type: "REPLACE"; path: number[]; newNode: VNode }
  | { type: "UPDATE_ATTRS"; path: number[]; attributes: Record<string, string> }
  | { type: "UPDATE_STYLES"; path: number[]; styles: Record<string, string> }
  | { type: "UPDATE_TEXT"; path: number[]; content: string }
  | { type: "MOUNT_COMPONENT"; path: number[]; componentId: string; props: any; index: number }
  | { type: "UPDATE_COMPONENT_PROPS"; path: number[]; props: any }
  | { type: "UNMOUNT_COMPONENT"; path: number[] };

/**
 * Pure diff - compares two VNodes and returns abstract patches
 * No DOM references - completely serializable
 */
export function diff(
  oldNode: VNode | null,
  newNode: VNode | null,
  path: number[] = []
): Patch[] {
  const patches: Patch[] = [];

  // Both null - no change
  if (!oldNode && !newNode) {
    return patches;
  }

  // New node added
  if (!oldNode && newNode) {
    // This will be handled by parent
    return patches;
  }

  // Node removed
  if (oldNode && !newNode) {
    patches.push({ type: "REMOVE", path });
    return patches;
  }

  if (!oldNode || !newNode) {
    return patches;
  }

  // Different node types - replace
  if (oldNode.type !== newNode.type) {
    patches.push({ type: "REPLACE", path, newNode });
    return patches;
  }

  // Text nodes
  if (oldNode.type === "Text" && newNode.type === "Text") {
    if (oldNode.content !== newNode.content) {
      patches.push({
        type: "UPDATE_TEXT",
        path,
        content: newNode.content,
      });
    }
    return patches;
  }

  // Element nodes
  if (oldNode.type === "Element" && newNode.type === "Element") {
    // Different tags - replace
    if (oldNode.tag !== newNode.tag) {
      patches.push({ type: "REPLACE", path, newNode });
      return patches;
    }

    // Update attributes
    const attrsChanged =
      JSON.stringify(oldNode.attributes) !== JSON.stringify(newNode.attributes);
    if (attrsChanged) {
      patches.push({
        type: "UPDATE_ATTRS",
        path,
        attributes: newNode.attributes,
      });
    }

    // Update styles
    const stylesChanged =
      JSON.stringify(oldNode.styles) !== JSON.stringify(newNode.styles);
    if (stylesChanged) {
      patches.push({
        type: "UPDATE_STYLES",
        path,
        styles: newNode.styles,
      });
    }

    // Diff children
    const oldChildren = oldNode.children || [];
    const newChildren = newNode.children || [];
    const maxLength = Math.max(oldChildren.length, newChildren.length);

    for (let i = 0; i < maxLength; i++) {
      const oldChild = oldChildren[i];
      const newChild = newChildren[i];
      const childPath = [...path, i];

      if (!oldChild && newChild) {
        // New child
        patches.push({
          type: "CREATE",
          path,
          node: newChild,
          index: i,
        });
      } else if (oldChild && !newChild) {
        // Removed child
        patches.push({ type: "REMOVE", path: childPath });
      } else if (oldChild && newChild) {
        // Recurse
        patches.push(...diff(oldChild, newChild, childPath));
      }
    }
  }

  return patches;
}

/**
 * Create a real DOM node from a VNode
 */
export function createElement(vnode: VNode): Node {
  if (vnode.type === "Text") {
    return document.createTextNode(vnode.content);
  }

  if (vnode.type === "Comment") {
    return document.createComment(vnode.content);
  }

  if (vnode.type === "Component") {
    // Component nodes should be handled by hybridPatchApplier
    // For now, create a placeholder
    const placeholder = document.createElement("div");
    placeholder.setAttribute("data-component-id", vnode.componentId);
    placeholder.textContent = `[Component: ${vnode.componentId}]`;
    return placeholder;
  }

  // Element
  const el = document.createElement(vnode.tag);

  // Set attributes
  for (const [key, value] of Object.entries(vnode.attributes)) {
    el.setAttribute(key, value);
  }

  // Set styles
  for (const [key, value] of Object.entries(vnode.styles)) {
    const styleKey = key.replace(/-([a-z])/g, (g) => g[1].toUpperCase());
    (el.style as any)[styleKey] = value;
  }

  // Add ID if present
  if (vnode.id) {
    el.setAttribute("data-vdom-id", vnode.id);
  }

  // Append children
  for (const child of vnode.children) {
    el.appendChild(createElement(child));
  }

  return el;
}

/**
 * PatchApplier interface - allows different patch application strategies
 */
export interface PatchApplier<T> {
  apply(patches: Patch[], target: T): T;
}

/**
 * Helper to walk a path and find a DOM node
 */
function walkPath(root: Node, path: number[]): Node | null {
  let current: Node | null = root;

  for (const index of path) {
    if (!current || !current.childNodes) {
      return null;
    }
    current = current.childNodes[index] || null;
  }

  return current;
}

/**
 * DOM patch applier factory
 * Returns an applier that applies patches to real DOM elements
 */
export function domPatchApplier(): PatchApplier<Element> {
  return {
    apply(patches: Patch[], element: Element): Element {
      for (const p of patches) {
        switch (p.type) {
          case "CREATE": {
            const parent = walkPath(element, p.path);
            if (!parent) {
              console.warn("CREATE: Parent not found at path", p.path);
              break;
            }

            const newElement = createElement(p.node);
            if (p.index < parent.childNodes.length) {
              parent.insertBefore(newElement, parent.childNodes[p.index]);
            } else {
              parent.appendChild(newElement);
            }
            break;
          }

          case "REMOVE": {
            const node = walkPath(element, p.path);
            if (!node) {
              console.warn("REMOVE: Node not found at path", p.path);
              break;
            }
            node.parentNode?.removeChild(node);
            break;
          }

          case "REPLACE": {
            const oldNode = walkPath(element, p.path);
            if (!oldNode) {
              console.warn("REPLACE: Node not found at path", p.path);
              break;
            }
            const newElement = createElement(p.newNode);
            oldNode.parentNode?.replaceChild(newElement, oldNode);
            break;
          }

          case "UPDATE_ATTRS": {
            const node = walkPath(element, p.path);
            if (!node || node.nodeType !== Node.ELEMENT_NODE) {
              console.warn("UPDATE_ATTRS: Element not found at path", p.path);
              break;
            }
            const el = node as Element;

            // Remove old attributes
            for (const attr of Array.from(el.attributes)) {
              if (!(attr.name in p.attributes)) {
                el.removeAttribute(attr.name);
              }
            }
            // Set new attributes
            for (const [key, value] of Object.entries(p.attributes)) {
              el.setAttribute(key, value);
            }
            break;
          }

          case "UPDATE_STYLES": {
            const node = walkPath(element, p.path);
            if (!node || node.nodeType !== Node.ELEMENT_NODE) {
              console.warn("UPDATE_STYLES: Element not found at path", p.path);
              break;
            }
            const htmlEl = node as HTMLElement;

            // Clear old styles
            htmlEl.removeAttribute("style");
            // Set new styles
            for (const [key, value] of Object.entries(p.styles)) {
              const styleKey = key.replace(/-([a-z])/g, (g) => g[1].toUpperCase());
              (htmlEl.style as any)[styleKey] = value;
            }
            break;
          }

          case "UPDATE_TEXT": {
            const node = walkPath(element, p.path);
            if (!node || node.nodeType !== Node.TEXT_NODE) {
              console.warn("UPDATE_TEXT: Text node not found at path", p.path);
              break;
            }
            node.textContent = p.content;
            break;
          }
        }
      }

      return element;
    }
  };
}

/**
 * Generic patch function - applies patches using a given applier
 * This is the main API: diff(old, new) → patches → patch(patches, target, applier)
 */
export function patch<T>(patches: Patch[], target: T, applier: PatchApplier<T>): T {
  return applier.apply(patches, target);
}
