# Paperclip CSS Compiler

Compiles Paperclip `.pc` files to CSS stylesheets.

## Overview

This package extracts styles from Paperclip components and generates scoped CSS. It uses the evaluator to process style blocks and convert them to standard CSS rules with unique class names.

## Features

- ✅ Style extraction from components
- ✅ Scoped CSS class names (prevents conflicts)
- ✅ Nested element styles
- ✅ Design token support
- ✅ Multiple component compilation
- ✅ Clean, readable CSS output

## Usage

```rust
use paperclip_parser::parse;
use paperclip_compiler_css::compile_to_css;

fn main() {
    let source = r#"
        public component Button {
            render button {
                style {
                    padding: 8px 16px
                    background: #3366FF
                    color: white
                }
                text "Click me"
            }
        }
    "#;

    let document = parse(source).expect("Failed to parse");
    let css = compile_to_css(&document).expect("Failed to compile");

    println!("{}", css);
}
```

## Example Output

**Input (`.pc`):**
```javascript
public component Button {
    render button {
        style {
            padding: 8px 16px
            background: #3366FF
            color: white
            border: none
            border-radius: 4px
        }
        text "Click me"
    }
}
```

**Output (`.css`):**
```css
._Button-button-175a7583-5 {
  padding: 8px 16px;
  background: #3366FF;
  color: white;
  border: none;
  border-radius: 4px;
}
```

## Scoped Class Names

Class names are automatically generated using the pattern:
```
._{ComponentName}-{element}-{documentId}-{nodeId}
```

This ensures:
- **No naming conflicts** between components
- **Clear component association** in dev tools
- **Stable identifiers** across builds (when using same document path)

## Design Tokens

Tokens are expanded to their values in the CSS:

**Input:**
```javascript
public token primaryColor #3366FF

public component Button {
    render button {
        style {
            background: {primaryColor}
        }
        text "Click"
    }
}
```

**Output:**
```css
._Button-button-xyz {
  background: #3366FF;
}
```

## API

### `compile_to_css(document: &Document) -> CssResult<String>`

Compiles a Paperclip document to CSS.

**Parameters:**
- `document`: The parsed Paperclip AST

**Returns:**
- `Ok(String)`: The generated CSS
- `Err(CssError)`: Compilation error

### `compile_to_css_with_path(document: &Document, path: &str) -> CssResult<String>`

Compiles with a specific document path for stable ID generation.

**Parameters:**
- `document`: The parsed Paperclip AST
- `path`: Document path (used for generating stable IDs)

**Returns:**
- `Ok(String)`: The generated CSS
- `Err(CssError)`: Compilation error

## CLI Usage

```bash
# Compile to CSS
paperclip compile --target css

# Output to stdout
paperclip compile --target css --stdout

# Custom output directory
paperclip compile --target css --out-dir build/styles
```

## Integration with React Compiler

The React compiler generates `className` references that match the CSS compiler output:

**React Output:**
```jsx
<button className={cx("_Button-button-175a7583-5")}>
  Click me
</button>
```

**CSS Output:**
```css
._Button-button-175a7583-5 {
  padding: 8px 16px;
  background: #3366FF;
  color: white;
}
```

Import both in your React app:
```jsx
import "./styles.css";  // CSS from paperclip compile --target css
import { Button } from "./styles.jsx";  // React from paperclip compile --target react
```

## Testing

Run the test suite:

```bash
cargo test --package paperclip-compiler-css
```

All tests cover:
- Simple style compilation
- Nested elements
- Multiple components
- Token expansion

## Architecture

```
Paperclip AST
     ↓
CSS Evaluator (from paperclip-evaluator)
     ↓
Virtual CSS Document
     ↓
CSS Text Output
```

The compiler is a thin wrapper around the evaluator's CSS functionality.

## Future Enhancements

- [ ] CSS variable generation for tokens
- [ ] CSS modules support
- [ ] Source maps
- [ ] Minification
- [ ] Autoprefixer integration
- [ ] CSS custom properties fallbacks
- [ ] Media query extraction
- [ ] Keyframe animations support

## See Also

- [React Compiler](../compiler-react/README.md)
- [Evaluator](../evaluator/README.md)
- [Parser](../parser/README.md)
