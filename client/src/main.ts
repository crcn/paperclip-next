import { VDocument, VNode, diff, patch, createElement } from "./vdom";

/**
 * Simple demo of the Virtual DOM differ/patcher
 */

// Example Virtual DOM documents (simulating server responses)
const vdoc1: VDocument = {
  nodes: [
    {
      type: "Element",
      tag: "button",
      attributes: {},
      styles: {
        padding: "8px 16px",
        background: "#3366FF",
        color: "white",
        border: "none",
        "border-radius": "4px",
      },
      children: [
        {
          type: "Text",
          content: "Click me",
        },
      ],
    },
  ],
  styles: [],
};

const vdoc2: VDocument = {
  nodes: [
    {
      type: "Element",
      tag: "button",
      attributes: {},
      styles: {
        padding: "12px 24px",
        background: "#FF3366",
        color: "white",
        border: "none",
        "border-radius": "8px",
      },
      children: [
        {
          type: "Text",
          content: "Click me now!",
        },
      ],
    },
  ],
  styles: [],
};

const vdoc3: VDocument = {
  nodes: [
    {
      type: "Element",
      tag: "div",
      attributes: {},
      styles: {
        padding: "16px",
        background: "#f0f0f0",
        "border-radius": "8px",
      },
      children: [
        {
          type: "Element",
          tag: "h1",
          attributes: {},
          styles: {
            margin: "0 0 16px 0",
            color: "#333",
          },
          children: [
            {
              type: "Text",
              content: "Hello Paperclip!",
            },
          ],
        },
        {
          type: "Element",
          tag: "button",
          attributes: {},
          styles: {
            padding: "8px 16px",
            background: "#3366FF",
            color: "white",
            border: "none",
            "border-radius": "4px",
          },
          children: [
            {
              type: "Text",
              content: "Click me",
            },
          ],
        },
      ],
    },
  ],
  styles: [],
};

// State
let currentVDoc: VDocument | null = null;
let currentRoot: Element | null = null;
let currentIndex = 0;
const vdocs = [vdoc1, vdoc2, vdoc3];

// Mount initial Virtual DOM
function mount(container: Element, vdoc: VDocument) {
  currentVDoc = vdoc;
  currentRoot = container;

  // Clear container
  container.innerHTML = "";

  // Create and append elements
  for (const node of vdoc.nodes) {
    const element = createElement(node);
    container.appendChild(element);
  }
}

// Update to new Virtual DOM
function update(vdoc: VDocument) {
  if (!currentVDoc || !currentRoot) {
    return;
  }

  // Diff each root node
  const patches = [];
  const maxLength = Math.max(currentVDoc.nodes.length, vdoc.nodes.length);

  for (let i = 0; i < maxLength; i++) {
    const oldNode = currentVDoc.nodes[i] || null;
    const newNode = vdoc.nodes[i] || null;
    const element = currentRoot.childNodes[i] || null;

    patches.push(...diff(oldNode, newNode, element));
  }

  // Apply patches
  patch(patches);

  currentVDoc = vdoc;
}

// Demo controls
function initDemo() {
  const container = document.getElementById("preview");
  const btnNext = document.getElementById("btn-next");
  const btnPrev = document.getElementById("btn-prev");
  const status = document.getElementById("status");

  if (!container || !btnNext || !btnPrev || !status) {
    console.error("Required DOM elements not found");
    return;
  }

  // Mount initial
  mount(container, vdocs[currentIndex]);
  status.textContent = `Document ${currentIndex + 1}/${vdocs.length}`;

  // Next button
  btnNext.addEventListener("click", () => {
    currentIndex = (currentIndex + 1) % vdocs.length;
    update(vdocs[currentIndex]);
    status.textContent = `Document ${currentIndex + 1}/${vdocs.length}`;
  });

  // Previous button
  btnPrev.addEventListener("click", () => {
    currentIndex = (currentIndex - 1 + vdocs.length) % vdocs.length;
    update(vdocs[currentIndex]);
    status.textContent = `Document ${currentIndex + 1}/${vdocs.length}`;
  });

  console.log("Demo initialized");
  console.log("Virtual DOM differ/patcher ready");
}

// Initialize when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initDemo);
} else {
  initDemo();
}
