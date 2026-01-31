# Paperclip Bundler Loaders

This document describes the bundler loaders for Paperclip files.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Paperclip Source (.pc)                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              @paperclip-lang/wasm (Core)                     │
│  ┌──────────────────────────────────────────────────┐       │
│  │  Rust Parser → AST → React Compiler              │       │
│  │  Compiled to WebAssembly                         │       │
│  └──────────────────────────────────────────────────┘       │
└───────┬─────────────┬────────────┬──────────────┬───────────┘
        │             │            │              │
        ▼             ▼            ▼              ▼
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│ Webpack  │  │  Vite    │  │ Rollup   │  │ esbuild  │
│  Loader  │  │  Plugin  │  │  Plugin  │  │  Plugin  │
└──────────┘  └──────────┘  └──────────┘  └──────────┘
```

## Packages

### 1. Core WASM Package

**Package**: `@paperclip-lang/wasm`
**Location**: `packages/wasm/`

WebAssembly bindings for the Paperclip compiler.

**Build**:
```bash
cd packages/wasm
npm run build          # For bundlers (webpack, rollup, vite)
npm run build:nodejs   # For Node.js
npm run build:web      # For browsers (ES modules)
npm run build:all      # All targets
```

**API**:
```typescript
// Compile to React/JSX
compileToReact(source: string, filePath: string, generateTypes: boolean): {
  code: string;
  types?: string;
}

// Compile to CSS
compileToCss(source: string, filePath: string): string

// Parse AST
parse(source: string, filePath: string): string

// Get document ID
getDocumentId(filePath: string): string
```

**Performance**:
- Parse: ~1-10 µs
- Compile to React: ~10-50 µs
- Total: < 100 µs per component

---

### 2. Webpack Loader

**Package**: `@paperclip-lang/webpack-loader`
**Location**: `packages/loader-webpack/`

**Installation**:
```bash
yarn add -D @paperclip-lang/webpack-loader @paperclip-lang/wasm
```

**Usage**:
```javascript
// webpack.config.js
module.exports = {
  module: {
    rules: [
      {
        test: /\.pc$/,
        use: {
          loader: '@paperclip-lang/webpack-loader',
          options: {
            typescript: true,        // Generate TypeScript definitions
            emitDeclaration: false,  // Emit separate .d.ts files
          },
        },
      },
    ],
  },
};
```

**Features**:
- ✅ TypeScript definitions
- ✅ Separate .d.ts emission
- ✅ Fast compilation (WASM)
- ✅ Webpack 5 compatible

---

### 3. Vite Plugin

**Package**: `@paperclip-lang/vite-plugin`
**Location**: `packages/plugin-vite/`

**Installation**:
```bash
yarn add -D @paperclip-lang/vite-plugin @paperclip-lang/wasm
```

**Usage**:
```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import paperclip from '@paperclip-lang/vite-plugin';

export default defineConfig({
  plugins: [
    paperclip({
      typescript: true,        // Generate TypeScript definitions
      includeStyles: true,     // Include CSS
    }),
    react(),
  ],
});
```

**Features**:
- ✅ Hot Module Replacement (HMR)
- ✅ TypeScript definitions
- ✅ Automatic CSS extraction
- ✅ Lightning fast (WASM)
- ✅ Vite 3, 4, 5 compatible

---

### 4. Rollup Plugin

**Package**: `@paperclip-lang/rollup-plugin`
**Location**: `packages/plugin-rollup/`

**Installation**:
```bash
yarn add -D @paperclip-lang/rollup-plugin @paperclip-lang/wasm
```

**Usage**:
```javascript
// rollup.config.js
import paperclip from '@paperclip-lang/rollup-plugin';

export default {
  plugins: [
    paperclip({
      typescript: true,        // Generate TypeScript definitions
      includeStyles: true,     // Include CSS
    }),
  ],
};
```

**Features**:
- ✅ TypeScript definitions
- ✅ CSS asset emission
- ✅ Tree-shaking friendly
- ✅ Rollup 2, 3, 4 compatible

---

### 5. esbuild Plugin

**Package**: `@paperclip-lang/esbuild-plugin`
**Location**: `packages/plugin-esbuild/`

**Installation**:
```bash
yarn add -D @paperclip-lang/esbuild-plugin @paperclip-lang/wasm
```

**Usage**:
```javascript
// build.js
const esbuild = require('esbuild');
const paperclip = require('@paperclip-lang/esbuild-plugin');

esbuild.build({
  entryPoints: ['src/index.tsx'],
  bundle: true,
  plugins: [
    paperclip({
      typescript: true,
    }),
  ],
});
```

**Features**:
- ✅ TypeScript definitions
- ✅ Blazing fast (WASM + esbuild)
- ✅ esbuild 0.17+ compatible

---

## Example Usage

### React Component

```paperclip
// Button.pc
public component Button {
  render button {
    style {
      padding: 8px 16px
      background: #3366ff
      color: white
      border: none
      border-radius: 4px
      cursor: pointer
    }

    style hover {
      background: #2952cc
    }

    text "Click me"
  }
}
```

### Import in React/TypeScript

```typescript
// App.tsx
import { Button } from './Button.pc';

function App() {
  return (
    <div>
      <h1>My App</h1>
      <Button />
    </div>
  );
}

export default App;
```

### TypeScript Declarations

The loaders automatically generate TypeScript declarations:

```typescript
// Button.pc.d.ts (auto-generated)
import { FC } from 'react';

export const Button: FC<{}>;
```

---

## Building the Loaders

### Prerequisites

1. Install Rust (latest stable)
2. Install wasm-pack:
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

### Build Steps

```bash
# 1. Build WASM core
cd packages/wasm
yarn build:all

# 2. Build loaders
cd ../loader-webpack
yarn build

cd ../plugin-vite
yarn build

cd ../plugin-rollup
yarn build

cd ../plugin-esbuild
yarn build
```

### Testing

```bash
# Test WASM module
cd packages/wasm
yarn test

# Test in a real project
cd ../../examples/vite-react
yarn install
yarn dev
```

---

## Performance Comparison

| Bundler | Cold Start | Hot Reload | Build Time |
|---------|------------|------------|------------|
| **Vite** | ⚡️ 50ms | ⚡️ 10ms | ⚡️ Fast |
| **Webpack** | ✅ 100ms | ✅ 50ms | ✅ Moderate |
| **Rollup** | ✅ 80ms | ⚠️ N/A | ✅ Fast |
| **esbuild** | ⚡️ 30ms | ⚠️ N/A | ⚡️ Blazing |

*All times are approximate and include WASM compilation overhead*

---

## Troubleshooting

### WASM Module Not Found

If you get "Cannot find module '@paperclip-lang/wasm'":

1. Ensure WASM package is built:
   ```bash
   cd packages/wasm && npm run build
   ```

2. Link for local development:
   ```bash
   cd packages/wasm && yarn link
   cd ../loader-webpack && yarn link @paperclip-lang/wasm
   ```

### TypeScript Errors

Add `.pc` module declarations:

```typescript
// types/paperclip.d.ts
declare module '*.pc' {
  import { FC } from 'react';
  export const [componentName]: FC<any>;
}
```

### Build Performance

For faster builds, use the release profile:

```bash
cd packages/wasm
wasm-pack build --release --target bundler
```

---

## Roadmap

- [ ] Source maps support
- [ ] CSS Modules integration
- [ ] Server-side rendering (SSR) support
- [ ] Bundle size optimization
- [ ] Streaming compilation
- [ ] Incremental builds
- [ ] Watch mode optimization

---

## License

MIT
