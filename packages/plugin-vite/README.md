# @paperclip-lang/vite-plugin

Vite plugin for Paperclip files. Compiles `.pc` files to React/JSX using WebAssembly with hot module replacement support.

## Installation

```bash
yarn add -D @paperclip-lang/vite-plugin @paperclip-lang/wasm
```

## Usage

### vite.config.ts

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import paperclip from '@paperclip-lang/vite-plugin';

export default defineConfig({
  plugins: [
    paperclip(),
    react(),
  ],
});
```

### With Options

```typescript
import { defineConfig } from 'vite';
import paperclip from '@paperclip-lang/vite-plugin';

export default defineConfig({
  plugins: [
    paperclip({
      typescript: true,        // Generate TypeScript definitions (default: true)
      includeStyles: true,     // Include CSS (default: true)
      filter: (id) => id.endsWith('.pc'),  // Custom filter
    }),
  ],
});
```

### TypeScript Support

Add `.pc` files to your TypeScript module declarations:

```typescript
// vite-env.d.ts
/// <reference types="vite/client" />

declare module '*.pc' {
  import { FC } from 'react';

  export const Button: FC<any>;
  export const Card: FC<any>;
  // ... other components
}
```

### Using Paperclip Components

```typescript
// button.pc
public component Button {
  render button {
    style {
      padding: 8px 16px
      background: #3366ff
      color: white
      border: none
      border-radius: 4px
    }
    text "Click me"
  }
}
```

```typescript
// App.tsx
import { Button } from './button.pc';

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

## Options

### `typescript`

Type: `boolean`
Default: `true`

Generate TypeScript type definitions for components.

### `includeStyles`

Type: `boolean`
Default: `true`

Automatically extract and inject CSS from Paperclip files.

### `filter`

Type: `(id: string) => boolean`
Default: `(id) => id.endsWith('.pc')`

Custom filter function to determine which files to process.

## Features

### ⚡️ Lightning Fast

Powered by WebAssembly for instant compilation:
- **Parse + Compile**: < 100 microseconds
- **Hot Module Replacement**: Instant updates
- **No performance impact** on dev server

### 🔥 Hot Module Replacement

Full HMR support with instant component updates during development.

### 🎨 CSS Extraction

Automatically extracts and injects scoped CSS from your components.

### 📦 TypeScript

First-class TypeScript support with auto-generated type definitions.

## Example Project

```bash
# Create a new Vite + React + TypeScript project
yarn create vite my-app --template react-ts
cd my-app

# Install Paperclip plugin
yarn add -D @paperclip-lang/vite-plugin @paperclip-lang/wasm

# Add plugin to vite.config.ts
# ... (see usage above)

# Create a .pc file
cat > src/Button.pc << 'EOF'
public component Button {
  render button {
    style {
      padding: 8px 16px
      background: #3366ff
      color: white
    }
    text "Click me"
  }
}
EOF

# Use in your app
# ... (see usage above)

# Start dev server
yarn dev
```

## Comparison with Other Loaders

| Feature | Vite Plugin | Webpack Loader | Rollup Plugin |
|---------|-------------|----------------|---------------|
| Speed | ⚡️ Fastest | ✅ Fast | ✅ Fast |
| HMR | ✅ Yes | ✅ Yes | ❌ No |
| CSS Extraction | ✅ Auto | ⚠️ Manual | ⚠️ Manual |
| TypeScript | ✅ Yes | ✅ Yes | ✅ Yes |
| Config | 🎯 Simple | ⚙️ Moderate | ⚙️ Moderate |

## License

MIT
