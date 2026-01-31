# Paperclip Feature Comparison: Old vs New

Comparison between the original Paperclip repo (`~/Developer/crcn/paperclip`) and the new implementation (`paperclip-next`).

## ✅ Fully Implemented Features

| Feature | Old Syntax | New Status | Notes |
|---------|-----------|------------|-------|
| **Tokens** | `public token color #333` | ✅ Implemented | Design tokens for reusable values |
| **Style Mixins** | `public style name { ... }` | ✅ Implemented | Reusable CSS blocks |
| **Style Extends** | `style extends font1, font2` | ✅ Implemented | Extend multiple styles |
| **Components** | `public component Name { ... }` | ✅ Implemented | Reusable UI components |
| **Variants** | `variant hover` | ✅ Implemented | State-based styling variations |
| **Variant Triggers** | `variant hover trigger { ":hover" }` | ✅ Implemented | CSS selectors/media queries |
| **Style Variants** | `style variant hover { ... }` | ✅ Implemented | Conditional styles per variant |
| **Slots** | `slot children { ... }` | ✅ Implemented | Content insertion points |
| **Slot Defaults** | Default content in slot body | ✅ Implemented | Fallback content for slots |
| **Slot Insert** | `header` or `slot header` | ✅ Implemented | Insert content into slots |
| **Import** | `import "./file.pc" as name` | ✅ Implemented | Import other PC files |
| **Render** | `render div { ... }` | ✅ Implemented | Define component render tree |
| **Text Nodes** | `text "Hello"` | ✅ Implemented | Text content |
| **Elements** | `div, span, button, etc.` | ✅ Implemented | HTML elements |
| **Attributes** | `(class: x, onClick: y)` | ✅ Implemented | Element attributes/props |
| **Inline Styles** | `style { color: red }` | ✅ Implemented | Element-level styles |
| **Expressions** | `{variable}`, `{obj.prop}` | ✅ Implemented | Variable references, member access |
| **Conditionals** | `if condition { ... }` | ✅ Implemented | Conditional rendering |
| **Iteration** | `repeat item in items { ... }` | ✅ Implemented | List rendering |

## ❌ NOT Yet Implemented

| Feature | Old Syntax | Impact | Notes |
|---------|-----------|--------|-------|
| **Combination Variants** | `style variant a + b + c { ... }` | 🔴 High | Multiple variants together |
| **Global Triggers** | `trigger mobile { "@media ..." }` | 🟡 Medium | Reusable trigger definitions |
| **Script Directive** | `script(src: "...", target: "react")` | 🟡 Medium | Bind to external components |
| **Element Names** | `div myElement (...)` | 🟢 Low | Designer-friendly names |
| **@frame Annotations** | `@frame(x: 100, y: 200, ...)` | 🟢 Low | Designer positioning |
| **Insert Syntax** | `insert slotName { ... }` | 🟡 Medium | Explicit slot insertion |
| **Binary Operations** | `{count + 1}`, `{x * y}` | 🔴 High | Math/comparison in expressions |
| **Function Calls** | `{formatDate(date)}` | 🟡 Medium | Function expressions |
| **Template Strings** | String interpolation | 🟡 Medium | `"Hello {name}"` |

## 🔍 Feature Details

### ✅ What Works Now

#### 1. Basic Component Definition
```javascript
public component Button {
    variant hover trigger {
        ":hover"
    }
    slot children
    render button {
        style {
            background: blue
        }
        style variant hover {
            background: darkblue
        }
        children
    }
}
```

#### 2. Tokens & Styles
```javascript
public token primaryColor #3366FF
public token fontSize 16px

public style defaultFont {
    font-family: Inter
    font-size: var(fontSize)
}

public style button extends defaultFont {
    padding: 8px 16px
}
```

#### 3. Slots with Defaults
```javascript
public component Card {
    slot header {
        text "Default Header"
    }
    render div {
        header
    }
}
```

#### 4. Conditionals & Iteration
```javascript
public component List {
    render div {
        if showHeader {
            text "Header"
        }
        repeat item in items {
            div {
                text {item.name}
            }
        }
    }
}
```

### ❌ Missing Features Deep Dive

#### 1. Combination Variants (HIGH Priority) 🔴

**Old Syntax:**
```javascript
public component Button {
    variant v2 trigger { ".v2" }
    variant hover trigger { ":hover" }

    render button {
        style variant hover {
            background: lightblue
        }
        style variant v2 {
            border-radius: 8px
        }
        // Multiple variants together
        style variant v2 + hover {
            background: darkblue
        }
    }
}
```

**Impact:** Can't style combinations of states (e.g., "v2 version when hovered")

**Parser Change Needed:**
- Update `StyleBlock` to support `variant_combination: Vec<String>` instead of `variant: Option<String>`
- Parser needs to handle `+` operator in variant expressions
- CSS generation needs to handle combined selectors

---

#### 2. Global Trigger Definitions (MEDIUM Priority) 🟡

**Old Syntax:**
```javascript
// Define reusable triggers at document level
trigger mobile {
    "@media screen and (max-width: 480px)"
}

trigger tablet {
    "@media screen and (max-width: 768px)"
}

public component Page {
    variant mobile trigger {
        mobile  // Reference the global trigger
    }
}
```

**Impact:** Can't create reusable breakpoint definitions

**Parser Change Needed:**
- Add `TriggerDecl` to Document AST
- Allow variants to reference global triggers by name
- Resolve trigger references during compilation

---

#### 3. Script Directive (MEDIUM Priority) 🟡

**Old Syntax:**
```javascript
public component DataTable {
    script(src: "./DataTable.tsx", target: "react", name: "DataTable")

    render div {
        // Visual structure only, logic in React component
    }
}
```

**Impact:** Can't bind visual components to external logic/framework code

**Parser Change Needed:**
- Add `script: Option<ScriptDirective>` to `Component`
- Parse script parameters (src, target, name)
- Compiler needs to generate imports/bindings

---

#### 4. Element Names (LOW Priority) 🟢

**Old Syntax:**
```javascript
render div mainContainer (class: "flex") {
    div sidebar {
        text "Sidebar"
    }
    div content {
        text "Main content"
    }
}
```

**Impact:** Designer shows generic names instead of semantic ones

**Parser Change Needed:**
- Add `name: Option<String>` to `Element::Tag`
- Parse element name before attributes
- Designer/tooling can use these names

---

#### 5. Insert Syntax (MEDIUM Priority) 🟡

**Old Syntax:**
```javascript
MyComponent {
    insert header {
        text "Custom Header"
    }
    insert footer {
        text "Custom Footer"
    }
}
```

**Current Workaround:**
```javascript
MyComponent {
    // Just use the slot name directly
    header {
        text "Custom Header"
    }
}
```

**Impact:** Less explicit, but functionally equivalent

**Parser Change Needed:**
- Add `insert` keyword support in component instance children
- May be syntactic sugar over current approach

---

#### 6. Binary Operations & Function Calls (HIGH Priority) 🔴

**Old Syntax:**
```javascript
render div {
    text "Count: {count + 1}"
    text "Price: {price * quantity}"
    text "Name: {formatName(user.firstName, user.lastName)}"
}
```

**Current Limitation:** Parser only supports simple expressions

**Impact:**
- Can't do math in templates
- Type inference for binary ops is implemented but unused
- Limits expressiveness significantly

**Parser Change Needed:**
- Add binary operator parsing to expression parser
- Add function call parsing
- This is a MAJOR parser enhancement

---

## Priority Roadmap

### Phase 1: Core Expression Support (Critical)
1. **Binary Operations** - Math, comparisons, string concat
2. **Function Calls** - Enable calling functions in expressions
3. **Template Strings** - String interpolation support

**Why:** These are fundamental expression capabilities that unlock the type inference system we just built.

### Phase 2: Advanced Styling (Important)
1. **Combination Variants** - `style variant a + b { ... }`
2. **Global Triggers** - Reusable breakpoint/trigger definitions

**Why:** Critical for real-world design systems with complex state combinations.

### Phase 3: Integration (Nice to Have)
1. **Script Directive** - Bind to external framework components
2. **Element Names** - Better designer experience
3. **Insert Syntax** - More explicit slot filling

**Why:** Improves integration and tooling but doesn't block core functionality.

## What This Means

### ✅ Current Capabilities (Excellent!)
- **Core component model:** Fully functional
- **Styling system:** Tokens, mixins, extends, inline styles all work
- **Slots & variants:** Complete implementation
- **Basic expressions:** Variables and member access work
- **Control flow:** Conditionals and iteration work
- **Type inference:** Sophisticated multi-pass engine (ready for binary ops)

### ❌ Gaps vs Original (Need Attention)
1. **Expression richness:** Can't do math or call functions in templates
2. **Variant combinations:** Can't style multiple variant states together
3. **Reusability:** No global trigger definitions
4. **Integration:** No script directive for framework bindings

### 🎯 Recommendation

**FOCUS ON PHASE 1 IMMEDIATELY:**
The parser needs binary operations and function calls to reach feature parity with the old repo. The inference engine you just built is ready to handle these - it just needs the parser to support them.

**Quick Win:** Implementing binary ops would immediately unlock better type inference (the `test_infer_binary_op` test you marked as ignored would pass).

## Testing Current Features

```bash
# What works now (should all pass):
cargo test -p paperclip-parser      # 112 tests
cargo test -p paperclip-evaluator   # 39 tests
cargo test -p paperclip-inference   # 32 tests
cargo test -p paperclip-compiler-react  # 13 tests

# What doesn't work yet:
# - Combination variants (parser doesn't support `+` in variants)
# - Binary operations (parser doesn't support operators in expressions)
# - Global triggers (not in AST)
# - Script directive (not in AST)
```

## Summary

You have ~80% feature parity with the original Paperclip. The architecture is solid, the type inference is sophisticated, but **expression parsing needs enhancement** to reach full parity. The good news: your inference engine is already built and ready for these features!
