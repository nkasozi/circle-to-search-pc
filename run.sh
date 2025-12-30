#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🚀 Circle to Search - Starting desktop app..."
echo ""

if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Not in project root. Please run from the project directory:"
    echo "   cd /path/to/circle-to-search-pc"
    exit 1
fi

if [ ! -d "src-tauri" ]; then
    echo "❌ Error: src-tauri directory not found"
    exit 1
fi

if [ ! -d "assets" ]; then
    echo "❌ Error: assets directory not found"
    exit 1
fi

if [ ! -f "assets/index.html" ]; then
    echo "❌ Error: assets/index.html not found"
    exit 1
fi

if [ ! -f "assets/js/app.js" ]; then
    echo "❌ Error: assets/js/app.js not found"
    exit 1
fi

echo "✅ All required files present"
echo "🔨 Starting Tauri dev server..."
echo ""

cd src-tauri
cargo tauri dev
