# @paperclip-lang/webpack-loader

Webpack loader for Paperclip files. Compiles `.pc` files to React/JSX using WebAssembly.

## Installation

```bash
yarn add -D @paperclip-lang/webpack-loader @paperclip-lang/wasm
```

## Usage

### webpack.config.js

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.pc$/,
        use: '@paperclip-lang/webpack-loader',
      },
    ],
  },
};
```

### With Options

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.pc$/,
        use: {
          loader: '@paperclip-lang/webpack-loader',
          options: {
            typescript: true,        // Generate TypeScript definitions (default: true)
            emitDeclaration: false,  // Emit separate .d.ts files (default: false)
          },
        },
      },
    ],
  },
};
```

### TypeScript Support

Add `.pc` files to your TypeScript module declarations:

```typescript
// types/paperclip.d.ts
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
    text "Click me"
  }
}
```

```typescript
// app.tsx
import { Button } from './button.pc';

function App() {
  return <Button />;
}
```

## Options

### `typescript`

Type: `boolean`
Default: `true`

Generate TypeScript type definitions for components.

### `emitDeclaration`

Type: `boolean`
Default: `false`

Emit TypeScript declarations as separate `.d.ts` files.

## Performance

The loader uses WebAssembly for extremely fast compilation:
- **Parse + Compile**: < 100 microseconds per component
- **No performance impact** on webpack build times

## Examples

### React + TypeScript

```javascript
// webpack.config.js
module.exports = {
  entry: './src/index.tsx',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
      },
      {
        test: /\.pc$/,
        use: {
          loader: '@paperclip-lang/webpack-loader',
          options: {
            typescript: true,
            emitDeclaration: true,
          },
        },
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js', '.pc'],
  },
};
```

### With CSS Loader

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.pc$/,
        use: [
          {
            loader: '@paperclip-lang/webpack-loader',
            options: { typescript: true },
          },
          // Add CSS extraction if needed
          'css-loader',
        ],
      },
    ],
  },
};
```

## License

MIT
