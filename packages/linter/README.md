# Paperclip Linter

Configurable linting engine for Paperclip `.pc` files.

## Features

- 🔍 **Configurable rules** - Enable/disable rules per project
- 🎯 **Multiple severity levels** - Error, warning, info
- 📊 **Actionable diagnostics** - Clear error messages with locations
- ⚡ **Fast analysis** - Built on high-performance parser
- 🔧 **Extensible** - Easy to add custom rules

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
paperclip-linter = { path = "../linter" }
paperclip-parser = { path = "../parser" }
```

## Usage

### Basic Linting

```rust
use paperclip_parser::parse_with_path;
use paperclip_linter::{Linter, LintConfig};

fn main() {
    let source = r#"
        public component Button {
            render button {
                text "Click me"
            }
        }
    "#;

    let doc = parse_with_path(source, "button.pc").expect("Failed to parse");
    let config = LintConfig::default();
    let linter = Linter::new(config);

    let diagnostics = linter.lint(&doc);

    for diagnostic in diagnostics {
        println!("{}: {} at line {}",
            diagnostic.severity,
            diagnostic.message,
            diagnostic.line
        );
    }
}
```

### Custom Configuration

```rust
use paperclip_linter::{LintConfig, RuleConfig, Severity};

let mut config = LintConfig::default();

// Configure specific rules
config.rules.insert(
    "unused-component".to_string(),
    RuleConfig {
        enabled: true,
        severity: Severity::Warning,
    }
);

config.rules.insert(
    "missing-key".to_string(),
    RuleConfig {
        enabled: true,
        severity: Severity::Error,
    }
);

let linter = Linter::new(config);
```

## CLI Usage

```bash
# Lint a single file
paperclip lint button.pc

# Lint a directory
paperclip lint src/components

# With custom config
paperclip lint src/ --config .paperclip-lint.json

# Output format
paperclip lint src/ --format json
```

## Built-in Rules

### `unused-component`
Detects components that are declared but never used.

**Severity:** Warning
**Example:**
```javascript
// Warning: Component 'Helper' is never used
component Helper {
    render div { text "help" }
}
```

### `missing-key`
Detects repeat loops without explicit keys.

**Severity:** Warning
**Example:**
```javascript
// Warning: Repeat without key attribute
repeat item in items {
    Card()
}

// Fixed:
repeat item in items {
    Card(key={item.id})
}
```

### `duplicate-variant`
Detects duplicate variant names in a component.

**Severity:** Error
**Example:**
```javascript
public component Button {
    variant primary
    variant primary  // Error: Duplicate variant 'primary'
    render button { }
}
```

### `empty-component`
Detects components with no render body.

**Severity:** Warning
**Example:**
```javascript
// Warning: Component has no render body
component Empty {
    // No render statement
}
```

### `invalid-css-property`
Detects unknown CSS properties.

**Severity:** Warning (configurable)
**Example:**
```javascript
style {
    paddin: 16px  // Warning: Unknown CSS property 'paddin', did you mean 'padding'?
}
```

## Configuration File

Create `.paperclip-lint.json` in your project root:

```json
{
  "rules": {
    "unused-component": {
      "enabled": true,
      "severity": "warning"
    },
    "missing-key": {
      "enabled": true,
      "severity": "warning"
    },
    "duplicate-variant": {
      "enabled": true,
      "severity": "error"
    },
    "empty-component": {
      "enabled": false
    }
  }
}
```

## API Reference

### `Linter`

Main linter struct.

#### Methods

**`new(config: LintConfig) -> Self`**

Create a new linter with the given configuration.

**`lint(&self, document: &Document) -> Vec<Diagnostic>`**

Lint a parsed document and return diagnostics.

### `LintConfig`

Configuration for the linter.

```rust
pub struct LintConfig {
    pub rules: HashMap<String, RuleConfig>,
}
```

#### Methods

**`default() -> Self`**

Create default configuration with all rules enabled.

**`from_file(path: &Path) -> Result<Self, Error>`**

Load configuration from a JSON file.

### `RuleConfig`

Configuration for a single rule.

```rust
pub struct RuleConfig {
    pub enabled: bool,
    pub severity: Severity,
}
```

### `Diagnostic`

A linting diagnostic.

```rust
pub struct Diagnostic {
    pub rule: String,
    pub severity: Severity,
    pub message: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub span: Option<Range<usize>>,
}
```

### `Severity`

Diagnostic severity level.

```rust
pub enum Severity {
    Error,
    Warning,
    Info,
}
```

## Writing Custom Rules

Create a custom rule by implementing the `LintRule` trait:

```rust
use paperclip_linter::{LintRule, Diagnostic, Severity};
use paperclip_parser::ast::Document;

pub struct MyCustomRule;

impl LintRule for MyCustomRule {
    fn name(&self) -> &str {
        "my-custom-rule"
    }

    fn check(&self, document: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Analyze document and collect diagnostics
        for component in &document.components {
            if component.name.len() > 20 {
                diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    severity: Severity::Warning,
                    message: "Component name is too long".to_string(),
                    file_path: "".to_string(),
                    line: 0,
                    column: 0,
                    span: Some(component.span.start..component.span.end),
                });
            }
        }

        diagnostics
    }
}
```

Register your custom rule:

```rust
let mut linter = Linter::new(config);
linter.add_rule(Box::new(MyCustomRule));
```

## Testing

Run tests:

```bash
cargo test -p paperclip-linter
```

Run with output:

```bash
cargo test -p paperclip-linter -- --nocapture
```

## Integration

### With CLI

The linter is integrated into the Paperclip CLI:

```bash
paperclip lint src/
```

### With LSP

The linter can be integrated with Language Server Protocol implementations for real-time diagnostics in editors.

### With CI/CD

Add linting to your CI pipeline:

```yaml
# .github/workflows/lint.yml
- name: Lint Paperclip files
  run: paperclip lint src/ --format json > lint-results.json
```

## Performance

The linter is designed for speed:
- **Parse once** - Uses existing parsed AST
- **Parallel rule execution** - Rules run concurrently (future)
- **Incremental analysis** - Only re-check changed files (future)

Typical performance:
- Small project (< 50 files): < 100ms
- Medium project (< 500 files): < 1s
- Large project (1000+ files): < 5s

## Future Rules

Planned rules for future releases:

- `unused-import` - Detect unused import statements
- `unused-token` - Detect unused design tokens
- `unused-slot` - Detect unused slot definitions
- `inconsistent-naming` - Enforce naming conventions
- `max-component-size` - Limit component complexity
- `required-documentation` - Require doc comments
- `accessibility-checks` - A11y validation

## License

MIT
