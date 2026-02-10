#!/bin/bash
set -e

echo "Installing Rust..."
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env" 2>/dev/null || source "/rust/env" 2>/dev/null || true
export PATH="$HOME/.cargo/bin:/rust/bin:$PATH"

echo "Installing wasm-pack..."
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

echo "Building WASM module..."
cd ../core
wasm-pack build --target web --out-dir ../web-demo/src/wasm

echo "Building web application..."
cd ../web-demo
npm run build
