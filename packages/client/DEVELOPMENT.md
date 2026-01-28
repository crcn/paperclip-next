# Development Guidelines

## Architecture Principles

### Avoid Global State for Web-Related Code

**Important:** Web-related code should avoid using globals. This refactor demonstrates the preferred approach.

#### Why Avoid Globals?

1. **Hard to test** - Globals create implicit dependencies
2. **Not composable** - Can't run multiple instances
3. **Side effects** - Hard to reason about data flow
4. **Coupling** - Creates tight coupling between modules
5. **Server-side issues** - Breaks SSR and concurrent rendering
6. **Not OT-compatible** - Operational transformations need pure functions

#### Bad: Global State Pattern

```typescript
// ❌ Don't do this
let currentElement: Element | null = null;

export function diff(oldNode: VNode, newNode: VNode) {
  // Uses global currentElement
  const patches = diffRecursive(oldNode, newNode, currentElement);
  return patches;
}

export function patch(patches: Patch[]) {
  // Mutates global currentElement
  applyToGlobal(patches);
}
```

**Problems:**
- `diff()` and `patch()` rely on hidden global state
- Can't diff multiple trees simultaneously
- Hard to test (must set up global state)
- Not serializable (patches contain DOM references)

#### Good: Pure Functions with Explicit Parameters

```typescript
// ✅ Do this instead
export function diff(
  oldNode: VNode,
  newNode: VNode,
  path: number[] = []
): Patch[] {
  // Pure function - no side effects
  return patches;
}

export function patch<T>(
  patches: Patch[],
  target: T,
  applier: PatchApplier<T>
): T {
  // Explicit dependencies
  return applier.apply(patches, target);
}
```

**Benefits:**
- All dependencies are explicit
- Pure functions are easy to test
- Composable - can run multiple instances
- Serializable - patches are pure data
- Thread-safe - no shared mutable state

#### Example: Composable API

```typescript
// Multiple independent diff/patch operations
const patches1 = diff(oldTree1, newTree1);
const patches2 = diff(oldTree2, newTree2);

// Can apply to different targets
patch(patches1, element1, domPatchApplier());
patch(patches2, element2, domPatchApplier());

// Can serialize and send over network
const json = JSON.stringify(patches1);
stream.send(json);
```

### Other Best Practices

#### Prefer Pure Functions

```typescript
// ✅ Pure - same input always gives same output
function add(a: number, b: number): number {
  return a + b;
}

// ❌ Impure - depends on external state
let count = 0;
function increment() {
  return ++count;
}
```

#### Make Dependencies Explicit

```typescript
// ✅ Dependencies as parameters
function render(
  vnode: VNode,
  target: Element,
  renderer: Renderer
): Element {
  return renderer.render(vnode, target);
}

// ❌ Hidden dependencies
let globalRenderer: Renderer;
function render(vnode: VNode, target: Element): Element {
  return globalRenderer.render(vnode, target);
}
```

#### Use Factory Functions for Configuration

```typescript
// ✅ Factory returns configured instance
function createRenderer(config: Config): Renderer {
  return {
    render(vnode, target) {
      // Uses config from closure
      return renderWithConfig(vnode, target, config);
    }
  };
}

// Usage
const renderer = createRenderer({ mode: 'fast' });
renderer.render(vnode, element);
```

#### Dependency Injection Over Singletons

```typescript
// ✅ Inject dependencies
class Component {
  constructor(
    private renderer: Renderer,
    private differ: Differ
  ) {}

  update(vnode: VNode) {
    const patches = this.differ.diff(this.old, vnode);
    this.renderer.apply(patches);
  }
}

// ❌ Singleton globals
class Component {
  update(vnode: VNode) {
    const patches = GlobalDiffer.diff(this.old, vnode);
    GlobalRenderer.apply(patches);
  }
}
```

## Testing

Pure functions with explicit dependencies are easy to test:

```typescript
test('diff generates correct patches', () => {
  const patches = diff(oldNode, newNode);

  expect(patches).toEqual([
    { type: 'UPDATE_TEXT', path: [0], content: 'new' }
  ]);

  // No DOM setup needed!
  // No global state to manage!
});

test('patch applies with custom applier', () => {
  const mockApplier = {
    apply: jest.fn()
  };

  patch(patches, target, mockApplier);

  expect(mockApplier.apply).toHaveBeenCalledWith(patches, target);
});
```

## References

- **OT_REFACTOR.md** - Details on the OT-style refactor
- **src/vdom.ts** - Example of pure, composable API
- **src/main.ts** - Usage examples

---

**Remember:** If you find yourself reaching for a global variable in web code, stop and consider:
1. Can I pass this as a parameter?
2. Can I use a factory function?
3. Can I use dependency injection?
4. Can I make this a pure function?

The answer is usually yes! 🎯
