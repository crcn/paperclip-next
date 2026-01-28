# Paperclip CLI

Command-line interface for Paperclip - the visual component builder for the AI age.

## Installation

```bash
cargo install --path packages/cli
```

Or build from source:

```bash
cargo build --package paperclip-cli --release
```

The binary will be at `target/release/paperclip`.

## Commands

### `paperclip init`

Initialize a new Paperclip project with configuration and example files.

```bash
paperclip init
```

**Options:**
- `-t, --target <TARGET>` - Target format (react, html, css, all) [default: react]
- `-s, --src-dir <DIR>` - Source directory [default: src]
- `-f, --force` - Force overwrite existing config

**Example:**
```bash
# Initialize with React target
paperclip init

# Initialize with all targets
paperclip init --target all

# Custom source directory
paperclip init --src-dir components
```

**Creates:**
- `paperclip.config.json` - Project configuration
- `src/` - Source directory
- `src/example.pc` - Example component

### `paperclip compile`

Compile `.pc` files to target format (React, HTML, CSS).

```bash
paperclip compile
```

**Options:**
- `[PATH]` - Directory to compile [default: .]
- `-t, --target <TARGET>` - Target format (react, html, css) [default: react]
- `--stdout` - Output to stdout instead of files
- `-o, --out-dir <DIR>` - Output directory (overrides config)
- `--typescript` - Generate TypeScript definitions
- `-w, --watch` - Watch for file changes (coming soon)

**Examples:**
```bash
# Compile current directory
paperclip compile

# Compile specific directory
paperclip compile ./components

# Output to stdout
paperclip compile --stdout

# Generate TypeScript definitions
paperclip compile --typescript

# Custom output directory
paperclip compile --out-dir build

# Different target
paperclip compile --target html
```

**Output:**
- React: `.jsx` files + optional `.d.ts` files
- HTML: `.html` files (coming soon)
- CSS: `.css` files (coming soon)

### `paperclip designer`

Start the visual designer (coming soon).

```bash
paperclip designer
```

**Options:**
- `-p, --port <PORT>` - Port to run on [default: 3000]
- `--open` - Open browser automatically

## Configuration

The `paperclip.config.json` file controls how Paperclip compiles your components.

**Example:**
```json
{
  "srcDir": "src",
  "moduleDirs": ["node_modules"],
  "compilerOptions": [
    {
      "emit": ["react", "css"],
      "outDir": "dist"
    }
  ]
}
```

**Fields:**
- `srcDir` - Source directory containing `.pc` files
- `moduleDirs` - Directories to search for imports
- `compilerOptions` - Array of compiler configurations
  - `emit` - Output formats to generate
  - `outDir` - Output directory (optional)

## Quick Start

1. **Initialize a project:**
   ```bash
   paperclip init
   ```

2. **Edit your component:**
   ```bash
   # Edit src/example.pc
   ```

3. **Compile:**
   ```bash
   paperclip compile
   ```

4. **Check output:**
   ```bash
   cat dist/example.jsx
   ```

## Example Workflow

```bash
# Create new project
mkdir my-components
cd my-components
paperclip init

# Create a component
cat > src/button.pc << 'EOF'
public component Button {
    render button(type="button") {
        style {
            padding: 8px 16px
            background: #3366FF
            color: white
            border: none
            border-radius: 4px
        }
        text {label}
    }
}
EOF

# Compile with TypeScript
paperclip compile --typescript

# Use in React
cat > app.jsx << 'EOF'
import { Button } from './dist/button';

export function App() {
  return <Button label="Click me!" />;
}
EOF
```

## Architecture

The CLI is a thin wrapper around the compiler packages:

```
paperclip CLI
    ├── paperclip-parser      (Parse .pc files to AST)
    ├── paperclip-compiler-react (Compile to React/JSX)
    └── paperclip-evaluator   (Evaluate to Virtual DOM)
```

## Development

Run the CLI during development:

```bash
cargo run --package paperclip-cli -- init
cargo run --package paperclip-cli -- compile --typescript
```

Run tests:

```bash
cargo test --package paperclip-cli
```

## Features

- ✅ Project initialization
- ✅ React compilation (.jsx)
- ✅ TypeScript definitions (.d.ts)
- ✅ Stdout output
- ✅ Config file support
- ✅ Color-coded output
- ⬜ Watch mode
- ⬜ HTML compilation
- ⬜ CSS compilation
- ⬜ Visual designer
- ⬜ Live reload
- ⬜ Production optimizations

## Error Handling

The CLI provides clear, colored error messages:

```bash
$ paperclip compile --target unknown
Error: Unknown target: unknown

$ paperclip compile nonexistent
Error: Source directory does not exist: "./nonexistent"
```

## Exit Codes

- `0` - Success
- `1` - Error occurred

## See Also

- [Paperclip Parser](../parser/README.md)
- [React Compiler](../compiler-react/README.md)
- [Main Documentation](../../README.md)
