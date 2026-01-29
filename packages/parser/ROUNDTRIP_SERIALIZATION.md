# Roundtrip Serialization (Spike 0.4)

**Status:** ✅ Complete

## Goal

Prove that AST edits can be serialized back to `.pc` source code while preserving original formatting, whitespace, and comments.

## Approach: Span-Based Preservation

Instead of capturing whitespace/comments as trivia tokens (high memory overhead), we use **existing spans** to preserve formatting.

### How It Works

```rust
// 1. Parse source with spans
let source = "component Button { ... }";
let ast = parse(source)?;

// 2. Edit AST
ast.components[0].name = "BigButton";

// 3. Serialize using spans
let serializer = LosslessSerializer::new(source);
serializer.mark_dirty(&ast.components[0].span.id);
let output = serializer.serialize(&ast);
```

### Key Concepts

1. **Clean nodes** → Copy original source verbatim using spans
2. **Dirty nodes** → Re-serialize using regular serializer
3. **Whitespace between nodes** → Preserved automatically via spans

### Implementation: `LosslessSerializer`

**File:** `packages/parser/src/lossless_serializer.rs`

```rust
pub struct LosslessSerializer<'a> {
    source: &'a str,                // Original source
    dirty_spans: HashSet<String>,  // Modified node IDs
}
```

**Algorithm:**

1. Collect all top-level spans sorted by position
2. For each node:
   - Preserve whitespace before node (source[last_end..span.start])
   - If dirty: re-serialize
   - If clean: copy source[span.start..span.end]
3. Preserve trailing whitespace

### Tests

**Test 1: No changes → Perfect roundtrip**
```rust
let source = "public component Button { ... }";
let ast = parse(source)?;
let output = LosslessSerializer::new(source).serialize(&ast);
assert_eq!(output, source); // ✓ Exact match
```

**Test 2: Preserve comments**
```rust
let source = "// Button\npublic component Button { ... }";
// Comments preserved ✓
```

**Test 3: Preserve extra whitespace**
```rust
let source = "\n\n\npublic component Button { ... }\n\n\n";
// All whitespace preserved ✓
```

**Test 4: Dirty node re-serialized**
```rust
doc.components[0].name = "BigButton";
serializer.mark_dirty(&doc.components[0].span.id);
// Output contains "BigButton" ✓
// Original "Button" gone ✓
```

## Performance Characteristics

- **Memory:** O(source length) - stores original source
- **Time:** O(n) where n = number of nodes
- **No parser changes required** - uses existing spans

## Advantages of Span-Based Approach

✅ **Zero parser changes** - Works with existing tokenizer
✅ **Low memory overhead** - No trivia token storage
✅ **Fast** - Simple string slicing
✅ **Exact preservation** - Byte-perfect for clean nodes
✅ **Simple mental model** - Easy to understand and debug

## Limitations

1. **Re-serialized nodes lose original formatting**
   - Dirty nodes use default formatting (2-space indent)
   - Could be improved with format-preserving edits

2. **Requires original source**
   - Must store source alongside AST
   - Not a problem for editor use case

3. **Span IDs must be unique**
   - Already enforced by ID generator

## Usage in Editor

```rust
// Load document
let source = fs::read_to_string("button.pc")?;
let mut doc = parse(&source)?;

// Make edit
doc.components[0].name = "BigButton".to_string();

// Serialize with preservation
let mut serializer = LosslessSerializer::new(&source);
serializer.mark_dirty(&doc.components[0].span.id);
let output = serializer.serialize(&doc);

// Save
fs::write("button.pc", output)?;
```

## Integration with Editor Package

The `packages/editor/` mutation system should:

1. Track dirty spans when mutations are applied
2. Use `LosslessSerializer` for save operations
3. Pass dirty span list to serializer

Example:

```rust
impl Document {
    pub fn apply_mutation(&mut self, mutation: Mutation) {
        match mutation {
            Mutation::UpdateText { node_id, content } => {
                // Find node and update
                let node = self.find_node_mut(&node_id)?;
                node.content = content;

                // Track as dirty
                self.dirty_spans.insert(node.span.id);
            }
        }
    }

    pub fn save(&self) -> String {
        let mut serializer = LosslessSerializer::new(&self.original_source);
        for span_id in &self.dirty_spans {
            serializer.mark_dirty(span_id);
        }
        serializer.serialize(&self.ast)
    }
}
```

## Future Enhancements

- **Format-preserving edits:** Detect whitespace style in dirty nodes
- **Incremental serialization:** Only serialize changed subtrees
- **Source map generation:** Track line/column changes after edits

## Validation: Spike 0.4 Complete ✅

- ✅ Lossless roundtrip for unmodified files
- ✅ Comments preserved
- ✅ Whitespace preserved
- ✅ Dirty nodes correctly re-serialized
- ✅ All tests passing

**This validates that the parser approach supports lossless roundtrip editing, enabling the visual designer to save changes back to `.pc` files.**
