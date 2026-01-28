# CSSOM Evaluation Mode

Paperclip supports **dual evaluation modes**: DOM evaluation for preview rendering and CSSOM evaluation for CSS production.

## Overview

PC files are evaluated in two separate pipelines:

1. **DOM Evaluation** → VDocument (for live preview)
2. **CSSOM Evaluation** → CssDocument (for production CSS)

This separation enables:
- ✅ Live preview with full component interactivity
- ✅ Production CSS generation with scoped selectors
- ✅ Separate optimization strategies for each use case

## Architecture

```
PC File (AST)
     │
     ├─→ DOM Evaluator → VDocument → Preview Rendering
     │
     └─→ CSS Evaluator → CssDocument → Production CSS
```

### DOM Evaluator
- Evaluates component structure
- Generates virtual DOM nodes
- Handles component expansion
- Used for preview rendering

### CSS Evaluator
- Extracts style declarations
- Generates scoped CSS rules
- Resolves design tokens
- Outputs production-ready CSS

## Usage

### Basic Dual Evaluation

```rust
use paperclip_parser::parse;
use paperclip_evaluator::{Evaluator, CssEvaluator};

let source = r#"
    token primaryColor #3366FF

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

let doc = parse(source)?;

// DOM Evaluation (for preview)
let mut dom_evaluator = Evaluator::new();
let vdoc = dom_evaluator.evaluate(&doc)?;

// CSS Evaluation (for production)
let mut css_evaluator = CssEvaluator::new();
let css_doc = css_evaluator.evaluate(&doc)?;

// Export CSS to file
let css_text = css_doc.to_css();
std::fs::write("output.css", css_text)?;
```

### Workspace Integration

The workspace automatically evaluates both modes:

```rust
use paperclip_workspace::WorkspaceState;

let mut workspace = WorkspaceState::new();

// Update file - evaluates both DOM and CSS
let patches = workspace.update_file(
    path,
    new_source,
    &project_root,
)?;

// Access cached results
let file_state = workspace.get_file(&path)?;
let vdom = &file_state.vdom;   // DOM evaluation result
let css = &file_state.css;      // CSS evaluation result
```

## CSS Scoping Strategy

CSS rules are scoped to prevent style collisions:

### Component-Scoped Selectors

Input:
```pc
public component Button {
    render button {
        style {
            padding: 8px
        }
    }
}
```

Output:
```css
.Button-1 .button-2 {
  padding: 8px;
}
```

The scoping strategy:
1. Generate unique class for component (`.Button-1`)
2. Generate unique class for each element (`.button-2`)
3. Combine into nested selector

### Global Styles

Input:
```pc
public style ButtonBase {
    padding: 8px 16px
    font-family: sans-serif
}
```

Output:
```css
.ButtonBase {
  padding: 8px 16px;
  font-family: sans-serif;
}
```

Global styles use their declaration name as the selector.

## Design Token Resolution

Tokens are resolved during CSS evaluation:

Input:
```pc
token primaryColor #3366FF
token spacing 16px

public component Button {
    render button {
        style {
            padding: {spacing}
            background: {primaryColor}
        }
    }
}
```

Output:
```css
.Button-1 .button-2 {
  padding: 16px;
  background: #3366FF;
}
```

**Note**: Token resolution in CSS values is currently being implemented. Use literal values for now.

## Public vs Private Components

Only **public** components generate CSS:

```pc
// Private - no CSS generated
component PrivateHelper {
    render div {
        style { margin: 8px }
    }
}

// Public - CSS generated
public component PublicComponent {
    render div {
        style { padding: 16px }
    }
}
```

This prevents internal implementation details from leaking into production CSS.

## CSS Output Format

### CssDocument Structure

```rust
pub struct CssDocument {
    pub rules: Vec<CssRule>,
}

pub struct CssRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
}
```

### Converting to CSS Text

```rust
let css_doc = css_evaluator.evaluate(&doc)?;
let css_text = css_doc.to_css();

// Output:
// .Button-1 .button-2 {
//   padding: 8px;
//   background: #3366FF;
// }
```

## Performance Characteristics

CSS evaluation is lightweight:

| Operation | Time (µs) | Notes |
|-----------|-----------|-------|
| Simple component | ~2-3 | Extract styles from one component |
| 10 components | ~20-30 | Linear scaling |
| Global styles | ~1-2 | Direct mapping |

CSS evaluation is independent of DOM evaluation, enabling:
- **Parallel processing**: Both can run concurrently
- **Cached results**: Only re-evaluate when source changes
- **Incremental updates**: Diff CSS documents for hot reload

## Limitations & Roadmap

### Current Limitations

1. **Token References**: `{tokenName}` syntax in CSS values not fully implemented yet
2. **Style Inheritance**: `extends` keyword not implemented
3. **Media Queries**: Not supported yet
4. **Pseudo-classes**: `:hover`, `:focus` variants not implemented
5. **CSS Variables**: Not converted to CSS custom properties yet

### Roadmap

- [ ] Token reference resolution in CSS values
- [ ] Style inheritance via `extends`
- [ ] Media query support
- [ ] Pseudo-class handling from variants
- [ ] CSS custom properties generation
- [ ] Source maps for debugging
- [ ] CSS minification
- [ ] Critical CSS extraction

## Best Practices

### 1. Use Design Tokens

Define tokens for consistency:

```pc
token primaryColor #3366FF
token spacing16 16px
token spacing8 8px
```

### 2. Component Scoping

Styles are automatically scoped - no need for manual BEM or CSS Modules:

```pc
public component Button {
    render button {
        style {
            padding: 8px
        }
        div {
            style {
                margin: 4px
            }
        }
    }
}
```

### 3. Global Styles for Utilities

Use global styles for reusable utility classes:

```pc
public style flexRow {
    display: flex
    flex-direction: row
}
```

### 4. Public Only What's Needed

Only mark components as `public` if they're used externally:

```pc
// Internal - stays private
component InternalButton { ... }

// External API - mark public
public component Button { ... }
```

## Integration Examples

### Build Pipeline

```rust
fn build_css(source_dir: &Path, output: &Path) -> Result<()> {
    let mut workspace = WorkspaceState::new();
    let mut all_css = CssDocument::new();

    // Process all PC files
    for entry in glob("**/*.pc")? {
        let source = fs::read_to_string(&entry)?;
        let patches = workspace.update_file(entry, source, source_dir)?;

        // Collect CSS from each file
        if let Some(state) = workspace.get_file(&entry) {
            for rule in &state.css.rules {
                all_css.add_rule(rule.clone());
            }
        }
    }

    // Write combined CSS
    fs::write(output, all_css.to_css())?;
    Ok(())
}
```

### Watch Mode

```rust
async fn watch_and_rebuild(dir: &Path) -> Result<()> {
    let mut watcher = notify::recommended_watcher(|event| {
        if let Ok(event) = event {
            for path in event.paths {
                if path.extension() == Some("pc") {
                    rebuild_css(&path)?;
                }
            }
        }
    })?;

    watcher.watch(dir, RecursiveMode::Recursive)?;
    // ... event loop
}
```

## Testing

Comprehensive test coverage for CSSOM:

- ✅ Dual evaluation (DOM + CSS)
- ✅ CSS scoping
- ✅ Token registration
- ✅ Global style declarations
- ✅ Multiple components
- ✅ Public vs private components
- ✅ CSS text output

Run tests:
```bash
cargo test -p paperclip-evaluator cssom
```

## Debugging

Enable logging to see CSS evaluation:

```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

let mut evaluator = CssEvaluator::new();
let css_doc = evaluator.evaluate(&doc)?;
```

Output:
```
DEBUG evaluate{components=1 tokens=2}: Starting CSS evaluation
DEBUG evaluate{components=1 tokens=2}: token_name="primaryColor" token_value="#3366FF" Registering CSS token
DEBUG evaluate{components=1 tokens=2}: component_name="Button" Processing component styles
INFO evaluate{components=1 tokens=2}: rules=1 CSS evaluation complete
```

## API Reference

### CssEvaluator

```rust
pub struct CssEvaluator;

impl CssEvaluator {
    pub fn new() -> Self;
    pub fn evaluate(&mut self, doc: &Document) -> CssResult<CssDocument>;
    pub fn tokens(&self) -> &HashMap<String, String>;
}
```

### CssDocument

```rust
pub struct CssDocument {
    pub rules: Vec<CssRule>,
}

impl CssDocument {
    pub fn new() -> Self;
    pub fn add_rule(&mut self, rule: CssRule);
    pub fn to_css(&self) -> String;
}
```

### CssRule

```rust
pub struct CssRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
}
```

## See Also

- [Evaluator Documentation](./README.md) - DOM evaluation
- [VDOM Differ](./src/vdom_differ.rs) - Patch generation
- [Benchmarks](./BENCHMARKS.md) - Performance characteristics
