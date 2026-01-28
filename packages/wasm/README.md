# @paperclip-lang/wasm

WebAssembly bindings for the Paperclip compiler. This package compiles Paperclip (`.pc`) files to React/JSX and CSS in the browser or Node.js.

## Installation

```bash
npm install @paperclip-lang/wasm
```

## Prerequisites

You need [wasm-pack](https://rustwasm.github.io/wasm-pack/) to build:

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

## Building

```bash
# Build for bundlers (webpack, rollup, vite)
npm run build

# Build for Node.js
npm run build:nodejs

# Build for web (ES modules)
npm run build:web

# Build all targets
npm run build:all
```

## Usage

### Compile to React

```typescript
import { compileToReact } from '@paperclip-lang/wasm';

const source = `
  public component Button {
    render button {
      text "Click me"
    }
  }
`;

const result = compileToReact(source, '/components/button.pc', true);
console.log(result.code);      // React/JSX code
console.log(result.types);     // TypeScript definitions (if requested)
```

### Compile to CSS

```typescript
import { compileToCss } from '@paperclip-lang/wasm';

const source = `
  public style buttonStyle {
    padding: 8px 16px
    background: blue
  }
`;

const css = compileToCss(source, '/styles.pc');
console.log(css);  // CSS output
```

### Parse AST

```typescript
import { parse } from '@paperclip-lang/wasm';

const source = `
  component Card {
    render div {
      text "Hello"
    }
  }
`;

const ast = parse(source, '/card.pc');
console.log(JSON.parse(ast));  // AST as JSON
```

## API

### `compileToReact(source: string, filePath: string, generateTypes: boolean): CompileResult`

Compiles a Paperclip source to React/JSX.

**Returns:**
- `code: string` - The compiled React/JSX code
- `types: string | undefined` - TypeScript definitions (if `generateTypes` is true)

### `compileToCss(source: string, filePath: string): string`

Compiles a Paperclip source to CSS.

### `parse(source: string, filePath: string): string`

Parses a Paperclip source and returns the AST as JSON string.

### `getDocumentId(filePath: string): string`

Returns the deterministic document ID for a file path (CRC32-based).

## Performance

WASM compilation is extremely fast:
- **Parse**: ~1-10 microseconds
- **Compile to React**: ~10-50 microseconds
- **Total**: < 100 microseconds for most components

This makes it suitable for real-time compilation in dev servers and bundlers.

## Browser Support

The WASM module works in all modern browsers and Node.js 14+.

## License

MIT
