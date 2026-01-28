# Paperclip + Webpack + React Example

A complete example showing how to use Paperclip with Webpack and React.

## Prerequisites

Build the WASM package first:

```bash
# From the project root
cd packages/wasm
yarn build
yarn link

# Link in this example
cd ../../examples/webpack-react
yarn link @paperclip-lang/wasm
```

## Quick Start

```bash
# Install dependencies
yarn install

# Start dev server
yarn dev

# Build for production
yarn build
```

## Webpack Configuration

The key part is the Paperclip loader in `webpack.config.js`:

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.pc$/,
        use: {
          loader: '@paperclip-lang/webpack-loader',
          options: {
            typescript: true,
            emitDeclaration: false,
          },
        },
      },
    ],
  },
};
```

## Project Structure

```
src/
├── components/        # .pc component files
├── index.tsx          # React entry point
└── App.tsx            # Main app component
```

You can copy the Paperclip components from the `../vite-react/src/components/` directory to use them here.

## Key Differences from Vite

- **Build Tool**: Webpack instead of Vite
- **Dev Server**: Webpack Dev Server (port 3000)
- **Hot Reload**: Webpack HMR (slightly slower than Vite)
- **Config**: More verbose webpack.config.js

## Features

- ✅ TypeScript definitions
- ✅ Hot Module Replacement
- ✅ Production builds
- ✅ CSS extraction

## Learn More

- [Webpack Loader Documentation](../../packages/loader-webpack/README.md)
- [Vite Example](../vite-react/) - Similar example with Vite

## License

MIT
