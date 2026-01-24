#!/bin/bash

set -e

APP_NAME="Circle to Search"
BUNDLE_NAME="Circle to Search.app"
BUNDLE_ID="com.circle-to-search.app"
EXECUTABLE_NAME="circle-to-search-pc"
VERSION="0.1.0.0"

echo "Resetting macOS permissions for fresh install..."
tccutil reset ScreenCapture "$BUNDLE_ID" 2>/dev/null && echo "  Screen Recording reset for $BUNDLE_ID" || echo "  Screen Recording: no existing entry"
tccutil reset Accessibility "$BUNDLE_ID" 2>/dev/null && echo "  Accessibility reset for $BUNDLE_ID" || echo "  Accessibility: no existing entry"
tccutil reset ListenEvent "$BUNDLE_ID" 2>/dev/null && echo "  Input Monitoring reset for $BUNDLE_ID" || echo "  Input Monitoring: no existing entry"
echo ""

echo ""
echo "Building release binary..."
cargo build --release

echo "Creating app bundle structure..."
rm -rf "target/release/${BUNDLE_NAME}"
mkdir -p "target/release/${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "target/release/${BUNDLE_NAME}/Contents/Resources"

echo "Copying executable..."
cp "target/release/${EXECUTABLE_NAME}" "target/release/${BUNDLE_NAME}/Contents/MacOS/"

echo "Copying Info.plist..."
cp Info.plist "target/release/${BUNDLE_NAME}/Contents/"

if [ -f "src/assets/icon.png" ]; then
    echo "Copying icon..."
    cp "src/assets/icon.png" "target/release/${BUNDLE_NAME}/Contents/Resources/icon.png"
fi

if [ -d "tessdata" ]; then
    echo "Copying tessdata..."
    cp -r tessdata "target/release/${BUNDLE_NAME}/Contents/Resources/"
fi

echo ""
echo "Stopping any running instance and cleaning up old installation..."
pkill -f "${EXECUTABLE_NAME}" 2>/dev/null && echo "  Stopped running instance" || echo "  No running instance found"
if [ -d "/Applications/${BUNDLE_NAME}" ]; then
    rm -rf "/Applications/${BUNDLE_NAME}"
    echo "  Removed old installation from /Applications"
else
    echo "  No existing installation in /Applications"
fi

echo ""
echo "Installing to /Applications..."
cp -r "target/release/${BUNDLE_NAME}" /Applications/
echo "  Installed to /Applications/${BUNDLE_NAME}"

echo ""
echo "App bundle created and installed successfully!"
echo ""
echo "The app will run as a system tray application (no dock icon)."
echo ""
echo "To launch, open 'Circle to Search' from Applications or Spotlight."