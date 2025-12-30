#!/bin/bash

echo "🚀 Circle to Search - PC Edition"
echo "================================"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Install from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust toolchain found"
echo ""

echo "🔨 Building project..."
cargo check --all || exit 1

echo ""
echo "✅ Build successful!"
echo ""
echo "🎯 To run the app:"
echo "   cd src-tauri && cargo tauri dev"
echo ""
echo "📝 After launching, press ⌘ + S to activate the search overlay"
