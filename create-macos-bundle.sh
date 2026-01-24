#!/bin/bash

set -e

APP_NAME="Circle to Search"
BUNDLE_NAME="Circle to Search.app"
BUNDLE_ID="com.circle-to-search.app"
EXECUTABLE_NAME="circle-to-search-pc"
VERSION="0.1.0.0"

echo "========================================"
echo "  Building $APP_NAME for macOS"
echo "========================================"
echo ""

echo "[1/7] Checking prerequisites..."
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo not found. Please install from https://rustup.rs"
    exit 1
fi
echo "  ✓ Cargo found"

if [ ! -f "tessdata/eng.traineddata" ]; then
    echo "  tessdata not found, downloading..."
    if ! ./download-tessdata.sh; then
        echo "ERROR: Failed to download tessdata"
        exit 1
    fi
fi
echo "  ✓ tessdata found"

if [ ! -f "Info.plist" ]; then
    echo "ERROR: Info.plist not found in project root"
    exit 1
fi
echo "  ✓ Info.plist found"

echo ""
echo "[2/7] Resetting macOS permissions for fresh install..."
tccutil reset ScreenCapture "$BUNDLE_ID" 2>/dev/null && echo "  ✓ Screen Recording reset for $BUNDLE_ID" || echo "  ✓ Screen Recording: no existing entry"
tccutil reset Accessibility "$BUNDLE_ID" 2>/dev/null && echo "  ✓ Accessibility reset for $BUNDLE_ID" || echo "  ✓ Accessibility: no existing entry"
tccutil reset ListenEvent "$BUNDLE_ID" 2>/dev/null && echo "  ✓ Input Monitoring reset for $BUNDLE_ID" || echo "  ✓ Input Monitoring: no existing entry"

echo ""
echo "[3/7] Building release binary..."
cargo build --release
echo "  ✓ Build successful"

echo ""
echo "[4/7] Creating app bundle structure..."
rm -rf "target/release/${BUNDLE_NAME}"
mkdir -p "target/release/${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "target/release/${BUNDLE_NAME}/Contents/Resources"
echo "  ✓ Bundle structure created"

echo ""
echo "[5/7] Copying files..."
cp "target/release/${EXECUTABLE_NAME}" "target/release/${BUNDLE_NAME}/Contents/MacOS/"
echo "  ✓ Copied executable"

cp Info.plist "target/release/${BUNDLE_NAME}/Contents/"
echo "  ✓ Copied Info.plist"

if [ -f "src/assets/icon.png" ]; then
    cp "src/assets/icon.png" "target/release/${BUNDLE_NAME}/Contents/Resources/icon.png"
    echo "  ✓ Copied icon"
fi

cp -r tessdata "target/release/${BUNDLE_NAME}/Contents/Resources/"
echo "  ✓ Copied tessdata"

echo ""
echo "[6/7] Stopping any running instance..."
if pkill -f "${EXECUTABLE_NAME}" 2>/dev/null; then
    sleep 1
    echo "  ✓ Stopped existing instance"
else
    echo "  ✓ No running instance found"
fi

if [ -d "/Applications/${BUNDLE_NAME}" ]; then
    rm -rf "/Applications/${BUNDLE_NAME}"
    echo "  ✓ Removed old installation from /Applications"
else
    echo "  ✓ No existing installation in /Applications"
fi

echo ""
echo "[7/7] Installing to /Applications..."
cp -r "target/release/${BUNDLE_NAME}" /Applications/
echo "  ✓ Installed to /Applications/${BUNDLE_NAME}"

echo ""
echo "========================================"
echo "  Installation Complete!"
echo "========================================"
echo ""
echo "The app will run as a system tray application (no dock icon)."
echo ""
echo "To launch:"
echo "  1. Open 'Circle to Search' from Applications or Spotlight"
echo "  2. Grant Screen Recording and Accessibility permissions when prompted"
echo ""
echo "Note: First launch requires right-click → Open due to Gatekeeper."
echo ""