# Paperclip Examples

This directory contains example projects demonstrating Paperclip integration with various bundlers and frameworks.

## Examples

### 1. Vite + React + TypeScript

**Location**: `vite-react/`

The most complete example showing Paperclip with modern tooling.

**Features**:
- ⚡️ Lightning fast HMR with Vite
- 🎨 Multiple Paperclip components (Button, Card, Hero, Feature)
- 📦 Demonstrates slots and component composition
- 🔒 Full TypeScript support
- 💅 Scoped styles with hover states

**Quick Start**:
```bash
cd vite-react
yarn install
yarn dev
```

**Best for**: New projects, modern dev experience

---

### 2. Webpack + React + TypeScript

**Location**: `webpack-react/`

Example using Paperclip with Webpack.

**Features**:
- 📦 Webpack 5 configuration
- 🔥 Hot Module Replacement
- 🔒 TypeScript support
- 🏗️ Production builds

**Quick Start**:
```bash
cd webpack-react
yarn install
yarn dev
```

**Best for**: Existing Webpack projects, enterprise setups

---

## Prerequisites

Before running any example, you must build the WASM package:

```bash
# From project root
cd packages/wasm
yarn build

# Optionally link for local development
yarn link
```

Then in each example directory:
```bash
yarn link @paperclip-lang/wasm
yarn install
yarn dev
```

## What's Demonstrated

All examples show:

1. **Component Definition** - Writing `.pc` files
2. **Slots** - Flexible content insertion
3. **Styles** - Scoped CSS with pseudo-selectors
4. **TypeScript** - Type-safe component usage
5. **Hot Reload** - Instant updates during development
6. **Production Builds** - Optimized output

## Creating Your Own Example

Want to add a new example? Follow this pattern:

```
examples/your-example/
├── package.json          # Dependencies and scripts
├── README.md             # Setup instructions
├── [bundler].config.js   # Bundler configuration
└── src/
    ├── components/       # .pc component files
    └── index.tsx         # Entry point
```

### Key Points

1. **Add Paperclip plugin/loader** to your bundler config
2. **Declare .pc modules** in TypeScript (if using TS)
3. **Import components** like regular React components
4. **Use slots** with named `slot` attributes

## Example Component

```paperclip
// Button.pc
public component Button {
    slot children {
        text "Click me"
    }

    render button {
        style {
            padding: 12px 24px
            background: #667eea
            color: white
            border-radius: 8px
        }

        style hover {
            background: #764ba2
        }

        children
    }
}
```

```tsx
// App.tsx
import { Button } from './components/Button.pc';

function App() {
  return (
    <div>
      <Button>Custom Text</Button>
      <Button>
        <span>🎉</span> With Icon
      </Button>
    </div>
  );
}
```

## Comparison Table

| Example | Bundler | HMR Speed | Setup Complexity | Best For |
|---------|---------|-----------|------------------|----------|
| **Vite + React** | Vite 5 | ⚡️ Instant | 🟢 Simple | New projects, modern DX |
| **Webpack + React** | Webpack 5 | ✅ Fast | 🟡 Moderate | Existing projects, enterprise |

## Troubleshooting

### WASM Module Not Found

```bash
cd packages/wasm
yarn build
yarn link
cd ../../examples/[example-name]
yarn link @paperclip-lang/wasm
```

### TypeScript Errors

Add `.pc` module declarations:
```typescript
declare module '*.pc' {
  import { FC } from 'react';
  export const ComponentName: FC<any>;
}
```

### Hot Reload Not Working

Restart the dev server after adding new `.pc` files.

## Contributing

To add a new example:

1. Create a new directory in `examples/`
2. Add bundler configuration
3. Create sample components
4. Write a detailed README
5. Update this main README

## Learn More

- [Paperclip Documentation](../README.md)
- [Loaders Documentation](../LOADERS.md)
- [WASM API](../packages/wasm/README.md)

## License

MIT
