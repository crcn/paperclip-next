#!/bin/bash

set -e

echo "🚀 Setting up Paperclip examples..."
echo

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found!"
    echo "Install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Build WASM package
echo "📦 Building WASM package..."
cd ../packages/wasm
wasm-pack build --target bundler --out-dir pkg
npm link
echo "✅ WASM package built and linked"
echo

# Setup Vite example
if [ -d "../examples/vite-react" ]; then
    echo "📦 Setting up Vite + React example..."
    cd ../../examples/vite-react
    yarn link @paperclip-lang/wasm
    yarn install
    echo "✅ Vite example ready"
    echo "   Run: cd examples/vite-react && yarn dev"
    echo
fi

# Setup Webpack example
if [ -d "../webpack-react" ]; then
    echo "📦 Setting up Webpack + React example..."
    cd ../webpack-react
    yarn link @paperclip-lang/wasm
    yarn install
    echo "✅ Webpack example ready"
    echo "   Run: cd examples/webpack-react && yarn dev"
    echo
fi

echo "🎉 All examples are ready!"
echo
echo "Quick start:"
echo "  cd examples/vite-react && yarn dev"
