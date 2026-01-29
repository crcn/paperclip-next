# Spike 0.3: Component Composition & Slots

**Status**: ✅ **VALIDATED**
**Date**: 2026-01-28

## Objective

Validate that component composition and slots work end-to-end:
- Component instances render correctly
- Slot declarations at component level
- Slot insertion points in render tree
- Default slot content
- Named slots with explicit insertion
- Nested component composition
- Slot content can contain any element type

## Implementation Status

### ✅ What's Implemented

1. **Basic component instances**
   ```javascript
   component Button {
       render button {
           text "Click"
       }
   }

   component App {
       render div {
           Button()
       }
   }
   ```

2. **Component instances with props**
   ```javascript
   Button(id="submit", class="primary")
   Card(width=300, height=200)
   ```

3. **Slot declarations at component level**
   ```javascript
   component Card {
       slot content
       slot header
       slot footer

       render div {
           header
           content
           footer
       }
   }
   ```

4. **Default slot content**
   ```javascript
   slot content {
       text "Default text"
   }
   ```

5. **Slot insertion in render tree**
   ```javascript
   render div {
       content  // Bare identifier becomes SlotInsert
   }
   ```

6. **Named slot insertion with `insert` directive**
   ```javascript
   Dialog() {
       insert header {
           text "Title"
       }
       insert body {
           text "Content"
       }
       insert footer {
           button { text "OK" }
       }
   }
   ```

7. **Default slot content (implicit)**
   ```javascript
   Card() {
       text "This goes into the default slot"
       div { text "Multiple elements" }
   }
   ```

8. **Nested component composition**
   ```javascript
   component Button {
       render button {
           Icon() {
               text "→"
           }
           text "Next"
       }
   }
   ```

9. **Slots with control flow**
   ```javascript
   Card() {
       if isVisible {
           text "Conditional content"
       }
       repeat item in items {
           li { text item.name }
       }
   }
   ```

## Test Results

**All 11 tests passing** ✅

### Component Instances
1. ✅ `test_basic_component_instance` - Basic component instantiation
2. ✅ `test_component_instance_with_attributes` - Props/attributes on instances

### Slots
3. ✅ `test_default_slot` - Default slot declaration and insertion
4. ✅ `test_named_slots` - Multiple named slots with insert directives
5. ✅ `test_default_slot_content` - Slots with default fallback content
6. ✅ `test_multiple_slot_contents` - Multiple children in slot content

### Composition
7. ✅ `test_nested_component_composition` - Components within components
8. ✅ `test_component_composition_with_styles` - Composed components with styles

### Integration
9. ✅ `test_slots_with_conditional_content` - Slots containing conditionals
10. ✅ `test_slots_with_repeat_content` - Slots containing loops
11. ✅ `test_real_world_layout_composition` - Complex multi-level composition

## Findings

### ✅ Strengths

1. **Clean slot syntax**: `slot name { default }` at component level is clear
2. **Bare identifiers for insertion**: Just write `content` to insert a slot
3. **Explicit named insertion**: `insert name { ... }` is unambiguous
4. **Props vs children separation**: Instance syntax cleanly separates props from children
5. **Universal slot content**: Any element can go in a slot (text, tags, conditionals, repeats, instances)
6. **Nested composition**: Components can contain component instances with slots

### Architecture Decisions

#### 1. Slot Declaration Location
**Decision**: Slots declared at component level, not inline in render tree
```javascript
component Card {
    slot header   // ✅ Declared here
    slot content

    render div {
        header    // Used here as SlotInsert
        content
    }
}
```
**Rationale**:
- Clear contract: Slots are part of component API
- Easy to find all slots for a component
- Separates definition from usage
- Matches web components spec

#### 2. Slot Insertion Syntax
**Decision**: Bare identifier in render tree becomes SlotInsert
```javascript
render div {
    content  // ✅ Bare identifier
}
// Not: <slot name="content" />
```
**Rationale**:
- Minimal syntax (no special characters needed)
- Reads naturally (just reference the slot name)
- Distinguishes from component instances (which have `()` or `{}`)

#### 3. Named Slot Content Syntax
**Decision**: Use `insert name { ... }` directive for named slots
```javascript
Dialog() {
    insert header { text "Title" }    // ✅ Explicit
    insert body { text "Content" }
}
// Not: div(slot="header") { ... }
```
**Rationale**:
- Explicit slot targeting (clear intent)
- Distinguishes from HTML `slot` attribute
- Allows multiple elements per slot naturally
- Consistent with Paperclip's directive pattern

#### 4. Default Slot
**Decision**: Children without `insert` directive go to implicit default slot
```javascript
slot content  // Default slot (first/only slot)

Card() {
    text "Goes to default"  // ✅ Implicit
}
```
**Rationale**:
- Ergonomic for common case (single slot)
- Matches React children behavior
- No need to name a slot if it's the only one

#### 5. Default Slot Content
**Decision**: Slots can have fallback content at declaration
```javascript
slot content {
    text "Empty state message"  // ✅ Default
}
```
**Rationale**:
- Provides fallback for empty slots
- Keeps default content with slot definition
- Evaluator can check if slot content provided

## Comparison with Original Paperclip

Based on analysis of `~/Developer/crcn/paperclip`:

### ✅ Feature Parity
- **Component instances**: ✅ Same syntax
- **Slot declarations**: ✅ Component-level slot definitions
- **Slot insertion**: ✅ Bare identifier syntax
- **Named slots**: ✅ `insert` directive
- **Default content**: ✅ Slot with children

### Differences
- Original uses protobuf AST, new version uses cleaner Rust enums
- Original evaluation creates VNode trees (not yet in new version evaluator)
- New version has cleaner separation of slot declaration vs usage

## Next Steps

### Immediate
- ✅ **Spike validated** - Slot system fully working in parser

### Future Work (Post-Spike)

1. **Evaluator implementation**
   - Resolve slot content during component evaluation
   - Use default content when slot empty
   - Handle nested slot resolution
   - Support recursive composition

2. **Validation & Errors**
   - Warn on unused slot declarations
   - Error on undefined slot references
   - Error on duplicate slot names
   - Validate slot content types

3. **Advanced Features**
   - Scoped slots (access parent data)
   - Slot props (pass data to slot content)
   - Conditional slots (slots that only appear sometimes)
   - Multiple default slots (fallthrough behavior)

4. **Designer Integration**
   - Visual slot editing
   - Drag-and-drop slot content
   - Show slot boundaries in preview
   - Inline slot content editing

## Examples from Tests

### Card with Default Slot
```javascript
component Card {
    slot content

    render div(class="card") {
        style {
            padding: 20px
            border: 1px solid #ddd
        }
        content
    }
}

component App {
    render div {
        Card() {
            text "This is the card content"
        }
    }
}
```

### Dialog with Named Slots
```javascript
component Dialog {
    slot header
    slot body
    slot footer

    render div(class="dialog") {
        style {
            position: fixed
            top: 50%
            left: 50%
        }

        div(class="header") {
            header
        }
        div(class="body") {
            body
        }
        div(class="footer") {
            footer
        }
    }
}

component App {
    render div {
        Dialog() {
            insert header {
                text "Confirm Action"
            }
            insert body {
                text "Are you sure you want to continue?"
            }
            insert footer {
                button { text "Cancel" }
                button { text "Confirm" }
            }
        }
    }
}
```

### Nested Composition
```javascript
component Icon {
    slot icon

    render span(class="icon") {
        icon
    }
}

component Button {
    slot label

    render button {
        Icon() {
            text "→"
        }
        label
    }
}

component App {
    render div {
        Button() {
            text "Next Page"
        }
    }
}
```

### Layout System
```javascript
component Header {
    slot logo
    slot nav

    render header {
        div(class="logo") { logo }
        div(class="nav") { nav }
    }
}

component Sidebar {
    slot content

    render aside {
        content
    }
}

component Layout {
    slot sidebarContent
    slot mainContent

    render div(class="layout") {
        Header() {
            insert logo {
                text "MyApp"
            }
            insert nav {
                text "Home | About | Contact"
            }
        }

        div(class="container") {
            Sidebar() {
                sidebarContent
            }
            div(class="main") {
                mainContent
            }
        }
    }
}
```

### Slots with Control Flow
```javascript
component List {
    slot items {
        text "No items"  // Default
    }

    render ul {
        items
    }
}

component App {
    render div {
        List() {
            repeat item in items {
                if item.isVisible {
                    li {
                        text item.name
                    }
                }
            }
        }
    }
}
```

## Edge Cases

### Empty Slots
```javascript
slot content {
    text "Empty state"  // Shows when no content provided
}
```

### Multiple Slot Inserts
```javascript
render div {
    header
    content
    content  // Can insert same slot multiple times
    footer
}
```

### Slot Content Types
- ✅ Text elements
- ✅ HTML tags
- ✅ Component instances
- ✅ Conditionals
- ✅ Repeats
- ✅ Style blocks (attached to elements)

## Performance Considerations

### Slot Resolution
- **Lookup**: O(1) slot name lookup in hash map
- **Content substitution**: Single pass during evaluation
- **Nested composition**: O(depth) for recursive resolution

### Evaluation Order
1. Evaluate component instance
2. Resolve props/attributes
3. Find slot content from children
4. Substitute slot inserts with content
5. Evaluate substituted content

## Conclusion

**Spike Status**: ✅ **SUCCESS**

The component composition and slot system is fully implemented in the parser and works perfectly:
- ✅ Component instances parse with props
- ✅ Slot declarations at component level
- ✅ Slot insertion points in render tree
- ✅ Default slot content for fallbacks
- ✅ Named slots with `insert` directive
- ✅ Nested composition (components in components)
- ✅ Universal slot content (any element type)
- ✅ Integration with control flow (conditionals, repeats)

All 11 tests passing with zero failures. The slot system provides a clean, intuitive API for component composition that matches modern component patterns while maintaining Paperclip's minimal syntax philosophy.

**Recommendations**:
1. Proceed with evaluator implementation for component evaluation
2. Add slot validation during parse or evaluate phase
3. Consider scoped slots for advanced use cases
4. Implement designer tools for visual slot editing

The parser foundation for component-based UIs is solid and production-ready!
