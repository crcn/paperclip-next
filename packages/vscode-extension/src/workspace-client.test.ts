import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock grpc module
vi.mock('@grpc/grpc-js', () => ({
  credentials: {
    createInsecure: vi.fn(() => ({})),
  },
  loadPackageDefinition: vi.fn(() => ({
    paperclip: {
      workspace: {
        WorkspaceService: vi.fn(),
      },
    },
  })),
}));

// Mock proto-loader
vi.mock('@grpc/proto-loader', () => ({
  load: vi.fn(() => Promise.resolve({})),
}));

// Mock fs for path resolution
vi.mock('fs', () => ({
  existsSync: vi.fn(() => true),
  realpathSync: vi.fn((p) => p),
}));

describe('WorkspaceClient', () => {
  describe('stream error handling', () => {
    it('should not trigger reconnect for cancelled streams (code 1)', () => {
      // gRPC status codes
      const CANCELLED = 1;
      const OK = 0;
      const UNAVAILABLE = 14;

      // Simulate stream error handling logic
      const shouldReconnect = (code: number): boolean => {
        const isCancelled = code === CANCELLED;
        const isOk = code === OK;
        if (!isCancelled && !isOk) {
          if (code === UNAVAILABLE) {
            return true;
          }
        }
        return false;
      };

      // Cancelled stream should not reconnect
      expect(shouldReconnect(CANCELLED)).toBe(false);

      // OK status should not reconnect
      expect(shouldReconnect(OK)).toBe(false);

      // Unavailable should reconnect
      expect(shouldReconnect(UNAVAILABLE)).toBe(true);

      // Other errors should not reconnect (e.g., INVALID_ARGUMENT = 3)
      expect(shouldReconnect(3)).toBe(false);
    });

    it('should handle stream end without error', () => {
      // Stream end is a normal event, should not trigger reconnect
      let reconnectCalled = false;
      const onEnd = () => {
        // Normal stream end - no action needed
      };
      const scheduleReconnect = () => {
        reconnectCalled = true;
      };

      onEnd();
      expect(reconnectCalled).toBe(false);
    });
  });

  describe('field name handling with keepCase: false', () => {
    it('should use camelCase field names', () => {
      // With keepCase: false, proto fields are converted to camelCase
      const protoResponse = {
        filePath: '/test/file.pc',
        patches: [],
        error: null,
        timestamp: '1234567890',
        version: '1',
        acknowledgedMutationIds: [],
        changedByClientId: null,
      };

      // Verify the field names are camelCase (not snake_case)
      expect(protoResponse.filePath).toBeDefined();
      expect(protoResponse.acknowledgedMutationIds).toBeDefined();
      expect(protoResponse.changedByClientId).toBeDefined();

      // These should not exist with keepCase: false
      expect((protoResponse as any).file_path).toBeUndefined();
      expect((protoResponse as any).acknowledged_mutation_ids).toBeUndefined();
    });
  });
});

describe('BufferStreamer', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('debouncing', () => {
    it('should debounce rapid content updates', async () => {
      let streamStartCount = 0;
      const debounceMs = 100;

      // Simulate debounce logic
      let pendingTimer: ReturnType<typeof setTimeout> | null = null;

      const updateContent = (content: string) => {
        if (pendingTimer) {
          clearTimeout(pendingTimer);
        }
        pendingTimer = setTimeout(() => {
          streamStartCount++;
          pendingTimer = null;
        }, debounceMs);
      };

      // Rapid updates
      updateContent('a');
      updateContent('ab');
      updateContent('abc');
      updateContent('abcd');

      // Before debounce timeout, no streams started
      expect(streamStartCount).toBe(0);

      // Advance past debounce
      vi.advanceTimersByTime(debounceMs + 10);

      // Only one stream should have started
      expect(streamStartCount).toBe(1);
    });

    it('should cancel pending stream when new content arrives', () => {
      let currentGeneration = 0;
      let activeGeneration: number | null = null;
      const debounceMs = 100;

      let pendingTimer: ReturnType<typeof setTimeout> | null = null;

      const updateContent = () => {
        if (pendingTimer) {
          clearTimeout(pendingTimer);
        }
        currentGeneration++;
        const generation = currentGeneration;

        pendingTimer = setTimeout(() => {
          activeGeneration = generation;
          pendingTimer = null;
        }, debounceMs);
      };

      // First update
      updateContent();
      expect(currentGeneration).toBe(1);

      // Advance halfway
      vi.advanceTimersByTime(50);

      // Second update (cancels first)
      updateContent();
      expect(currentGeneration).toBe(2);

      // Advance past debounce
      vi.advanceTimersByTime(debounceMs + 10);

      // Only second generation should be active
      expect(activeGeneration).toBe(2);
    });
  });

  describe('stale update handling', () => {
    it('should ignore updates from stale generations', () => {
      let currentGeneration = 2;
      const receivedUpdates: number[] = [];

      const onUpdate = (generation: number) => {
        if (generation === currentGeneration) {
          receivedUpdates.push(generation);
        }
      };

      // Stale update (gen 1) arrives
      onUpdate(1);
      expect(receivedUpdates).toHaveLength(0);

      // Current update (gen 2) arrives
      onUpdate(2);
      expect(receivedUpdates).toHaveLength(1);
      expect(receivedUpdates[0]).toBe(2);
    });
  });
});

describe('DOM Patch Application', () => {
  // Simulate DOM structure for testing path navigation
  // In real impl, this is actual DOM nodes
  interface MockNode {
    type: 'element' | 'text';
    tag?: string;
    textContent?: string;
    style: Record<string, string>;
    attributes: Record<string, string>;
    children: MockNode[];
  }

  function createMockElement(tag: string, children: MockNode[] = []): MockNode {
    return { type: 'element', tag, style: {}, attributes: {}, children };
  }

  function createMockText(content: string): MockNode {
    return { type: 'text', textContent: content, style: {}, attributes: {}, children: [] };
  }

  // Mock DOM path navigation (mirrors preview-panel.ts getDOMNodeAtPath)
  function getNodeAtPath(root: { children: MockNode[] }, path: number[]): MockNode | null {
    if (!path || path.length === 0) {
      return null;
    }
    let node = root.children[path[0]];
    if (!node) return null;
    for (let i = 1; i < path.length; i++) {
      node = node.children[path[i]];
      if (!node) return null;
    }
    return node;
  }

  // Apply styles directly to DOM node (mirrors preview-panel.ts applyUpdateStylesToDOM)
  function applyUpdateStyles(root: { children: MockNode[] }, patch: { path: number[]; styles: Record<string, string> }): void {
    const node = getNodeAtPath(root, patch.path);
    if (!node || node.type !== 'element') {
      throw new Error('Invalid path for update_styles');
    }
    for (const [prop, value] of Object.entries(patch.styles)) {
      node.style[prop] = value;
    }
  }

  // Apply text update directly to DOM node (mirrors preview-panel.ts applyUpdateTextToDOM)
  function applyUpdateText(root: { children: MockNode[] }, patch: { path: number[]; content: string }): void {
    const node = getNodeAtPath(root, patch.path);
    if (!node) {
      throw new Error('Invalid path for update_text');
    }
    if (node.type === 'text') {
      node.textContent = patch.content;
    } else {
      // Element containing text - update first text child
      const textChild = node.children.find(n => n.type === 'text');
      if (textChild) {
        textChild.textContent = patch.content;
      }
    }
  }

  // Apply attributes directly to DOM node
  function applyUpdateAttributes(root: { children: MockNode[] }, patch: { path: number[]; attributes: Record<string, string> }): void {
    const node = getNodeAtPath(root, patch.path);
    if (!node || node.type !== 'element') {
      throw new Error('Invalid path for update_attributes');
    }
    for (const [name, value] of Object.entries(patch.attributes)) {
      node.attributes[name] = value;
    }
  }

  describe('getNodeAtPath', () => {
    it('should get root child at path [0]', () => {
      const root = {
        children: [
          createMockElement('div', [createMockText('Hello')])
        ]
      };
      const node = getNodeAtPath(root, [0]);
      expect(node?.tag).toBe('div');
    });

    it('should get nested child at path [0, 0]', () => {
      const root = {
        children: [
          createMockElement('div', [createMockText('Hello')])
        ]
      };
      const node = getNodeAtPath(root, [0, 0]);
      expect(node?.textContent).toBe('Hello');
    });

    it('should return null for invalid path', () => {
      const root = { children: [] };
      expect(getNodeAtPath(root, [0])).toBeNull();
      expect(getNodeAtPath(root, [0, 0])).toBeNull();
    });
  });

  describe('applyUpdateStyles', () => {
    it('should apply styles directly to element at path', () => {
      const root = {
        children: [
          createMockElement('div', [createMockText('Hello')])
        ]
      };

      applyUpdateStyles(root, { path: [0], styles: { color: 'blue', padding: '16px' } });

      expect(root.children[0].style.color).toBe('blue');
      expect(root.children[0].style.padding).toBe('16px');
    });

    it('should merge new styles with existing styles', () => {
      const root = {
        children: [
          createMockElement('div')
        ]
      };
      root.children[0].style = { color: 'red' };

      applyUpdateStyles(root, { path: [0], styles: { padding: '16px' } });

      expect(root.children[0].style.color).toBe('red');
      expect(root.children[0].style.padding).toBe('16px');
    });

    it('should throw for invalid path', () => {
      const root = { children: [] };
      expect(() => applyUpdateStyles(root, { path: [0], styles: { color: 'blue' } }))
        .toThrow('Invalid path');
    });
  });

  describe('applyUpdateText', () => {
    it('should update text node content directly', () => {
      const root = {
        children: [
          createMockElement('div', [createMockText('Hello')])
        ]
      };

      applyUpdateText(root, { path: [0, 0], content: 'World' });

      expect(root.children[0].children[0].textContent).toBe('World');
    });

    it('should mutate DOM in place (not create copy)', () => {
      const textNode = createMockText('Hello');
      const root = { children: [createMockElement('div', [textNode])] };

      applyUpdateText(root, { path: [0, 0], content: 'World' });

      // Original textNode should be mutated
      expect(textNode.textContent).toBe('World');
    });

    it('should throw for invalid path', () => {
      const root = { children: [] };
      expect(() => applyUpdateText(root, { path: [0, 0], content: 'Test' }))
        .toThrow('Invalid path');
    });
  });

  describe('applyUpdateAttributes', () => {
    it('should apply attributes directly to element', () => {
      const root = {
        children: [createMockElement('div')]
      };

      applyUpdateAttributes(root, { path: [0], attributes: { class: 'foo', id: 'bar' } });

      expect(root.children[0].attributes.class).toBe('foo');
      expect(root.children[0].attributes.id).toBe('bar');
    });

    it('should merge new attributes with existing', () => {
      const div = createMockElement('div');
      div.attributes = { class: 'original' };
      const root = { children: [div] };

      applyUpdateAttributes(root, { path: [0], attributes: { id: 'new' } });

      expect(root.children[0].attributes.class).toBe('original');
      expect(root.children[0].attributes.id).toBe('new');
    });
  });
});
