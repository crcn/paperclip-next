/**
 * Virtual DOM types matching the Rust evaluator output
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
 * Diff two VNodes and return a list of patches
 */
export type Patch =
  | { type: "CREATE"; node: VNode; parent: Node; index: number }
  | { type: "REMOVE"; element: Node }
  | { type: "REPLACE"; oldElement: Node; newNode: VNode }
  | { type: "UPDATE_ATTRS"; element: Element; attributes: Record<string, string> }
  | { type: "UPDATE_STYLES"; element: Element; styles: Record<string, string> }
  | { type: "UPDATE_TEXT"; element: Text; content: string };

export function diff(
  oldNode: VNode | null,
  newNode: VNode | null,
  element: Node | null
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
  if (oldNode && !newNode && element) {
    patches.push({ type: "REMOVE", element });
    return patches;
  }

  if (!oldNode || !newNode || !element) {
    return patches;
  }

  // Different node types - replace
  if (oldNode.type !== newNode.type) {
    patches.push({ type: "REPLACE", oldElement: element, newNode });
    return patches;
  }

  // Text nodes
  if (oldNode.type === "Text" && newNode.type === "Text") {
    if (oldNode.content !== newNode.content) {
      patches.push({
        type: "UPDATE_TEXT",
        element: element as Text,
        content: newNode.content,
      });
    }
    return patches;
  }

  // Element nodes
  if (oldNode.type === "Element" && newNode.type === "Element") {
    const el = element as Element;

    // Different tags - replace
    if (oldNode.tag !== newNode.tag) {
      patches.push({ type: "REPLACE", oldElement: element, newNode });
      return patches;
    }

    // Update attributes
    const attrsChanged =
      JSON.stringify(oldNode.attributes) !== JSON.stringify(newNode.attributes);
    if (attrsChanged) {
      patches.push({
        type: "UPDATE_ATTRS",
        element: el,
        attributes: newNode.attributes,
      });
    }

    // Update styles
    const stylesChanged =
      JSON.stringify(oldNode.styles) !== JSON.stringify(newNode.styles);
    if (stylesChanged) {
      patches.push({
        type: "UPDATE_STYLES",
        element: el,
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
      const childElement = el.childNodes[i] || null;

      if (!oldChild && newChild) {
        // New child
        patches.push({
          type: "CREATE",
          node: newChild,
          parent: el,
          index: i,
        });
      } else if (oldChild && !newChild && childElement) {
        // Removed child
        patches.push({ type: "REMOVE", element: childElement });
      } else if (oldChild && newChild && childElement) {
        // Recurse
        patches.push(...diff(oldChild, newChild, childElement));
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
 * Apply a list of patches to the DOM
 */
export function patch(patches: Patch[]): void {
  for (const p of patches) {
    switch (p.type) {
      case "CREATE": {
        const newElement = createElement(p.node);
        if (p.index < p.parent.childNodes.length) {
          p.parent.insertBefore(newElement, p.parent.childNodes[p.index]);
        } else {
          p.parent.appendChild(newElement);
        }
        break;
      }

      case "REMOVE": {
        p.element.parentNode?.removeChild(p.element);
        break;
      }

      case "REPLACE": {
        const newElement = createElement(p.newNode);
        p.oldElement.parentNode?.replaceChild(newElement, p.oldElement);
        break;
      }

      case "UPDATE_ATTRS": {
        // Remove old attributes
        for (const attr of Array.from(p.element.attributes)) {
          if (!(attr.name in p.attributes)) {
            p.element.removeAttribute(attr.name);
          }
        }
        // Set new attributes
        for (const [key, value] of Object.entries(p.attributes)) {
          p.element.setAttribute(key, value);
        }
        break;
      }

      case "UPDATE_STYLES": {
        const htmlEl = p.element as HTMLElement;
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
        p.element.textContent = p.content;
        break;
      }
    }
  }
}
