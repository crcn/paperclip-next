# Component Recursion Behavior in Paperclip

## Summary

When you render `component AB { render div { AB() } }`, it causes **infinite recursion** and **stack overflow**.

## Detailed Behavior

### Syntax Matters

There are two ways to reference "AB" inside a component:

#### 1. Bare Identifier: `AB` → Slot Insert
```javascript
component AB {
    render div {
        AB  // ← Bare identifier becomes SlotInsert { name: "AB" }
    }
}
```

**Result**: Evaluator looks for a slot named "AB" in component AB, doesn't find it, and produces an error node:
```
Error: "Slot 'AB' not found in component 'AB'"
```

**No recursion occurs** because it's looking for a slot, not trying to render the component.

#### 2. Instance Syntax: `AB()` or `AB { }` → Component Instance
```javascript
component AB {
    render div {
        AB()  // ← With parens/braces becomes Instance { name: "AB" }
    }
}
```

**Result**: **Stack overflow** due to infinite recursion.

### Why Stack Overflow Happens

Evaluation flow:
1. `evaluator.evaluate_component("AB")`
2. Evaluates component body: `div { AB() }`
3. Encounters `AB()` instance
4. Calls `evaluate_component_with_props_and_children("AB", ...)`
5. Creates new scoped evaluator (clones context)
6. Evaluates body again: `div { AB() }`
7. **Go to step 3** → Infinite loop

No recursion depth limit or component stack tracking exists in the evaluator.

### Actual Error Output

```
thread 'test_direct_recursion' has overflowed its stack
fatal runtime error: stack overflow, aborting
process didn't exit successfully (signal: 6, SIGABRT)
```

## Indirect Recursion

Indirect recursion also causes stack overflow:

```javascript
component A {
    render div {
        B()
    }
}

component B {
    render div {
        A()
    }
}

public component App {
    render A
}
```

**Result**: Same stack overflow. No cycle detection.

## Conditional Recursion

Even with a conditional, without variable updates, it still recurses infinitely:

```javascript
component Countdown {
    render div {
        text count
        if count > 0 {
            Countdown()
        }
    }
}
```

If `count` is set to `3` and never decremented, this will:
1. Render "3"
2. Check `count > 0` → true
3. Render `Countdown()` again
4. Render "3" again (same `count` value)
5. Check `count > 0` → true
6. **Go to step 3** → Infinite loop

**Why?** The scoped evaluator clones the context, so `count` stays at 3 forever. There's no mechanism to decrement it in the component.

## Current Protection: None

**No recursion detection exists**:
- ✗ No recursion depth limit
- ✗ No component call stack tracking
- ✗ No cycle detection
- ✗ No warning or error before overflow

## Potential Solutions

### 1. Recursion Depth Limit (Simple)

Add max depth counter to context:

```rust
pub struct EvalContext {
    // ... existing fields
    recursion_depth: usize,
    max_recursion_depth: usize,  // e.g., 100
}

impl Evaluator {
    fn evaluate_component_with_props_and_children(...) -> EvalResult<VNode> {
        // Check depth before evaluating
        if self.context.recursion_depth >= self.context.max_recursion_depth {
            return Err(EvalError::RecursionLimitExceeded {
                component_name: name.to_string(),
                depth: self.context.recursion_depth,
            });
        }

        // Increment depth
        self.context.recursion_depth += 1;

        // ... evaluate component

        // Decrement depth after
        self.context.recursion_depth -= 1;

        result
    }
}
```

**Pros**: Simple, catches all recursion
**Cons**: Legitimate deep nesting might hit limit

### 2. Component Stack Tracking (Better)

Track component instances being evaluated:

```rust
pub struct EvalContext {
    // ... existing fields
    component_stack: Vec<String>,  // Stack of component names being evaluated
}

impl Evaluator {
    fn evaluate_component_with_props_and_children(...) -> EvalResult<VNode> {
        // Check if component is already in stack
        if self.context.component_stack.contains(&name.to_string()) {
            return Err(EvalError::CircularDependency {
                component_name: name.to_string(),
                stack: self.context.component_stack.clone(),
            });
        }

        // Push to stack
        self.context.component_stack.push(name.to_string());

        // ... evaluate component

        // Pop from stack
        self.context.component_stack.pop();

        result
    }
}
```

**Pros**: Detects actual cycles, allows deep nesting
**Cons**: Slightly more complex

### 3. Cycle Detection with Path (Most Robust)

Track full semantic path to detect cycles:

```rust
impl Evaluator {
    fn evaluate_component_with_props_and_children(...) -> EvalResult<VNode> {
        // Build semantic ID for this component instance
        let semantic_id = self.context.get_semantic_id();

        // Check if this exact instance is already being evaluated
        if self.context.is_evaluating(&semantic_id) {
            return Err(EvalError::CircularReference {
                semantic_id,
                component_name: name.to_string(),
            });
        }

        // ... evaluate
    }
}
```

**Pros**: Most accurate, allows same component in different branches
**Cons**: Most complex

## Recommendation

**Implement Solution #2: Component Stack Tracking**

Reasons:
1. Catches all circular dependencies
2. Provides clear error message with call stack
3. Simple to implement
4. Allows legitimate deep nesting
5. Aligns with common patterns (React has similar detection)

Error message would be:
```
Error: Circular component dependency detected
Component: AB
Call stack: AB → AB
```

Or for indirect:
```
Error: Circular component dependency detected
Component: A
Call stack: A → B → A
```

## Comparison with React

React has similar protection:
- **React**: "Maximum update depth exceeded" after ~50 renders
- **Vue**: "Maximum recursive updates exceeded"
- **Svelte**: Stack overflow (no built-in protection)

Paperclip should have protection to provide better developer experience.

## Valid Use Cases for Recursion

Some recursion patterns are valid (with proper termination):

### Tree Rendering (Valid with props)
```javascript
component TreeNode {
    render div {
        text node.label
        if node.children {
            repeat child in node.children {
                TreeNode(node=child)  // ✅ Valid - different data each time
            }
        }
    }
}
```

**Why this works**: Each recursive call has different `node` prop, eventually reaching leaf nodes.

### Menu with Submenus (Valid with props)
```javascript
component Menu {
    render ul {
        repeat item in items {
            li {
                text item.label
                if item.submenu {
                    Menu(items=item.submenu)  // ✅ Valid - different items
                }
            }
        }
    }
}
```

**Why this works**: Recursion terminates when `item.submenu` is empty/null.

## Testing

Test file created: `packages/evaluator/tests/test_recursion.rs`

Tests:
- ✅ Direct recursion detection
- ✅ Indirect recursion (A → B → A)
- ✅ Conditional recursion without updates

All currently cause stack overflow as expected.

## Action Items

1. **Add component stack tracking to `EvalContext`**
2. **Check for cycles before evaluating component**
3. **Return `EvalError::CircularDependency` with stack trace**
4. **Add tests for recursion error handling**
5. **Update documentation with recursion guidelines**

## Current Status

**⚠️ UNPROTECTED**: Recursive components cause stack overflow with no helpful error message.

Developer experience: Poor (crashes with cryptic "stack overflow" from runtime)
