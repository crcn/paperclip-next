# Paperclip Evaluator Formal Specification

**Version:** 0.1.0
**Last Updated:** January 2026

This document provides the formal semantics for the Paperclip evaluator, which transforms parsed AST into Virtual DOM with semantic identity.

---

## Table of Contents

1. [Overview](#overview)
2. [Type System](#type-system)
3. [Evaluation Context](#evaluation-context)
4. [Semantic Identity](#semantic-identity)
5. [Expression Evaluation](#expression-evaluation)
6. [Element Evaluation](#element-evaluation)
7. [Component Instantiation](#component-instantiation)
8. [Style Evaluation](#style-evaluation)
9. [Bundle Semantics](#bundle-semantics)
10. [Error Handling](#error-handling)
11. [Validation Rules](#validation-rules)

---

## 1. Overview

### 1.1 Purpose

The Paperclip evaluator transforms Abstract Syntax Trees (AST) produced by the parser into Virtual DOM documents with stable semantic identities. The evaluator is **deterministic**: given the same input AST and context, it produces identical output.

### 1.2 Pipeline

```
Document (AST) → Evaluator → VirtualDomDocument
```

Where:
- **Input**: `Document` - Parsed AST containing components, styles, and tokens
- **Output**: `VirtualDomDocument` - Virtual DOM nodes with semantic IDs and styles

### 1.3 Key Properties

**Determinism**: `∀ doc, ctx : evaluate(doc, ctx) = evaluate(doc, ctx)`

**Semantic Stability**: Semantic IDs remain stable across refactoring that preserves structure

**Error Recovery**: Evaluation continues on errors, producing `VNode::Error` nodes instead of propagating failures

---

## 2. Type System

### 2.1 AST Types

```rust
Document ::= {
    components: Component*,
    styles: StyleDecl*,
    tokens: TokenDecl*,
    imports: Import*
}

Component ::= {
    name: String,
    public: Bool,
    variants: Variant*,
    slots: SlotDecl*,
    body: Element?
}

Element ::=
    | Tag { tag: String, attributes: Attributes, children: Element*, ... }
    | Text { content: Expression, ... }
    | Instance { name: String, props: Attributes, children: Element*, ... }
    | Conditional { condition: Expression, then_branch: Element*, else_branch: Element*, ... }
    | Repeat { collection: Expression, body: Element*, ... }
    | SlotInsert { name: String?, ... }

Expression ::=
    | Literal { value: String | Number | Bool }
    | Variable { name: String }
    | BinaryOp { left: Expression, op: Operator, right: Expression }
    | MemberAccess { object: Expression, property: String }
```

### 2.2 Virtual DOM Types

```rust
VirtualDomDocument ::= {
    nodes: VNode*,
    styles: StyleBlock*
}

VNode ::=
    | Element {
        tag: String,
        attributes: Map<String, String>,
        styles: Map<String, String>,
        children: VNode*,
        semantic_id: SemanticID,
        key: String?,
        id: String?
    }
    | Text {
        content: String
    }
    | Comment {
        content: String
    }
    | Error {
        message: String,
        span: Span?,
        semantic_id: SemanticID
    }

SemanticID ::= {
    segments: SemanticSegment*
}

SemanticSegment ::=
    | Element { tag: String, role: String?, ast_id: String }
    | Component { name: String, key: String? }
    | RepeatItem { repeat_id: String, key: String }
    | ConditionalBranch { branch_type: BranchType, condition_id: String }
    | SlotVariant { variant: SlotVariantType, slot_id: String }
```

### 2.3 Runtime Value Types

```rust
Value ::=
    | String(String)
    | Number(f64)
    | Bool(bool)
    | Array(Vec<Value>)
    | Object(Map<String, Value>)
    | Null
```

---

## 3. Evaluation Context

### 3.1 Context State

The evaluation context maintains:

```rust
Context ::= {
    semantic_id_stack: SemanticSegment*,  // Current semantic path
    variables: Map<String, Value>,         // Variable bindings
    props: Map<String, Value>              // Component props
}
```

### 3.2 Context Operations

**Push Segment**: Adds segment to semantic path
```
push_segment(ctx, segment) = ctx { semantic_id_stack: ctx.semantic_id_stack + [segment] }
```

**Pop Segment**: Removes last segment from semantic path
```
pop_segment(ctx) = ctx { semantic_id_stack: init(ctx.semantic_id_stack) }
```

**Get Semantic ID**: Constructs current semantic identifier
```
get_semantic_id(ctx) = SemanticID { segments: ctx.semantic_id_stack }
```

**Set Variable**: Binds value to variable name
```
set_variable(ctx, name, value) = ctx { variables: ctx.variables[name ↦ value] }
```

---

## 4. Semantic Identity

### 4.1 Purpose

Semantic IDs provide **stable, hierarchical identifiers** that:
- Remain constant across refactoring
- Enable efficient diffing by ID comparison
- Support time-travel debugging and undo/redo

### 4.2 Segment Types

#### 4.2.1 Element Segment

```
Element { tag: "div", role: None, ast_id: "div-0" }
```

Generated for HTML elements. The `ast_id` is deterministically computed from the element's position in the AST.

#### 4.2.2 Component Segment

```
Component { name: "Button", key: Some("Button-0") }
```

Generated for component instances. The `key` is auto-generated based on component name and instance count, or explicitly provided.

#### 4.2.3 RepeatItem Segment

```
RepeatItem { repeat_id: "repeat-0", key: "item-0" }
```

Generated for items within a repeat block. The `key` can be:
- **Explicit**: Extracted from `key` attribute on first child element
- **Auto-generated**: `"item-{index}"` if no explicit key provided

**Rule**: Keys MUST be unique within a repeat block.

#### 4.2.4 ConditionalBranch Segment

```
ConditionalBranch { branch_type: Then, condition_id: "cond-0" }
```

Generated for conditional branches (then/else).

#### 4.2.5 SlotVariant Segment

```
SlotVariant { variant: Inserted, slot_id: "slot-0" }
```

Generated for slot content, indicating whether content is default or inserted.

### 4.3 Selector Format

Semantic IDs serialize to CSS-like selectors:

```
Button{"Button-0"}::div[div-0]::RepeatItem[repeat-0,"item-5"]::li[li-1]
```

**Properties**:
- **Hierarchical**: Shows full path from root to node
- **Deterministic**: Same structure produces same selector
- **Unique**: No two nodes have the same selector

---

## 5. Expression Evaluation

### 5.1 Evaluation Rules

**Literal**:
```
⟦ Literal(v) ⟧(ctx) = v
```

**Variable Lookup**:
```
⟦ Variable(name) ⟧(ctx) =
    if name ∈ ctx.variables then ctx.variables[name]
    else if name ∈ ctx.props then ctx.props[name]
    else Error("Undefined variable: {name}")
```

**Binary Operation**:
```
⟦ BinaryOp(left, op, right) ⟧(ctx) =
    let v1 = ⟦ left ⟧(ctx)
    let v2 = ⟦ right ⟧(ctx)
    apply_op(op, v1, v2)
```

**Member Access**:
```
⟦ MemberAccess(object, property) ⟧(ctx) =
    let obj = ⟦ object ⟧(ctx)
    match obj:
        Object(map) -> map.get(property) or Error("Property not found")
        _ -> Error("Cannot access property on non-object")
```

### 5.2 Operator Semantics

**String Concatenation** (`+` on strings):
```
apply_op(Add, String(s1), String(s2)) = String(s1 + s2)
```

**Numeric Addition** (`+` on numbers):
```
apply_op(Add, Number(n1), Number(n2)) = Number(n1 + n2)
```

**Equality** (`==`):
```
apply_op(Eq, v1, v2) = Bool(v1 == v2)
```

---

## 6. Element Evaluation

### 6.1 Tag Element

**Rule**:
```
⟦ Tag(tag, attributes, styles, children) ⟧(ctx) =
    let segment = Element { tag, role: None, ast_id }
    let ctx' = push_segment(ctx, segment)
    let semantic_id = get_semantic_id(ctx')

    let attrs = { k: ⟦ v ⟧(ctx') | (k, v) ∈ attributes }
    let inline_styles = { k: ⟦ v ⟧(ctx') | (k, v) ∈ styles }
    let child_nodes = [ ⟦ child ⟧(ctx') | child ∈ children ]

    VNode::Element {
        tag,
        attributes: attrs,
        styles: inline_styles,
        children: child_nodes,
        semantic_id,
        key: extract_key(attrs),
        id: attrs.get("id")
    }
```

**Error Handling**: If attribute or child evaluation fails, emit `VNode::Error` for that attribute/child and continue.

### 6.2 Text Element

**Rule**:
```
⟦ Text(content) ⟧(ctx) =
    match ⟦ content ⟧(ctx):
        Ok(value) -> VNode::Text { content: to_string(value) }
        Err(err) -> VNode::Error {
            message: format!("Error: {}", err),
            span: content.span,
            semantic_id: get_semantic_id(ctx)
        }
```

### 6.3 Conditional Element

**Rule**:
```
⟦ Conditional(condition, then_branch, else_branch) ⟧(ctx) =
    let wrapper = VNode::element("div", get_semantic_id(ctx))

    match ⟦ condition ⟧(ctx):
        Ok(Bool(true)) ->
            let segment = ConditionalBranch { branch_type: Then, condition_id }
            let ctx' = push_segment(ctx, segment)
            wrapper.with_children([ ⟦ elem ⟧(ctx') | elem ∈ then_branch ])

        Ok(Bool(false)) ->
            let segment = ConditionalBranch { branch_type: Else, condition_id }
            let ctx' = push_segment(ctx, segment)
            wrapper.with_children([ ⟦ elem ⟧(ctx') | elem ∈ else_branch ])

        Err(err) ->
            wrapper.with_child(VNode::error(err, span, semantic_id))
```

### 6.4 Repeat Element

**Rule**:
```
⟦ Repeat(collection, item_var, body) ⟧(ctx) =
    let wrapper = VNode::element("div", get_semantic_id(ctx))

    match ⟦ collection ⟧(ctx):
        Ok(Array(items)) ->
            for (index, item) in items.enumerate():
                // Extract explicit key from first child's attributes
                let explicit_key = extract_explicit_key(body[0], ctx)
                let item_key = explicit_key or format!("item-{}", index)

                let segment = RepeatItem { repeat_id, key: item_key }
                let ctx' = push_segment(set_variable(ctx, item_var, item), segment)

                for child in body:
                    match ⟦ child ⟧(ctx'):
                        Ok(vnode) ->
                            // Apply key to first child if explicit
                            if child == body[0] and explicit_key.is_some():
                                wrapper.add_child(vnode.with_key(item_key))
                            else:
                                wrapper.add_child(vnode)
                        Err(err) ->
                            wrapper.add_child(VNode::error(err, span, semantic_id))

        Err(err) ->
            wrapper.with_child(VNode::error(err, span, semantic_id))

    wrapper
```

**Key Extraction**:
```
extract_explicit_key(element, ctx) =
    if element is Tag with attributes:
        if "key" ∈ attributes:
            return Some(⟦ attributes["key"] ⟧(ctx).to_string())
    return None
```

---

## 7. Component Instantiation

### 7.1 Component Rendering

**Rule**:
```
⟦ Instance(name, props, inserted_children) ⟧(ctx, bundle) =
    let (component, file) = bundle.find_component(name, current_file)
    let key = auto_generate_component_key(name)

    let segment = Component { name, key: Some(key) }
    let ctx' = push_segment(ctx, segment)

    // Bind props
    let ctx'' = fold(set_prop, ctx', props)

    // Evaluate component body with inserted slots
    match component.body:
        Some(body) ->
            ⟦ body ⟧(ctx'', slots: inserted_children)
        None ->
            VNode::element("div", get_semantic_id(ctx''))
```

### 7.2 Slot Resolution

**Default Slot**:
```
⟦ SlotInsert(None) ⟧(ctx, slots) =
    let segment = SlotVariant { variant: Inserted, slot_id }
    let ctx' = push_segment(ctx, segment)

    if slots.default.is_empty():
        // Use default content from component definition
        let segment' = SlotVariant { variant: Default, slot_id }
        let ctx'' = push_segment(ctx, segment')
        [ ⟦ elem ⟧(ctx'') | elem ∈ default_content ]
    else:
        // Use inserted content
        [ ⟦ elem ⟧(ctx') | elem ∈ slots.default ]
```

**Named Slot**:
```
⟦ SlotInsert(Some(name)) ⟧(ctx, slots) =
    let segment = SlotVariant { variant: Inserted, slot_id }
    let ctx' = push_segment(ctx, segment)

    if slots[name].is_empty():
        // Use default content
        let segment' = SlotVariant { variant: Default, slot_id }
        let ctx'' = push_segment(ctx, segment')
        [ ⟦ elem ⟧(ctx'') | elem ∈ default_content ]
    else:
        // Use inserted content
        [ ⟦ elem ⟧(ctx') | elem ∈ slots[name] ]
```

---

## 8. Style Evaluation

### 8.1 Style Block

**Rule**:
```
⟦ StyleDecl(name, properties, extends) ⟧(bundle) =
    let base_styles = fold(merge_styles, {}, [ bundle.find_style(ext) | ext ∈ extends ])
    let own_styles = { k: resolve_value(v, bundle) | (k, v) ∈ properties }

    StyleBlock {
        selector: format!(".{}", namespaced_name),
        properties: merge_styles(base_styles, own_styles)
    }
```

### 8.2 Style Resolution

**Token Resolution**:
```
resolve_value(value, bundle) =
    if value starts with "$":
        let token_name = value[1..]
        match bundle.find_token(token_name):
            Some(token) -> token.value
            None -> value  // Keep original if not found
    else:
        value
```

**Style Inheritance**:
```
merge_styles(base, override) = base ∪ override
```

Where `override` takes precedence over `base` for duplicate keys.

---

## 9. Bundle Semantics

### 9.1 Dependency Graph

**Invariant**: The dependency graph MUST be acyclic.

```
∀ file ∈ Bundle.documents :
    ¬∃ path : file →* file
```

**Topological Order**: Documents are evaluated in dependency order (dependencies before dependents).

### 9.2 Import Resolution

**Relative Import**:
```
resolve_import("./foo.pc", importing_file) =
    canonicalize(parent(importing_file) / "foo.pc")
```

**Absolute Import**:
```
resolve_import("foo.pc", _, project_root) =
    canonicalize(project_root / "foo.pc")
```

### 9.3 Alias Resolution

**Definition**:
```
import "./theme.pc" as theme
```

Creates binding:
```
resolver.aliases[(current_file, "theme")] = resolved_path
```

**Lookup**:
```
resolve_alias("theme.fontBold", current_file, bundle) =
    let target_file = resolver.aliases[(current_file, "theme")]
    let doc = bundle.documents[target_file]
    find_style("fontBold", doc) where style.public == true
```

---

## 10. Error Handling

### 10.1 Partial Evaluation

**Principle**: Errors do not halt evaluation. They are converted to `VNode::Error` nodes inline.

**Error Node**:
```
VNode::Error {
    message: String,        // Human-readable error description
    span: Option<Span>,     // Source location if available
    semantic_id: SemanticID // Stable ID for the error location
}
```

**Rendering**: Error nodes render as styled `<span>` elements:
```html
<span class="paperclip-error"
      style="color: red; font-weight: bold; background: #fee; padding: 2px 4px; border: 1px solid red;"
      title="{message}">
    ⚠ {message}
</span>
```

### 10.2 Error Scenarios

**Undefined Variable**:
```
⟦ Variable("foo") ⟧(ctx) where "foo" ∉ ctx.variables ∪ ctx.props
    → VNode::Error { message: "Undefined variable: foo", ... }
```

**Invalid Repeat Collection**:
```
⟦ Repeat(collection, ...) ⟧(ctx) where ⟦ collection ⟧(ctx) ≠ Array(_)
    → wrapper.with_child(VNode::Error { message: "Invalid repeat collection", ... })
```

**Expression Error**:
```
⟦ BinaryOp(left, Add, right) ⟧(ctx) where typeof(left) ≠ typeof(right)
    → Error("Type mismatch in binary operation")
```

---

## 11. Validation Rules

### 11.1 Dev Mode Validation

Validation runs ONLY in development mode (`dev_mode = true`). In production, validation is skipped for zero overhead.

### 11.2 Validation Checks

**Auto-Generated Keys** (Warning):
```
∀ segment ∈ semantic_id.segments :
    if segment = RepeatItem { key, .. } and key.starts_with("item-"):
        Warning("Auto-generated key detected. Consider explicit keys for stability.")
```

**Duplicate Keys** (Error):
```
∀ repeat_block :
    let keys = { item.key | item ∈ repeat_block.items }
    if |keys| < |repeat_block.items|:
        Error("Duplicate key detected in repeat block")
```

**Duplicate Semantic IDs** (Error):
```
let selectors = { node.semantic_id.to_selector() | node ∈ all_nodes }
if |selectors| < |all_nodes|:
    Error("Duplicate semantic ID detected")
```

**Missing Component Keys** (Warning):
```
∀ segment ∈ semantic_id.segments :
    if segment = Component { key: None, .. }:
        Warning("Component instance has no explicit key. Auto-generated keys may not be stable.")
```

---

## 12. Appendix: Formal Notation

### 12.1 Notation Guide

- `⟦ expr ⟧(ctx)` - Evaluation of expression in context
- `∀ x : P(x)` - For all x, property P holds
- `∃ x : P(x)` - There exists x such that P holds
- `¬P` - Negation of P
- `P ∧ Q` - P and Q
- `P ∨ Q` - P or Q
- `P → Q` - P implies Q
- `a →* b` - Transitive closure (a reaches b through zero or more steps)
- `{ x | P(x) }` - Set of all x satisfying P
- `[a, b, c]` - List/sequence
- `{ k: v }` - Map/dictionary
- `A ∪ B` - Set union
- `A ∩ B` - Set intersection
- `|A|` - Cardinality (size) of set A
- `a ∈ A` - Element a is in set A
- `f(x) = y` - Function f maps x to y
- `a[b ↦ c]` - Map a with key b updated to value c

### 12.2 Type Notation

- `Type*` - Zero or more of Type
- `Type+` - One or more of Type
- `Type?` - Optional Type
- `Type1 | Type2` - Sum type (either Type1 or Type2)

---

## 13. Implementation Notes

### 13.1 Performance Characteristics

- **Parse Time**: O(n) where n = source size
- **Evaluate Time**: O(m) where m = AST node count
- **Semantic ID Generation**: O(d) where d = tree depth
- **Validation**: O(n) where n = VNode count (dev mode only)

### 13.2 Memory Usage

- **Zero-copy parsing**: AST references source strings
- **Arena allocation**: VNodes allocated in contiguous memory
- **Interning**: Repeated strings (tag names, property names) are interned

### 13.3 Determinism Guarantees

Given:
- Same source code
- Same import graph
- Same evaluation order (topological)

The evaluator produces:
- Identical VNodes (structural equality)
- Identical semantic IDs
- Identical style blocks

**Exception**: Auto-generated keys depend on evaluation order within a file.

---

## 14. Change Log

### Version 0.1.0 (January 2026)
- Initial specification
- Semantic identity system
- Error recovery with VNode::Error
- Slot implementation
- Explicit key support for repeat blocks
- Duplicate key validation

---

## References

1. **AST Definition**: See `packages/parser/src/ast.rs`
2. **Evaluator Implementation**: See `packages/evaluator/src/evaluator.rs`
3. **Semantic Identity**: See `packages/evaluator/src/semantic_identity.rs`
4. **Validation Rules**: See `packages/evaluator/src/validator.rs`
5. **Bundle System**: See `packages/evaluator/src/bundle.rs`
