#!/bin/bash

set -e

echo "🔨 Building Paperclip Bundler Loaders..."
echo

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found!"
    echo "Install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Build WASM core
echo "📦 Building WASM core..."
cd packages/wasm
wasm-pack build --target bundler --out-dir pkg
wasm-pack build --target nodejs --out-dir pkg-node
echo "✅ WASM core built"
echo

cd ../..

# Build webpack loader
if [ -d "packages/loader-webpack" ]; then
    echo "📦 Building webpack loader..."
    cd packages/loader-webpack
    yarn install 2>/dev/null || true
    yarn build
    echo "✅ Webpack loader built"
    echo
    cd ../..
fi

# Build vite plugin
if [ -d "packages/plugin-vite" ]; then
    echo "📦 Building vite plugin..."
    cd packages/plugin-vite
    yarn install 2>/dev/null || true
    yarn build
    echo "✅ Vite plugin built"
    echo
    cd ../..
fi

# Build rollup plugin
if [ -d "packages/plugin-rollup" ]; then
    echo "📦 Building rollup plugin..."
    cd packages/plugin-rollup
    yarn install 2>/dev/null || true
    yarn build
    echo "✅ Rollup plugin built"
    echo
    cd ../..
fi

echo "🎉 All loaders built successfully!"
echo
echo "Next steps:"
echo "  1. Test WASM: cd packages/wasm && cargo test"
echo "  2. Link locally: cd packages/wasm && yarn link"
echo "  3. Create example: See examples/ directory"
