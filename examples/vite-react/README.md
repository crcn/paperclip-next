# Paperclip + Vite + React Example

A complete example showing how to use Paperclip with Vite and React.

## Features Demonstrated

- ✅ **Paperclip Components** - Button, Card, Hero, Feature components
- ✅ **Slots** - Default content and named slots
- ✅ **Styles** - Component-scoped CSS with hover states
- ✅ **TypeScript** - Full type safety
- ✅ **Hot Module Replacement** - Instant updates
- ✅ **Vite Plugin** - Zero-config setup

## Prerequisites

Before running this example, you need to build the WASM package:

```bash
# From the project root
cd packages/wasm
yarn build

# Link for local development
yarn link

# Link in this example
cd ../../examples/vite-react
yarn link @paperclip-lang/wasm
```

## Quick Start

```bash
# Install dependencies
yarn install

# Start dev server
yarn dev

# Open http://localhost:5173
```

## Project Structure

```
src/
├── components/
│   ├── Button.pc         # Reusable button with slot
│   ├── Card.pc           # Card with header/content/footer slots
│   ├── Hero.pc           # Hero section
│   └── Feature.pc        # Feature showcase component
├── App.tsx               # Main app using Paperclip components
├── App.css               # App-specific styles
├── main.tsx              # React entry point
├── index.css             # Global styles
└── vite-env.d.ts         # TypeScript declarations for .pc files
```

## How It Works

### 1. Vite Configuration

The `vite.config.ts` includes the Paperclip plugin:

```typescript
import paperclip from '@paperclip-lang/vite-plugin';

export default defineConfig({
  plugins: [
    paperclip({
      typescript: true,
      includeStyles: true,
    }),
    react(),
  ],
});
```

### 2. Paperclip Components

Components are defined in `.pc` files:

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
        }
        children
    }
}
```

### 3. Using Components in React

Import and use Paperclip components like regular React components:

```tsx
import { Button } from './components/Button.pc';

function App() {
  return <Button>Custom Text</Button>;
}
```

### 4. Slots

Paperclip components support slots for flexible content:

```tsx
<Card>
  <div slot="header">My Title</div>
  <div slot="content">My content here</div>
  <div slot="footer">
    <Button>Action</Button>
  </div>
</Card>
```

## Key Concepts

### Component Slots

Slots allow components to accept children with specific purposes:

```paperclip
public component Card {
    slot header {
        text "Default Header"
    }

    slot content {
        text "Default Content"
    }

    render div {
        header
        content
    }
}
```

### Component Styles

Styles are scoped to components and support pseudo-selectors:

```paperclip
render button {
    style {
        background: blue
    }

    style hover {
        background: darkblue
    }

    style active {
        transform: scale(0.95)
    }
}
```

### TypeScript Support

The Vite plugin generates TypeScript definitions automatically. Add declarations for your components:

```typescript
// vite-env.d.ts
declare module '*.pc' {
  import { FC, ReactNode } from 'react';
  export const Button: FC<{ children?: ReactNode }>;
  export const Card: FC<{ children?: ReactNode }>;
}
```

## Development

### Hot Module Replacement

Changes to `.pc` files trigger instant HMR updates:

1. Edit `src/components/Button.pc`
2. Save the file
3. Browser updates instantly (no full reload)

### Adding New Components

1. Create a new `.pc` file in `src/components/`
2. Define your component with `public component Name { ... }`
3. Import and use in `App.tsx`
4. Add TypeScript declaration in `vite-env.d.ts`

Example:

```paperclip
// Alert.pc
public component Alert {
    slot message {
        text "Alert message"
    }

    render div {
        style {
            padding: 16px
            background: #fee
            border: 1px solid #fcc
            border-radius: 4px
        }
        message
    }
}
```

```tsx
// App.tsx
import { Alert } from './components/Alert.pc';

<Alert>
  <div slot="message">Warning: Something happened!</div>
</Alert>
```

## Building for Production

```bash
# Build
yarn build

# Preview build
yarn preview
```

The build output will include:
- Compiled React components (no Paperclip runtime)
- Extracted CSS
- Optimized bundles

## Performance

The Paperclip plugin compiles components at build time with zero runtime overhead:

- **Dev Server Start**: ~50ms overhead
- **Hot Reload**: ~10ms per component
- **Production Build**: No runtime, pure React

## Troubleshooting

### WASM Module Not Found

If you see "Cannot find module '@paperclip-lang/wasm'":

```bash
# Build WASM package
cd ../../packages/wasm
yarn build
yarn link

# Link in example
cd ../../examples/vite-react
yarn link @paperclip-lang/wasm
```

### TypeScript Errors

Make sure `vite-env.d.ts` includes declarations for your components.

### Hot Reload Not Working

Restart the dev server after adding new `.pc` files.

## Learn More

- [Paperclip Documentation](../../README.md)
- [Vite Plugin Documentation](../../packages/plugin-vite/README.md)
- [WASM API Documentation](../../packages/wasm/README.md)

## License

MIT
