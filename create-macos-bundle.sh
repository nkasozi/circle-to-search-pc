#!/bin/bash

set -e

APP_NAME="Circle to Search"
BUNDLE_NAME="Circle to Search.app"
EXECUTABLE_NAME="circle-to-search-pc"
VERSION="0.1.0"

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

echo "App bundle created at: target/release/${BUNDLE_NAME}"
echo ""
echo "To install, copy the app bundle to /Applications:"
echo "  cp -r \"target/release/${BUNDLE_NAME}\" /Applications/"
echo ""
echo "The app will run as a system tray application (no dock icon)."
